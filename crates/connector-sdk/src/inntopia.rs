use async_trait::async_trait;
use platform_core::payload_hash;
use serde_json::{json, Value};

use crate::{CanonicalEvent, CanonicalTraceContext, ConnectorAdapter, ConnectorError, RawEvent};

#[derive(Debug, Default, Clone)]
pub struct InntopiaAdapter;

#[async_trait]
impl ConnectorAdapter for InntopiaAdapter {
    fn source_system(&self) -> &'static str {
        "inntopia"
    }

    async fn normalize(&self, raw: RawEvent) -> Result<CanonicalEvent, ConnectorError> {
        let RawEvent {
            source_event_id,
            occurred_at,
            payload,
        } = raw;

        let reservation_id = required_string(&payload, "reservation_id")?;
        let tenant_id = required_string(&payload, "tenant_id")?;
        let legal_entity_id = required_string(&payload, "legal_entity_id")?;

        let total_amount_minor = first_i64(
            &payload,
            &[
                "/total_amount_minor",
                "/amount_minor",
                "/totals/grand_total_minor",
            ],
        )
        .ok_or_else(|| ConnectorError::Normalize("missing total amount".to_string()))?;
        let currency = first_string(
            &payload,
            &[
                "/currency",
                "/totals/currency",
                "/totals/grand_total/currency",
            ],
        )
        .unwrap_or_else(|| "USD".to_string());
        let business_date = first_string(&payload, &["/business_date"])
            .unwrap_or_else(|| occurred_at.format("%Y-%m-%d").to_string());

        let canonical_payload = json!({
            "reservation_id": reservation_id,
            "reservation_status": first_string(&payload, &["/reservation_status"])
                .unwrap_or_else(|| "CAPTURED".to_string()),
            "business_date": business_date,
            "currency": currency,
            "total_amount_minor": total_amount_minor,
            "arrival_date": payload.get("arrival_date"),
            "departure_date": payload.get("departure_date"),
            "extensions": {
                "source_payload": payload
            }
        });

        let payload_digest = payload_hash(&canonical_payload);
        let event_id = format!(
            "inntopia-{}-{}",
            source_event_id,
            &payload_digest[..12].to_ascii_lowercase()
        );

        let trace_context = CanonicalTraceContext {
            idempotency_key: first_string(
                &canonical_payload,
                &["/extensions/source_payload/idempotency_key"],
            )
            .unwrap_or_else(|| format!("inntopia:{source_event_id}:reservation.captured")),
            correlation_id: first_string(
                &canonical_payload,
                &["/extensions/source_payload/correlation_id"],
            )
            .unwrap_or_else(|| reservation_id.clone()),
            causation_id: first_string(
                &canonical_payload,
                &["/extensions/source_payload/causation_id"],
            ),
            traceparent: first_string(
                &canonical_payload,
                &[
                    "/extensions/source_payload/trace/traceparent",
                    "/extensions/source_payload/traceparent",
                ],
            ),
            tracestate: first_string(
                &canonical_payload,
                &[
                    "/extensions/source_payload/trace/tracestate",
                    "/extensions/source_payload/tracestate",
                ],
            ),
        };

        Ok(CanonicalEvent {
            event_id,
            event_type: "inntopia.reservation.captured.v1".to_string(),
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

fn required_string(payload: &Value, key: &str) -> Result<String, ConnectorError> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| ConnectorError::Normalize(format!("missing field `{key}`")))
}

fn first_string(payload: &Value, pointers: &[&str]) -> Option<String> {
    pointers
        .iter()
        .find_map(|pointer| payload.pointer(pointer).and_then(Value::as_str))
        .map(ToString::to_string)
}

fn first_i64(payload: &Value, pointers: &[&str]) -> Option<i64> {
    pointers.iter().find_map(|pointer| {
        payload.pointer(pointer).and_then(|value| match value {
            Value::Number(number) => number.as_i64(),
            Value::String(string) => string.parse().ok(),
            _ => None,
        })
    })
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use crate::ConnectorAdapter;

    use super::InntopiaAdapter;

    #[tokio::test]
    async fn normalizes_inntopia_reservation_with_trace_context() {
        let adapter = InntopiaAdapter;
        let raw = crate::RawEvent {
            source_event_id: "evt_123".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "reservation_id": "resv_001",
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01",
                "total_amount_minor": 41250,
                "currency": "USD",
                "business_date": "2026-02-21",
                "idempotency_key": "inntopia:evt_123",
                "correlation_id": "corr_456",
                "trace": {
                    "traceparent": "00-4bf92f3577b34da6a3ce929d0e0e4736-1111111111111111-01"
                }
            }),
        };

        let canonical = adapter.normalize(raw).await.unwrap();
        assert_eq!(canonical.event_type, "inntopia.reservation.captured.v1");
        assert_eq!(canonical.trace_context.idempotency_key, "inntopia:evt_123");
        assert_eq!(canonical.trace_context.correlation_id, "corr_456");
        assert_eq!(
            canonical.payload["total_amount_minor"].as_i64(),
            Some(41250)
        );
    }

    #[tokio::test]
    async fn normalize_fails_when_required_fields_missing() {
        let adapter = InntopiaAdapter;
        let raw = crate::RawEvent {
            source_event_id: "evt_123".to_string(),
            occurred_at: Utc::now(),
            payload: json!({
                "tenant_id": "tenant_1",
                "legal_entity_id": "US_CO_01"
            }),
        };

        let error = adapter.normalize(raw).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "normalization failed: missing field `reservation_id`"
        );
    }
}
