use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

const IDEMPOTENCY_STORE_FILENAME: &str = "idempotency_store.json";
const AUDIT_SEAL_STORE_FILENAME: &str = "audit_seal_store.json";

enum WriteBehindCommand<T> {
    Persist(T),
    Flush(Sender<io::Result<()>>),
    Shutdown,
}

struct WriteBehind<T> {
    tx: Sender<WriteBehindCommand<T>>,
}

impl<T> Drop for WriteBehind<T> {
    fn drop(&mut self) {
        let _ = self.tx.send(WriteBehindCommand::Shutdown);
    }
}

impl<T> WriteBehind<T>
where
    T: Serialize + Send + 'static,
{
    fn new(path: PathBuf, worker_name: &str) -> io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name(worker_name.to_string())
            .spawn(move || run_write_behind_worker(path, rx))?;
        Ok(Self { tx })
    }

    fn persist(&self, snapshot: T) {
        let _ = self.tx.send(WriteBehindCommand::Persist(snapshot));
    }

    fn flush(&self) -> io::Result<()> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.tx
            .send(WriteBehindCommand::Flush(ack_tx))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "persistence worker stopped"))?;
        ack_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "persistence worker stopped"))?
    }
}

fn run_write_behind_worker<T>(path: PathBuf, rx: Receiver<WriteBehindCommand<T>>)
where
    T: Serialize,
{
    let mut latest_snapshot = None;
    loop {
        match rx.recv() {
            Ok(WriteBehindCommand::Persist(snapshot)) => {
                latest_snapshot = Some(snapshot);
            }
            Ok(WriteBehindCommand::Flush(ack)) => {
                let _ = ack.send(persist_latest_snapshot(&path, &mut latest_snapshot));
                continue;
            }
            Ok(WriteBehindCommand::Shutdown) => {
                let _ = persist_latest_snapshot(&path, &mut latest_snapshot);
                return;
            }
            Err(_) => {
                let _ = persist_latest_snapshot(&path, &mut latest_snapshot);
                return;
            }
        }

        loop {
            match rx.try_recv() {
                Ok(WriteBehindCommand::Persist(snapshot)) => {
                    latest_snapshot = Some(snapshot);
                }
                Ok(WriteBehindCommand::Flush(ack)) => {
                    let _ = ack.send(persist_latest_snapshot(&path, &mut latest_snapshot));
                }
                Ok(WriteBehindCommand::Shutdown) => {
                    let _ = persist_latest_snapshot(&path, &mut latest_snapshot);
                    return;
                }
                Err(TryRecvError::Empty) => {
                    let _ = persist_latest_snapshot(&path, &mut latest_snapshot);
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    let _ = persist_latest_snapshot(&path, &mut latest_snapshot);
                    return;
                }
            }
        }
    }
}

fn persist_latest_snapshot<T>(path: &Path, snapshot: &mut Option<T>) -> io::Result<()>
where
    T: Serialize,
{
    if let Some(value) = snapshot.take() {
        persist_snapshot(path, &value)?;
    }
    Ok(())
}

fn persist_snapshot<T>(path: &Path, snapshot: &T) -> io::Result<()>
where
    T: Serialize,
{
    let encoded = serde_json::to_vec(snapshot).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to serialize persistence snapshot: {error}"),
        )
    })?;
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, encoded)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}

fn load_snapshot_or_default<T>(path: &Path) -> io::Result<T>
where
    T: DeserializeOwned + Default,
{
    match fs::read(path) {
        Ok(encoded) => serde_json::from_slice(&encoded).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to deserialize persistence snapshot: {error}"),
            )
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(T::default()),
        Err(error) => Err(error),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyEntry {
    pub key: String,
    pub payload_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyStatus {
    FirstSeen,
    Replay,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdempotencyError {
    #[error("idempotency key has different payload hash")]
    PayloadHashMismatch,
    #[error("idempotency store poisoned")]
    StorePoisoned,
}

#[derive(Clone)]
pub struct InMemoryIdempotencyStore {
    inner: Arc<Mutex<HashMap<String, IdempotencyEntry>>>,
    persistence: Option<Arc<WriteBehind<HashMap<String, IdempotencyEntry>>>>,
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            persistence: None,
        }
    }
}

impl InMemoryIdempotencyStore {
    pub fn with_persistence_dir(dir: impl AsRef<Path>) -> io::Result<Self> {
        let path = dir.as_ref().join(IDEMPOTENCY_STORE_FILENAME);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let loaded = load_snapshot_or_default(&path)?;
        let persistence = Arc::new(WriteBehind::new(path, "idempotency-write-behind")?);
        Ok(Self {
            inner: Arc::new(Mutex::new(loaded)),
            persistence: Some(persistence),
        })
    }

    pub fn flush_persistence(&self) -> io::Result<()> {
        match &self.persistence {
            Some(persistence) => persistence.flush(),
            None => Ok(()),
        }
    }

    pub fn check_or_insert(
        &self,
        key: &str,
        payload: &Value,
    ) -> Result<IdempotencyStatus, IdempotencyError> {
        let hash = payload_hash(payload);
        let mut store = self
            .inner
            .lock()
            .map_err(|_| IdempotencyError::StorePoisoned)?;

        let (status, snapshot) = match store.get(key) {
            Some(existing) if existing.payload_hash == hash => (IdempotencyStatus::Replay, None),
            Some(_) => return Err(IdempotencyError::PayloadHashMismatch),
            None => {
                store.insert(
                    key.to_string(),
                    IdempotencyEntry {
                        key: key.to_string(),
                        payload_hash: hash,
                    },
                );
                let snapshot = self.persistence.as_ref().map(|_| store.clone());
                (IdempotencyStatus::FirstSeen, snapshot)
            }
        };
        drop(store);

        if let Some(snapshot) = snapshot {
            if let Some(persistence) = &self.persistence {
                persistence.persist(snapshot);
            }
        }

        Ok(status)
    }
}

pub fn payload_hash(payload: &Value) -> String {
    let encoded = serde_json::to_vec(payload).expect("payload serialization should not fail");
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditSealEntry {
    pub sequence: u64,
    pub event_type: String,
    pub entity_scope: Vec<String>,
    pub payload_hash: String,
    pub previous_seal: String,
    pub seal: String,
    pub created_at_ns: i64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuditSealError {
    #[error("audit seal store poisoned")]
    StorePoisoned,
    #[error("audit seal chain broken at sequence {sequence}")]
    ChainBroken { sequence: u64 },
    #[error("audit seal tampered at sequence {sequence}")]
    Tampered { sequence: u64 },
}

#[derive(Clone)]
pub struct InMemoryAuditSealStore {
    inner: Arc<Mutex<Vec<AuditSealEntry>>>,
    persistence: Option<Arc<WriteBehind<Vec<AuditSealEntry>>>>,
}

impl Default for InMemoryAuditSealStore {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            persistence: None,
        }
    }
}

impl InMemoryAuditSealStore {
    pub fn with_persistence_dir(dir: impl AsRef<Path>) -> io::Result<Self> {
        let path = dir.as_ref().join(AUDIT_SEAL_STORE_FILENAME);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let loaded = load_snapshot_or_default(&path)?;
        let persistence = Arc::new(WriteBehind::new(path, "audit-seal-write-behind")?);
        Ok(Self {
            inner: Arc::new(Mutex::new(loaded)),
            persistence: Some(persistence),
        })
    }

    pub fn flush_persistence(&self) -> io::Result<()> {
        match &self.persistence {
            Some(persistence) => persistence.flush(),
            None => Ok(()),
        }
    }

    pub fn append(
        &self,
        event_type: &str,
        entity_scope: &[String],
        payload: &Value,
        created_at_ns: i64,
    ) -> Result<AuditSealEntry, AuditSealError> {
        let mut store = self
            .inner
            .lock()
            .map_err(|_| AuditSealError::StorePoisoned)?;
        let sequence = (store.len() as u64) + 1;
        let previous_seal = store
            .last()
            .map(|entry| entry.seal.clone())
            .unwrap_or_else(|| "GENESIS".to_string());
        let payload_digest = payload_hash(payload);
        let canonical_scope = canonical_entity_scope(entity_scope);
        let seal = payload_hash(&json!({
            "sequence": sequence,
            "event_type": event_type,
            "entity_scope": canonical_scope,
            "payload_hash": payload_digest,
            "previous_seal": previous_seal,
            "created_at_ns": created_at_ns
        }));

        let entry = AuditSealEntry {
            sequence,
            event_type: event_type.to_string(),
            entity_scope: canonical_scope,
            payload_hash: payload_digest,
            previous_seal,
            seal,
            created_at_ns,
        };
        store.push(entry.clone());
        let snapshot = self.persistence.as_ref().map(|_| store.clone());
        drop(store);

        if let Some(snapshot) = snapshot {
            if let Some(persistence) = &self.persistence {
                persistence.persist(snapshot);
            }
        }

        Ok(entry)
    }

    pub fn verify_chain(&self) -> Result<(), AuditSealError> {
        let entries = self
            .inner
            .lock()
            .map_err(|_| AuditSealError::StorePoisoned)?
            .clone();
        let mut previous_seal = "GENESIS".to_string();

        for (index, entry) in entries.iter().enumerate() {
            let expected_sequence = (index + 1) as u64;
            if entry.sequence != expected_sequence || entry.previous_seal != previous_seal {
                return Err(AuditSealError::ChainBroken {
                    sequence: entry.sequence,
                });
            }

            let expected_seal = payload_hash(&json!({
                "sequence": entry.sequence,
                "event_type": entry.event_type,
                "entity_scope": canonical_entity_scope(&entry.entity_scope),
                "payload_hash": entry.payload_hash,
                "previous_seal": entry.previous_seal,
                "created_at_ns": entry.created_at_ns
            }));
            if expected_seal != entry.seal {
                return Err(AuditSealError::Tampered {
                    sequence: entry.sequence,
                });
            }

            previous_seal = entry.seal.clone();
        }

        Ok(())
    }

    pub fn len(&self) -> Result<usize, AuditSealError> {
        let store = self
            .inner
            .lock()
            .map_err(|_| AuditSealError::StorePoisoned)?;
        Ok(store.len())
    }
}

fn canonical_entity_scope(entity_scope: &[String]) -> Vec<String> {
    let mut scope = entity_scope
        .iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    scope.sort();
    scope.dedup();
    scope
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScaleSample {
    pub active_users: u32,
    pub throughput_rps: f64,
    pub cpu_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoBendReadiness {
    pub target_active_users: u32,
    pub linearity_ratio_min: f64,
    pub measured_linearity_ratio: f64,
    pub comfortable: bool,
}

pub fn evaluate_no_bend_readiness(
    samples: &[ScaleSample],
    target_active_users: u32,
    linearity_ratio_min: f64,
) -> Option<NoBendReadiness> {
    if samples.len() < 2 {
        return None;
    }

    let mut sorted = samples.to_vec();
    sorted.sort_by_key(|sample| sample.active_users);
    let baseline = sorted.first()?;
    let target = sorted
        .iter()
        .find(|sample| sample.active_users >= target_active_users)?;
    if baseline.active_users == 0 || target.active_users == 0 {
        return None;
    }

    let user_scale = target.active_users as f64 / baseline.active_users as f64;
    let throughput_scale = target.throughput_rps / baseline.throughput_rps.max(0.0001);
    let measured_linearity_ratio = throughput_scale / user_scale.max(0.0001);

    Some(NoBendReadiness {
        target_active_users,
        linearity_ratio_min,
        measured_linearity_ratio,
        comfortable: measured_linearity_ratio >= linearity_ratio_min,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(prefix: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be monotonic from epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "platform-core-{prefix}-{}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("test temp dir should be creatable");
            Self { path }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn first_seen_then_replay() {
        let store = InMemoryIdempotencyStore::default();
        let payload = json!({"event": "order.captured.v1", "amount": 100});

        let first = store.check_or_insert("key-1", &payload).unwrap();
        let replay = store.check_or_insert("key-1", &payload).unwrap();

        assert_eq!(first, IdempotencyStatus::FirstSeen);
        assert_eq!(replay, IdempotencyStatus::Replay);
    }

    #[test]
    fn same_key_different_hash_is_conflict() {
        let store = InMemoryIdempotencyStore::default();
        let payload_a = json!({"event": "order.captured.v1", "amount": 100});
        let payload_b = json!({"event": "order.captured.v1", "amount": 200});

        store.check_or_insert("key-1", &payload_a).unwrap();
        let conflict = store.check_or_insert("key-1", &payload_b).unwrap_err();

        assert_eq!(conflict, IdempotencyError::PayloadHashMismatch);
    }

    #[test]
    fn audit_seal_chain_verifies_after_append() {
        let store = InMemoryAuditSealStore::default();
        let entity_scope = vec!["US_CO_01".to_string()];
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j1"}),
                1700000000000000000,
            )
            .unwrap();
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j2"}),
                1700000001000000000,
            )
            .unwrap();

        assert_eq!(store.len().unwrap(), 2);
        assert_eq!(store.verify_chain(), Ok(()));
    }

    #[test]
    fn audit_seal_detects_payload_tampering() {
        let store = InMemoryAuditSealStore::default();
        let entity_scope = vec!["US_CO_01".to_string()];
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j1"}),
                1700000000000000000,
            )
            .unwrap();
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j2"}),
                1700000001000000000,
            )
            .unwrap();

        {
            let mut guard = store.inner.lock().unwrap();
            guard[1].payload_hash = "tampered".to_string();
        }

        assert_eq!(
            store.verify_chain(),
            Err(AuditSealError::Tampered { sequence: 2 })
        );
    }

    #[test]
    fn idempotency_flush_persists_to_disk() {
        let temp_dir = TempDirGuard::new("idempotency-flush");
        let store = InMemoryIdempotencyStore::with_persistence_dir(&temp_dir.path).unwrap();
        let payload = json!({"event": "order.captured.v1", "amount": 100});

        assert_eq!(
            store.check_or_insert("key-1", &payload).unwrap(),
            IdempotencyStatus::FirstSeen
        );
        store.flush_persistence().unwrap();

        let persisted = fs::read(temp_dir.path.join(IDEMPOTENCY_STORE_FILENAME)).unwrap();
        let entries: HashMap<String, IdempotencyEntry> =
            serde_json::from_slice(&persisted).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries["key-1"].payload_hash, payload_hash(&payload));
    }

    #[test]
    fn idempotency_reloads_after_restart() {
        let temp_dir = TempDirGuard::new("idempotency-restart");
        let payload = json!({"event": "order.captured.v1", "amount": 100});

        {
            let store = InMemoryIdempotencyStore::with_persistence_dir(&temp_dir.path).unwrap();
            assert_eq!(
                store.check_or_insert("key-1", &payload).unwrap(),
                IdempotencyStatus::FirstSeen
            );
            store.flush_persistence().unwrap();
        }

        let reloaded = InMemoryIdempotencyStore::with_persistence_dir(&temp_dir.path).unwrap();
        assert_eq!(
            reloaded.check_or_insert("key-1", &payload).unwrap(),
            IdempotencyStatus::Replay
        );
    }

    #[test]
    fn audit_seal_flush_persists_to_disk() {
        let temp_dir = TempDirGuard::new("audit-flush");
        let store = InMemoryAuditSealStore::with_persistence_dir(&temp_dir.path).unwrap();
        let entity_scope = vec!["US_CO_01".to_string()];

        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j1"}),
                1700000000000000000,
            )
            .unwrap();
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j2"}),
                1700000001000000000,
            )
            .unwrap();
        store.flush_persistence().unwrap();

        let persisted = fs::read(temp_dir.path.join(AUDIT_SEAL_STORE_FILENAME)).unwrap();
        let entries: Vec<AuditSealEntry> = serde_json::from_slice(&persisted).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].sequence, 2);
    }

    #[test]
    fn audit_seal_reloads_after_restart() {
        let temp_dir = TempDirGuard::new("audit-restart");
        let entity_scope = vec!["US_CO_01".to_string()];

        {
            let store = InMemoryAuditSealStore::with_persistence_dir(&temp_dir.path).unwrap();
            store
                .append(
                    "posting.posted",
                    &entity_scope,
                    &json!({"journal_id": "j1"}),
                    1700000000000000000,
                )
                .unwrap();
            store
                .append(
                    "posting.posted",
                    &entity_scope,
                    &json!({"journal_id": "j2"}),
                    1700000001000000000,
                )
                .unwrap();
            store.flush_persistence().unwrap();
        }

        let reloaded = InMemoryAuditSealStore::with_persistence_dir(&temp_dir.path).unwrap();
        assert_eq!(reloaded.len().unwrap(), 2);
        assert_eq!(reloaded.verify_chain(), Ok(()));
        let appended = reloaded
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j3"}),
                1700000002000000000,
            )
            .unwrap();
        assert_eq!(appended.sequence, 3);
    }

    #[test]
    fn no_bend_readiness_passes_with_linear_scaling() {
        let samples = vec![
            ScaleSample {
                active_users: 500,
                throughput_rps: 82.0,
                cpu_percent: 42.0,
            },
            ScaleSample {
                active_users: 1000,
                throughput_rps: 164.0,
                cpu_percent: 56.0,
            },
            ScaleSample {
                active_users: 2000,
                throughput_rps: 329.0,
                cpu_percent: 73.0,
            },
        ];
        let readiness = evaluate_no_bend_readiness(&samples, 2000, 0.95).unwrap();
        assert!(readiness.comfortable);
        assert!(readiness.measured_linearity_ratio >= 0.95);
    }

    #[test]
    fn no_bend_readiness_fails_with_bending_curve() {
        let samples = vec![
            ScaleSample {
                active_users: 500,
                throughput_rps: 82.0,
                cpu_percent: 42.0,
            },
            ScaleSample {
                active_users: 1000,
                throughput_rps: 130.0,
                cpu_percent: 70.0,
            },
            ScaleSample {
                active_users: 2000,
                throughput_rps: 190.0,
                cpu_percent: 90.0,
            },
        ];
        let readiness = evaluate_no_bend_readiness(&samples, 2000, 0.95).unwrap();
        assert!(!readiness.comfortable);
        assert!(readiness.measured_linearity_ratio < 0.95);
    }
}
