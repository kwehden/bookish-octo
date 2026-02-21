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
pub enum ReconReasonCode {
    AmountMismatch,
    CurrencyMismatch,
    MissingGatewayReference,
    MissingBankReference,
    DuplicateCandidate,
    PartialAllocationRequired,
    ToleranceMatchReview,
    HighRiskInvestigate,
    Unclassified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconRoutingDecision {
    pub reason_code: ReconReasonCode,
    pub owner_queue: String,
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

pub fn route_reason_code(
    exception_type: &str,
    outcome: MatchOutcome,
    severity: &str,
) -> ReconReasonCode {
    if outcome == MatchOutcome::Duplicate {
        return ReconReasonCode::DuplicateCandidate;
    }
    if outcome == MatchOutcome::MatchedTolerance {
        return ReconReasonCode::ToleranceMatchReview;
    }
    if outcome == MatchOutcome::PartialMatch {
        return ReconReasonCode::PartialAllocationRequired;
    }

    let normalized_exception_type = normalize(exception_type);
    let normalized_severity = normalize(severity);

    if outcome == MatchOutcome::Investigate
        && (normalized_severity == "HIGH" || normalized_severity == "CRITICAL")
    {
        return ReconReasonCode::HighRiskInvestigate;
    }
    if normalized_exception_type.contains("AMOUNT") {
        return ReconReasonCode::AmountMismatch;
    }
    if normalized_exception_type.contains("CURRENCY") || normalized_exception_type.contains("FX") {
        return ReconReasonCode::CurrencyMismatch;
    }
    if normalized_exception_type.contains("BANK") {
        return ReconReasonCode::MissingBankReference;
    }
    if normalized_exception_type.contains("GATEWAY") || normalized_exception_type.contains("STRIPE")
    {
        return ReconReasonCode::MissingGatewayReference;
    }

    ReconReasonCode::Unclassified
}

pub fn route_owner_queue(reason_code: ReconReasonCode) -> &'static str {
    match reason_code {
        ReconReasonCode::AmountMismatch => "PAYMENTS_OPS",
        ReconReasonCode::CurrencyMismatch => "PAYMENTS_OPS",
        ReconReasonCode::MissingGatewayReference => "PAYMENTS_OPS",
        ReconReasonCode::MissingBankReference => "TREASURY_OPS",
        ReconReasonCode::DuplicateCandidate => "DATA_QUALITY",
        ReconReasonCode::PartialAllocationRequired => "PAYMENTS_OPS",
        ReconReasonCode::ToleranceMatchReview => "AUTO_CLEAR_REVIEW",
        ReconReasonCode::HighRiskInvestigate => "RISK_CONTROL",
        ReconReasonCode::Unclassified => "RECON_ANALYST",
    }
}

pub fn route_exception(input: &ReconException, outcome: MatchOutcome) -> ReconRoutingDecision {
    let reason_code = route_reason_code(&input.exception_type, outcome, &input.severity);
    let owner_queue = route_owner_queue(reason_code.clone()).to_string();

    ReconRoutingDecision {
        reason_code,
        owner_queue,
    }
}

fn normalize(input: &str) -> String {
    input
        .trim()
        .to_ascii_uppercase()
        .replace(' ', "_")
        .replace('-', "_")
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

    fn sample_exception(exception_type: &str, severity: &str) -> ReconException {
        ReconException {
            exception_id: "e1".into(),
            exception_type: exception_type.into(),
            severity: severity.into(),
            opened_at: Utc::now(),
            owner: "recon_ops".into(),
            sla_due_at: Utc::now(),
            resolution_code: None,
        }
    }

    #[test]
    fn duplicate_outcome_takes_precedence_over_exception_type() {
        let ex = sample_exception("AMOUNT_MISMATCH", "LOW");
        let route = route_exception(&ex, MatchOutcome::Duplicate);

        assert_eq!(route.reason_code, ReconReasonCode::DuplicateCandidate);
        assert_eq!(route.owner_queue, "DATA_QUALITY");
    }

    #[test]
    fn routes_case_insensitive_bank_reference_exceptions() {
        let ex = sample_exception("bank reference missing", "MEDIUM");
        let route = route_exception(&ex, MatchOutcome::Unmatched);

        assert_eq!(route.reason_code, ReconReasonCode::MissingBankReference);
        assert_eq!(route.owner_queue, "TREASURY_OPS");
    }

    #[test]
    fn high_risk_investigate_routes_to_risk_control() {
        let ex = sample_exception("investigate_manual", "CRITICAL");
        let route = route_exception(&ex, MatchOutcome::Investigate);

        assert_eq!(route.reason_code, ReconReasonCode::HighRiskInvestigate);
        assert_eq!(route.owner_queue, "RISK_CONTROL");
    }

    #[test]
    fn tolerance_match_routes_to_auto_clear_review() {
        let ex = sample_exception("amount mismatch", "LOW");
        let route = route_exception(&ex, MatchOutcome::MatchedTolerance);

        assert_eq!(route.reason_code, ReconReasonCode::ToleranceMatchReview);
        assert_eq!(route.owner_queue, "AUTO_CLEAR_REVIEW");
    }

    #[test]
    fn unknown_route_defaults_to_recon_analyst() {
        let ex = sample_exception("vendor_note", "LOW");
        let route = route_exception(&ex, MatchOutcome::Unmatched);

        assert_eq!(route.reason_code, ReconReasonCode::Unclassified);
        assert_eq!(route.owner_queue, "RECON_ANALYST");
    }
}
