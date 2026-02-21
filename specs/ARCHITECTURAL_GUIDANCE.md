# Architectural Guidance

Last updated: February 21, 2026
Status: Approved baseline for technical design

## 1. Locked Sponsor Decisions
1. Accounting basis: dual-book from MVP (`US_GAAP` + `IFRS`).
2. Operational finance UI: `ERPNext` first.
3. ERP compatibility: support `Odoo` and future compatible ERPs via adapter contracts.
4. Pilot vertical: ski resorts.
5. Initial countries: United States and Canada.
6. Payments: Stripe primary provider, with configurable gateway abstraction.
7. Implementation language: Rust.
8. Runtime style: Ractor actor model for event-driven state updates.
9. Distributed transactions: required for cross-service workflows.
10. Initial storage profile: DuckDB.
11. Storage portability: parameterized support for compatible distributed databases later.

## 2. Governing Design Rules (Pike Priority)
These rules override optimization preferences:
1. Do not guess bottlenecks.
2. Measure before tuning.
3. Prefer simple algorithms over fancy/high-constant approaches for pilot scale.
4. Prefer simpler data structures and execution paths where correctness is equivalent.
5. Data model correctness dominates architecture choices.

Implication: start with the simplest architecture that preserves audit correctness, deterministic replay, and future migration options.

## 3. Architecture Shape

### 3.1 Service Topology (Pilot)
Use a small number of deployables first:
1. `acct-core-svc` (single write authority for accounting domain)
- Ingestion normalization
- Canonical events
- Posting engine
- Revenue recognition (dual-book)
- Reconciliation
- Intercompany + consolidation
- Close controls
2. `acct-integration-svc`
- Connector runtime (Inntopia, Square/Simphony as needed)
- Payment provider adapters (Stripe first)
- Retry/replay and DLQ handling
3. `acct-erp-bridge-svc`
- ERPNext adapter implementation
- Odoo adapter contract stub + conformance tests
4. `acct-query-svc`
- Read models
- BI/reporting APIs

Rationale: fewer distributed boundaries reduce complexity and failure modes while still preserving modular boundaries in code.

### 3.2 Internal Module Boundaries (Inside `acct-core-svc`)
1. `canonical`
2. `ledger`
3. `revrec`
4. `payments`
5. `reconciliation`
6. `federation`
7. `controls`
8. `country_pack`

Each module owns its write model and invariants; cross-module communication uses internal event contracts.

## 4. Actor Runtime Model (Rust + Ractor)

### 4.1 Actor Pattern
1. Root supervisor per service.
2. Shard supervisors by `tenant_id` + `legal_entity_id`.
3. Single-writer actors per aggregate key for deterministic updates.
4. Bounded mailboxes and backpressure (fail fast with retriable errors).

### 4.2 Message Contract
Every command/event includes:
- `tenant_id`
- `legal_entity_id`
- `event_id`
- `idempotency_key`
- `correlation_id`
- `causation_id`
- `occurred_at`
- `schema_version`

### 4.3 Supervision Policy
1. `one_for_one` for aggregate actors.
2. `rest_for_one` for coupled pipeline actors.
3. Exponential retry with jitter for transient dependencies.
4. Circuit-break and operator escalation after retry budget exhaustion.

## 5. Data Model Invariants (Non-Negotiable)
1. Journals balance by transaction currency and base currency.
2. Posted journals are immutable; corrections are reversal + replacement.
3. Deterministic replay: same canonical event + same policy package + same FX set => same output.
4. Dual-book parity: every in-scope posting generates both books or neither.
5. Mandatory posting provenance:
- `book_policy_id`
- `policy_version`
- `fx_rate_set_id`
- `ruleset_version`
- `workflow_id`

## 6. Distributed Transaction Strategy

### 6.1 Default Pattern
Use local ACID + outbox/inbox idempotency as the default.

### 6.2 Saga Usage Rule
Use saga orchestration only when a flow crosses service boundaries and requires compensating actions.

### 6.3 Required Primitives
1. Local transaction writes domain state + outbox atomically.
2. Inbox dedupe key on consumer side.
3. Compensation command defined for each saga step.
4. Step timeouts and dead-letter escalation.

### 6.4 Compensation Semantics
1. Never delete history.
2. Compensation is expressed as business reversal events/journals.
3. Compensation paths are testable and replay-safe.

## 7. Storage Architecture (DuckDB-First, Pluggable)

### 7.1 Storage Interface
Define storage via Rust traits, not direct DB coupling:
- `JournalStore`
- `EventStore`
- `IdempotencyStore`
- `OutboxStore`
- `InboxStore`
- `ProjectionStore`
- `FxRateStore`

### 7.2 Engine Profiles
1. `duckdb` profile (initial)
- Fast local analytics + simple operational model
- Single-writer expectations explicitly managed by actor design
2. `distributed_sql` profile (future)
- Postgres-compatible distributed engine support via adapter
- No canonical contract change required

### 7.3 Portability Rules
1. No engine-specific SQL in domain modules.
2. All SQL variants isolated in adapter packages.
3. CI runs contract tests against at least two profiles (`duckdb` and one alternate).

## 8. ERP and Gateway Abstraction

### 8.1 ERP Adapter Contract
1. `erp-bridge` receives canonical domain events.
2. ERP-specific mapping occurs in adapter layer only.
3. MVP production adapter: ERPNext.
4. Odoo: contract tests + stub implementation in MVP; production rollout after pilot close success.

### 8.2 Payment Gateway Contract
1. Generic interface for auth/capture/refund/dispute/settlement ingestion.
2. MVP provider implementation: Stripe.
3. Provider-specific fields stored in extensible metadata without polluting canonical contracts.

## 9. Country and Domain Guidance (Ski + US/CA)
1. First-class ski flows:
- passes
- lessons
- rentals
- reservations
- weather/closure refunds and reclassification
2. Country packs must include:
- US and CA tax model differences
- dual-book close evidence
- statutory reporting support artifacts

## 10. Security and Compliance Baseline
1. Keycloak + OPA for authn/authz and SoD policy enforcement.
2. Non-bypassable controls for posting, period lock, policy package changes, and payment release.
3. Tamper-evident audit trail across event, posting, reconciliation, and close workflows.
4. PCI scope-minimizing payment architecture (no PAN handling in core platform).

## 11. 2,000 User Scalability Specification (No Bend)

### 11.1 Workload Model
1. Target active users: 2,000 concurrent active users with >=30% headroom.
2. Average request rate: ~167 RPS.
3. Peak sustained rate: ~330 RPS.
4. Peak burst rate (<=60s): ~500 RPS.
5. Traffic mix:
- 65% reads
- 25% writes
- 7% reporting/background
- 3% payment/webhook

### 11.2 SLO Targets
1. Availability: 99.95% monthly.
2. Read latency: p95 <= 150ms, p99 <= 350ms.
3. Write latency: p95 <= 250ms, p99 <= 600ms.
4. 5xx rate <= 0.1%.
5. Timeouts <= 0.05%.
6. Duplicate business effects: 0.

### 11.3 Linear Scaling Definition
“No bend” means:
1. Scaling efficiency `E(N) = T(N) / (N * T(1)) >= 0.80` for N=2..8 replicas.
2. p95 degradation per scaling step <= 10% at equivalent per-replica load.
3. Error rate increase <= 0.05% absolute across scaling steps.

### 11.4 Capacity Baseline
1. API/core replicas: 8 replicas, 2 vCPU / 4 GiB each.
2. Worker replicas: 4 replicas, 2 vCPU / 4 GiB each.
3. DuckDB node: 16 vCPU / 64 GiB / NVMe for pilot profile.

### 11.5 Test Gates
Mandatory load tests:
1. 500-user baseline (30m).
2. 2,000-user target (60m).
3. 2,500-user step test.
4. 3,000-user spike (10m).
5. 2,000-user soak (8h).

Pass criteria:
1. All SLOs pass in target + soak tests.
2. Linear scaling criteria pass.
3. No data integrity violations.
4. Queue backlogs drain in <=2 minutes after burst.

## 12. DuckDB Exit Criteria and Migration Trigger
Start migration planning when any two soft triggers hold for two consecutive weeks:
1. write commit p95 > 200ms at <=70% infra utilization
2. write queue depth > 1000 for >15 min/day
3. sustained writes >120 tx/s during normal peak
4. checkpoint/maintenance windows >30 min
5. storage >300 GB or hot tables >200M rows

Immediate migration trigger (hard):
1. consecutive target-load failures attributable to storage contention
2. inability to meet required RPO/RTO
3. requirement for multi-writer HA beyond profile constraints

## 13. Implementation Guardrails
1. No new distributed boundary unless it removes a measured bottleneck or an explicit compliance risk.
2. No new abstraction layer without two concrete implementations or a proven near-term need.
3. Any optimization must include benchmark evidence and rollback criteria.
4. Every epic must include deterministic replay tests.

## 14. ADR Queue
1. ADR-001 service topology (pilot macroservice vs microservices).
2. ADR-002 distributed transaction standard (outbox/inbox + selective saga).
3. ADR-003 storage profile and migration contract.
4. ADR-004 dual-book policy package versioning.
5. ADR-005 ERP adapter contract and rollout gating.
6. ADR-006 payment gateway contract and Stripe-first implementation.
7. ADR-007 scale SLO and no-bend acceptance gates.
