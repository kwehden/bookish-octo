use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread;

use chrono::{DateTime, NaiveDate, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

const JOURNAL_STORE_FILENAME: &str = "journal_store.json";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalHeader {
    pub journal_id: Uuid,
    pub journal_number: String,
    pub status: JournalStatus,
    pub tenant_id: String,
    pub legal_entity_id: String,
    pub ledger_book: String,
    pub accounting_date: NaiveDate,
    pub posted_at: DateTime<Utc>,
    pub source_event_ids: Vec<String>,
    pub posting_run_id: String,
    pub book_policy_id: String,
    pub policy_version: String,
    pub fx_rate_set_id: String,
    pub ruleset_version: String,
    pub workflow_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalLine {
    pub line_number: u32,
    pub account_id: String,
    pub entry_side: EntrySide,
    pub amount_minor: i64,
    pub currency: String,
    pub base_amount_minor: i64,
    pub base_currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalRecord {
    pub header: JournalHeader,
    pub lines: Vec<JournalLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntrySide {
    Debit,
    Credit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalStatus {
    Posted,
    Reversed,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("journal already exists")]
    JournalExists,
    #[error("journal is unbalanced")]
    Unbalanced,
    #[error("posted journal is immutable")]
    Immutable,
    #[error("journal not found")]
    NotFound,
    #[error("journal already reversed")]
    AlreadyReversed,
}

pub struct InMemoryJournalRepository {
    journals: HashMap<Uuid, JournalRecord>,
    persistence: Option<Arc<WriteBehind<HashMap<Uuid, JournalRecord>>>>,
}

impl Default for InMemoryJournalRepository {
    fn default() -> Self {
        Self {
            journals: HashMap::new(),
            persistence: None,
        }
    }
}

impl InMemoryJournalRepository {
    pub fn with_persistence_dir(dir: impl AsRef<Path>) -> io::Result<Self> {
        let path = dir.as_ref().join(JOURNAL_STORE_FILENAME);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let loaded = load_snapshot_or_default(&path)?;
        let persistence = Arc::new(WriteBehind::new(path, "journal-write-behind")?);
        Ok(Self {
            journals: loaded,
            persistence: Some(persistence),
        })
    }

    pub fn flush_persistence(&self) -> io::Result<()> {
        match &self.persistence {
            Some(persistence) => persistence.flush(),
            None => Ok(()),
        }
    }

    pub fn insert_posted(&mut self, record: JournalRecord) -> Result<(), LedgerError> {
        if self.journals.contains_key(&record.header.journal_id) {
            return Err(LedgerError::JournalExists);
        }
        validate_balanced(&record.lines)?;
        self.journals.insert(record.header.journal_id, record);
        if let Some(persistence) = &self.persistence {
            persistence.persist(self.journals.clone());
        }
        Ok(())
    }

    pub fn get(&self, journal_id: &Uuid) -> Option<&JournalRecord> {
        self.journals.get(journal_id)
    }

    pub fn all(&self) -> Vec<JournalRecord> {
        self.journals.values().cloned().collect()
    }

    pub fn update_posted(
        &mut self,
        _journal_id: &Uuid,
        _record: JournalRecord,
    ) -> Result<(), LedgerError> {
        Err(LedgerError::Immutable)
    }

    pub fn reverse(&mut self, journal_id: &Uuid) -> Result<(), LedgerError> {
        {
            let record = self
                .journals
                .get_mut(journal_id)
                .ok_or(LedgerError::NotFound)?;
            if record.header.status == JournalStatus::Reversed {
                return Err(LedgerError::AlreadyReversed);
            }
            record.header.status = JournalStatus::Reversed;
        }
        if let Some(persistence) = &self.persistence {
            persistence.persist(self.journals.clone());
        }
        Ok(())
    }
}

pub fn validate_balanced(lines: &[JournalLine]) -> Result<(), LedgerError> {
    let mut debit_total = 0_i64;
    let mut credit_total = 0_i64;
    let mut base_debit_total = 0_i64;
    let mut base_credit_total = 0_i64;

    for line in lines {
        match line.entry_side {
            EntrySide::Debit => {
                debit_total += line.amount_minor;
                base_debit_total += line.base_amount_minor;
            }
            EntrySide::Credit => {
                credit_total += line.amount_minor;
                base_credit_total += line.base_amount_minor;
            }
        }
    }

    if debit_total == credit_total && base_debit_total == base_credit_total {
        Ok(())
    } else {
        Err(LedgerError::Unbalanced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                "ledger-posting-{prefix}-{}-{nanos}",
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

    fn sample_header() -> JournalHeader {
        JournalHeader {
            journal_id: Uuid::new_v4(),
            journal_number: "USCO01-2026-000001".to_string(),
            status: JournalStatus::Posted,
            tenant_id: "tenant_1".to_string(),
            legal_entity_id: "US_CO_01".to_string(),
            ledger_book: "US_GAAP".to_string(),
            accounting_date: NaiveDate::from_ymd_opt(2026, 2, 21).unwrap(),
            posted_at: Utc::now(),
            source_event_ids: vec!["evt_1".to_string()],
            posting_run_id: "run_1".to_string(),
            book_policy_id: "policy_dual_book".to_string(),
            policy_version: "1.0.0".to_string(),
            fx_rate_set_id: "fx_2026_02_21".to_string(),
            ruleset_version: "v1".to_string(),
            workflow_id: Some("wf_1".to_string()),
        }
    }

    fn balanced_lines() -> Vec<JournalLine> {
        vec![
            JournalLine {
                line_number: 1,
                account_id: "1105".to_string(),
                entry_side: EntrySide::Debit,
                amount_minor: 10000,
                currency: "USD".to_string(),
                base_amount_minor: 10000,
                base_currency: "USD".to_string(),
            },
            JournalLine {
                line_number: 2,
                account_id: "4000".to_string(),
                entry_side: EntrySide::Credit,
                amount_minor: 10000,
                currency: "USD".to_string(),
                base_amount_minor: 10000,
                base_currency: "USD".to_string(),
            },
        ]
    }

    #[test]
    fn inserted_journal_must_be_balanced() {
        let mut repo = InMemoryJournalRepository::default();
        let mut lines = balanced_lines();
        lines[1].amount_minor = 9000;

        let result = repo.insert_posted(JournalRecord {
            header: sample_header(),
            lines,
        });

        assert_eq!(result, Err(LedgerError::Unbalanced));
    }

    #[test]
    fn posted_journal_is_immutable() {
        let mut repo = InMemoryJournalRepository::default();
        let record = JournalRecord {
            header: sample_header(),
            lines: balanced_lines(),
        };
        let journal_id = record.header.journal_id;

        repo.insert_posted(record.clone()).unwrap();
        let update = repo.update_posted(&journal_id, record);

        assert_eq!(update, Err(LedgerError::Immutable));
    }

    #[test]
    fn reverse_transitions_posted_to_reversed_once() {
        let mut repo = InMemoryJournalRepository::default();
        let record = JournalRecord {
            header: sample_header(),
            lines: balanced_lines(),
        };
        let journal_id = record.header.journal_id;
        repo.insert_posted(record).unwrap();

        repo.reverse(&journal_id).unwrap();
        let status = repo.get(&journal_id).unwrap().header.status.clone();
        assert_eq!(status, JournalStatus::Reversed);

        let duplicate_reverse = repo.reverse(&journal_id);
        assert_eq!(duplicate_reverse, Err(LedgerError::AlreadyReversed));
    }

    #[test]
    fn reverse_fails_for_missing_journal() {
        let mut repo = InMemoryJournalRepository::default();
        let missing = Uuid::new_v4();

        let error = repo.reverse(&missing);
        assert_eq!(error, Err(LedgerError::NotFound));
    }

    #[test]
    fn flush_persists_journal_store_to_disk() {
        let temp_dir = TempDirGuard::new("journal-flush");
        let mut repo = InMemoryJournalRepository::with_persistence_dir(&temp_dir.path).unwrap();
        let record = JournalRecord {
            header: sample_header(),
            lines: balanced_lines(),
        };
        let journal_id = record.header.journal_id;

        repo.insert_posted(record).unwrap();
        repo.flush_persistence().unwrap();

        let reloaded = InMemoryJournalRepository::with_persistence_dir(&temp_dir.path).unwrap();
        assert!(reloaded.get(&journal_id).is_some());
    }

    #[test]
    fn reloaded_journal_store_preserves_reverse_status() {
        let temp_dir = TempDirGuard::new("journal-restart");
        let journal_id = {
            let mut repo = InMemoryJournalRepository::with_persistence_dir(&temp_dir.path).unwrap();
            let record = JournalRecord {
                header: sample_header(),
                lines: balanced_lines(),
            };
            let journal_id = record.header.journal_id;
            repo.insert_posted(record).unwrap();
            repo.reverse(&journal_id).unwrap();
            repo.flush_persistence().unwrap();
            journal_id
        };

        let reloaded = InMemoryJournalRepository::with_persistence_dir(&temp_dir.path).unwrap();
        let status = reloaded.get(&journal_id).unwrap().header.status.clone();
        assert_eq!(status, JournalStatus::Reversed);
    }
}
