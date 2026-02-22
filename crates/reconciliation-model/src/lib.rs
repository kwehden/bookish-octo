use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_MATCH_TOLERANCE_MINOR: i64 = 100;
const ONE_HUNDRED_PERCENT_BPS: u32 = 10_000;
const MIN_DRY_RUN_ENTITY_COUNT: usize = 2;
const MAX_DRY_RUN_ENTITY_COUNT: usize = 3;

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
pub struct ReconOrder {
    pub order_id: String,
    pub payment_id: String,
    pub payout_id: String,
    pub currency: String,
    pub amount_minor: i64,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconPayment {
    pub payment_id: String,
    pub order_id: String,
    pub payout_id: String,
    pub currency: String,
    pub amount_minor: i64,
    pub settled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconPayout {
    pub payout_id: String,
    pub payment_id: String,
    pub bank_reference: String,
    pub currency: String,
    pub amount_minor: i64,
    pub settled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconRunInput {
    pub run_id: String,
    pub run_started_at: DateTime<Utc>,
    pub orders: Vec<ReconOrder>,
    pub payments: Vec<ReconPayment>,
    pub payouts: Vec<ReconPayout>,
    pub tolerance_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconMatchRecord {
    pub order_id: String,
    pub expected_payment_id: String,
    pub expected_payout_id: String,
    pub matched_payment_id: Option<String>,
    pub matched_payout_id: Option<String>,
    pub outcome: MatchOutcome,
    pub reason_code: Option<ReconReasonCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconExceptionQueueItem {
    pub exception_id: String,
    pub order_id: String,
    pub payment_id: String,
    pub payout_id: String,
    pub reason_code: ReconReasonCode,
    pub owner_queue: String,
    pub opened_at: DateTime<Utc>,
    pub sla_due_at: DateTime<Utc>,
    pub outcome: MatchOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconRunMetrics {
    pub total_candidates: u32,
    pub auto_matched: u32,
    pub non_auto_candidates: u32,
    pub routed_exceptions: u32,
    pub auto_match_rate_bps: u32,
    pub routed_exception_rate_bps: u32,
}

impl ReconRunMetrics {
    pub fn auto_match_rate_percent(&self) -> f64 {
        self.auto_match_rate_bps as f64 / 100.0
    }

    pub fn routed_exception_rate_percent(&self) -> f64 {
        self.routed_exception_rate_bps as f64 / 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconRunResult {
    pub run_id: String,
    pub run_started_at: DateTime<Utc>,
    pub matches: Vec<ReconMatchRecord>,
    pub exception_queue: Vec<ReconExceptionQueueItem>,
    pub metrics: ReconRunMetrics,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CloseDependencyStatus {
    Pending,
    InProgress,
    Satisfied,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloseChecklistDependency {
    pub dependency_id: String,
    pub description: String,
    pub required_for_close: bool,
    pub status: CloseDependencyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CloseChecklistStatus {
    InProgress,
    Blocked,
    ReadyToClose,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityCloseChecklist {
    pub checklist_id: String,
    pub legal_entity_id: String,
    pub period_id: String,
    pub status: CloseChecklistStatus,
    pub dependencies: Vec<CloseChecklistDependency>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloseChecklistProgression {
    pub checklist_id: String,
    pub legal_entity_id: String,
    pub period_id: String,
    pub status: CloseChecklistStatus,
    pub can_progress: bool,
    pub unresolved_blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloseChecklistActorContext {
    pub actor_id: String,
    pub actor_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiEntityCloseDryRunInput {
    pub run_id: String,
    pub run_started_at: DateTime<Utc>,
    pub checklists: Vec<EntityCloseChecklist>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityCloseDryRunResult {
    pub checklist_id: String,
    pub legal_entity_id: String,
    pub status: CloseChecklistStatus,
    pub can_progress: bool,
    pub close_ready: bool,
    pub unresolved_blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiEntityCloseDryRunResult {
    pub run_id: String,
    pub run_started_at: DateTime<Utc>,
    pub entity_results: Vec<EntityCloseDryRunResult>,
    pub passed: bool,
    pub failed_entities: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CloseChecklistError {
    #[error("dependency `{dependency_id}` not found in checklist")]
    DependencyNotFound { dependency_id: String },
    #[error("invalid dependency transition from `{from:?}` to `{to:?}`")]
    InvalidDependencyTransition {
        from: CloseDependencyStatus,
        to: CloseDependencyStatus,
    },
    #[error("multi-entity close dry run requires 2-3 entities, got {entity_count}")]
    UnsupportedEntityCount { entity_count: usize },
}

pub fn evaluate_entity_close_checklist(
    checklist: &EntityCloseChecklist,
) -> CloseChecklistProgression {
    let unresolved_blockers = checklist
        .dependencies
        .iter()
        .filter(|dependency| dependency.status == CloseDependencyStatus::Blocked)
        .map(|dependency| dependency.dependency_id.clone())
        .collect::<Vec<_>>();
    let can_progress = unresolved_blockers.is_empty();
    let status = derive_close_checklist_status(checklist, can_progress);

    CloseChecklistProgression {
        checklist_id: checklist.checklist_id.clone(),
        legal_entity_id: checklist.legal_entity_id.clone(),
        period_id: checklist.period_id.clone(),
        status,
        can_progress,
        unresolved_blockers,
    }
}

pub fn evaluate_entity_close_checklist_for_actor(
    checklist: &EntityCloseChecklist,
    _actor: &CloseChecklistActorContext,
) -> CloseChecklistProgression {
    evaluate_entity_close_checklist(checklist)
}

pub fn transition_close_dependency_status(
    checklist: &EntityCloseChecklist,
    dependency_id: &str,
    next_status: CloseDependencyStatus,
    updated_at: DateTime<Utc>,
) -> Result<EntityCloseChecklist, CloseChecklistError> {
    let mut updated = checklist.clone();
    let dependency = updated
        .dependencies
        .iter_mut()
        .find(|dependency| dependency.dependency_id == dependency_id)
        .ok_or_else(|| CloseChecklistError::DependencyNotFound {
            dependency_id: dependency_id.to_string(),
        })?;

    if !is_valid_close_dependency_transition(&dependency.status, &next_status) {
        return Err(CloseChecklistError::InvalidDependencyTransition {
            from: dependency.status.clone(),
            to: next_status,
        });
    }

    dependency.status = next_status;
    updated.updated_at = updated_at;
    let progression = evaluate_entity_close_checklist(&updated);
    updated.status = progression.status;

    Ok(updated)
}

pub fn simulate_multi_entity_close_dry_run(
    input: &MultiEntityCloseDryRunInput,
) -> Result<MultiEntityCloseDryRunResult, CloseChecklistError> {
    simulate_multi_entity_close_dry_run_internal(input, None)
}

pub fn simulate_multi_entity_close_dry_run_for_actor(
    input: &MultiEntityCloseDryRunInput,
    actor: &CloseChecklistActorContext,
) -> Result<MultiEntityCloseDryRunResult, CloseChecklistError> {
    simulate_multi_entity_close_dry_run_internal(input, Some(actor))
}

pub fn reconcile_v1(input: &ReconRunInput) -> ReconRunResult {
    let mut sorted_orders = input.orders.clone();
    sorted_orders.sort_by(|left, right| left.order_id.cmp(&right.order_id));

    let payment_index = build_payment_index(&input.payments);
    let payout_index = build_payout_index(&input.payouts);
    let tolerance_minor = if input.tolerance_minor < 0 {
        DEFAULT_MATCH_TOLERANCE_MINOR
    } else {
        input.tolerance_minor
    };
    let normalized_run_id = normalize_run_id(&input.run_id);

    let mut matches = Vec::with_capacity(sorted_orders.len());
    let mut exception_queue = Vec::new();
    let mut auto_matched = 0u32;
    let mut exception_sequence = 1u32;

    for order in &sorted_orders {
        let mut outcome = MatchOutcome::MatchedExact;
        let mut reason_code = None;
        let mut matched_payment = None;
        let mut matched_payout = None;

        let payment_candidates = payment_index
            .get(&order.payment_id)
            .cloned()
            .unwrap_or_default();
        match payment_candidates.as_slice() {
            [] => {
                outcome = MatchOutcome::Unmatched;
                reason_code = Some(ReconReasonCode::MissingGatewayReference);
            }
            [payment] => {
                matched_payment = Some(payment.clone());
            }
            _ => {
                outcome = MatchOutcome::Duplicate;
                reason_code = Some(ReconReasonCode::DuplicateCandidate);
            }
        }

        let payout_candidates = payout_index
            .get(&order.payout_id)
            .cloned()
            .unwrap_or_default();
        if reason_code.is_none() {
            match payout_candidates.as_slice() {
                [] => {
                    outcome = MatchOutcome::Unmatched;
                    reason_code = Some(ReconReasonCode::MissingBankReference);
                }
                [payout] => {
                    matched_payout = Some(payout.clone());
                }
                _ => {
                    outcome = MatchOutcome::Duplicate;
                    reason_code = Some(ReconReasonCode::DuplicateCandidate);
                }
            }
        }

        if reason_code.is_none() {
            let payment = matched_payment
                .as_ref()
                .expect("payment must exist when reason_code is none");
            let payout = matched_payout
                .as_ref()
                .expect("payout must exist when reason_code is none");
            let order_currency = normalize_currency(&order.currency);
            let payment_currency = normalize_currency(&payment.currency);
            let payout_currency = normalize_currency(&payout.currency);

            if payment.order_id != order.order_id
                || payment.payout_id != order.payout_id
                || payout.payment_id != payment.payment_id
            {
                outcome = MatchOutcome::PartialMatch;
                reason_code = Some(ReconReasonCode::PartialAllocationRequired);
            } else if order_currency != payment_currency || payment_currency != payout_currency {
                outcome = MatchOutcome::Unmatched;
                reason_code = Some(ReconReasonCode::CurrencyMismatch);
            } else {
                let delta_order_payment = (order.amount_minor - payment.amount_minor).abs();
                let delta_payment_payout = (payment.amount_minor - payout.amount_minor).abs();

                if delta_order_payment == 0 && delta_payment_payout == 0 {
                    outcome = MatchOutcome::MatchedExact;
                    reason_code = None;
                } else if delta_order_payment <= tolerance_minor
                    && delta_payment_payout <= tolerance_minor
                {
                    outcome = MatchOutcome::MatchedTolerance;
                    reason_code = None;
                } else if delta_order_payment <= tolerance_minor
                    || delta_payment_payout <= tolerance_minor
                {
                    outcome = MatchOutcome::PartialMatch;
                    reason_code = Some(ReconReasonCode::PartialAllocationRequired);
                } else {
                    outcome = MatchOutcome::Unmatched;
                    reason_code = Some(ReconReasonCode::AmountMismatch);
                }
            }
        }

        if is_auto_match(&outcome) {
            auto_matched += 1;
        }

        let reason_for_match = reason_code.clone();
        matches.push(ReconMatchRecord {
            order_id: order.order_id.clone(),
            expected_payment_id: order.payment_id.clone(),
            expected_payout_id: order.payout_id.clone(),
            matched_payment_id: matched_payment.as_ref().map(|item| item.payment_id.clone()),
            matched_payout_id: matched_payout.as_ref().map(|item| item.payout_id.clone()),
            outcome: outcome.clone(),
            reason_code: reason_for_match,
        });

        if let Some(reason_code) = reason_code {
            let owner_queue = route_owner_queue(reason_code.clone()).to_string();
            let sla_due_at = input.run_started_at + sla_offset(reason_code.clone());
            let exception_id = format!("{normalized_run_id}-EX-{exception_sequence:04}");
            exception_sequence += 1;

            exception_queue.push(ReconExceptionQueueItem {
                exception_id,
                order_id: order.order_id.clone(),
                payment_id: order.payment_id.clone(),
                payout_id: order.payout_id.clone(),
                reason_code,
                owner_queue,
                opened_at: input.run_started_at,
                sla_due_at,
                outcome: outcome.clone(),
            });
        }
    }

    let total_candidates = matches.len() as u32;
    let non_auto_candidates = total_candidates.saturating_sub(auto_matched);
    let routed_exceptions = exception_queue.len() as u32;

    ReconRunResult {
        run_id: input.run_id.clone(),
        run_started_at: input.run_started_at,
        matches,
        exception_queue,
        metrics: ReconRunMetrics {
            total_candidates,
            auto_matched,
            non_auto_candidates,
            routed_exceptions,
            auto_match_rate_bps: ratio_to_bps(auto_matched, total_candidates),
            routed_exception_rate_bps: ratio_to_bps(routed_exceptions, non_auto_candidates),
        },
    }
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

fn simulate_multi_entity_close_dry_run_internal(
    input: &MultiEntityCloseDryRunInput,
    actor: Option<&CloseChecklistActorContext>,
) -> Result<MultiEntityCloseDryRunResult, CloseChecklistError> {
    let entity_count = input.checklists.len();
    if !(MIN_DRY_RUN_ENTITY_COUNT..=MAX_DRY_RUN_ENTITY_COUNT).contains(&entity_count) {
        return Err(CloseChecklistError::UnsupportedEntityCount { entity_count });
    }

    let mut sorted_checklists = input.checklists.clone();
    sorted_checklists.sort_by(|left, right| {
        left.legal_entity_id
            .cmp(&right.legal_entity_id)
            .then(left.checklist_id.cmp(&right.checklist_id))
    });

    let mut entity_results = Vec::with_capacity(sorted_checklists.len());
    let mut failed_entities = Vec::new();
    for checklist in sorted_checklists {
        let progression = if let Some(actor) = actor {
            evaluate_entity_close_checklist_for_actor(&checklist, actor)
        } else {
            evaluate_entity_close_checklist(&checklist)
        };
        let close_ready = progression.status == CloseChecklistStatus::ReadyToClose
            || progression.status == CloseChecklistStatus::Closed;
        if !progression.can_progress || !close_ready {
            failed_entities.push(checklist.legal_entity_id.clone());
        }

        entity_results.push(EntityCloseDryRunResult {
            checklist_id: checklist.checklist_id,
            legal_entity_id: checklist.legal_entity_id,
            status: progression.status,
            can_progress: progression.can_progress,
            close_ready,
            unresolved_blockers: progression.unresolved_blockers,
        });
    }

    Ok(MultiEntityCloseDryRunResult {
        run_id: input.run_id.clone(),
        run_started_at: input.run_started_at,
        entity_results,
        passed: failed_entities.is_empty(),
        failed_entities,
    })
}

fn derive_close_checklist_status(
    checklist: &EntityCloseChecklist,
    can_progress: bool,
) -> CloseChecklistStatus {
    if !can_progress {
        return CloseChecklistStatus::Blocked;
    }
    if checklist.status == CloseChecklistStatus::Closed {
        return CloseChecklistStatus::Closed;
    }

    let ready_to_close = checklist
        .dependencies
        .iter()
        .filter(|dependency| dependency.required_for_close)
        .all(|dependency| dependency.status == CloseDependencyStatus::Satisfied);
    if ready_to_close {
        CloseChecklistStatus::ReadyToClose
    } else {
        CloseChecklistStatus::InProgress
    }
}

fn is_valid_close_dependency_transition(
    from: &CloseDependencyStatus,
    to: &CloseDependencyStatus,
) -> bool {
    if from == to {
        return true;
    }

    matches!(
        (from, to),
        (
            CloseDependencyStatus::Pending,
            CloseDependencyStatus::InProgress
        ) | (
            CloseDependencyStatus::Pending,
            CloseDependencyStatus::Satisfied
        ) | (
            CloseDependencyStatus::Pending,
            CloseDependencyStatus::Blocked
        ) | (
            CloseDependencyStatus::InProgress,
            CloseDependencyStatus::Satisfied
        ) | (
            CloseDependencyStatus::InProgress,
            CloseDependencyStatus::Blocked
        ) | (
            CloseDependencyStatus::Blocked,
            CloseDependencyStatus::InProgress
        ) | (
            CloseDependencyStatus::Blocked,
            CloseDependencyStatus::Satisfied
        )
    )
}

fn normalize(input: &str) -> String {
    input
        .trim()
        .to_ascii_uppercase()
        .replace(' ', "_")
        .replace('-', "_")
}

fn normalize_currency(input: &str) -> String {
    input.trim().to_ascii_uppercase()
}

fn normalize_run_id(input: &str) -> String {
    let normalized = normalize(input);
    if normalized.is_empty() {
        "RECON_RUN".to_string()
    } else {
        normalized
    }
}

fn ratio_to_bps(numerator: u32, denominator: u32) -> u32 {
    if denominator == 0 {
        ONE_HUNDRED_PERCENT_BPS
    } else {
        ((numerator as u64 * ONE_HUNDRED_PERCENT_BPS as u64) / denominator as u64) as u32
    }
}

fn is_auto_match(outcome: &MatchOutcome) -> bool {
    *outcome == MatchOutcome::MatchedExact || *outcome == MatchOutcome::MatchedTolerance
}

fn sla_offset(reason_code: ReconReasonCode) -> Duration {
    match reason_code {
        ReconReasonCode::HighRiskInvestigate => Duration::hours(2),
        ReconReasonCode::AmountMismatch => Duration::hours(4),
        ReconReasonCode::CurrencyMismatch => Duration::hours(4),
        ReconReasonCode::PartialAllocationRequired => Duration::hours(4),
        ReconReasonCode::MissingGatewayReference => Duration::hours(8),
        ReconReasonCode::MissingBankReference => Duration::hours(8),
        ReconReasonCode::ToleranceMatchReview => Duration::hours(12),
        ReconReasonCode::DuplicateCandidate => Duration::hours(24),
        ReconReasonCode::Unclassified => Duration::hours(24),
    }
}

fn build_payment_index(payments: &[ReconPayment]) -> BTreeMap<String, Vec<ReconPayment>> {
    let mut index: BTreeMap<String, Vec<ReconPayment>> = BTreeMap::new();
    for payment in payments {
        index
            .entry(payment.payment_id.clone())
            .or_default()
            .push(payment.clone());
    }
    for candidates in index.values_mut() {
        candidates.sort_by(|left, right| {
            left.order_id
                .cmp(&right.order_id)
                .then(left.payout_id.cmp(&right.payout_id))
                .then(left.amount_minor.cmp(&right.amount_minor))
                .then(left.currency.cmp(&right.currency))
        });
    }
    index
}

fn build_payout_index(payouts: &[ReconPayout]) -> BTreeMap<String, Vec<ReconPayout>> {
    let mut index: BTreeMap<String, Vec<ReconPayout>> = BTreeMap::new();
    for payout in payouts {
        index
            .entry(payout.payout_id.clone())
            .or_default()
            .push(payout.clone());
    }
    for candidates in index.values_mut() {
        candidates.sort_by(|left, right| {
            left.payment_id
                .cmp(&right.payment_id)
                .then(left.amount_minor.cmp(&right.amount_minor))
                .then(left.currency.cmp(&right.currency))
                .then(left.bank_reference.cmp(&right.bank_reference))
        });
    }
    index
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

    #[test]
    fn reconcile_v1_is_deterministic_for_seeded_fixture() {
        let fixture = seeded_fixture();
        let first = reconcile_v1(&fixture);
        let second = reconcile_v1(&fixture);

        assert_eq!(first, second);
    }

    #[test]
    fn reconcile_v1_routes_seeded_mismatches_to_exception_queue() {
        let fixture = seeded_fixture();
        let result = reconcile_v1(&fixture);

        assert_eq!(result.exception_queue.len(), 3);
        assert_eq!(result.metrics.routed_exception_rate_bps, 10_000);

        let currency = result
            .exception_queue
            .iter()
            .find(|item| item.order_id == "O-009")
            .expect("currency mismatch should be present");
        assert_eq!(currency.reason_code, ReconReasonCode::CurrencyMismatch);
        assert_eq!(currency.owner_queue, "PAYMENTS_OPS");
        assert_eq!(
            currency.sla_due_at,
            fixture.run_started_at + Duration::hours(4)
        );

        let missing_payout = result
            .exception_queue
            .iter()
            .find(|item| item.order_id == "O-010")
            .expect("missing payout should be present");
        assert_eq!(
            missing_payout.reason_code,
            ReconReasonCode::MissingBankReference
        );
        assert_eq!(missing_payout.owner_queue, "TREASURY_OPS");
        assert_eq!(
            missing_payout.sla_due_at,
            fixture.run_started_at + Duration::hours(8)
        );

        let duplicate = result
            .exception_queue
            .iter()
            .find(|item| item.order_id == "O-011")
            .expect("duplicate should be present");
        assert_eq!(duplicate.reason_code, ReconReasonCode::DuplicateCandidate);
        assert_eq!(duplicate.owner_queue, "DATA_QUALITY");
        assert_eq!(
            duplicate.sla_due_at,
            fixture.run_started_at + Duration::hours(24)
        );
    }

    #[test]
    fn reconcile_v1_fixture_auto_match_rate_meets_gate() {
        let fixture = seeded_fixture();
        let result = reconcile_v1(&fixture);

        assert_eq!(result.metrics.total_candidates, 11);
        assert_eq!(result.metrics.auto_matched, 8);
        assert!(result.metrics.auto_match_rate_percent() >= 70.0);
    }

    #[test]
    fn dependency_state_transitions_promote_entity_to_ready_to_close() {
        let base = sample_close_checklist(
            "LE-US",
            vec![("bank_stmt_reconciled", CloseDependencyStatus::Pending, true)],
            CloseChecklistStatus::InProgress,
        );
        let updated_at = fixed_ts() + Duration::minutes(15);

        let in_progress = transition_close_dependency_status(
            &base,
            "bank_stmt_reconciled",
            CloseDependencyStatus::InProgress,
            updated_at,
        )
        .expect("pending to in-progress transition should be valid");
        assert_eq!(in_progress.status, CloseChecklistStatus::InProgress);

        let ready = transition_close_dependency_status(
            &in_progress,
            "bank_stmt_reconciled",
            CloseDependencyStatus::Satisfied,
            updated_at + Duration::minutes(15),
        )
        .expect("in-progress to satisfied transition should be valid");
        assert_eq!(ready.status, CloseChecklistStatus::ReadyToClose);
    }

    #[test]
    fn dependency_state_transition_rejects_regression_from_satisfied() {
        let checklist = sample_close_checklist(
            "LE-US",
            vec![(
                "bank_stmt_reconciled",
                CloseDependencyStatus::Satisfied,
                true,
            )],
            CloseChecklistStatus::ReadyToClose,
        );

        let result = transition_close_dependency_status(
            &checklist,
            "bank_stmt_reconciled",
            CloseDependencyStatus::Blocked,
            fixed_ts() + Duration::minutes(10),
        );
        assert_eq!(
            result,
            Err(CloseChecklistError::InvalidDependencyTransition {
                from: CloseDependencyStatus::Satisfied,
                to: CloseDependencyStatus::Blocked,
            })
        );
    }

    #[test]
    fn unresolved_blockers_block_close_progression() {
        let checklist = sample_close_checklist(
            "LE-CA",
            vec![
                (
                    "intercompany_eliminations_posted",
                    CloseDependencyStatus::Blocked,
                    true,
                ),
                (
                    "fx_translation_complete",
                    CloseDependencyStatus::Satisfied,
                    true,
                ),
            ],
            CloseChecklistStatus::InProgress,
        );

        let progression = evaluate_entity_close_checklist(&checklist);
        assert!(!progression.can_progress);
        assert_eq!(progression.status, CloseChecklistStatus::Blocked);
        assert_eq!(
            progression.unresolved_blockers,
            vec!["intercompany_eliminations_posted".to_string()]
        );
    }

    #[test]
    fn checklist_evaluation_is_authorization_neutral() {
        let checklist = sample_close_checklist(
            "LE-US",
            vec![
                (
                    "bank_stmt_reconciled",
                    CloseDependencyStatus::Satisfied,
                    true,
                ),
                (
                    "fx_translation_complete",
                    CloseDependencyStatus::Satisfied,
                    true,
                ),
            ],
            CloseChecklistStatus::InProgress,
        );
        let finance_actor = CloseChecklistActorContext {
            actor_id: "u-finance".to_string(),
            actor_role: "FINANCE_MANAGER".to_string(),
        };
        let qa_actor = CloseChecklistActorContext {
            actor_id: "u-qa".to_string(),
            actor_role: "QA_RELEASE".to_string(),
        };

        let finance_result = evaluate_entity_close_checklist_for_actor(&checklist, &finance_actor);
        let qa_result = evaluate_entity_close_checklist_for_actor(&checklist, &qa_actor);

        assert_eq!(finance_result, qa_result);
        assert_eq!(finance_result.status, CloseChecklistStatus::ReadyToClose);
    }

    #[test]
    fn multi_entity_close_dry_run_passes_for_two_ready_entities() {
        let input = MultiEntityCloseDryRunInput {
            run_id: "sprint4-dry-run-pass".to_string(),
            run_started_at: fixed_ts(),
            checklists: vec![
                sample_close_checklist(
                    "LE-US",
                    vec![
                        (
                            "bank_stmt_reconciled",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                        (
                            "fx_translation_complete",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                    ],
                    CloseChecklistStatus::InProgress,
                ),
                sample_close_checklist(
                    "LE-CA",
                    vec![
                        (
                            "bank_stmt_reconciled",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                        (
                            "fx_translation_complete",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                    ],
                    CloseChecklistStatus::InProgress,
                ),
            ],
        };

        let result =
            simulate_multi_entity_close_dry_run(&input).expect("2-entity dry run should execute");
        assert!(result.passed);
        assert!(result.failed_entities.is_empty());
        assert_eq!(result.entity_results.len(), 2);
        assert!(result.entity_results.iter().all(|item| item.close_ready));
    }

    #[test]
    fn multi_entity_close_dry_run_fails_when_one_entity_has_blocker() {
        let input = MultiEntityCloseDryRunInput {
            run_id: "sprint4-dry-run-fail".to_string(),
            run_started_at: fixed_ts(),
            checklists: vec![
                sample_close_checklist(
                    "LE-US",
                    vec![
                        (
                            "bank_stmt_reconciled",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                        (
                            "fx_translation_complete",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                    ],
                    CloseChecklistStatus::InProgress,
                ),
                sample_close_checklist(
                    "LE-CA",
                    vec![
                        ("bank_stmt_reconciled", CloseDependencyStatus::Blocked, true),
                        (
                            "fx_translation_complete",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                    ],
                    CloseChecklistStatus::InProgress,
                ),
                sample_close_checklist(
                    "LE-HQ",
                    vec![
                        (
                            "bank_stmt_reconciled",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                        (
                            "fx_translation_complete",
                            CloseDependencyStatus::Satisfied,
                            true,
                        ),
                    ],
                    CloseChecklistStatus::InProgress,
                ),
            ],
        };
        let actor = CloseChecklistActorContext {
            actor_id: "u-controller".to_string(),
            actor_role: "CONTROLLER".to_string(),
        };

        let result = simulate_multi_entity_close_dry_run_for_actor(&input, &actor)
            .expect("3-entity dry run should execute");
        assert!(!result.passed);
        assert_eq!(result.failed_entities, vec!["LE-CA".to_string()]);
        assert!(result
            .entity_results
            .iter()
            .any(|item| item.legal_entity_id == "LE-CA"
                && !item.can_progress
                && !item.unresolved_blockers.is_empty()));
    }

    #[test]
    fn multi_entity_close_dry_run_requires_two_to_three_entities() {
        let input = MultiEntityCloseDryRunInput {
            run_id: "sprint4-dry-run-invalid".to_string(),
            run_started_at: fixed_ts(),
            checklists: vec![sample_close_checklist(
                "LE-US",
                vec![(
                    "bank_stmt_reconciled",
                    CloseDependencyStatus::Satisfied,
                    true,
                )],
                CloseChecklistStatus::ReadyToClose,
            )],
        };

        let result = simulate_multi_entity_close_dry_run(&input);
        assert_eq!(
            result,
            Err(CloseChecklistError::UnsupportedEntityCount { entity_count: 1 })
        );
    }

    fn sample_close_checklist(
        legal_entity_id: &str,
        dependency_specs: Vec<(&str, CloseDependencyStatus, bool)>,
        status: CloseChecklistStatus,
    ) -> EntityCloseChecklist {
        EntityCloseChecklist {
            checklist_id: format!("CHK-{legal_entity_id}"),
            legal_entity_id: legal_entity_id.to_string(),
            period_id: "2026-02".to_string(),
            status,
            dependencies: dependency_specs
                .into_iter()
                .map(|(dependency_id, dependency_status, required_for_close)| {
                    CloseChecklistDependency {
                        dependency_id: dependency_id.to_string(),
                        description: dependency_id.to_string(),
                        required_for_close,
                        status: dependency_status,
                    }
                })
                .collect(),
            updated_at: fixed_ts(),
        }
    }

    fn seeded_fixture() -> ReconRunInput {
        let run_started_at = fixed_ts();
        ReconRunInput {
            run_id: "sprint3_fixture".into(),
            run_started_at,
            tolerance_minor: DEFAULT_MATCH_TOLERANCE_MINOR,
            orders: vec![
                sample_order("O-001", "P-001", "PO-001", "USD", 10_000, run_started_at),
                sample_order("O-002", "P-002", "PO-002", "USD", 2_500, run_started_at),
                sample_order("O-003", "P-003", "PO-003", "USD", 3_000, run_started_at),
                sample_order("O-004", "P-004", "PO-004", "USD", 1_500, run_started_at),
                sample_order("O-005", "P-005", "PO-005", "USD", 6_000, run_started_at),
                sample_order("O-006", "P-006", "PO-006", "USD", 4_200, run_started_at),
                sample_order("O-007", "P-007", "PO-007", "USD", 3_300, run_started_at),
                sample_order("O-008", "P-008", "PO-008", "USD", 2_000, run_started_at),
                sample_order("O-009", "P-009", "PO-009", "USD", 5_000, run_started_at),
                sample_order("O-010", "P-010", "PO-010", "USD", 7_500, run_started_at),
                sample_order("O-011", "P-011", "PO-011", "USD", 9_000, run_started_at),
            ],
            payments: vec![
                sample_payment("P-001", "O-001", "PO-001", "USD", 10_000, run_started_at),
                sample_payment("P-002", "O-002", "PO-002", "USD", 2_500, run_started_at),
                sample_payment("P-003", "O-003", "PO-003", "USD", 3_000, run_started_at),
                sample_payment("P-004", "O-004", "PO-004", "USD", 1_500, run_started_at),
                sample_payment("P-005", "O-005", "PO-005", "USD", 6_000, run_started_at),
                sample_payment("P-006", "O-006", "PO-006", "USD", 4_200, run_started_at),
                sample_payment("P-007", "O-007", "PO-007", "USD", 3_300, run_started_at),
                sample_payment("P-008", "O-008", "PO-008", "USD", 2_080, run_started_at),
                sample_payment("P-009", "O-009", "PO-009", "CAD", 5_000, run_started_at),
                sample_payment("P-010", "O-010", "PO-010", "USD", 7_500, run_started_at),
                sample_payment("P-011", "O-011", "PO-011", "USD", 9_000, run_started_at),
                sample_payment("P-011", "O-011", "PO-011", "USD", 9_000, run_started_at),
            ],
            payouts: vec![
                sample_payout("PO-001", "P-001", "BANK-001", "USD", 10_000, run_started_at),
                sample_payout("PO-002", "P-002", "BANK-002", "USD", 2_500, run_started_at),
                sample_payout("PO-003", "P-003", "BANK-003", "USD", 3_000, run_started_at),
                sample_payout("PO-004", "P-004", "BANK-004", "USD", 1_500, run_started_at),
                sample_payout("PO-005", "P-005", "BANK-005", "USD", 6_000, run_started_at),
                sample_payout("PO-006", "P-006", "BANK-006", "USD", 4_200, run_started_at),
                sample_payout("PO-007", "P-007", "BANK-007", "USD", 3_300, run_started_at),
                sample_payout("PO-008", "P-008", "BANK-008", "USD", 2_080, run_started_at),
                sample_payout("PO-009", "P-009", "BANK-009", "CAD", 5_000, run_started_at),
                sample_payout("PO-011", "P-011", "BANK-011", "USD", 9_000, run_started_at),
            ],
        }
    }

    fn sample_order(
        order_id: &str,
        payment_id: &str,
        payout_id: &str,
        currency: &str,
        amount_minor: i64,
        at: DateTime<Utc>,
    ) -> ReconOrder {
        ReconOrder {
            order_id: order_id.into(),
            payment_id: payment_id.into(),
            payout_id: payout_id.into(),
            currency: currency.into(),
            amount_minor,
            captured_at: at,
        }
    }

    fn sample_payment(
        payment_id: &str,
        order_id: &str,
        payout_id: &str,
        currency: &str,
        amount_minor: i64,
        at: DateTime<Utc>,
    ) -> ReconPayment {
        ReconPayment {
            payment_id: payment_id.into(),
            order_id: order_id.into(),
            payout_id: payout_id.into(),
            currency: currency.into(),
            amount_minor,
            settled_at: at,
        }
    }

    fn sample_payout(
        payout_id: &str,
        payment_id: &str,
        bank_reference: &str,
        currency: &str,
        amount_minor: i64,
        at: DateTime<Utc>,
    ) -> ReconPayout {
        ReconPayout {
            payout_id: payout_id.into(),
            payment_id: payment_id.into(),
            bank_reference: bank_reference.into(),
            currency: currency.into(),
            amount_minor,
            settled_at: at,
        }
    }

    fn fixed_ts() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-02-21T00:00:00Z")
            .expect("fixture timestamp must parse")
            .with_timezone(&Utc)
    }
}
