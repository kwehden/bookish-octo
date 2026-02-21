use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchOutcome {
    MatchedExact,
    MatchedTolerance,
    PartialMatch,
    Unmatched,
    Duplicate,
    Investigate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconException {
    pub exception_id: String,
    pub exception_type: String,
    pub severity: String,
    pub opened_at: DateTime<Utc>,
    pub owner: String,
    pub sla_due_at: DateTime<Utc>,
    pub resolution_code: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReconValidationError {
    #[error("exception owner is required")]
    MissingOwner,
    #[error("severity is required")]
    MissingSeverity,
}

pub fn validate_exception(input: &ReconException) -> Result<(), ReconValidationError> {
    if input.owner.trim().is_empty() {
        return Err(ReconValidationError::MissingOwner);
    }
    if input.severity.trim().is_empty() {
        return Err(ReconValidationError::MissingSeverity);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_is_required() {
        let ex = ReconException {
            exception_id: "e1".into(),
            exception_type: "AMOUNT_MISMATCH".into(),
            severity: "HIGH".into(),
            opened_at: Utc::now(),
            owner: " ".into(),
            sla_due_at: Utc::now(),
            resolution_code: None,
        };

        let result = validate_exception(&ex);
        assert_eq!(result, Err(ReconValidationError::MissingOwner));
    }
}
