use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread;

use chrono::{Datelike, NaiveDate};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const PERIOD_STORE_FILENAME: &str = "period_store.json";

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct PeriodKey {
    tenant_id: String,
    legal_entity_id: String,
    ledger_book: String,
    period_id: String,
}

impl PeriodKey {
    fn new(tenant_id: &str, legal_entity_id: &str, ledger_book: &str, period_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            legal_entity_id: legal_entity_id.to_string(),
            ledger_book: ledger_book.to_string(),
            period_id: period_id.to_string(),
        }
    }
}

pub struct InMemoryPeriodRepository {
    closed: HashSet<PeriodKey>,
    persistence: Option<Arc<WriteBehind<HashSet<PeriodKey>>>>,
}

impl Default for InMemoryPeriodRepository {
    fn default() -> Self {
        Self {
            closed: HashSet::new(),
            persistence: None,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PeriodError {
    #[error("invalid period id `{0}`")]
    InvalidPeriodId(String),
    #[error("period is closed: {0}")]
    PeriodClosed(String),
}

impl InMemoryPeriodRepository {
    pub fn with_persistence_dir(dir: impl AsRef<Path>) -> io::Result<Self> {
        let path = dir.as_ref().join(PERIOD_STORE_FILENAME);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let loaded = load_snapshot_or_default(&path)?;
        let persistence = Arc::new(WriteBehind::new(path, "period-write-behind")?);
        Ok(Self {
            closed: loaded,
            persistence: Some(persistence),
        })
    }

    pub fn flush_persistence(&self) -> io::Result<()> {
        match &self.persistence {
            Some(persistence) => persistence.flush(),
            None => Ok(()),
        }
    }

    pub fn lock_period(
        &mut self,
        tenant_id: &str,
        legal_entity_id: &str,
        ledger_book: &str,
        period_id: &str,
    ) -> Result<(), PeriodError> {
        if !is_valid_period_id(period_id) {
            return Err(PeriodError::InvalidPeriodId(period_id.to_string()));
        }
        self.closed.insert(PeriodKey::new(
            tenant_id,
            legal_entity_id,
            ledger_book,
            period_id,
        ));
        if let Some(persistence) = &self.persistence {
            persistence.persist(self.closed.clone());
        }
        Ok(())
    }

    pub fn ensure_open(
        &self,
        tenant_id: &str,
        legal_entity_id: &str,
        ledger_book: &str,
        accounting_date: NaiveDate,
    ) -> Result<(), PeriodError> {
        let period_id = period_id_from_date(accounting_date);
        let period = PeriodKey::new(tenant_id, legal_entity_id, ledger_book, &period_id);
        if self.closed.contains(&period) {
            return Err(PeriodError::PeriodClosed(period_id));
        }
        Ok(())
    }
}

pub fn period_id_from_date(date: NaiveDate) -> String {
    format!("{:04}-{:02}", date.year(), date.month())
}

fn is_valid_period_id(period_id: &str) -> bool {
    period_id.len() == 7
        && period_id.chars().nth(4) == Some('-')
        && period_id[..4].chars().all(|c| c.is_ascii_digit())
        && period_id[5..].chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{period_id_from_date, InMemoryPeriodRepository, PeriodError};

    struct TempDirGuard {
        path: std::path::PathBuf,
    }

    impl TempDirGuard {
        fn new(prefix: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be monotonic from epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "posting-api-period-{prefix}-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("test temp dir should be creatable");
            Self { path }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn lock_period_rejects_invalid_period_id() {
        let mut repo = InMemoryPeriodRepository::default();
        let err = repo
            .lock_period("tenant_1", "US_CO_01", "US_GAAP", "202602")
            .unwrap_err();
        assert_eq!(err, PeriodError::InvalidPeriodId("202602".to_string()));
    }

    #[test]
    fn locked_period_rejects_posting_date() {
        let mut repo = InMemoryPeriodRepository::default();
        repo.lock_period("tenant_1", "US_CO_01", "US_GAAP", "2026-02")
            .unwrap();

        let err = repo
            .ensure_open(
                "tenant_1",
                "US_CO_01",
                "US_GAAP",
                NaiveDate::from_ymd_opt(2026, 2, 21).unwrap(),
            )
            .unwrap_err();
        assert_eq!(err, PeriodError::PeriodClosed("2026-02".to_string()));
    }

    #[test]
    fn open_period_allows_posting_date() {
        let repo = InMemoryPeriodRepository::default();
        let result = repo.ensure_open(
            "tenant_1",
            "US_CO_01",
            "US_GAAP",
            NaiveDate::from_ymd_opt(2026, 2, 21).unwrap(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn derives_period_id_from_date() {
        let period = period_id_from_date(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
        assert_eq!(period, "2026-12");
    }

    #[test]
    fn flush_persists_locked_periods_to_disk() {
        let temp_dir = TempDirGuard::new("period-flush");
        let mut repo = InMemoryPeriodRepository::with_persistence_dir(&temp_dir.path).unwrap();
        repo.lock_period("tenant_1", "US_CO_01", "US_GAAP", "2026-02")
            .unwrap();
        repo.flush_persistence().unwrap();

        let reloaded = InMemoryPeriodRepository::with_persistence_dir(&temp_dir.path).unwrap();
        let err = reloaded
            .ensure_open(
                "tenant_1",
                "US_CO_01",
                "US_GAAP",
                NaiveDate::from_ymd_opt(2026, 2, 21).unwrap(),
            )
            .unwrap_err();
        assert_eq!(err, PeriodError::PeriodClosed("2026-02".to_string()));
    }

    #[test]
    fn reloaded_period_store_preserves_multiple_locks() {
        let temp_dir = TempDirGuard::new("period-restart");
        {
            let mut repo = InMemoryPeriodRepository::with_persistence_dir(&temp_dir.path).unwrap();
            repo.lock_period("tenant_1", "US_CO_01", "US_GAAP", "2026-01")
                .unwrap();
            repo.lock_period("tenant_1", "CA_BC_01", "IFRS", "2026-02")
                .unwrap();
            repo.flush_persistence().unwrap();
        }

        let reloaded = InMemoryPeriodRepository::with_persistence_dir(&temp_dir.path).unwrap();
        let us_err = reloaded
            .ensure_open(
                "tenant_1",
                "US_CO_01",
                "US_GAAP",
                NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            )
            .unwrap_err();
        assert_eq!(us_err, PeriodError::PeriodClosed("2026-01".to_string()));
        let ca_err = reloaded
            .ensure_open(
                "tenant_1",
                "CA_BC_01",
                "IFRS",
                NaiveDate::from_ymd_opt(2026, 2, 2).unwrap(),
            )
            .unwrap_err();
        assert_eq!(ca_err, PeriodError::PeriodClosed("2026-02".to_string()));
    }
}
