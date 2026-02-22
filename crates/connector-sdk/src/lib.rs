use async_trait::async_trait;
use chrono::{DateTime, Utc};
use platform_core::payload_hash;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use thiserror::Error;

pub mod inntopia;
pub mod square;
pub use inntopia::InntopiaAdapter;
pub use square::SquareAdapter;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayBackfillTelemetry {
    pub total_events: u32,
    pub total_attempts: u32,
    pub first_attempt_failures: u32,
    pub recovered_events: u32,
    pub failed_events: u32,
    pub simulated_recovery_time_ms: u64,
    pub recovery_target_ms: u64,
    pub objective_met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayBackfillResult {
    pub hashes: Vec<String>,
    pub telemetry: ReplayBackfillTelemetry,
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

pub async fn run_replay_backfill_resiliency<A: ConnectorAdapter + Sync>(
    adapter: &A,
    raw_events: &[RawEvent],
    transient_failure_indices: &BTreeSet<usize>,
    recovery_target_ms: u64,
    retry_penalty_ms: u64,
) -> Result<ReplayBackfillResult, ConnectorError> {
    let mut hashes = Vec::with_capacity(raw_events.len());
    let mut total_attempts = 0_u32;
    let mut first_attempt_failures = 0_u32;
    let mut recovered_events = 0_u32;
    let retry_penalty = if retry_penalty_ms == 0 {
        1
    } else {
        retry_penalty_ms
    };
    let mut simulated_recovery_time_ms = 0_u64;

    for (index, raw) in raw_events.iter().enumerate() {
        total_attempts += 1;
        if transient_failure_indices.contains(&index) {
            first_attempt_failures += 1;
            total_attempts += 1;
            recovered_events += 1;
            simulated_recovery_time_ms += retry_penalty;
        }

        let canonical = adapter.normalize(raw.clone()).await?;
        let encoded = serde_json::to_value(canonical)
            .map_err(|e| ConnectorError::Normalize(e.to_string()))?;
        hashes.push(payload_hash(&encoded));
    }

    let telemetry = ReplayBackfillTelemetry {
        total_events: raw_events.len() as u32,
        total_attempts,
        first_attempt_failures,
        recovered_events,
        failed_events: 0,
        simulated_recovery_time_ms,
        recovery_target_ms,
        objective_met: simulated_recovery_time_ms <= recovery_target_ms,
    };

    Ok(ReplayBackfillResult { hashes, telemetry })
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

    #[tokio::test]
    async fn replay_backfill_resiliency_tracks_recovery_objective() {
        let adapter = FakeAdapter;
        let events = vec![
            RawEvent {
                source_event_id: "evt_1".to_string(),
                occurred_at: Utc::now(),
                payload: json!({"a": 1}),
            },
            RawEvent {
                source_event_id: "evt_2".to_string(),
                occurred_at: Utc::now(),
                payload: json!({"a": 2}),
            },
        ];
        let failures = BTreeSet::from([1_usize]);

        let result = run_replay_backfill_resiliency(&adapter, &events, &failures, 1000, 250)
            .await
            .unwrap();

        assert_eq!(result.hashes.len(), 2);
        assert_eq!(result.telemetry.total_events, 2);
        assert_eq!(result.telemetry.total_attempts, 3);
        assert_eq!(result.telemetry.first_attempt_failures, 1);
        assert_eq!(result.telemetry.recovered_events, 1);
        assert_eq!(result.telemetry.failed_events, 0);
        assert_eq!(result.telemetry.simulated_recovery_time_ms, 250);
        assert!(result.telemetry.objective_met);
    }
}
