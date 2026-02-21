use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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
}

#[derive(Default)]
pub struct InMemoryJournalRepository {
    journals: HashMap<Uuid, JournalRecord>,
}

impl InMemoryJournalRepository {
    pub fn insert_posted(&mut self, record: JournalRecord) -> Result<(), LedgerError> {
        if self.journals.contains_key(&record.header.journal_id) {
            return Err(LedgerError::JournalExists);
        }
        validate_balanced(&record.lines)?;
        self.journals.insert(record.header.journal_id, record);
        Ok(())
    }

    pub fn get(&self, journal_id: &Uuid) -> Option<&JournalRecord> {
        self.journals.get(journal_id)
    }

    pub fn update_posted(
        &mut self,
        _journal_id: &Uuid,
        _record: JournalRecord,
    ) -> Result<(), LedgerError> {
        Err(LedgerError::Immutable)
    }

    pub fn reverse(&mut self, journal_id: &Uuid) -> Result<(), LedgerError> {
        let record = self
            .journals
            .get_mut(journal_id)
            .ok_or(LedgerError::NotFound)?;
        record.header.status = JournalStatus::Reversed;
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
}
