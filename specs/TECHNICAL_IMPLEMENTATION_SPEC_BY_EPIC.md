# Technical Implementation Specification by Epic

Last updated: February 21, 2026
Status: Execution spec

## 1. Scope and Constraints
This spec is authoritative for implementation execution and supersedes conflicting assumptions in older planning docs.

Locked constraints:
1. Rust + Ractor runtime.
2. Dual-book from MVP (`US_GAAP` + `IFRS`).
3. ERPNext first, Odoo-compatible adapter contract.
4. Ski-resort pilot domain.
5. US + Canada country packs from MVP.
6. Stripe first, pluggable gateway interface.
7. DuckDB initial storage profile with pluggable DB backends.
8. Scale target: comfortable 2,000 active users with linear scaling behavior.

## 2. Cross-Epic Technical Standards
1. APIs
- External: REST JSON (`/v1/...`), idempotent write semantics.
- Internal: gRPC and event contracts.
2. Identity and policy
- OIDC with Keycloak.
- SoD and policy checks via OPA.
3. Message envelope required fields
- `tenant_id`, `legal_entity_id`, `event_id`, `idempotency_key`, `correlation_id`, `schema_version`, `occurred_at`.
4. Data integrity
- Decimal arithmetic only.
- Deterministic rounding policy per currency.
- Journal balancing enforced at write time.
5. Distributed consistency
- Outbox/inbox required for all cross-service write flows.
- Saga only where local ACID is insufficient.

## 3. Epic Map
1. Epic A: Platform Core and Runtime
2. Epic B: Ledger and Posting Engine
3. Epic C: Revenue Recognition (Dual-Book)
4. Epic D: Integrations and Canonical Ingestion (Ski)
5. Epic E: Payment Gateway Layer (Stripe-first)
6. Epic F: Reconciliation and Exception Ops
7. Epic G: Federation, Intercompany, Consolidation
8. Epic H: Compliance, Audit, Security Controls
9. Epic I: ERP Adapter Layer (ERPNext + Odoo contract)
10. Epic J: Country Packs (US + CA)
11. Epic K: Scale and Performance Engineering (2,000 users)

## 4. Epic Specifications

### Epic A: Platform Core and Runtime
Objective:
- Build shared runtime primitives and contracts used by all domains.

Scope:
- Actor supervision framework.
- Idempotency service.
- Schema registry and envelope validators.
- Outbox/inbox libraries.

Interfaces:
```http
POST /v1/platform/events/ingest
POST /v1/platform/idempotency/check
```

```rust
pub trait IdempotencyStore {
    async fn check_or_insert(&self, key: &str, payload_hash: &str) -> Result<IdempotencyResult>;
}
```

Acceptance criteria:
1. Duplicate request with same hash returns prior result.
2. Duplicate request with different hash returns conflict.
3. Actor restart does not lose committed messages.

Test strategy:
- Property tests for idempotency.
- Chaos tests for actor restarts.

Rollout:
- Deploy as shared runtime crate and service foundation before any domain endpoints.

Risks:
- Envelope drift across services.

### Epic B: Ledger and Posting Engine
Objective:
- Deterministic, immutable, balanced journals with strict provenance.

Scope:
- Posting rule engine.
- Journal writer.
- Period lock.
- Reversal workflows.

Interfaces:
```http
POST /v1/ledger/posting-runs
POST /v1/ledger/journals/{journal_id}/reverse
POST /v1/ledger/periods/{period_id}/lock
```

Data contract additions required:
- `book_policy_id`
- `policy_version`
- `fx_rate_set_id`
- `ruleset_version`
- `workflow_id`

Acceptance criteria:
1. 100% journals balanced by tx/base currency.
2. Posted journals immutable.
3. Deterministic replay checksum stable across reruns.
4. Both books post or neither posts.

Test strategy:
- Golden tests for deterministic replay.
- Mutation tests for rule precedence.

Rollout:
- Enable for POS + reservation core flows first.

Risks:
- Rule ambiguity causing non-determinism.

### Epic C: Revenue Recognition (Dual-Book)
Objective:
- Run ASC 606/IFRS 15 logic from same contracts/events with explicit policy deltas.

Scope:
- Obligation modeling.
- SSP allocation.
- Deferred revenue schedules.
- Breakage and refund liability.

Interfaces:
```http
POST /v1/revrec/contracts
POST /v1/revrec/fulfillment-events
POST /v1/revrec/runs
GET /v1/revrec/rollforward?book=US_GAAP
GET /v1/revrec/rollforward?book=IFRS
```

Acceptance criteria:
1. Daily dual-book rollforwards reconcile to liability accounts.
2. Policy package changes are versioned and auditable.
3. Breakage estimate changes are prospective and approved.

Test strategy:
- Fixture-based differential tests (`US_GAAP` vs `IFRS`).

Rollout:
- Season pass + lesson bundle + rental package scenarios first.

Risks:
- Incomplete policy package versioning.

### Epic D: Integrations and Canonical Ingestion (Ski)
Objective:
- Normalize ski-domain source systems into canonical events.

Scope:
- Connector SDK.
- Inntopia integration.
- POS adapter (Square or Simphony pilot choice).
- Replay and dead-letter handling.

Interfaces:
```rust
#[async_trait::async_trait]
pub trait ConnectorAdapter {
    async fn pull_since(&self, cursor: &str) -> Result<Vec<RawEvent>>;
    async fn normalize(&self, raw: RawEvent) -> Result<CanonicalEvent>;
}
```

Acceptance criteria:
1. >=98% valid events ingest without manual intervention.
2. Replay can reconstruct missing windows deterministically.
3. DLQ items are triaged with reason codes.

Test strategy:
- Contract tests per connector.
- Backfill/replay simulation tests.

Rollout:
- Inntopia first, then pilot POS source.

Risks:
- Vendor API drift.

### Epic E: Payment Gateway Layer (Stripe-first)
Objective:
- Isolate gateway logic and normalize payment events.

Scope:
- Gateway trait.
- Stripe implementation.
- Settlement and dispute ingestion.
- Provider metadata normalization.

Interfaces:
```rust
#[async_trait::async_trait]
pub trait PaymentGateway {
    async fn capture(&self, req: CaptureRequest) -> Result<CaptureResult>;
    async fn refund(&self, req: RefundRequest) -> Result<RefundResult>;
    async fn ingest_settlement(&self, batch_ref: &str) -> Result<Vec<SettlementLine>>;
}
```

Acceptance criteria:
1. Stripe workflows pass end-to-end for auth/capture/refund/dispute/settlement.
2. Provider switch conformance tests pass against gateway contract.
3. No gateway-specific data leaks into canonical core schema.

Test strategy:
- API mocks + signed webhook replay tests.

Rollout:
- Stripe production path enabled; non-Stripe adapters remain contract-only in MVP.

Risks:
- Settlement shape differences across providers.

### Epic F: Reconciliation and Exception Ops
Objective:
- Match operational and financial realities with controlled exceptions.

Scope:
- Order-payment-payout-bank matcher.
- Exception queue and SLA policies.
- Auto-adjustment proposals.

Interfaces:
```http
POST /v1/recon/runs
GET /v1/recon/exceptions
POST /v1/recon/exceptions/{id}/resolve
```

Acceptance criteria:
1. Auto-match >=85% on pilot workload.
2. First triage SLA <24h for exceptions.
3. Reconciliation integrity checks pass before close.

Test strategy:
- Seeded mismatch sets.
- False-positive/false-negative benchmark tests.

Rollout:
- Daily reconciliation with operator queues.

Risks:
- Source data quality variability.

### Epic G: Federation, Intercompany, Consolidation
Objective:
- Support multi-entity operations and group close.

Scope:
- Intercompany due-to/due-from.
- Elimination rules.
- FX translation.
- Consolidation outputs.

Interfaces:
```http
POST /v1/federation/intercompany/post
POST /v1/federation/consolidation/runs
GET /v1/federation/consolidation/{run_id}
```

Acceptance criteria:
1. Consolidated close for pilot entities reproducible and auditable.
2. Intercompany mismatch dashboard and controls operational.
3. FX translation provenance retained.

Test strategy:
- Multi-entity close simulation.

Rollout:
- Start with 2-3 entities in pilot, expand post-stabilization.

Risks:
- Policy inconsistencies between entities.

### Epic H: Compliance, Audit, Security Controls
Objective:
- Enforce non-bypassable controls and auditable evidence.

Scope:
- SoD policies.
- Immutable audit chain.
- Access review exports.
- Retention/legal hold controls.

Interfaces:
```http
GET /v1/compliance/evidence/export
POST /v1/compliance/policy/evaluate
```

Acceptance criteria:
1. Full evidence chain from source event to financial statements.
2. SoD policy violations blocked and logged.
3. Control exports accepted by internal audit stakeholders.

Test strategy:
- Policy regression suite.
- Tamper-evidence verification tests.

Rollout:
- Controls enabled in stage early; production enforcement before go-live.

Risks:
- Control gaps detected late.

### Epic I: ERP Adapter Layer (ERPNext + Odoo contract)
Objective:
- Keep core accounting independent from ERP vendor behavior.

Scope:
- ERP adapter contract.
- ERPNext adapter implementation.
- Odoo conformance test harness + stub.

Interfaces:
```rust
#[async_trait::async_trait]
pub trait ErpAdapter {
    async fn push_document(&self, doc: ErpDocument) -> Result<ErpResult>;
    async fn fetch_status(&self, external_id: &str) -> Result<ErpStatus>;
}
```

Acceptance criteria:
1. ERPNext sync flows pass for required finance operations.
2. Odoo contract test suite passes with stub adapter.
3. Core modules contain zero ERP-specific conditionals.

Test strategy:
- Consumer-driven contract tests.

Rollout:
- ERPNext first; Odoo production adapter after first successful close.

Risks:
- ERP data model drift and mapping errors.

### Epic J: Country Packs (US + CA)
Objective:
- Localize accounting/tax/compliance without forking core logic.

Scope:
- US and CA tax models.
- Country-specific controls and report packs.
- Dual-book close signoff gates.

Interfaces:
```rust
pub trait CountryPack {
    fn country_code(&self) -> &'static str;
    fn tax_rules(&self) -> TaxRuleSet;
    fn validation_rules(&self) -> ValidationRuleSet;
}
```

Acceptance criteria:
1. US and CA packs pass regression suites.
2. Country-specific rollout checklist evidence complete.
3. Dual-book signoff required for each country close pack.

Test strategy:
- Country-specific UAT packs.
- Tax regression fixtures by province/state.

Rollout:
- US first, CA shadow then production.

Risks:
- Local compliance changes and tax rule drift.

### Epic K: Scale and Performance Engineering (2,000 users)
Objective:
- Prove comfortable operation for 2,000 active users with no meaningful scaling bend.

Scope:
- Load model codification.
- Capacity planning.
- Performance test automation.
- Bottleneck observability.

Interfaces:
```http
GET /v1/ops/slo
GET /v1/ops/capacity
POST /v1/ops/loadtest/trigger
```

Acceptance criteria:
1. 2,000-user target test passes all SLOs.
2. Scaling efficiency `E(N) >= 0.80` for 2..8 replicas.
3. p95 step degradation <=10% and error-rate drift <=0.05% absolute.
4. No ledger integrity failures under soak/spike tests.

Test strategy:
1. 500 baseline.
2. 2,000 target (60m).
3. 2,500 step.
4. 3,000 spike.
5. 2,000 soak (8h).

Rollout:
- Performance gates are release-blocking for pilot go-live.

Risks:
- Storage profile bottlenecks and actor hotspotting.

## 5. Cross-Epic Milestones
1. M0: Contracts and decision lock.
2. M1: Platform core and envelope stability.
3. M2: Ledger posting + period controls.
4. M3: Dual-book revenue engine.
5. M4: Ski connectors and canonical ingestion.
6. M5: Stripe gateway + settlement ingest.
7. M6: Reconciliation and exception operations.
8. M7: Federation/consolidation baseline.
9. M8: Compliance and audit gates.
10. M9: ERPNext production adapter + Odoo contract pass.
11. M10: US/CA country pack readiness.
12. M11: 2,000-user scale gate + pilot launch.

## 6. Required Updates to Existing Specs
1. `specs/OPEN_SOURCE_ACCOUNTING_SYSTEM_MASTER_SPEC.md`
- Replace “Decisions Needed” with locked decisions.
- Promote dual-book from Should to Must.
2. `specs/IMPLEMENTATION_PLAN_12_WEEKS.md`
- Replace US-GAAP-only assumption.
- Add scale gate (2,000 users) and no-bend metrics.
3. `specs/CANONICAL_DATA_MODEL_SPEC.md`
- Add required provenance fields listed in Epic B.
- Update payment example to Stripe for pilot default.
4. `specs/COUNTRY_ROLLOUT_PACK_TEMPLATE_US_CA.md`
- Add dual-book evidence checklist and ski-domain UAT evidence gates.

## 7. Definition of Done
1. All epic acceptance criteria pass in stage and production preflight.
2. Dual-book close completes for pilot entities/countries.
3. 2,000-user no-bend scale gate passes.
4. Operational runbooks, rollback plans, and control evidence are approved.
