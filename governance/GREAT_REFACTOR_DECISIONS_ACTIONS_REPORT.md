# Great Refactor - Decisions, Actions, and Updates

Date: 2026-02-22
Scope: Remediation of Sprint 4-6 review findings using squad-aligned agent execution.

## Pike's 5 Rules Used for Decisioning
1. Do not guess bottlenecks.
2. Measure before optimization.
3. Prefer simple algorithms/data structures for common cases.
4. Prefer simple implementations to reduce defects.
5. Data model quality dominates system behavior.

## Decisions and Actions by Finding

### 1) External attestations blocker (carry, track, continue)
Decision:
- Carry this item as an explicit open task and proceed with in-repo engineering work.
Actions:
- Added TODO tracking for external business-owner UAT and 2,000-user no-bend performance attestation.
- File: `TODO.md`
Status:
- Open (external, out-of-repo).
Pike alignment:
- Rule 2: gate on measured external evidence.
- Rule 5: treat attestation artifacts as release-critical data inputs.

### 2) Sprint 4 finance/controls evidence closure (in-repo remediation)
Decision:
- Close in-repo Sprint 4 evidence/signoff gaps and separate external dependencies clearly.
Actions:
- Updated Sprint 4 checklist/signoff/gate/completion artifacts to reflect in-repo closures and explicit external carryovers.
- Files:
  - `finance/US_CA_FINANCE_EVIDENCE_CHECKLIST_V1.md`
  - `governance/sprint4_signoff_packet.md`
  - `governance/SPRINT4_PLAN_COMPLETION_REPORT.md`
  - `qa/SPRINT4_GATE_REPORT.md`
  - `specs/SPRINT4_CLOSE_CHECKLIST_CONTRACT.md`
Status:
- In-repo closure complete.
Pike alignment:
- Rule 4: use explicit checklists and deterministic gate artifacts.
- Rule 5: approvals/evidence are first-class operational data.

### 3) Stripe integration gap (implemented)
Decision:
- Implement Stripe adapter and include replay/cutover tests to match Sprint scope.
Actions:
- Added `StripeAdapter` normalization and replay/cutover tests.
- Files:
  - `crates/connector-sdk/src/stripe.rs`
  - `crates/connector-sdk/src/lib.rs`
Status:
- Complete.
Pike alignment:
- Rule 3: adapter scope kept focused and minimal.
- Rule 2: replay/cutover behavior validated by tests.

### 4) Canonical event contract mismatch (implemented)
Decision:
- Align connector canonical envelope with schema-required fields and envelope shape.
Actions:
- Added top-level `occurred_at`, `business_date`, `idempotency_key` to canonical events.
- Updated Inntopia/Square adapters and tests.
- Added `trace_context` schema property for envelope compatibility.
- Files:
  - `crates/connector-sdk/src/lib.rs`
  - `crates/connector-sdk/src/inntopia.rs`
  - `crates/connector-sdk/src/square.rs`
  - `contracts/canonical_event_v1.schema.json`
Status:
- Complete.
Pike alignment:
- Rule 5: canonical event data contract is the integration backbone.
- Rule 4: explicit schema and tests reduce integration ambiguity.

### 5) In-memory store durability model (implemented + architecture review)
Decision:
- Keep in-memory authoritative path, add eventual-consistent write-behind persistence for actor state.
Actions:
- Added async write-behind persistence with flush/reload support for idempotency and audit-seal stores.
- Added opt-in posting API state bootstrapping from persistence directory.
- Added persistence tests for flush/restart recovery.
- Files:
  - `crates/platform-core/src/lib.rs`
  - `crates/posting-api/src/lib.rs`
Status:
- Complete for idempotency/audit-seal state; additional actor-state hardening still recommended (see open actions).
Pike alignment:
- Rule 3: preserve fast common-path in-memory operations.
- Rule 2: durability behavior tested (flush/restart).
- Rule 5: persistent actor state treated as central data plane.

### 6) Square legal-entity enrichment fallback (implemented)
Decision:
- Support legal-entity fallback by known location map for US/CA pilot while preserving strict failure for unknown mappings.
Actions:
- Added location->legal_entity fallback mapping in Square adapter.
- Added tests for fallback success and unknown-location failure.
- File: `crates/connector-sdk/src/square.rs`
Status:
- Complete.
Pike alignment:
- Rule 3: simple deterministic map for pilot scope.
- Rule 4: explicit failure path for unmapped locations.

### 7) Zero-denominator metric semantics (implemented)
Decision:
- Zero denominator must return 0 bps (not 100%) to avoid false success signals.
Actions:
- Updated ratio behavior and added regression tests.
- File: `crates/reconciliation-model/src/lib.rs`
Status:
- Complete.
Pike alignment:
- Rule 5: metric semantics are control-plane data correctness.
- Rule 2: regression tests verify measured outcomes.

### 8) Reconciliation cloning hot path (implemented)
Decision:
- Remove avoidable cloning in candidate lookup path; use borrowed slices/references.
Actions:
- Updated `reconcile_v1` candidate handling to reference index slices instead of cloning vectors/items.
- File: `crates/reconciliation-model/src/lib.rs`
Status:
- Complete.
Pike alignment:
- Rule 1 and Rule 2: optimize identified hot path, not speculative paths.
- Rule 3: simpler reference-based flow.

## Persistent Actor Lens - Cross-Squad Updates
- Platform: write-behind durability now present for idempotency/audit-seal state.
- Integration: canonical envelope now includes required top-level contract fields; Stripe added.
- Data: deterministic no-data metric semantics and lower-copy reconciliation path.
- Controls/Finance: Sprint 4 in-repo evidence closure updated; external attestations still tracked as open.
- QA/Release: gate flow remains green in-repo; external attestations remain release blockers.

## Open Actions
1. Attach external business-owner UAT attestation into Sprint 5/6 signoff and QA gate packets.
2. Attach external 2,000-user no-bend performance certification into Sprint 5/6 signoff and QA gate packets.
3. Evaluate optional persistence extension for non-critical in-memory stores (for example `legal_holds` and idempotency result cache) based on multi-instance operations policy.

## Sprint 7 Update
- Item 2 addressed: persistent actor pattern expanded to journal + period authoritative state with write-behind persistence and restart reload tests.
- Item 3 addressed: Sprint 4 dry-run now enforces elimination/FX coupled-output validity with deterministic blockers/tests; Sprint 4 criterion #6 moved to full closure.
- Item 4 addressed: Sprint 1-3 finance/governance artifact debt backfilled with explicit in-repo closure references and external-attestation non-claims.

## Validation Snapshot
- `python3 scripts/validate_canonical_schema.py`: PASS
- `cargo test -p connector-sdk`: PASS
- `./scripts/run_contract_gates.sh`: PASS
