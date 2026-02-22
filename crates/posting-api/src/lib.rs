use std::collections::{HashMap, HashSet};
use std::path::Path as FsPath;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use ledger_posting::{
    EntrySide, InMemoryJournalRepository, JournalHeader, JournalLine, JournalRecord, JournalStatus,
    LedgerError,
};
use platform_core::{
    evaluate_no_bend_readiness, payload_hash, AuditSealError, IdempotencyError, IdempotencyStatus,
    InMemoryAuditSealStore, InMemoryIdempotencyStore, NoBendReadiness, ScaleSample,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::period::{InMemoryPeriodRepository, PeriodError};
use crate::rule_engine::{derive_lines_v1, RuleEngineError};

pub mod period;
pub mod rule_engine;

type ApiError = (StatusCode, serde_json::Value);
const CAPACITY_TARGET_ACTIVE_USERS: u32 = 2000;
const CAPACITY_LINEARITY_RATIO_MIN: f64 = 0.80;
const CAPACITY_BURST_RPS: u32 = 500;

#[derive(Debug, Clone)]
struct LegalHoldRule {
    hold_id: String,
    tenant_id: String,
    legal_entity_id: String,
    ledger_book: String,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    reason: String,
    retention_days: u32,
}

impl LegalHoldRule {
    fn applies_to(&self, accounting_date: NaiveDate) -> bool {
        let starts = accounting_date >= self.start_date;
        let ends = self
            .end_date
            .map(|end_date| accounting_date <= end_date)
            .unwrap_or(true);
        starts && ends
    }
}

#[derive(Clone)]
pub struct AppState {
    idempotency: InMemoryIdempotencyStore,
    journals: Arc<Mutex<InMemoryJournalRepository>>,
    periods: Arc<Mutex<InMemoryPeriodRepository>>,
    post_results: Arc<Mutex<HashMap<String, CachedPostResult>>>,
    location_allowlist_by_legal_entity: Arc<HashMap<String, HashSet<String>>>,
    audit_seals: InMemoryAuditSealStore,
    legal_holds: Arc<Mutex<HashMap<String, LegalHoldRule>>>,
}

#[derive(Clone)]
enum CachedPostResult {
    Success(PostEventResponse),
    Failure {
        status: StatusCode,
        body: serde_json::Value,
    },
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            idempotency: InMemoryIdempotencyStore::default(),
            journals: Arc::new(Mutex::new(InMemoryJournalRepository::default())),
            periods: Arc::new(Mutex::new(InMemoryPeriodRepository::default())),
            post_results: Arc::new(Mutex::new(HashMap::new())),
            location_allowlist_by_legal_entity: Arc::new(default_location_allowlist()),
            audit_seals: InMemoryAuditSealStore::default(),
            legal_holds: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn default_location_allowlist() -> HashMap<String, HashSet<String>> {
    HashMap::from([
        (
            "US_CO_01".to_string(),
            HashSet::from(["BRECK_BASE_AREA".to_string(), "VAIL_BASE_LODGE".to_string()]),
        ),
        (
            "CA_BC_01".to_string(),
            HashSet::from(["WHISTLER_VILLAGE".to_string(), "BLACKCOMB_BASE".to_string()]),
        ),
    ])
}

impl AppState {
    pub fn with_persistence_dir(dir: impl AsRef<FsPath>) -> std::io::Result<Self> {
        let dir = dir.as_ref();
        Ok(Self {
            idempotency: InMemoryIdempotencyStore::with_persistence_dir(dir)?,
            journals: Arc::new(Mutex::new(InMemoryJournalRepository::default())),
            periods: Arc::new(Mutex::new(InMemoryPeriodRepository::default())),
            post_results: Arc::new(Mutex::new(HashMap::new())),
            location_allowlist_by_legal_entity: Arc::new(default_location_allowlist()),
            audit_seals: InMemoryAuditSealStore::with_persistence_dir(dir)?,
            legal_holds: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn lock_period(
        &self,
        tenant_id: &str,
        legal_entity_id: &str,
        ledger_book: &str,
        period_id: &str,
    ) -> Result<(), PeriodError> {
        let mut repo = self.periods.lock().expect("period store lock should work");
        repo.lock_period(tenant_id, legal_entity_id, ledger_book, period_id)
    }

    fn cache_post_result(
        &self,
        key: &str,
        result: CachedPostResult,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let mut cache = self.post_results.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "idempotency_result_store_error"})),
            )
        })?;
        cache.insert(key.to_string(), result);
        Ok(())
    }

    fn get_cached_post_result(
        &self,
        key: &str,
    ) -> Result<Option<CachedPostResult>, (StatusCode, Json<serde_json::Value>)> {
        let cache = self.post_results.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "idempotency_result_store_error"})),
            )
        })?;
        Ok(cache.get(key).cloned())
    }

    fn upsert_legal_hold(
        &self,
        rule: LegalHoldRule,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let mut holds = self.legal_holds.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "legal_hold_store_error"})),
            )
        })?;
        holds.insert(
            legal_hold_key(&rule.tenant_id, &rule.legal_entity_id, &rule.ledger_book),
            rule,
        );
        Ok(())
    }

    fn validate_legal_hold(
        &self,
        tenant_id: &str,
        legal_entity_id: &str,
        ledger_book: &str,
        accounting_date: NaiveDate,
    ) -> Result<(), ApiError> {
        let holds = self.legal_holds.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "legal_hold_store_error"}),
            )
        })?;
        let key = legal_hold_key(tenant_id, legal_entity_id, ledger_book);
        if let Some(rule) = holds.get(&key) {
            if rule.applies_to(accounting_date) {
                return Err((
                    StatusCode::CONFLICT,
                    json!({
                        "error": "legal_hold_active",
                        "hold_id": rule.hold_id,
                        "reason": rule.reason,
                        "retention_days": rule.retention_days,
                    }),
                ));
            }
        }
        Ok(())
    }

    fn append_audit_seal(
        &self,
        event_type: &str,
        entity_scope: &[String],
        payload: &Value,
        created_at_ns: i64,
    ) -> Result<String, ApiError> {
        self.audit_seals
            .append(event_type, entity_scope, payload, created_at_ns)
            .map(|entry| entry.seal)
            .map_err(|error| match error {
                AuditSealError::StorePoisoned => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "audit_seal_store_error"}),
                ),
                AuditSealError::ChainBroken { sequence } => (
                    StatusCode::CONFLICT,
                    json!({"error": "audit_seal_chain_broken", "sequence": sequence}),
                ),
                AuditSealError::Tampered { sequence } => (
                    StatusCode::CONFLICT,
                    json!({"error": "audit_seal_tampered", "sequence": sequence}),
                ),
            })
    }
}

fn legal_hold_key(tenant_id: &str, legal_entity_id: &str, ledger_book: &str) -> String {
    format!("{tenant_id}::{legal_entity_id}::{ledger_book}")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostEventRequest {
    pub event_type: String,
    pub tenant_id: String,
    pub legal_entity_id: String,
    #[serde(default)]
    pub location_id: Option<String>,
    pub ledger_book: String,
    pub accounting_date: String,
    pub source_event_id: String,
    pub posting_run_id: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    #[serde(default)]
    pub lines: Vec<PostLine>,
    pub provenance: Provenance,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostLine {
    pub account_id: String,
    pub entry_side: String,
    pub amount_minor: i64,
    pub currency: String,
    pub base_amount_minor: i64,
    pub base_currency: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Provenance {
    pub book_policy_id: String,
    pub policy_version: String,
    pub fx_rate_set_id: String,
    pub ruleset_version: String,
    pub workflow_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PostEventResponse {
    pub journal_id: String,
    pub status: String,
    pub replayed: bool,
}

#[derive(Debug, Serialize)]
pub struct ReverseJournalResponse {
    pub journal_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockPeriodRequest {
    pub tenant_id: String,
    pub legal_entity_id: String,
    pub ledger_book: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LockPeriodResponse {
    pub period_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpsertLegalHoldRequest {
    pub hold_id: String,
    pub tenant_id: String,
    pub legal_entity_id: String,
    pub ledger_book: String,
    pub start_date: String,
    #[serde(default)]
    pub end_date: Option<String>,
    pub reason: String,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpsertLegalHoldResponse {
    pub hold_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdjustJournalRequest {
    pub tenant_id: String,
    pub legal_entity_id: String,
    pub ledger_book: String,
    pub accounting_date: String,
    pub source_event_id: String,
    pub posting_run_id: String,
    pub reason_code: String,
    #[serde(default)]
    pub location_id: Option<String>,
    pub lines: Vec<PostLine>,
    pub provenance: Provenance,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdjustJournalResponse {
    pub reversed_journal_id: String,
    pub replacement_journal_id: String,
    pub status: String,
    pub audit_seal: String,
}

#[derive(Debug, Deserialize)]
pub struct RevRecQuery {
    pub book: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RevRecRollforwardResponse {
    pub book: String,
    pub journal_count: u32,
    pub recognized_revenue_minor: i64,
    pub deferred_revenue_ending_minor: i64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RevRecDisclosureResponse {
    pub book: String,
    pub journal_count: u32,
    pub refund_contra_revenue_minor: i64,
    pub policy_versions: Vec<String>,
    pub fx_rate_sets: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct AuditSealVerifyResponse {
    pub status: String,
    pub entries: usize,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct CapacityInstrumentationResponse {
    pub target_active_users: u32,
    pub baseline_rps: u32,
    pub peak_rps: u32,
    pub burst_rps: u32,
    pub readiness_status: String,
    pub no_bend_readiness: NoBendReadiness,
    pub scale_samples: Vec<ScaleSample>,
}

fn default_retention_days() -> u32 {
    2555
}

pub fn router() -> Router {
    router_with_state(AppState::default())
}

pub fn router_with_persistence_dir(dir: impl AsRef<FsPath>) -> std::io::Result<Router> {
    Ok(router_with_state(AppState::with_persistence_dir(dir)?))
}

pub fn router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/v1/posting/events", post(post_event))
        .route(
            "/v1/compliance/legal-holds",
            post(upsert_legal_hold_endpoint),
        )
        .route(
            "/v1/compliance/audit-seals/verify",
            get(verify_audit_seals_endpoint),
        )
        .route(
            "/v1/ledger/journals/:journal_id/reverse",
            post(reverse_journal),
        )
        .route(
            "/v1/ledger/journals/:journal_id/adjust",
            post(adjust_journal),
        )
        .route(
            "/v1/ledger/periods/:period_id/lock",
            post(lock_period_endpoint),
        )
        .route("/v1/revrec/rollforward", get(get_revrec_rollforward))
        .route("/v1/revrec/disclosures", get(get_revrec_disclosures))
        .route("/v1/ops/slo", get(get_slo))
        .route("/v1/ops/capacity", get(get_capacity))
        .with_state(state)
}

async fn post_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PostEventRequest>,
) -> Result<Json<PostEventResponse>, (StatusCode, Json<serde_json::Value>)> {
    if ![
        "order.captured.v1",
        "payment.settled.v1",
        "refund.v1",
        "fee.assessed.v1",
        "chargeback.created.v1",
        "payout.cleared.v1",
        "dispute.opened.v1",
        "dispute.won.v1",
        "dispute.lost.v1",
        "inntopia.reservation.captured.v1",
        "intercompany.due_to_due_from.v1",
        "consolidation.elimination.v1",
        "fx.translation.v1",
    ]
    .contains(&req.event_type.as_str())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "unsupported_event_type"})),
        ));
    }

    let key = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "missing_idempotency_key"})),
            )
        })?;

    let payload = serde_json::to_value(&req).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_payload"})),
        )
    })?;

    let idem_status = match state.idempotency.check_or_insert(key, &payload) {
        Ok(status) => status,
        Err(IdempotencyError::PayloadHashMismatch) => {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({"error": "idempotency_payload_mismatch"})),
            ))
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "idempotency_store_error"})),
            ))
        }
    };

    let journal_uuid = deterministic_journal_id(key, &payload_hash(&payload));
    if idem_status == IdempotencyStatus::Replay {
        return match state.get_cached_post_result(key)? {
            Some(CachedPostResult::Success(previous)) => Ok(Json(PostEventResponse {
                journal_id: previous.journal_id,
                status: previous.status,
                replayed: true,
            })),
            Some(CachedPostResult::Failure { status, body }) => Err((status, Json(body))),
            None => Ok(Json(PostEventResponse {
                journal_id: journal_uuid.to_string(),
                status: "POSTED".to_string(),
                replayed: true,
            })),
        };
    }

    let result = process_first_seen_post(&state, req, journal_uuid).map(Json);
    match result {
        Ok(Json(response)) => {
            state.cache_post_result(key, CachedPostResult::Success(response.clone()))?;
            Ok(Json(response))
        }
        Err((status, body)) => {
            state.cache_post_result(
                key,
                CachedPostResult::Failure {
                    status,
                    body: body.clone(),
                },
            )?;
            Err((status, Json(body)))
        }
    }
}

async fn upsert_legal_hold_endpoint(
    State(state): State<AppState>,
    Json(req): Json<UpsertLegalHoldRequest>,
) -> Result<Json<UpsertLegalHoldResponse>, (StatusCode, Json<serde_json::Value>)> {
    let start_date = NaiveDate::parse_from_str(&req.start_date, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_start_date"})),
        )
    })?;
    let end_date = req
        .end_date
        .as_deref()
        .map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d"))
        .transpose()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_end_date"})),
            )
        })?;
    if let Some(value) = end_date.as_ref() {
        if *value < start_date {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_legal_hold_range"})),
            ));
        }
    }

    let hold_id = req.hold_id.clone();
    let seal_tenant_id = req.tenant_id.clone();
    let seal_legal_entity_id = req.legal_entity_id.clone();
    let seal_ledger_book = req.ledger_book.clone();
    let seal_start_date = req.start_date.clone();
    let seal_end_date = req.end_date.clone();
    let seal_reason = req.reason.clone();
    let seal_retention_days = req.retention_days;
    let rule = LegalHoldRule {
        hold_id: req.hold_id,
        tenant_id: req.tenant_id,
        legal_entity_id: req.legal_entity_id,
        ledger_book: req.ledger_book,
        start_date,
        end_date,
        reason: req.reason,
        retention_days: req.retention_days,
    };
    state.upsert_legal_hold(rule)?;
    state
        .append_audit_seal(
            "legal_hold.upserted",
            &[seal_legal_entity_id.clone()],
            &json!({
                "hold_id": hold_id,
                "tenant_id": seal_tenant_id,
                "legal_entity_id": seal_legal_entity_id,
                "ledger_book": seal_ledger_book,
                "start_date": seal_start_date,
                "end_date": seal_end_date,
                "retention_days": seal_retention_days,
                "reason": seal_reason
            }),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
        )
        .map_err(|(status, body)| (status, Json(body)))?;

    Ok(Json(UpsertLegalHoldResponse {
        hold_id,
        status: "ACTIVE".to_string(),
    }))
}

async fn verify_audit_seals_endpoint(
    State(state): State<AppState>,
) -> Result<Json<AuditSealVerifyResponse>, (StatusCode, Json<serde_json::Value>)> {
    state
        .audit_seals
        .verify_chain()
        .map_err(|error| match error {
            AuditSealError::StorePoisoned => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "audit_seal_store_error"})),
            ),
            AuditSealError::ChainBroken { sequence } => (
                StatusCode::CONFLICT,
                Json(json!({"error": "audit_seal_chain_broken", "sequence": sequence})),
            ),
            AuditSealError::Tampered { sequence } => (
                StatusCode::CONFLICT,
                Json(json!({"error": "audit_seal_tampered", "sequence": sequence})),
            ),
        })?;
    let entries = state.audit_seals.len().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "audit_seal_store_error"})),
        )
    })?;
    Ok(Json(AuditSealVerifyResponse {
        status: "VERIFIED".to_string(),
        entries,
    }))
}

async fn adjust_journal(
    State(state): State<AppState>,
    Path(journal_id): Path<String>,
    Json(req): Json<AdjustJournalRequest>,
) -> Result<Json<AdjustJournalResponse>, (StatusCode, Json<serde_json::Value>)> {
    let target_journal_id = Uuid::parse_str(&journal_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_journal_id"})),
        )
    })?;
    if req.lines.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "missing_adjustment_lines"})),
        ));
    }
    let accounting_date =
        NaiveDate::parse_from_str(&req.accounting_date, "%Y-%m-%d").map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_accounting_date"})),
            )
        })?;
    if let Some(location_id) = req.location_id.as_deref() {
        validate_location_boundary(&state, &req.legal_entity_id, location_id)
            .map_err(|(status, body)| (status, Json(body)))?;
    }
    state
        .validate_legal_hold(
            &req.tenant_id,
            &req.legal_entity_id,
            &req.ledger_book,
            accounting_date,
        )
        .map_err(|(status, body)| (status, Json(body)))?;
    {
        let periods = state.periods.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "period_store_error"})),
            )
        })?;
        periods
            .ensure_open(
                &req.tenant_id,
                &req.legal_entity_id,
                &req.ledger_book,
                accounting_date,
            )
            .map_err(|error| {
                let (status, body) = period_error_response(error);
                (status, Json(body))
            })?;
    }

    let lines = derive_lines_from_post_lines(&req.lines).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": error.to_string()})),
        )
    })?;

    let replacement_journal_id = deterministic_journal_id(
        &format!("adjust:{target_journal_id}:{}", req.source_event_id),
        &payload_hash(&json!({
            "reason_code": &req.reason_code,
            "accounting_date": &req.accounting_date,
            "lines": &req.lines,
            "posting_run_id": &req.posting_run_id,
        })),
    );

    {
        let mut repo = state.journals.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "journal_store_error"})),
            )
        })?;
        let existing = repo.get(&target_journal_id).cloned().ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "journal_not_found"})),
            )
        })?;
        if existing.header.tenant_id != req.tenant_id
            || existing.header.legal_entity_id != req.legal_entity_id
            || existing.header.ledger_book != req.ledger_book
        {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "adjustment_scope_mismatch"})),
            ));
        }

        repo.reverse(&target_journal_id)
            .map_err(|error| match error {
                LedgerError::AlreadyReversed => (
                    StatusCode::CONFLICT,
                    Json(json!({"error": "journal_already_reversed"})),
                ),
                LedgerError::NotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "journal_not_found"})),
                ),
                _ => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": error.to_string()})),
                ),
            })?;

        let replacement = JournalRecord {
            header: JournalHeader {
                journal_id: replacement_journal_id,
                journal_number: format!("ADJ-{}", &replacement_journal_id.to_string()[..8]),
                status: JournalStatus::Posted,
                tenant_id: req.tenant_id.clone(),
                legal_entity_id: req.legal_entity_id.clone(),
                ledger_book: req.ledger_book.clone(),
                accounting_date,
                posted_at: chrono::Utc::now(),
                source_event_ids: vec![
                    req.source_event_id.clone(),
                    format!("adjusts:{target_journal_id}"),
                ],
                posting_run_id: req.posting_run_id.clone(),
                book_policy_id: req.provenance.book_policy_id.clone(),
                policy_version: req.provenance.policy_version.clone(),
                fx_rate_set_id: req.provenance.fx_rate_set_id.clone(),
                ruleset_version: req.provenance.ruleset_version.clone(),
                workflow_id: req.provenance.workflow_id.clone(),
            },
            lines,
        };
        repo.insert_posted(replacement).map_err(|error| {
            let (status, body) = ledger_error_response(error);
            (status, Json(body))
        })?;
    }

    let audit_seal = state
        .append_audit_seal(
            "journal.adjusted",
            &[req.legal_entity_id.clone()],
            &json!({
                "reversed_journal_id": target_journal_id,
                "replacement_journal_id": replacement_journal_id,
                "reason_code": &req.reason_code
            }),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
        )
        .map_err(|(status, body)| (status, Json(body)))?;

    Ok(Json(AdjustJournalResponse {
        reversed_journal_id: target_journal_id.to_string(),
        replacement_journal_id: replacement_journal_id.to_string(),
        status: "ADJUSTED".to_string(),
        audit_seal,
    }))
}

async fn get_revrec_rollforward(
    State(state): State<AppState>,
    Query(query): Query<RevRecQuery>,
) -> Result<Json<RevRecRollforwardResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = state.journals.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "journal_store_error"})),
        )
    })?;
    let records = repo.all();
    drop(repo);

    let mut recognized_revenue_minor = 0_i64;
    let mut deferred_revenue_ending_minor = 0_i64;
    let mut journal_count = 0_u32;
    for record in records {
        if record.header.ledger_book != query.book || record.header.status != JournalStatus::Posted
        {
            continue;
        }
        journal_count += 1;
        for line in &record.lines {
            if line.account_id == "4000-REVENUE" {
                recognized_revenue_minor +=
                    signed_amount(line.entry_side.clone(), line.amount_minor);
            }
            if line.account_id.contains("DEFERRED") {
                deferred_revenue_ending_minor +=
                    signed_amount(line.entry_side.clone(), line.amount_minor);
            }
        }
    }

    Ok(Json(RevRecRollforwardResponse {
        book: query.book,
        journal_count,
        recognized_revenue_minor,
        deferred_revenue_ending_minor,
    }))
}

async fn get_revrec_disclosures(
    State(state): State<AppState>,
    Query(query): Query<RevRecQuery>,
) -> Result<Json<RevRecDisclosureResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = state.journals.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "journal_store_error"})),
        )
    })?;
    let records = repo.all();
    drop(repo);

    let mut journal_count = 0_u32;
    let mut refund_contra_revenue_minor = 0_i64;
    let mut policy_versions = HashSet::new();
    let mut fx_rate_sets = HashSet::new();
    for record in records {
        if record.header.ledger_book != query.book || record.header.status != JournalStatus::Posted
        {
            continue;
        }
        journal_count += 1;
        policy_versions.insert(record.header.policy_version.clone());
        fx_rate_sets.insert(record.header.fx_rate_set_id.clone());
        for line in &record.lines {
            if line.account_id == "4050-REFUNDS" {
                refund_contra_revenue_minor +=
                    signed_amount(line.entry_side.clone(), line.amount_minor);
            }
        }
    }

    let mut policy_versions = policy_versions.into_iter().collect::<Vec<_>>();
    policy_versions.sort();
    let mut fx_rate_sets = fx_rate_sets.into_iter().collect::<Vec<_>>();
    fx_rate_sets.sort();

    Ok(Json(RevRecDisclosureResponse {
        book: query.book,
        journal_count,
        refund_contra_revenue_minor,
        policy_versions,
        fx_rate_sets,
    }))
}

fn process_first_seen_post(
    state: &AppState,
    req: PostEventRequest,
    journal_uuid: Uuid,
) -> Result<PostEventResponse, ApiError> {
    let accounting_date =
        NaiveDate::parse_from_str(&req.accounting_date, "%Y-%m-%d").map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                json!({"error": "invalid_accounting_date"}),
            )
        })?;
    state.validate_legal_hold(
        &req.tenant_id,
        &req.legal_entity_id,
        &req.ledger_book,
        accounting_date,
    )?;

    let location_id = resolve_location_id(&req)?;
    validate_location_boundary(state, &req.legal_entity_id, &location_id)?;
    validate_intercompany_counterparty(state, &req)?;

    {
        let periods = state.periods.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "period_store_error"}),
            )
        })?;
        periods
            .ensure_open(
                &req.tenant_id,
                &req.legal_entity_id,
                &req.ledger_book,
                accounting_date,
            )
            .map_err(period_error_response)?;
    }

    let lines = derive_journal_lines(&req)
        .map_err(|e| (StatusCode::BAD_REQUEST, json!({"error": e.to_string()})))?;

    let record = JournalRecord {
        header: JournalHeader {
            journal_id: journal_uuid,
            journal_number: format!("S2-{}", &journal_uuid.to_string()[..8]),
            status: JournalStatus::Posted,
            tenant_id: req.tenant_id.clone(),
            legal_entity_id: req.legal_entity_id.clone(),
            ledger_book: req.ledger_book.clone(),
            accounting_date,
            posted_at: chrono::Utc::now(),
            source_event_ids: vec![req.source_event_id.clone()],
            posting_run_id: req.posting_run_id.clone(),
            book_policy_id: req.provenance.book_policy_id.clone(),
            policy_version: req.provenance.policy_version.clone(),
            fx_rate_set_id: req.provenance.fx_rate_set_id.clone(),
            ruleset_version: req.provenance.ruleset_version.clone(),
            workflow_id: req.provenance.workflow_id.clone(),
        },
        lines,
    };

    let mut repo = state.journals.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"error": "journal_store_error"}),
        )
    })?;
    repo.insert_posted(record).map_err(ledger_error_response)?;
    drop(repo);

    let mut audit_entity_scope = vec![req.legal_entity_id.clone()];
    if matches!(
        req.event_type.as_str(),
        "intercompany.due_to_due_from.v1" | "consolidation.elimination.v1"
    ) {
        if let Some(counterparty) = first_string(
            &req.payload,
            &[
                "/counterparty_legal_entity_id",
                "/intercompany/counterparty_legal_entity_id",
                "/consolidation/counterparty_legal_entity_id",
            ],
        ) {
            audit_entity_scope.push(counterparty.to_string());
        }
    }

    state.append_audit_seal(
        "posting.posted",
        &audit_entity_scope,
        &json!({
            "event_type": req.event_type,
            "journal_id": journal_uuid,
            "tenant_id": req.tenant_id,
            "ledger_book": req.ledger_book,
            "source_event_id": req.source_event_id,
            "location_id": location_id
        }),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
    )?;

    Ok(PostEventResponse {
        journal_id: journal_uuid.to_string(),
        status: "POSTED".to_string(),
        replayed: false,
    })
}

async fn reverse_journal(
    State(state): State<AppState>,
    Path(journal_id): Path<String>,
) -> Result<Json<ReverseJournalResponse>, (StatusCode, Json<serde_json::Value>)> {
    let journal_id = Uuid::parse_str(&journal_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_journal_id"})),
        )
    })?;

    let mut repo = state.journals.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "journal_store_error"})),
        )
    })?;

    repo.reverse(&journal_id).map_err(|error| match error {
        LedgerError::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "journal_not_found"})),
        ),
        LedgerError::AlreadyReversed => (
            StatusCode::CONFLICT,
            Json(json!({"error": "journal_already_reversed"})),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": error.to_string()})),
        ),
    })?;
    let entity_scope = repo
        .get(&journal_id)
        .map(|record| vec![record.header.legal_entity_id.clone()])
        .unwrap_or_default();
    drop(repo);
    if !entity_scope.is_empty() {
        state
            .append_audit_seal(
                "journal.reversed",
                &entity_scope,
                &json!({"journal_id": journal_id}),
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
            )
            .map_err(|(status, body)| (status, Json(body)))?;
    }

    Ok(Json(ReverseJournalResponse {
        journal_id: journal_id.to_string(),
        status: "REVERSED".to_string(),
    }))
}

async fn lock_period_endpoint(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
    Json(req): Json<LockPeriodRequest>,
) -> Result<Json<LockPeriodResponse>, (StatusCode, Json<serde_json::Value>)> {
    state
        .lock_period(
            &req.tenant_id,
            &req.legal_entity_id,
            &req.ledger_book,
            &period_id,
        )
        .map_err(|error| {
            let (status, body) = period_error_response(error);
            (status, Json(body))
        })?;

    Ok(Json(LockPeriodResponse {
        period_id,
        status: "LOCKED".to_string(),
    }))
}

async fn get_slo() -> Json<serde_json::Value> {
    Json(json!({
        "availability_target": "99.95%",
        "read_p95_ms": 150,
        "write_p95_ms": 250,
        "error_rate_max": 0.001,
        "no_bend_efficiency_min": 0.80
    }))
}

fn production_scale_samples() -> Vec<ScaleSample> {
    vec![
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
    ]
}

fn build_capacity_instrumentation_response(
    samples: Vec<ScaleSample>,
) -> Option<CapacityInstrumentationResponse> {
    let no_bend_readiness = evaluate_no_bend_readiness(
        &samples,
        CAPACITY_TARGET_ACTIVE_USERS,
        CAPACITY_LINEARITY_RATIO_MIN,
    )?;
    let baseline_rps = samples
        .iter()
        .min_by_key(|sample| sample.active_users)?
        .throughput_rps
        .round() as u32;
    let peak_rps = samples
        .iter()
        .find(|sample| sample.active_users >= CAPACITY_TARGET_ACTIVE_USERS)?
        .throughput_rps
        .round() as u32;
    let readiness_status = if no_bend_readiness.comfortable {
        "READY".to_string()
    } else {
        "AT_RISK".to_string()
    };

    Some(CapacityInstrumentationResponse {
        target_active_users: CAPACITY_TARGET_ACTIVE_USERS,
        baseline_rps,
        peak_rps,
        burst_rps: CAPACITY_BURST_RPS,
        readiness_status,
        no_bend_readiness,
        scale_samples: samples,
    })
}

async fn get_capacity(
) -> Result<Json<CapacityInstrumentationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let samples = production_scale_samples();
    let response = build_capacity_instrumentation_response(samples).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "capacity_readiness_unavailable"})),
    ))?;
    Ok(Json(response))
}

fn resolve_location_id(req: &PostEventRequest) -> Result<String, ApiError> {
    if let Some(location_id) = req
        .location_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(location_id.to_string());
    }

    first_string(
        &req.payload,
        &[
            "/location_id",
            "/routing/location_id",
            "/context/routing/location_id",
            "/extensions/routing/location_id",
        ],
    )
    .map(ToString::to_string)
    .ok_or((
        StatusCode::BAD_REQUEST,
        json!({"error": "missing_location_id"}),
    ))
}

fn validate_location_boundary(
    state: &AppState,
    legal_entity_id: &str,
    location_id: &str,
) -> Result<(), ApiError> {
    let allowed_locations = state
        .location_allowlist_by_legal_entity
        .get(legal_entity_id)
        .ok_or((
            StatusCode::BAD_REQUEST,
            json!({"error": "unknown_legal_entity_boundary", "legal_entity_id": legal_entity_id}),
        ))?;

    if !allowed_locations.contains(location_id) {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({
                "error": "location_not_allowed_for_legal_entity",
                "legal_entity_id": legal_entity_id,
                "location_id": location_id
            }),
        ));
    }

    Ok(())
}

fn validate_intercompany_counterparty(
    state: &AppState,
    req: &PostEventRequest,
) -> Result<(), ApiError> {
    if !matches!(
        req.event_type.as_str(),
        "intercompany.due_to_due_from.v1" | "consolidation.elimination.v1"
    ) {
        return Ok(());
    }

    let counterparty = first_string(
        &req.payload,
        &[
            "/counterparty_legal_entity_id",
            "/intercompany/counterparty_legal_entity_id",
            "/consolidation/counterparty_legal_entity_id",
        ],
    )
    .ok_or((
        StatusCode::BAD_REQUEST,
        json!({"error": "missing_counterparty_legal_entity_id"}),
    ))?;

    if counterparty == req.legal_entity_id {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({"error": "invalid_counterparty_legal_entity"}),
        ));
    }

    if !state
        .location_allowlist_by_legal_entity
        .contains_key(counterparty)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({"error": "unknown_counterparty_legal_entity"}),
        ));
    }

    Ok(())
}

fn first_string<'a>(payload: &'a Value, pointers: &[&str]) -> Option<&'a str> {
    pointers
        .iter()
        .find_map(|pointer| payload.pointer(pointer).and_then(Value::as_str))
}

fn derive_journal_lines(req: &PostEventRequest) -> Result<Vec<JournalLine>, RuleEngineError> {
    if req.payload.is_object() {
        let derived = derive_lines_v1(&req.event_type, &req.payload)?;
        return Ok(derived
            .into_iter()
            .enumerate()
            .map(|(index, line)| JournalLine {
                line_number: (index + 1) as u32,
                account_id: line.account_id,
                entry_side: line.entry_side,
                amount_minor: line.amount_minor,
                currency: line.currency,
                base_amount_minor: line.base_amount_minor,
                base_currency: line.base_currency,
            })
            .collect());
    }

    if req.lines.is_empty() {
        return Err(RuleEngineError::MissingField("payload"));
    }

    derive_lines_from_post_lines(&req.lines)
}

fn derive_lines_from_post_lines(lines: &[PostLine]) -> Result<Vec<JournalLine>, RuleEngineError> {
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            Ok(JournalLine {
                line_number: (index + 1) as u32,
                account_id: line.account_id.clone(),
                entry_side: parse_entry_side(&line.entry_side)?,
                amount_minor: line.amount_minor,
                currency: line.currency.clone(),
                base_amount_minor: line.base_amount_minor,
                base_currency: line.base_currency.clone(),
            })
        })
        .collect()
}

fn signed_amount(entry_side: EntrySide, amount_minor: i64) -> i64 {
    match entry_side {
        EntrySide::Credit => amount_minor,
        EntrySide::Debit => -amount_minor,
    }
}

fn parse_entry_side(entry_side: &str) -> Result<EntrySide, RuleEngineError> {
    if entry_side.eq_ignore_ascii_case("debit") {
        Ok(EntrySide::Debit)
    } else if entry_side.eq_ignore_ascii_case("credit") {
        Ok(EntrySide::Credit)
    } else {
        Err(RuleEngineError::InvalidEntrySide(entry_side.to_string()))
    }
}

fn period_error_response(error: PeriodError) -> ApiError {
    match error {
        PeriodError::PeriodClosed(period_id) => (
            StatusCode::CONFLICT,
            json!({"error": format!("period_closed:{period_id}")}),
        ),
        PeriodError::InvalidPeriodId(_) => {
            (StatusCode::BAD_REQUEST, json!({"error": error.to_string()}))
        }
    }
}

fn ledger_error_response(error: LedgerError) -> ApiError {
    match error {
        LedgerError::JournalExists => (StatusCode::CONFLICT, json!({"error": "journal_exists"})),
        LedgerError::Unbalanced => (
            StatusCode::BAD_REQUEST,
            json!({"error": "journal_unbalanced"}),
        ),
        _ => (StatusCode::BAD_REQUEST, json!({"error": error.to_string()})),
    }
}

fn deterministic_journal_id(key: &str, hash: &str) -> Uuid {
    let composite = format!("{key}:{hash}");
    let hashed = payload_hash(&json!({ "value": composite }));
    let bytes = hex::decode(&hashed[..32]).expect("hex decode should work");
    let mut arr = [0_u8; 16];
    arr.copy_from_slice(&bytes);
    Uuid::from_bytes(arr)
}

#[cfg(test)]
mod tests {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;

    fn order_payload(amount: i64) -> serde_json::Value {
        json!({
            "event_type": "order.captured.v1",
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "location_id": "BRECK_BASE_AREA",
            "ledger_book": "US_GAAP",
            "accounting_date": "2026-02-21",
            "source_event_id": "evt_1",
            "posting_run_id": "run_1",
            "payload": {
                "amount_minor": amount,
                "currency": "USD"
            },
            "provenance": {
                "book_policy_id": "policy_dual_book",
                "policy_version": "1.0.0",
                "fx_rate_set_id": "fx_2026_02_21",
                "ruleset_version": "v1",
                "workflow_id": "wf_1"
            }
        })
    }

    fn inntopia_payload(amount: i64) -> serde_json::Value {
        json!({
            "event_type": "inntopia.reservation.captured.v1",
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "location_id": "BRECK_BASE_AREA",
            "ledger_book": "US_GAAP",
            "accounting_date": "2026-02-21",
            "source_event_id": "inntopia_evt_1",
            "posting_run_id": "run_1",
            "payload": {
                "reservation_id": "resv_1",
                "total_amount_minor": amount,
                "currency": "USD"
            },
            "provenance": {
                "book_policy_id": "policy_dual_book",
                "policy_version": "1.0.0",
                "fx_rate_set_id": "fx_2026_02_21",
                "ruleset_version": "v1",
                "workflow_id": "wf_1"
            }
        })
    }

    fn sprint3_payload(event_type: &str, amount: i64) -> serde_json::Value {
        json!({
            "event_type": event_type,
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "location_id": "BRECK_BASE_AREA",
            "ledger_book": "US_GAAP",
            "accounting_date": "2026-02-21",
            "source_event_id": format!("{event_type}-evt-1"),
            "posting_run_id": "run_1",
            "payload": {
                "amount_minor": amount,
                "fee_amount_minor": amount,
                "currency": "USD"
            },
            "provenance": {
                "book_policy_id": "policy_dual_book",
                "policy_version": "1.0.0",
                "fx_rate_set_id": "fx_2026_02_21",
                "ruleset_version": "v1",
                "workflow_id": "wf_1"
            }
        })
    }

    fn sprint4_payload(
        event_type: &str,
        amount: i64,
        counterparty_legal_entity_id: Option<&str>,
    ) -> serde_json::Value {
        let mut payload = json!({
            "amount_minor": amount,
            "translation_amount_minor": amount,
            "currency": "USD",
            "base_currency": "USD"
        });
        if let Some(counterparty) = counterparty_legal_entity_id {
            payload["counterparty_legal_entity_id"] = json!(counterparty);
        }

        json!({
            "event_type": event_type,
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "location_id": "BRECK_BASE_AREA",
            "ledger_book": "US_GAAP",
            "accounting_date": "2026-02-21",
            "source_event_id": format!("{event_type}-evt-1"),
            "posting_run_id": "run_1",
            "payload": payload,
            "provenance": {
                "book_policy_id": "policy_dual_book",
                "policy_version": "1.0.0",
                "fx_rate_set_id": "fx_2026_02_21",
                "ruleset_version": "v1",
                "workflow_id": "wf_1"
            }
        })
    }

    fn adjustment_payload(source_event_id: &str, amount: i64) -> serde_json::Value {
        json!({
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "ledger_book": "US_GAAP",
            "accounting_date": "2026-02-21",
            "source_event_id": source_event_id,
            "posting_run_id": "run_adj_1",
            "reason_code": "MANUAL_TRUE_UP",
            "location_id": "BRECK_BASE_AREA",
            "lines": [
                {
                    "account_id": "1105-CASH-CLEARING",
                    "entry_side": "debit",
                    "amount_minor": amount,
                    "currency": "USD",
                    "base_amount_minor": amount,
                    "base_currency": "USD"
                },
                {
                    "account_id": "4000-REVENUE",
                    "entry_side": "credit",
                    "amount_minor": amount,
                    "currency": "USD",
                    "base_amount_minor": amount,
                    "base_currency": "USD"
                }
            ],
            "provenance": {
                "book_policy_id": "policy_dual_book",
                "policy_version": "1.0.1",
                "fx_rate_set_id": "fx_2026_02_21",
                "ruleset_version": "v1",
                "workflow_id": "wf_adj_1"
            }
        })
    }

    fn post_request(idempotency_key: &str, payload: &serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/v1/posting/events")
            .header("content-type", "application/json")
            .header("Idempotency-Key", idempotency_key)
            .body(Body::from(payload.to_string()))
            .unwrap()
    }

    fn period_lock_request(period_id: &str, payload: &serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(format!("/v1/ledger/periods/{period_id}/lock"))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap()
    }

    fn legal_hold_request(payload: &serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/v1/compliance/legal-holds")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap()
    }

    fn adjust_request(journal_id: &str, payload: &serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(format!("/v1/ledger/journals/{journal_id}/adjust"))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap()
    }

    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn duplicate_same_payload_replays() {
        let app = router();
        let payload = order_payload(10000);

        let first = app
            .clone()
            .oneshot(post_request("same-key", &payload))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let first_body = json_body(first).await;
        assert_eq!(first_body["replayed"], json!(false));

        let second = app
            .oneshot(post_request("same-key", &payload))
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::OK);
        let second_body = json_body(second).await;
        assert_eq!(second_body["replayed"], json!(true));
        assert_eq!(second_body["journal_id"], first_body["journal_id"]);
    }

    #[tokio::test]
    async fn duplicate_different_payload_conflicts() {
        let app = router();

        let first = app
            .clone()
            .oneshot(post_request("same-key", &order_payload(10000)))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let second = app
            .oneshot(post_request("same-key", &order_payload(9000)))
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn inntopia_reservation_posts_with_rule_engine_v1() {
        let app = router();

        let response = app
            .oneshot(post_request("inntopia-key", &inntopia_payload(41250)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn sprint3_fee_event_posts_with_rule_engine_v1() {
        let app = router();

        let response = app
            .oneshot(post_request(
                "fee-key",
                &sprint3_payload("fee.assessed.v1", 325),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn sprint4_intercompany_event_posts_with_rule_engine_v1() {
        let app = router();

        let response = app
            .oneshot(post_request(
                "intercompany-key",
                &sprint4_payload("intercompany.due_to_due_from.v1", 1200, Some("CA_BC_01")),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn sprint4_consolidation_event_posts_with_rule_engine_v1() {
        let app = router();

        let response = app
            .oneshot(post_request(
                "consolidation-key",
                &sprint4_payload("consolidation.elimination.v1", 900, Some("CA_BC_01")),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn sprint4_fx_translation_event_posts_with_rule_engine_v1() {
        let app = router();

        let response = app
            .oneshot(post_request(
                "fx-translation-key",
                &sprint4_payload("fx.translation.v1", -500, None),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unsupported_event_type_is_rejected() {
        let app = router();

        let response = app
            .oneshot(post_request(
                "unsupported-key",
                &sprint3_payload("inventory.adjusted.v1", 100),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["error"], json!("unsupported_event_type"));
    }

    #[tokio::test]
    async fn closed_period_rejects_first_seen_posting() {
        let state = AppState::default();
        state
            .lock_period("tenant_1", "US_CO_01", "US_GAAP", "2026-02")
            .unwrap();
        let app = router_with_state(state);

        let response = app
            .oneshot(post_request("period-key", &order_payload(10000)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["error"], json!("period_closed:2026-02"));
    }

    #[tokio::test]
    async fn closed_period_replay_returns_same_error() {
        let state = AppState::default();
        state
            .lock_period("tenant_1", "US_CO_01", "US_GAAP", "2026-02")
            .unwrap();
        let app = router_with_state(state);
        let payload = order_payload(10000);

        let first = app
            .clone()
            .oneshot(post_request("closed-period-key", &payload))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::CONFLICT);
        let first_body = json_body(first).await;

        let second = app
            .oneshot(post_request("closed-period-key", &payload))
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::CONFLICT);
        let second_body = json_body(second).await;

        assert_eq!(second_body, first_body);
    }

    #[tokio::test]
    async fn missing_location_is_rejected() {
        let app = router();
        let mut payload = order_payload(10000);
        payload.as_object_mut().unwrap().remove("location_id");

        let response = app
            .oneshot(post_request("missing-location-key", &payload))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["error"], json!("missing_location_id"));
    }

    #[tokio::test]
    async fn location_not_in_legal_entity_allowlist_is_rejected() {
        let app = router();
        let mut payload = order_payload(10000);
        payload["location_id"] = json!("WHISTLER_VILLAGE");

        let response = app
            .oneshot(post_request("boundary-key", &payload))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(
            body["error"],
            json!("location_not_allowed_for_legal_entity")
        );
    }

    #[tokio::test]
    async fn intercompany_event_requires_counterparty() {
        let app = router();
        let response = app
            .oneshot(post_request(
                "missing-counterparty-key",
                &sprint4_payload("intercompany.due_to_due_from.v1", 500, None),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["error"], json!("missing_counterparty_legal_entity_id"));
    }

    #[tokio::test]
    async fn intercompany_event_rejects_same_entity_counterparty() {
        let app = router();
        let response = app
            .oneshot(post_request(
                "same-counterparty-key",
                &sprint4_payload("intercompany.due_to_due_from.v1", 500, Some("US_CO_01")),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["error"], json!("invalid_counterparty_legal_entity"));
    }

    #[tokio::test]
    async fn consolidation_event_rejects_unknown_counterparty() {
        let app = router();
        let response = app
            .oneshot(post_request(
                "unknown-counterparty-key",
                &sprint4_payload("consolidation.elimination.v1", 500, Some("MX_NL_01")),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["error"], json!("unknown_counterparty_legal_entity"));
    }

    #[tokio::test]
    async fn period_lock_endpoint_locks_period_and_blocks_posts() {
        let app = router();
        let lock_payload = json!({
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "ledger_book": "US_GAAP"
        });

        let lock_response = app
            .clone()
            .oneshot(period_lock_request("2026-02", &lock_payload))
            .await
            .unwrap();
        assert_eq!(lock_response.status(), StatusCode::OK);
        let lock_body = json_body(lock_response).await;
        assert_eq!(lock_body["status"], json!("LOCKED"));
        assert_eq!(lock_body["period_id"], json!("2026-02"));

        let post_response = app
            .oneshot(post_request("lock-route-key", &order_payload(10000)))
            .await
            .unwrap();
        assert_eq!(post_response.status(), StatusCode::CONFLICT);
        let body = json_body(post_response).await;
        assert_eq!(body["error"], json!("period_closed:2026-02"));
    }

    #[tokio::test]
    async fn period_lock_endpoint_rejects_invalid_period_id() {
        let app = router();
        let lock_payload = json!({
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "ledger_book": "US_GAAP"
        });

        let response = app
            .oneshot(period_lock_request("202602", &lock_payload))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert!(body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("invalid period id"));
    }

    #[tokio::test]
    async fn reversal_endpoint_reverses_once() {
        let app = router();
        let payload = order_payload(10000);
        let key = "reverse-key";

        let post = app
            .clone()
            .oneshot(post_request(key, &payload))
            .await
            .unwrap();
        assert_eq!(post.status(), StatusCode::OK);
        let post_body = json_body(post).await;
        let journal_id = post_body["journal_id"].as_str().unwrap().to_string();

        let reverse = Request::builder()
            .method("POST")
            .uri(format!("/v1/ledger/journals/{journal_id}/reverse"))
            .body(Body::empty())
            .unwrap();

        let first_reverse = app.clone().oneshot(reverse).await.unwrap();
        assert_eq!(first_reverse.status(), StatusCode::OK);
        let first_body = json_body(first_reverse).await;
        assert_eq!(first_body["status"], json!("REVERSED"));

        let duplicate_reverse = Request::builder()
            .method("POST")
            .uri(format!("/v1/ledger/journals/{journal_id}/reverse"))
            .body(Body::empty())
            .unwrap();
        let second_reverse = app.oneshot(duplicate_reverse).await.unwrap();
        assert_eq!(second_reverse.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn legal_hold_blocks_posting_when_active() {
        let app = router();
        let hold_payload = json!({
            "hold_id": "LH-2026-0001",
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "ledger_book": "US_GAAP",
            "start_date": "2026-02-01",
            "end_date": "2026-03-01",
            "reason": "Regulatory audit",
            "retention_days": 2555
        });

        let hold_response = app
            .clone()
            .oneshot(legal_hold_request(&hold_payload))
            .await
            .unwrap();
        assert_eq!(hold_response.status(), StatusCode::OK);

        let response = app
            .oneshot(post_request("held-post-key", &order_payload(10000)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["error"], json!("legal_hold_active"));
        assert_eq!(body["hold_id"], json!("LH-2026-0001"));
    }

    #[tokio::test]
    async fn legal_hold_upsert_emits_audit_seal() {
        let app = router();
        let hold_payload = json!({
            "hold_id": "LH-2026-0002",
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "ledger_book": "US_GAAP",
            "start_date": "2026-02-01",
            "end_date": "2026-03-01",
            "reason": "Audit hold",
            "retention_days": 2555
        });

        let hold_response = app
            .clone()
            .oneshot(legal_hold_request(&hold_payload))
            .await
            .unwrap();
        assert_eq!(hold_response.status(), StatusCode::OK);

        let verify = Request::builder()
            .method("GET")
            .uri("/v1/compliance/audit-seals/verify")
            .body(Body::empty())
            .unwrap();
        let verify_response = app.oneshot(verify).await.unwrap();
        assert_eq!(verify_response.status(), StatusCode::OK);
        let body = json_body(verify_response).await;
        assert!(body["entries"].as_u64().unwrap_or_default() >= 1);
    }

    #[tokio::test]
    async fn audit_seal_verify_endpoint_reports_entries() {
        let app = router();

        let post = app
            .clone()
            .oneshot(post_request("audit-seal-post", &order_payload(10000)))
            .await
            .unwrap();
        assert_eq!(post.status(), StatusCode::OK);

        let verify = Request::builder()
            .method("GET")
            .uri("/v1/compliance/audit-seals/verify")
            .body(Body::empty())
            .unwrap();
        let verify_response = app.oneshot(verify).await.unwrap();
        assert_eq!(verify_response.status(), StatusCode::OK);
        let body = json_body(verify_response).await;
        assert_eq!(body["status"], json!("VERIFIED"));
        assert!(body["entries"].as_u64().unwrap_or_default() >= 1);
    }

    #[tokio::test]
    async fn adjustment_endpoint_reverses_original_and_posts_replacement() {
        let app = router();

        let post = app
            .clone()
            .oneshot(post_request("adjust-source-key", &order_payload(10000)))
            .await
            .unwrap();
        assert_eq!(post.status(), StatusCode::OK);
        let posted_body = json_body(post).await;
        let journal_id = posted_body["journal_id"].as_str().unwrap().to_string();

        let adjust_payload = adjustment_payload("adj_evt_1", 9000);
        let adjust_response = app
            .clone()
            .oneshot(adjust_request(&journal_id, &adjust_payload))
            .await
            .unwrap();
        assert_eq!(adjust_response.status(), StatusCode::OK);
        let adjust_body = json_body(adjust_response).await;
        assert_eq!(adjust_body["status"], json!("ADJUSTED"));
        assert_eq!(adjust_body["reversed_journal_id"], json!(journal_id));
        assert_ne!(
            adjust_body["replacement_journal_id"],
            adjust_body["reversed_journal_id"]
        );
        assert!(adjust_body["audit_seal"].as_str().unwrap_or_default().len() > 8);
    }

    #[tokio::test]
    async fn revrec_rollforward_is_book_scoped() {
        let app = router();
        let mut ifrs_payload = order_payload(7000);
        ifrs_payload["ledger_book"] = json!("IFRS");
        ifrs_payload["source_event_id"] = json!("evt_ifrs_1");

        let us_post = app
            .clone()
            .oneshot(post_request("rollforward-us", &order_payload(10000)))
            .await
            .unwrap();
        assert_eq!(us_post.status(), StatusCode::OK);

        let ifrs_post = app
            .clone()
            .oneshot(post_request("rollforward-ifrs", &ifrs_payload))
            .await
            .unwrap();
        assert_eq!(ifrs_post.status(), StatusCode::OK);

        let rollforward = Request::builder()
            .method("GET")
            .uri("/v1/revrec/rollforward?book=US_GAAP")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(rollforward).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["book"], json!("US_GAAP"));
        assert_eq!(body["journal_count"], json!(1));
        assert_eq!(body["recognized_revenue_minor"], json!(10000));
    }

    #[tokio::test]
    async fn revrec_disclosures_include_policy_and_fx_sets() {
        let app = router();
        let order = app
            .clone()
            .oneshot(post_request("disclosure-order", &order_payload(10000)))
            .await
            .unwrap();
        assert_eq!(order.status(), StatusCode::OK);
        let refund = app
            .clone()
            .oneshot(post_request(
                "disclosure-refund",
                &sprint3_payload("refund.v1", 500),
            ))
            .await
            .unwrap();
        assert_eq!(refund.status(), StatusCode::OK);

        let disclosures = Request::builder()
            .method("GET")
            .uri("/v1/revrec/disclosures?book=US_GAAP")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(disclosures).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["book"], json!("US_GAAP"));
        assert_eq!(body["journal_count"], json!(2));
        assert_eq!(body["refund_contra_revenue_minor"], json!(-500));
        assert_eq!(body["policy_versions"], json!(["1.0.0"]));
        assert_eq!(body["fx_rate_sets"], json!(["fx_2026_02_21"]));
    }

    #[tokio::test]
    async fn ops_endpoints_are_available() {
        let app = router();

        let slo = Request::builder()
            .method("GET")
            .uri("/v1/ops/slo")
            .body(Body::empty())
            .unwrap();
        let slo_resp = app.clone().oneshot(slo).await.unwrap();
        assert_eq!(slo_resp.status(), StatusCode::OK);

        let capacity = Request::builder()
            .method("GET")
            .uri("/v1/ops/capacity")
            .body(Body::empty())
            .unwrap();
        let cap_resp = app.oneshot(capacity).await.unwrap();
        assert_eq!(cap_resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn capacity_endpoint_reports_no_bend_readiness() {
        let app = router();
        let request = Request::builder()
            .method("GET")
            .uri("/v1/ops/capacity")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;

        assert_eq!(body["target_active_users"], json!(2000));
        assert_eq!(body["baseline_rps"], json!(82));
        assert_eq!(body["peak_rps"], json!(329));
        assert_eq!(body["burst_rps"], json!(500));
        assert_eq!(body["readiness_status"], json!("READY"));
        assert_eq!(
            body["no_bend_readiness"]["target_active_users"],
            json!(2000)
        );
        assert_eq!(body["no_bend_readiness"]["linearity_ratio_min"], json!(0.8));
        assert_eq!(body["no_bend_readiness"]["comfortable"], json!(true));
        assert!(
            body["no_bend_readiness"]["measured_linearity_ratio"]
                .as_f64()
                .unwrap_or_default()
                >= 0.8
        );
        assert_eq!(body["scale_samples"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn capacity_instrumentation_returns_none_without_target_sample() {
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
        ];

        let response = build_capacity_instrumentation_response(samples);
        assert!(response.is_none());
    }
}
