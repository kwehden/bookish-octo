use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;
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
}
