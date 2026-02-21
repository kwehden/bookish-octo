use std::collections::HashSet;

use chrono::{Datelike, NaiveDate};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Default)]
pub struct InMemoryPeriodRepository {
    closed: HashSet<PeriodKey>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PeriodError {
    #[error("invalid period id `{0}`")]
    InvalidPeriodId(String),
    #[error("period is closed: {0}")]
    PeriodClosed(String),
}

impl InMemoryPeriodRepository {
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

    use super::{period_id_from_date, InMemoryPeriodRepository, PeriodError};

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
}
