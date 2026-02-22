use async_trait::async_trait;
use chrono::{DateTime, Utc};
use platform_core::payload_hash;
use serde_json::{json, Value};

use crate::{CanonicalEvent, CanonicalTraceContext, ConnectorAdapter, ConnectorError, RawEvent};

#[derive(Debug, Default, Clone)]
pub struct SquareAdapter;

#[derive(Debug, Clone, Copy)]
enum SquareEventKind {
    Sale,
    Refund,
    Tender,
    Payout,
}

impl SquareEventKind {
    fn event_type(self) -> &'static str {
        match self {
            Self::Sale => "order.captured.v1",
            Self::Refund => "refund.v1",
            Self::Tender => "payment.settled.v1",
            Self::Payout => "payout.cleared.v1",
        }
    }

    fn idempotency_suffix(self) -> &'static str {
        match self {
            Self::Sale => "sale",
            Self::Refund => "refund",
            Self::Tender => "tender",
            Self::Payout => "payout",
        }
    }
}

#[async_trait]
impl ConnectorAdapter for SquareAdapter {
    fn source_system(&self) -> &'static str {
        "square"
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
        let location_id = required_string(
            &payload,
            &[
                "/location_id",
                "/merchant_location_id",
                "/data/object/location_id",
                "/location/id",
                "/data/object/location/id",
            ],
            "location_id",
        )?;
        let kind = detect_event_kind(&payload)?;
        let canonical_payload = match kind {
            SquareEventKind::Sale => normalize_sale(&payload, occurred_at)?,
            SquareEventKind::Refund => normalize_refund(&payload, occurred_at)?,
            SquareEventKind::Tender => normalize_tender(&payload, occurred_at)?,
            SquareEventKind::Payout => normalize_payout(&payload, occurred_at)?,
        };
        let canonical_payload =
            with_routing_context(canonical_payload, &legal_entity_id, &location_id);

        let payload_digest = payload_hash(&canonical_payload);
        let event_id = format!(
            "square-{}-{}",
            source_event_id,
            &payload_digest[..12].to_ascii_lowercase()
        );

        let correlation_id = first_string(
            &canonical_payload,
            &[
                "/extensions/source_payload/correlation_id",
                "/extensions/source_payload/headers/x-correlation-id",
                "/order_id",
                "/payment_id",
                "/refund_id",
                "/tender_id",
                "/payout_id",
            ],
        )
        .unwrap_or_else(|| source_event_id.clone());

        let trace_context = CanonicalTraceContext {
            idempotency_key: first_string(
                &canonical_payload,
                &[
                    "/extensions/source_payload/idempotency_key",
                    "/extensions/source_payload/metadata/idempotency_key",
                    "/extensions/source_payload/request/idempotency_key",
                ],
            )
            .unwrap_or_else(|| format!("square:{source_event_id}:{}", kind.idempotency_suffix())),
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
            tenant_id,
            legal_entity_id,
            payload: canonical_payload,
            trace_context,
        })
    }
}

fn with_routing_context(
    mut canonical_payload: Value,
    legal_entity_id: &str,
    location_id: &str,
) -> Value {
    if let Some(object) = canonical_payload.as_object_mut() {
        object.insert(
            "location_id".to_string(),
            Value::String(location_id.to_string()),
        );
        object.insert(
            "routing".to_string(),
            json!({
                "legal_entity_id": legal_entity_id,
                "location_id": location_id
            }),
        );
    }
    canonical_payload
}

fn detect_event_kind(payload: &Value) -> Result<SquareEventKind, ConnectorError> {
    if let Some(kind) = first_string(
        payload,
        &[
            "/entity",
            "/record_type",
            "/kind",
            "/event_type",
            "/type",
            "/data/type",
            "/data/object/type",
        ],
    ) {
        let normalized = kind.to_ascii_lowercase();
        if normalized.contains("refund") {
            return Ok(SquareEventKind::Refund);
        }
        if normalized.contains("payout") {
            return Ok(SquareEventKind::Payout);
        }
        if normalized.contains("tender") {
            return Ok(SquareEventKind::Tender);
        }
        if normalized.contains("sale")
            || normalized.contains("payment")
            || normalized.contains("order")
        {
            return Ok(SquareEventKind::Sale);
        }
    }

    if has_any(
        payload,
        &[
            "/payout_id",
            "/data/object/payout_id",
            "/data/object/arrival_date",
        ],
    ) {
        return Ok(SquareEventKind::Payout);
    }
    if has_any(payload, &["/refund_id", "/data/object/refund_id"]) {
        return Ok(SquareEventKind::Refund);
    }
    if has_any(payload, &["/tender_id", "/data/object/tender_id"]) {
        return Ok(SquareEventKind::Tender);
    }
    if has_any(payload, &["/order_id", "/data/object/order_id"]) {
        return Ok(SquareEventKind::Sale);
    }

    Err(ConnectorError::Normalize(
        "unsupported square event kind".to_string(),
    ))
}

fn normalize_sale(payload: &Value, occurred_at: DateTime<Utc>) -> Result<Value, ConnectorError> {
    let order_id = required_string(payload, &["/order_id", "/data/object/order_id"], "order_id")?;
    let amount_minor = required_amount(
        payload,
        &[
            "/amount_minor",
            "/total_amount_minor",
            "/amount_money/amount",
            "/total_money/amount",
            "/data/object/amount_minor",
            "/data/object/amount_money/amount",
            "/data/object/total_money/amount",
        ],
        "amount_minor",
    )?;
    let currency = first_string(
        payload,
        &[
            "/currency",
            "/amount_money/currency",
            "/total_money/currency",
            "/data/object/currency",
            "/data/object/amount_money/currency",
            "/data/object/total_money/currency",
        ],
    )
    .unwrap_or_else(|| "USD".to_string());

    Ok(json!({
        "order_id": order_id,
        "payment_id": first_string(payload, &["/payment_id", "/data/object/payment_id"]),
        "order_status": first_string(payload, &["/order_status", "/data/object/order_status"])
            .unwrap_or_else(|| "CAPTURED".to_string()),
        "business_date": first_date_string(payload, &["/business_date", "/data/object/business_date", "/created_at", "/data/object/created_at"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string()),
        "currency": currency,
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
            "/refund_amount_minor",
            "/amount_money/amount",
            "/data/object/amount_minor",
            "/data/object/amount_money/amount",
        ],
        "amount_minor",
    )?;
    let currency = first_string(
        payload,
        &[
            "/currency",
            "/amount_money/currency",
            "/data/object/currency",
            "/data/object/amount_money/currency",
        ],
    )
    .unwrap_or_else(|| "USD".to_string());

    Ok(json!({
        "refund_id": refund_id,
        "payment_id": first_string(payload, &["/payment_id", "/data/object/payment_id"]),
        "refund_status": first_string(payload, &["/refund_status", "/status", "/data/object/refund_status", "/data/object/status"])
            .unwrap_or_else(|| "COMPLETED".to_string()),
        "business_date": first_date_string(payload, &["/business_date", "/data/object/business_date", "/created_at", "/data/object/created_at"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string()),
        "currency": currency,
        "amount_minor": amount_minor,
        "extensions": {
            "source_payload": payload
        }
    }))
}

fn normalize_tender(payload: &Value, occurred_at: DateTime<Utc>) -> Result<Value, ConnectorError> {
    let tender_id = required_string(
        payload,
        &[
            "/tender_id",
            "/id",
            "/data/object/tender_id",
            "/data/object/id",
        ],
        "tender_id",
    )?;
    let gross_amount_minor = required_amount(
        payload,
        &[
            "/gross_amount_minor",
            "/amount_minor",
            "/amount_money/amount",
            "/data/object/gross_amount_minor",
            "/data/object/amount_minor",
            "/data/object/amount_money/amount",
        ],
        "gross_amount_minor",
    )?;
    let fee_amount_minor = optional_i64(
        payload,
        &[
            "/fee_amount_minor",
            "/processing_fee_minor",
            "/data/object/fee_amount_minor",
            "/data/object/processing_fee_minor",
            "/processing_fee_money/amount",
            "/data/object/processing_fee_money/amount",
        ],
    )
    .unwrap_or(0)
    .abs();
    let net_amount_minor = optional_i64(
        payload,
        &[
            "/net_amount_minor",
            "/net_money/amount",
            "/data/object/net_amount_minor",
            "/data/object/net_money/amount",
        ],
    )
    .unwrap_or(gross_amount_minor - fee_amount_minor);
    if net_amount_minor < 0 {
        return Err(ConnectorError::Normalize(
            "invalid net amount for tender".to_string(),
        ));
    }

    let currency = first_string(
        payload,
        &[
            "/currency",
            "/amount_money/currency",
            "/net_money/currency",
            "/data/object/currency",
            "/data/object/amount_money/currency",
            "/data/object/net_money/currency",
        ],
    )
    .unwrap_or_else(|| "USD".to_string());

    Ok(json!({
        "tender_id": tender_id,
        "order_id": first_string(payload, &["/order_id", "/data/object/order_id"]),
        "payment_id": first_string(payload, &["/payment_id", "/data/object/payment_id"]),
        "business_date": first_date_string(payload, &["/business_date", "/data/object/business_date", "/created_at", "/data/object/created_at"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string()),
        "currency": currency,
        "gross_amount_minor": gross_amount_minor,
        "fee_amount_minor": fee_amount_minor,
        "net_amount_minor": net_amount_minor,
        "extensions": {
            "source_payload": payload
        }
    }))
}

fn normalize_payout(payload: &Value, occurred_at: DateTime<Utc>) -> Result<Value, ConnectorError> {
    let payout_id = required_string(
        payload,
        &[
            "/payout_id",
            "/id",
            "/data/object/payout_id",
            "/data/object/id",
        ],
        "payout_id",
    )?;
    let amount_minor = required_amount(
        payload,
        &[
            "/amount_minor",
            "/net_amount_minor",
            "/amount_money/amount",
            "/data/object/amount_minor",
            "/data/object/net_amount_minor",
            "/data/object/amount_money/amount",
        ],
        "amount_minor",
    )?;
    let currency = first_string(
        payload,
        &[
            "/currency",
            "/amount_money/currency",
            "/data/object/currency",
            "/data/object/amount_money/currency",
        ],
    )
    .unwrap_or_else(|| "USD".to_string());

    Ok(json!({
        "payout_id": payout_id,
        "payout_status": first_string(payload, &["/status", "/payout_status", "/data/object/status", "/data/object/payout_status"])
            .unwrap_or_else(|| "PAID".to_string()),
        "payout_date": first_date_string(payload, &["/payout_date", "/arrival_date", "/effective_at", "/data/object/payout_date", "/data/object/arrival_date", "/data/object/effective_at"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string()),
        "currency": currency,
        "amount_minor": amount_minor,
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

    use crate::{run_replay_backfill_resiliency, ConnectorAdapter, RawEvent};

    use super::SquareAdapter;

    #[tokio::test]
    async fn normalizes_square_sale_with_trace_context() {
        let adapter = SquareAdapter;
        let raw = crate::RawEvent {
            source_event_id: "sq_evt_sale_1".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "type": "payment.created",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "location_id": "BRECK_BASE_AREA",
                "order_id": "ord_123",
                "amount_money": {"amount": 17120, "currency": "USD"},
                "idempotency_key": "square:idem:sale",
                "correlation_id": "corr_sale",
                "trace": {
                    "traceparent": "00-4bf92f3577b34da6a3ce929d0e0e4736-aaaaaaaaaaaaaaaa-01"
                }
            }),
        };

        let canonical = adapter.normalize(raw).await.unwrap();
        assert_eq!(canonical.event_type, "order.captured.v1");
        assert_eq!(canonical.payload["order_id"], json!("ord_123"));
        assert_eq!(canonical.payload["amount_minor"], json!(17120));
        assert_eq!(
            canonical.payload["routing"]["location_id"],
            json!("BRECK_BASE_AREA")
        );
        assert_eq!(
            canonical.trace_context.idempotency_key,
            "square:idem:sale".to_string()
        );
        assert_eq!(canonical.trace_context.correlation_id, "corr_sale");
        assert!(canonical.trace_context.traceparent.is_some());
    }

    #[tokio::test]
    async fn normalizes_square_refund() {
        let adapter = SquareAdapter;
        let raw = crate::RawEvent {
            source_event_id: "sq_evt_refund_1".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "entity": "refund",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "location_id": "BRECK_BASE_AREA",
                "refund_id": "rf_789",
                "payment_id": "pay_123",
                "amount_minor": 2500,
                "currency": "USD"
            }),
        };

        let canonical = adapter.normalize(raw).await.unwrap();
        assert_eq!(canonical.event_type, "refund.v1");
        assert_eq!(canonical.payload["refund_id"], json!("rf_789"));
        assert_eq!(canonical.payload["amount_minor"], json!(2500));
        assert_eq!(
            canonical.trace_context.idempotency_key,
            "square:sq_evt_refund_1:refund"
        );
    }

    #[tokio::test]
    async fn normalizes_square_tender_with_fee_math() {
        let adapter = SquareAdapter;
        let raw = crate::RawEvent {
            source_event_id: "sq_evt_tender_1".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "kind": "tender",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "location_id": "BRECK_BASE_AREA",
                "tender_id": "tnd_1",
                "order_id": "ord_123",
                "gross_amount_minor": 10000,
                "fee_amount_minor": 300,
                "currency": "USD"
            }),
        };

        let canonical = adapter.normalize(raw).await.unwrap();
        assert_eq!(canonical.event_type, "payment.settled.v1");
        assert_eq!(canonical.payload["gross_amount_minor"], json!(10000));
        assert_eq!(canonical.payload["fee_amount_minor"], json!(300));
        assert_eq!(canonical.payload["net_amount_minor"], json!(9700));
    }

    #[tokio::test]
    async fn normalizes_square_payout() {
        let adapter = SquareAdapter;
        let raw = crate::RawEvent {
            source_event_id: "sq_evt_payout_1".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "event_type": "payout.paid",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "location_id": "BRECK_BASE_AREA",
                "payout_id": "po_456",
                "amount_minor": 9750,
                "currency": "USD"
            }),
        };

        let canonical = adapter.normalize(raw).await.unwrap();
        assert_eq!(canonical.event_type, "payout.cleared.v1");
        assert_eq!(canonical.payload["payout_id"], json!("po_456"));
        assert_eq!(canonical.payload["amount_minor"], json!(9750));
        assert_eq!(canonical.trace_context.correlation_id, "po_456");
    }

    #[tokio::test]
    async fn normalize_fails_for_unknown_event_kind() {
        let adapter = SquareAdapter;
        let raw = crate::RawEvent {
            source_event_id: "sq_evt_unknown_1".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "location_id": "BRECK_BASE_AREA",
                "event_type": "inventory.adjusted"
            }),
        };

        let error = adapter.normalize(raw).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "normalization failed: unsupported square event kind"
        );
    }

    #[tokio::test]
    async fn normalize_fails_when_location_is_missing() {
        let adapter = SquareAdapter;
        let raw = crate::RawEvent {
            source_event_id: "sq_evt_sale_2".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "type": "payment.created",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "order_id": "ord_123",
                "amount_money": {"amount": 17120, "currency": "USD"}
            }),
        };

        let error = adapter.normalize(raw).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "normalization failed: missing field `location_id`"
        );
    }

    #[tokio::test]
    async fn replay_backfill_resiliency_meets_target_for_square() {
        let adapter = SquareAdapter;
        let events = vec![
            RawEvent {
                source_event_id: "sq_evt_sale_900".to_string(),
                occurred_at: Utc::now(),
                payload: json!({
                    "type": "payment.created",
                    "tenant_id": "tenant_1",
                    "legal_entity_id": "US_CO_01",
                    "location_id": "BRECK_BASE_AREA",
                    "order_id": "ord_900",
                    "amount_money": {"amount": 9999, "currency": "USD"}
                }),
            },
            RawEvent {
                source_event_id: "sq_evt_refund_901".to_string(),
                occurred_at: Utc::now(),
                payload: json!({
                    "entity": "refund",
                    "tenant_id": "tenant_1",
                    "legal_entity_id": "US_CO_01",
                    "location_id": "BRECK_BASE_AREA",
                    "refund_id": "rf_901",
                    "payment_id": "pay_901",
                    "amount_minor": 500,
                    "currency": "USD"
                }),
            },
        ];
        let failures = BTreeSet::from([0_usize]);

        let result = run_replay_backfill_resiliency(&adapter, &events, &failures, 1000, 150)
            .await
            .unwrap();
        assert_eq!(result.hashes.len(), 2);
        assert_eq!(result.telemetry.first_attempt_failures, 1);
        assert_eq!(result.telemetry.recovered_events, 1);
        assert!(result.telemetry.objective_met);
    }
}
