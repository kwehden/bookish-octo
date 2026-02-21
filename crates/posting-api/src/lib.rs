use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{NaiveDate, Utc};
use ledger_posting::{
    EntrySide, InMemoryJournalRepository, JournalHeader, JournalLine, JournalRecord, JournalStatus,
};
use platform_core::{payload_hash, IdempotencyError, IdempotencyStatus, InMemoryIdempotencyStore};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    idempotency: InMemoryIdempotencyStore,
    journals: Arc<Mutex<InMemoryJournalRepository>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            idempotency: InMemoryIdempotencyStore::default(),
            journals: Arc::new(Mutex::new(InMemoryJournalRepository::default())),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostEventRequest {
    pub event_type: String,
    pub tenant_id: String,
    pub legal_entity_id: String,
    pub ledger_book: String,
    pub accounting_date: String,
    pub source_event_id: String,
    pub posting_run_id: String,
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

#[derive(Debug, Serialize)]
pub struct PostEventResponse {
    pub journal_id: String,
    pub status: String,
    pub replayed: bool,
}

pub fn router() -> Router {
    Router::new()
        .route("/v1/posting/events", post(post_event))
        .route("/v1/ops/slo", get(get_slo))
        .route("/v1/ops/capacity", get(get_capacity))
        .with_state(AppState::default())
}

async fn post_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PostEventRequest>,
) -> Result<Json<PostEventResponse>, (StatusCode, Json<serde_json::Value>)> {
    if !["order.captured.v1", "payment.settled.v1", "refund.v1"].contains(&req.event_type.as_str())
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
    let accounting_date =
        NaiveDate::parse_from_str(&req.accounting_date, "%Y-%m-%d").map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_accounting_date"})),
            )
        })?;

    if idem_status == IdempotencyStatus::FirstSeen {
        let lines = req
            .lines
            .iter()
            .enumerate()
            .map(|(i, l)| JournalLine {
                line_number: (i + 1) as u32,
                account_id: l.account_id.clone(),
                entry_side: if l.entry_side.eq_ignore_ascii_case("debit") {
                    EntrySide::Debit
                } else {
                    EntrySide::Credit
                },
                amount_minor: l.amount_minor,
                currency: l.currency.clone(),
                base_amount_minor: l.base_amount_minor,
                base_currency: l.base_currency.clone(),
            })
            .collect::<Vec<_>>();

        let record = JournalRecord {
            header: JournalHeader {
                journal_id: journal_uuid,
                journal_number: format!("S1-{}", &journal_uuid.to_string()[..8]),
                status: JournalStatus::Posted,
                tenant_id: req.tenant_id,
                legal_entity_id: req.legal_entity_id,
                ledger_book: req.ledger_book,
                accounting_date,
                posted_at: Utc::now(),
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
                Json(json!({"error": "journal_store_error"})),
            )
        })?;

        repo.insert_posted(record).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    }

    Ok(Json(PostEventResponse {
        journal_id: journal_uuid.to_string(),
        status: "POSTED".to_string(),
        replayed: idem_status == IdempotencyStatus::Replay,
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
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;

    fn valid_payload(amount: i64) -> serde_json::Value {
        json!({
            "event_type": "order.captured.v1",
            "tenant_id": "tenant_1",
            "legal_entity_id": "US_CO_01",
            "ledger_book": "US_GAAP",
            "accounting_date": "2026-02-21",
            "source_event_id": "evt_1",
            "posting_run_id": "run_1",
            "lines": [
                {"account_id": "1105", "entry_side": "debit", "amount_minor": amount, "currency": "USD", "base_amount_minor": amount, "base_currency": "USD"},
                {"account_id": "4000", "entry_side": "credit", "amount_minor": amount, "currency": "USD", "base_amount_minor": amount, "base_currency": "USD"}
            ],
            "provenance": {
                "book_policy_id": "policy_dual_book",
                "policy_version": "1.0.0",
                "fx_rate_set_id": "fx_2026_02_21",
                "ruleset_version": "v1",
                "workflow_id": "wf_1"
            }
        })
    }

    #[tokio::test]
    async fn duplicate_same_payload_replays() {
        let app = router();
        let payload = valid_payload(10000);

        let request = Request::builder()
            .method("POST")
            .uri("/v1/posting/events")
            .header("content-type", "application/json")
            .header("Idempotency-Key", "same-key")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let first = app.clone().oneshot(request).await.unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let request = Request::builder()
            .method("POST")
            .uri("/v1/posting/events")
            .header("content-type", "application/json")
            .header("Idempotency-Key", "same-key")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let second = app.oneshot(request).await.unwrap();
        assert_eq!(second.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn duplicate_different_payload_conflicts() {
        let app = router();

        let request = Request::builder()
            .method("POST")
            .uri("/v1/posting/events")
            .header("content-type", "application/json")
            .header("Idempotency-Key", "same-key")
            .body(Body::from(valid_payload(10000).to_string()))
            .unwrap();
        let first = app.clone().oneshot(request).await.unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let request = Request::builder()
            .method("POST")
            .uri("/v1/posting/events")
            .header("content-type", "application/json")
            .header("Idempotency-Key", "same-key")
            .body(Body::from(valid_payload(9000).to_string()))
            .unwrap();

        let second = app.oneshot(request).await.unwrap();
        assert_eq!(second.status(), StatusCode::CONFLICT);
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
