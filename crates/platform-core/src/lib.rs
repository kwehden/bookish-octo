use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyEntry {
    pub key: String,
    pub payload_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyStatus {
    FirstSeen,
    Replay,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdempotencyError {
    #[error("idempotency key has different payload hash")]
    PayloadHashMismatch,
    #[error("idempotency store poisoned")]
    StorePoisoned,
}

#[derive(Clone, Default)]
pub struct InMemoryIdempotencyStore {
    inner: Arc<Mutex<HashMap<String, IdempotencyEntry>>>,
}

impl InMemoryIdempotencyStore {
    pub fn check_or_insert(
        &self,
        key: &str,
        payload: &Value,
    ) -> Result<IdempotencyStatus, IdempotencyError> {
        let hash = payload_hash(payload);
        let mut store = self
            .inner
            .lock()
            .map_err(|_| IdempotencyError::StorePoisoned)?;

        match store.get(key) {
            Some(existing) if existing.payload_hash == hash => Ok(IdempotencyStatus::Replay),
            Some(_) => Err(IdempotencyError::PayloadHashMismatch),
            None => {
                store.insert(
                    key.to_string(),
                    IdempotencyEntry {
                        key: key.to_string(),
                        payload_hash: hash,
                    },
                );
                Ok(IdempotencyStatus::FirstSeen)
            }
        }
    }
}

pub fn payload_hash(payload: &Value) -> String {
    let encoded = serde_json::to_vec(payload).expect("payload serialization should not fail");
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditSealEntry {
    pub sequence: u64,
    pub event_type: String,
    pub entity_scope: Vec<String>,
    pub payload_hash: String,
    pub previous_seal: String,
    pub seal: String,
    pub created_at_ns: i64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuditSealError {
    #[error("audit seal store poisoned")]
    StorePoisoned,
    #[error("audit seal chain broken at sequence {sequence}")]
    ChainBroken { sequence: u64 },
    #[error("audit seal tampered at sequence {sequence}")]
    Tampered { sequence: u64 },
}

#[derive(Clone, Default)]
pub struct InMemoryAuditSealStore {
    inner: Arc<Mutex<Vec<AuditSealEntry>>>,
}

impl InMemoryAuditSealStore {
    pub fn append(
        &self,
        event_type: &str,
        entity_scope: &[String],
        payload: &Value,
        created_at_ns: i64,
    ) -> Result<AuditSealEntry, AuditSealError> {
        let mut store = self
            .inner
            .lock()
            .map_err(|_| AuditSealError::StorePoisoned)?;
        let sequence = (store.len() as u64) + 1;
        let previous_seal = store
            .last()
            .map(|entry| entry.seal.clone())
            .unwrap_or_else(|| "GENESIS".to_string());
        let payload_digest = payload_hash(payload);
        let canonical_scope = canonical_entity_scope(entity_scope);
        let seal = payload_hash(&json!({
            "sequence": sequence,
            "event_type": event_type,
            "entity_scope": canonical_scope,
            "payload_hash": payload_digest,
            "previous_seal": previous_seal,
            "created_at_ns": created_at_ns
        }));

        let entry = AuditSealEntry {
            sequence,
            event_type: event_type.to_string(),
            entity_scope: canonical_scope,
            payload_hash: payload_digest,
            previous_seal,
            seal,
            created_at_ns,
        };
        store.push(entry.clone());
        Ok(entry)
    }

    pub fn verify_chain(&self) -> Result<(), AuditSealError> {
        let entries = self
            .inner
            .lock()
            .map_err(|_| AuditSealError::StorePoisoned)?
            .clone();
        let mut previous_seal = "GENESIS".to_string();

        for (index, entry) in entries.iter().enumerate() {
            let expected_sequence = (index + 1) as u64;
            if entry.sequence != expected_sequence || entry.previous_seal != previous_seal {
                return Err(AuditSealError::ChainBroken {
                    sequence: entry.sequence,
                });
            }

            let expected_seal = payload_hash(&json!({
                "sequence": entry.sequence,
                "event_type": entry.event_type,
                "entity_scope": canonical_entity_scope(&entry.entity_scope),
                "payload_hash": entry.payload_hash,
                "previous_seal": entry.previous_seal,
                "created_at_ns": entry.created_at_ns
            }));
            if expected_seal != entry.seal {
                return Err(AuditSealError::Tampered {
                    sequence: entry.sequence,
                });
            }

            previous_seal = entry.seal.clone();
        }

        Ok(())
    }

    pub fn len(&self) -> Result<usize, AuditSealError> {
        let store = self
            .inner
            .lock()
            .map_err(|_| AuditSealError::StorePoisoned)?;
        Ok(store.len())
    }
}

fn canonical_entity_scope(entity_scope: &[String]) -> Vec<String> {
    let mut scope = entity_scope
        .iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    scope.sort();
    scope.dedup();
    scope
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn first_seen_then_replay() {
        let store = InMemoryIdempotencyStore::default();
        let payload = json!({"event": "order.captured.v1", "amount": 100});

        let first = store.check_or_insert("key-1", &payload).unwrap();
        let replay = store.check_or_insert("key-1", &payload).unwrap();

        assert_eq!(first, IdempotencyStatus::FirstSeen);
        assert_eq!(replay, IdempotencyStatus::Replay);
    }

    #[test]
    fn same_key_different_hash_is_conflict() {
        let store = InMemoryIdempotencyStore::default();
        let payload_a = json!({"event": "order.captured.v1", "amount": 100});
        let payload_b = json!({"event": "order.captured.v1", "amount": 200});

        store.check_or_insert("key-1", &payload_a).unwrap();
        let conflict = store.check_or_insert("key-1", &payload_b).unwrap_err();

        assert_eq!(conflict, IdempotencyError::PayloadHashMismatch);
    }

    #[test]
    fn audit_seal_chain_verifies_after_append() {
        let store = InMemoryAuditSealStore::default();
        let entity_scope = vec!["US_CO_01".to_string()];
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j1"}),
                1700000000000000000,
            )
            .unwrap();
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j2"}),
                1700000001000000000,
            )
            .unwrap();

        assert_eq!(store.len().unwrap(), 2);
        assert_eq!(store.verify_chain(), Ok(()));
    }

    #[test]
    fn audit_seal_detects_payload_tampering() {
        let store = InMemoryAuditSealStore::default();
        let entity_scope = vec!["US_CO_01".to_string()];
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j1"}),
                1700000000000000000,
            )
            .unwrap();
        store
            .append(
                "posting.posted",
                &entity_scope,
                &json!({"journal_id": "j2"}),
                1700000001000000000,
            )
            .unwrap();

        {
            let mut guard = store.inner.lock().unwrap();
            guard[1].payload_hash = "tampered".to_string();
        }

        assert_eq!(
            store.verify_chain(),
            Err(AuditSealError::Tampered { sequence: 2 })
        );
    }
}
