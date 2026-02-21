use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use ledger_posting::{
    EntrySide, InMemoryJournalRepository, JournalHeader, JournalLine, JournalRecord, JournalStatus,
    LedgerError,
};
use platform_core::{payload_hash, IdempotencyError, IdempotencyStatus, InMemoryIdempotencyStore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::period::{InMemoryPeriodRepository, PeriodError};
use crate::rule_engine::{derive_lines_v1, RuleEngineError};

pub mod period;
pub mod rule_engine;

#[derive(Clone)]
pub struct AppState {
    idempotency: InMemoryIdempotencyStore,
    journals: Arc<Mutex<InMemoryJournalRepository>>,
    periods: Arc<Mutex<InMemoryPeriodRepository>>,
    post_results: Arc<Mutex<HashMap<String, CachedPostResult>>>,
    location_allowlist_by_legal_entity: Arc<HashMap<String, HashSet<String>>>,
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

pub fn router() -> Router {
    router_with_state(AppState::default())
}

pub fn router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/v1/posting/events", post(post_event))
        .route(
            "/v1/ledger/journals/:journal_id/reverse",
            post(reverse_journal),
        )
        .route(
            "/v1/ledger/periods/:period_id/lock",
            post(lock_period_endpoint),
        )
        .route("/v1/ops/slo", get(get_slo))
        .route("/v1/ops/capacity", get(get_capacity))
        .with_state(state)
}

type ApiError = (StatusCode, serde_json::Value);

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
            tenant_id: req.tenant_id,
            legal_entity_id: req.legal_entity_id,
            ledger_book: req.ledger_book,
            accounting_date,
            posted_at: chrono::Utc::now(),
            source_event_ids: vec![req.source_event_id],
            posting_run_id: req.posting_run_id,
            book_policy_id: req.provenance.book_policy_id,
            policy_version: req.provenance.policy_version,
            fx_rate_set_id: req.provenance.fx_rate_set_id,
            ruleset_version: req.provenance.ruleset_version,
            workflow_id: req.provenance.workflow_id,
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

async fn get_capacity() -> Json<serde_json::Value> {
    Json(json!({
        "target_active_users": 2000,
        "baseline_rps": 167,
        "peak_rps": 330,
        "burst_rps": 500
    }))
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

    req.lines
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
}
