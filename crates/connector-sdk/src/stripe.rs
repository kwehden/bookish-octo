use async_trait::async_trait;
use chrono::{DateTime, Utc};
use platform_core::payload_hash;
use serde_json::{json, Value};

use crate::{CanonicalEvent, CanonicalTraceContext, ConnectorAdapter, ConnectorError, RawEvent};

#[derive(Debug, Default, Clone)]
pub struct StripeAdapter;

#[derive(Debug, Clone, Copy)]
enum StripeEventKind {
    ChargeCaptured,
    Refund,
    Settlement,
}

impl StripeEventKind {
    fn event_type(self) -> &'static str {
        match self {
            Self::ChargeCaptured => "order.captured.v1",
            Self::Refund => "refund.v1",
            Self::Settlement => "payment.settled.v1",
        }
    }

    fn idempotency_suffix(self) -> &'static str {
        match self {
            Self::ChargeCaptured => "charge_captured",
            Self::Refund => "refund",
            Self::Settlement => "settlement",
        }
    }
}

#[async_trait]
impl ConnectorAdapter for StripeAdapter {
    fn source_system(&self) -> &'static str {
        "stripe"
    }

    async fn normalize(&self, raw: RawEvent) -> Result<CanonicalEvent, ConnectorError> {
        let RawEvent {
            source_event_id,
            occurred_at,
            payload,
        } = raw;

        let tenant_id = required_string(
            &payload,
            &["/tenant_id", "/data/object/tenant_id"],
            "tenant_id",
        )?;
        let legal_entity_id = required_string(
            &payload,
            &["/legal_entity_id", "/data/object/legal_entity_id"],
            "legal_entity_id",
        )?;
        let location_id = first_string(
            &payload,
            &[
                "/location_id",
                "/merchant_location_id",
                "/data/object/location_id",
            ],
        );

        let kind = detect_event_kind(&payload)?;
        let canonical_payload = match kind {
            StripeEventKind::ChargeCaptured => normalize_charge(&payload, occurred_at)?,
            StripeEventKind::Refund => normalize_refund(&payload, occurred_at)?,
            StripeEventKind::Settlement => normalize_settlement(&payload, occurred_at)?,
        };
        let canonical_payload =
            with_routing_context(canonical_payload, &legal_entity_id, location_id);

        let payload_digest = payload_hash(&canonical_payload);
        let event_id = format!(
            "stripe-{}-{}",
            source_event_id,
            &payload_digest[..12].to_ascii_lowercase()
        );
        let business_date = first_string(&canonical_payload, &["/business_date", "/payout_date"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string());
        let idempotency_key = first_string(
            &canonical_payload,
            &[
                "/extensions/source_payload/idempotency_key",
                "/extensions/source_payload/request/idempotency_key",
                "/extensions/source_payload/metadata/idempotency_key",
            ],
        )
        .unwrap_or_else(|| format!("stripe:{source_event_id}:{}", kind.idempotency_suffix()));
        let correlation_id = first_string(
            &canonical_payload,
            &[
                "/extensions/source_payload/correlation_id",
                "/extensions/source_payload/request/id",
                "/charge_id",
                "/refund_id",
                "/balance_transaction_id",
                "/payout_id",
            ],
        )
        .unwrap_or_else(|| source_event_id.clone());

        let trace_context = CanonicalTraceContext {
            idempotency_key: idempotency_key.clone(),
            correlation_id,
            causation_id: first_string(
                &canonical_payload,
                &[
                    "/extensions/source_payload/causation_id",
                    "/extensions/source_payload/parent_event_id",
                ],
            ),
            traceparent: first_string(
                &canonical_payload,
                &[
                    "/extensions/source_payload/trace/traceparent",
                    "/extensions/source_payload/traceparent",
                    "/extensions/source_payload/headers/traceparent",
                ],
            ),
            tracestate: first_string(
                &canonical_payload,
                &[
                    "/extensions/source_payload/trace/tracestate",
                    "/extensions/source_payload/tracestate",
                    "/extensions/source_payload/headers/tracestate",
                ],
            ),
        };

        Ok(CanonicalEvent {
            event_id,
            event_type: kind.event_type().to_string(),
            schema_version: "1.0.0".to_string(),
            source_system: self.source_system().to_string(),
            source_event_id,
            occurred_at,
            business_date,
            tenant_id,
            legal_entity_id,
            idempotency_key,
            payload: canonical_payload,
            trace_context,
        })
    }
}

fn with_routing_context(
    mut canonical_payload: Value,
    legal_entity_id: &str,
    location_id: Option<String>,
) -> Value {
    if let Some(object) = canonical_payload.as_object_mut() {
        object.insert(
            "routing".to_string(),
            json!({
                "legal_entity_id": legal_entity_id,
                "location_id": location_id
            }),
        );
        if let Some(location_id) = location_id {
            object.insert("location_id".to_string(), Value::String(location_id));
        }
    }
    canonical_payload
}

fn detect_event_kind(payload: &Value) -> Result<StripeEventKind, ConnectorError> {
    if let Some(kind) = first_string(
        payload,
        &["/type", "/event_type", "/record_type", "/data/object/type"],
    ) {
        let normalized = kind.to_ascii_lowercase();
        if normalized.contains("refund") {
            return Ok(StripeEventKind::Refund);
        }
        if normalized.contains("payout")
            || normalized.contains("balance")
            || normalized.contains("settlement")
        {
            return Ok(StripeEventKind::Settlement);
        }
        if normalized.contains("charge")
            || normalized.contains("payment_intent")
            || normalized.contains("payment")
        {
            return Ok(StripeEventKind::ChargeCaptured);
        }
    }

    if has_any(payload, &["/refund_id", "/data/object/refund_id"]) {
        return Ok(StripeEventKind::Refund);
    }
    if has_any(
        payload,
        &[
            "/payout_id",
            "/balance_transaction_id",
            "/data/object/payout_id",
            "/data/object/balance_transaction_id",
        ],
    ) {
        return Ok(StripeEventKind::Settlement);
    }
    if has_any(
        payload,
        &["/charge_id", "/payment_intent_id", "/data/object/charge_id"],
    ) {
        return Ok(StripeEventKind::ChargeCaptured);
    }

    Err(ConnectorError::Normalize(
        "unsupported stripe event kind".to_string(),
    ))
}

fn normalize_charge(payload: &Value, occurred_at: DateTime<Utc>) -> Result<Value, ConnectorError> {
    let charge_id = required_string(
        payload,
        &[
            "/charge_id",
            "/id",
            "/payment_intent_id",
            "/data/object/charge_id",
            "/data/object/id",
            "/data/object/payment_intent_id",
        ],
        "charge_id",
    )?;
    let amount_minor = required_amount(
        payload,
        &[
            "/amount_minor",
            "/amount",
            "/amount_captured",
            "/data/object/amount_minor",
            "/data/object/amount",
            "/data/object/amount_captured",
        ],
        "amount_minor",
    )?;

    Ok(json!({
        "charge_id": charge_id,
        "payment_intent_id": first_string(payload, &["/payment_intent_id", "/data/object/payment_intent_id"]),
        "order_id": first_string(payload, &["/order_id", "/data/object/order_id", "/metadata/order_id", "/data/object/metadata/order_id"]),
        "charge_status": first_string(payload, &["/status", "/data/object/status"])
            .unwrap_or_else(|| "SUCCEEDED".to_string()),
        "business_date": first_date_string(payload, &["/business_date", "/created", "/created_at", "/data/object/business_date", "/data/object/created", "/data/object/created_at"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string()),
        "currency": first_string(payload, &["/currency", "/data/object/currency"])
            .unwrap_or_else(|| "USD".to_string())
            .to_ascii_uppercase(),
        "amount_minor": amount_minor,
        "extensions": {
            "source_payload": payload
        }
    }))
}

fn normalize_refund(payload: &Value, occurred_at: DateTime<Utc>) -> Result<Value, ConnectorError> {
    let refund_id = required_string(
        payload,
        &[
            "/refund_id",
            "/id",
            "/data/object/refund_id",
            "/data/object/id",
        ],
        "refund_id",
    )?;
    let amount_minor = required_amount(
        payload,
        &[
            "/amount_minor",
            "/amount",
            "/data/object/amount_minor",
            "/data/object/amount",
        ],
        "amount_minor",
    )?;

    Ok(json!({
        "refund_id": refund_id,
        "charge_id": first_string(payload, &["/charge_id", "/payment_id", "/data/object/charge_id", "/data/object/payment_id"]),
        "refund_status": first_string(payload, &["/status", "/data/object/status"])
            .unwrap_or_else(|| "SUCCEEDED".to_string()),
        "business_date": first_date_string(payload, &["/business_date", "/created", "/created_at", "/data/object/business_date", "/data/object/created", "/data/object/created_at"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string()),
        "currency": first_string(payload, &["/currency", "/data/object/currency"])
            .unwrap_or_else(|| "USD".to_string())
            .to_ascii_uppercase(),
        "amount_minor": amount_minor,
        "extensions": {
            "source_payload": payload
        }
    }))
}

fn normalize_settlement(
    payload: &Value,
    occurred_at: DateTime<Utc>,
) -> Result<Value, ConnectorError> {
    let balance_transaction_id = required_string(
        payload,
        &[
            "/balance_transaction_id",
            "/id",
            "/data/object/balance_transaction_id",
            "/data/object/id",
        ],
        "balance_transaction_id",
    )?;
    let net_amount_minor = required_amount(
        payload,
        &[
            "/net_amount_minor",
            "/net",
            "/amount_minor",
            "/amount",
            "/data/object/net_amount_minor",
            "/data/object/net",
            "/data/object/amount_minor",
            "/data/object/amount",
        ],
        "net_amount_minor",
    )?;
    let fee_amount_minor = optional_i64(
        payload,
        &[
            "/fee_amount_minor",
            "/fee",
            "/data/object/fee_amount_minor",
            "/data/object/fee",
        ],
    )
    .unwrap_or(0)
    .abs();
    let gross_amount_minor = optional_i64(
        payload,
        &[
            "/gross_amount_minor",
            "/gross",
            "/data/object/gross_amount_minor",
            "/data/object/gross",
        ],
    )
    .unwrap_or(net_amount_minor + fee_amount_minor)
    .abs();

    Ok(json!({
        "balance_transaction_id": balance_transaction_id,
        "payout_id": first_string(payload, &["/payout_id", "/data/object/payout_id"]),
        "transaction_type": first_string(payload, &["/transaction_type", "/type", "/data/object/transaction_type", "/data/object/type"])
            .unwrap_or_else(|| "charge".to_string()),
        "payout_date": first_date_string(payload, &["/payout_date", "/available_on", "/data/object/payout_date", "/data/object/available_on"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string()),
        "currency": first_string(payload, &["/currency", "/data/object/currency"])
            .unwrap_or_else(|| "USD".to_string())
            .to_ascii_uppercase(),
        "gross_amount_minor": gross_amount_minor,
        "fee_amount_minor": fee_amount_minor,
        "net_amount_minor": net_amount_minor,
        "extensions": {
            "source_payload": payload
        }
    }))
}

fn required_string(
    payload: &Value,
    pointers: &[&str],
    field_name: &'static str,
) -> Result<String, ConnectorError> {
    first_string(payload, pointers)
        .ok_or_else(|| ConnectorError::Normalize(format!("missing field `{field_name}`")))
}

fn required_amount(
    payload: &Value,
    pointers: &[&str],
    field_name: &'static str,
) -> Result<i64, ConnectorError> {
    let value = optional_i64(payload, pointers)
        .ok_or_else(|| ConnectorError::Normalize(format!("missing field `{field_name}`")))?;
    let normalized = value.abs();
    if normalized == 0 {
        return Err(ConnectorError::Normalize(format!(
            "invalid amount for `{field_name}`"
        )));
    }
    Ok(normalized)
}

fn first_string(payload: &Value, pointers: &[&str]) -> Option<String> {
    pointers
        .iter()
        .find_map(|pointer| payload.pointer(pointer).and_then(Value::as_str))
        .map(ToString::to_string)
}

fn first_date_string(payload: &Value, pointers: &[&str]) -> Option<String> {
    pointers.iter().find_map(|pointer| {
        payload.pointer(pointer).and_then(|value| match value {
            Value::String(string) => string.get(0..10).map(ToString::to_string),
            Value::Number(number) => number
                .as_i64()
                .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0))
                .map(|timestamp| timestamp.format("%Y-%m-%d").to_string()),
            _ => None,
        })
    })
}

fn optional_i64(payload: &Value, pointers: &[&str]) -> Option<i64> {
    pointers.iter().find_map(|pointer| {
        payload.pointer(pointer).and_then(|value| match value {
            Value::Number(number) => number.as_i64(),
            Value::String(string) => string.parse().ok(),
            _ => None,
        })
    })
}

fn has_any(payload: &Value, pointers: &[&str]) -> bool {
    pointers
        .iter()
        .any(|pointer| payload.pointer(pointer).is_some())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use chrono::Utc;
    use serde_json::json;

    use crate::{
        evaluate_cutover_rehearsal, run_replay_backfill_resiliency, ConnectorAdapter,
        CutoverCheckpoint, RawEvent,
    };

    use super::StripeAdapter;

    #[tokio::test]
    async fn normalizes_stripe_charge_event() {
        let adapter = StripeAdapter;
        let raw = RawEvent {
            source_event_id: "st_evt_charge_1".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "type": "charge.succeeded",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "location_id": "BRECK_BASE_AREA",
                "charge_id": "ch_123",
                "amount": 17120,
                "currency": "usd",
                "idempotency_key": "stripe:idem:charge",
                "correlation_id": "corr_stripe_charge",
                "trace": {
                    "traceparent": "00-4bf92f3577b34da6a3ce929d0e0e4736-cccccccccccccccc-01"
                }
            }),
        };

        let canonical = adapter.normalize(raw).await.unwrap();
        assert_eq!(canonical.event_type, "order.captured.v1");
        assert_eq!(canonical.payload["charge_id"], json!("ch_123"));
        assert_eq!(canonical.payload["amount_minor"], json!(17120));
        assert_eq!(canonical.business_date.len(), 10);
        assert_eq!(canonical.idempotency_key, "stripe:idem:charge");
        assert_eq!(canonical.trace_context.correlation_id, "corr_stripe_charge");
    }

    #[tokio::test]
    async fn normalizes_stripe_refund_event() {
        let adapter = StripeAdapter;
        let raw = RawEvent {
            source_event_id: "st_evt_refund_1".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "type": "charge.refunded",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "refund_id": "re_456",
                "charge_id": "ch_123",
                "amount": 2500,
                "currency": "usd"
            }),
        };

        let canonical = adapter.normalize(raw).await.unwrap();
        assert_eq!(canonical.event_type, "refund.v1");
        assert_eq!(canonical.payload["refund_id"], json!("re_456"));
        assert_eq!(canonical.payload["amount_minor"], json!(2500));
        assert_eq!(canonical.idempotency_key, "stripe:st_evt_refund_1:refund");
    }

    #[tokio::test]
    async fn replay_backfill_resiliency_meets_target_for_stripe() {
        let adapter = StripeAdapter;
        let events = vec![
            RawEvent {
                source_event_id: "st_evt_charge_900".to_string(),
                occurred_at: Utc::now(),
                payload: json!({
                    "type": "charge.succeeded",
                    "tenant_id": "tenant_1",
                    "legal_entity_id": "US_CO_01",
                    "charge_id": "ch_900",
                    "amount": 9999,
                    "currency": "usd"
                }),
            },
            RawEvent {
                source_event_id: "st_evt_refund_901".to_string(),
                occurred_at: Utc::now(),
                payload: json!({
                    "type": "charge.refunded",
                    "tenant_id": "tenant_1",
                    "legal_entity_id": "US_CO_01",
                    "refund_id": "re_901",
                    "charge_id": "ch_900",
                    "amount": 500,
                    "currency": "usd"
                }),
            },
        ];
        let failures = BTreeSet::from([1_usize]);

        let result = run_replay_backfill_resiliency(&adapter, &events, &failures, 1000, 100)
            .await
            .unwrap();
        assert_eq!(result.hashes.len(), 2);
        assert_eq!(result.telemetry.first_attempt_failures, 1);
        assert_eq!(result.telemetry.recovered_events, 1);
        assert!(result.telemetry.objective_met);
    }

    #[tokio::test]
    async fn cutover_rehearsal_passes_for_stripe_when_all_checkpoints_pass() {
        let adapter = StripeAdapter;
        let events = vec![RawEvent {
            source_event_id: "st_evt_charge_950".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "type": "charge.succeeded",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "charge_id": "ch_950",
                "amount": 8800,
                "currency": "usd"
            }),
        }];

        let replay = run_replay_backfill_resiliency(&adapter, &events, &BTreeSet::new(), 1000, 100)
            .await
            .unwrap();
        let rehearsal = evaluate_cutover_rehearsal(
            &replay,
            true,
            &[CutoverCheckpoint {
                name: "stripe_cutover_dry_run".to_string(),
                passed: true,
            }],
        );

        assert!(rehearsal.passed);
        assert!(rehearsal.replay_objective_met);
        assert!(rehearsal.rollback_validated);
    }
}
