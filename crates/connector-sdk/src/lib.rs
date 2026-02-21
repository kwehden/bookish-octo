use async_trait::async_trait;
use chrono::{DateTime, Utc};
use platform_core::payload_hash;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub mod inntopia;
pub use inntopia::InntopiaAdapter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawEvent {
    pub source_event_id: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalTraceContext {
    pub idempotency_key: String,
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub traceparent: Option<String>,
    pub tracestate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanonicalEvent {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: String,
    pub source_system: String,
    pub source_event_id: String,
    pub tenant_id: String,
    pub legal_entity_id: String,
    pub payload: Value,
    pub trace_context: CanonicalTraceContext,
}

#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("normalization failed: {0}")]
    Normalize(String),
}

#[async_trait]
pub trait ConnectorAdapter {
    fn source_system(&self) -> &'static str;
    async fn normalize(&self, raw: RawEvent) -> Result<CanonicalEvent, ConnectorError>;
}

pub async fn replay_hashes<A: ConnectorAdapter + Sync>(
    adapter: &A,
    raw_events: &[RawEvent],
) -> Result<Vec<String>, ConnectorError> {
    let mut hashes = Vec::with_capacity(raw_events.len());
    for raw in raw_events {
        let canonical = adapter.normalize(raw.clone()).await?;
        let encoded = serde_json::to_value(canonical)
            .map_err(|e| ConnectorError::Normalize(e.to_string()))?;
        hashes.push(payload_hash(&encoded));
    }
    Ok(hashes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct FakeAdapter;

    #[async_trait]
    impl ConnectorAdapter for FakeAdapter {
        fn source_system(&self) -> &'static str {
            "fake"
        }

        async fn normalize(&self, raw: RawEvent) -> Result<CanonicalEvent, ConnectorError> {
            Ok(CanonicalEvent {
                event_id: format!("canon-{}", raw.source_event_id),
                event_type: "order.captured.v1".to_string(),
                schema_version: "1.0.0".to_string(),
                source_system: self.source_system().to_string(),
                source_event_id: raw.source_event_id,
                tenant_id: "tenant_1".to_string(),
                legal_entity_id: "US_CO_01".to_string(),
                payload: raw.payload,
                trace_context: CanonicalTraceContext {
                    idempotency_key: "fake:evt_1".to_string(),
                    correlation_id: "corr_1".to_string(),
                    causation_id: None,
                    traceparent: None,
                    tracestate: None,
                },
            })
        }
    }

    #[tokio::test]
    async fn replay_hashes_are_deterministic() {
        let adapter = FakeAdapter;
        let events = vec![RawEvent {
            source_event_id: "evt_1".to_string(),
            occurred_at: Utc::now(),
            payload: json!({"a": 1, "b": 2}),
        }];

        let first = replay_hashes(&adapter, &events).await.unwrap();
        let second = replay_hashes(&adapter, &events).await.unwrap();

        assert_eq!(first, second);
    }
}
