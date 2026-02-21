# Sprint 2 Squad Agent Execution Plan

Last updated: February 21, 2026
Scope: Sprint 2 (Weeks 3-4), Core Posting and Ingestion MVP (US + Canada ski-resort pilot)

## 1. Agent Topology
Coordinator:
- `ARCH-L` Architect Coordinator

Squads (1 Leader + 2 Workers each):
- Platform: `PL-L`, `PL-W1`, `PL-W2`
- Finance: `FI-L`, `FI-W1`, `FI-W2`
- Integration: `IN-L`, `IN-W1`, `IN-W2`
- Controls&Sec: `CS-L`, `CS-W1`, `CS-W2`
- Data/Reconciliation: `DA-L`, `DA-W1`, `DA-W2`
- QA/Release: `QA-L`, `QA-W1`, `QA-W2`

## 2. Sprint 2 Build Assignments by Squad
Verified Sprint 1 baseline input:
- Contract and policy gates are passing in repo (`cargo test`, `opa test`, contract scripts).
- Canonical contracts, control registers, and governance impact records exist and are active.
- Gaps carried into Sprint 2: Inntopia stage ingestion, posting-rule engine v1, finance approvals/evidence completion, SoD hardening, Stripe/bank recon adapters.

| Squad | Epic focus (Sprint 2) | Build assignments |
|---|---|---|
| Platform | A/B/K | Complete immutable journal + reversal semantics, period-open checks, posting rule engine v1, and stage throughput/no-bend instrumentation for 2,000-user comfort gates |
| Finance | B/C/J | Implement reservation/pass deferral schedules, liability mappings, dual-book tie-out fixtures, and close open US/CA evidence/signoff items |
| Integration | D/E/I | Deliver Inntopia connector v1 + canonical mapper + replay/DLQ workflow and ERP adapter contract alignment (ERPNext first, Odoo-compatible) |
| Controls&Sec | H + cross-epic gates | Enforce non-bypassable SoD for posting and mapping-rule changes, evidence-linked approvals, break-glass auditing, and control CI checks |
| Data/Reconciliation | F/G | Build Stripe settlement ingest + bank file parser v0, deterministic matching inputs, exception routing with reason codes, and trace links |
| QA/Release | Cross-epic quality gates | Stand up ingest -> post -> recon integration suite, source-to-journal trace checks, SoD regression gates, and Sprint 2 performance gate pack |

## 3. Per-Member Work Plan
### Platform
- `PL-L`: Own posting engine v1 scope closure and technical signoff packet for immutability/reversal/period checks.
- `PL-W1`: Implement immutable journal state machine, reversal API semantics, and period-open guardrails.
- `PL-W2`: Implement posting rule engine v1 and performance instrumentation for 500 -> 2,000 user no-bend slope checks.

### Finance
- `FI-L`: Drive Finance Impact Board decisions, close open approval items, and sign policy/evidence bundles.
- `FI-W1`: Implement deferral/liability mapping rules for reservations, passes, refunds, and cancellations.
- `FI-W2`: Build dual-book fixture tie-outs and versioned finance evidence package for US/CA pilot entities.

### Integration
- `IN-L`: Coordinate Inntopia connector release readiness in stage and dependency burn-down.
- `IN-W1`: Implement Inntopia pull/webhook ingestion + canonical mapping + schema conformance checks.
- `IN-W2`: Implement replay/backfill/DLQ handling and ERP contract harness updates for event-to-post trace.

### Controls&Sec
- `CS-L`: Own SoD/non-bypassable control decisions and Controls Impact Board approvals.
- `CS-W1`: Implement role/action control points for posting, period lock, and mapping/ruleset change workflows.
- `CS-W2`: Extend OPA policy pack and CI policy evidence exports for approval-aware merge gates.

### Data/Reconciliation
- `DA-L`: Coordinate payment settlement ingest scope and reconciliation dependency matrix.
- `DA-W1`: Implement Stripe settlement adapter and canonical reconciliation key generation.
- `DA-W2`: Implement bank parser v0 and deterministic exception routing with taxonomy reason codes.

### QA/Release
- `QA-L`: Own Sprint 2 exit-gate recommendation and enforcement dashboard.
- `QA-W1`: Build automated ingest -> post -> journal trace tests with failure triage hooks.
- `QA-W2`: Build SoD/control regression suite and 2,000-user comfort benchmark report pipeline.

## 4. Architect Coordination and Updates
Cadence:
1. Daily squad standups (internal, 15 min).
2. Daily Architect sync (all leaders, 20 min).
3. Tue/Thu contract and dependency board (30 min).
4. Friday sprint control review with Finance + Controls mandatory (45 min).

Architect update template:
```md
Squad: <...>
RAG: <Green|Amber|Red>
Completed (24h):
Next (24h):
Dependencies/Blockers:
Decisions Needed: <decision> | financial_impact:<yes/no> | control_impact:<yes/no>
Exit-Criteria Progress: inntopia_valid_rate:% | traceability_tests:% | sod_gate_pass:% | load_2k_gate:%
```

Architect Update 1 (current):
- Sprint 1 baseline verified via gate artifacts and repo checks.
- Sprint 2 scope narrowed to one vertical slice: Inntopia -> canonical -> posting -> recon evidence.
- Open Finance and Controls approvals remain explicit merge blockers.
- Pike-rule simplification retained: no broad orchestration rollout; local ACID + outbox/inbox default.

## 5. Decision Impact Workflow (Finance + Controls)
1. Any decision marked `financial_impact=yes` or `control_impact=yes` is blocked for merge.
2. Required approvals:
- Finance Impact Board approval (`FI-L` + delegate reviewers).
- Controls Impact Board approval (`CS-L` + delegate reviewers).
3. QA enforces merge gate: no merge without both approvals when flagged.
4. Architect resolves technical direction only after both checks complete.
5. Decisions affecting posting semantics, deferral estimates, period lock behavior, or mapping authority are always flagged.

## 6. Finance Impact Board (Required Decisions)
Required financial approval before merge:
1. Deferral schedule recognition timing for reservations/passes by product type.
2. Liability account mapping and dual-book handling across US and Canada entities.
3. Refund/cancellation/breakage estimation method versioning and effective dating.
4. FX handling policy for USD/CAD posting and dual-book tie-out treatment.
5. Stripe settlement fee/payout clearing mapping to canonical accounting outcomes.
6. Revenue cutover and period-open rules that affect close.

Minimum evidence:
- Signed deferral and liability mapping matrix by entity.
- Dual-book tie-out fixture results and variance report.
- Policy version changelog with effective dates and approvers.

## 7. Controls Impact Board (Required Decisions)
Required controls approval before merge:
1. Non-bypassable SoD points for posting, period lock, and mapping/ruleset change.
2. Break-glass role approval, TTL, and mandatory evidence capture.
3. Immutable trace chain completeness for source event -> journal -> recon exception.
4. CI merge policy requirements for authorization and control regression tests.
5. Exception ownership and remediation SLA workflow controls.

Minimum evidence:
- Updated role/action/entity matrix and authorization truth table.
- OPA positive/negative test reports tied to CI runs.
- Trace extract pack showing end-to-end audit link completeness.

## 8. Pike’s 5 Rules Review Outcome (Sprint 2)
Accepted corrections for Sprint 2:
1. Build one complete production-like vertical slice first (Inntopia flow) before widening connector scope.
2. Keep transaction boundaries simple: local ACID plus outbox/inbox, no generalized distributed saga engine.
3. Preserve explicit correctness invariants (dual-book atomicity, immutable journals, idempotency) as non-negotiable.
4. Keep reconciliation logic deterministic and reason-coded before adding optimization complexity.
5. Enforce quantitative no-bend scaling gates at 2,000-user comfort load and fail Sprint exit if slope bends materially.

## 9. Major Tool-Use Summary by Squad (Major Activities Only)
| Squad | Major activities | Tool use summary |
|---|---|---|
| Platform | Posting engine v1 + performance gates | Rust test suites, invariant fixtures, load-profile harnesses, latency/error telemetry checks |
| Finance | Deferral/liability policy implementation | Posting-rule fixtures, dual-book tie-out packs, policy approval artifact pipeline |
| Integration | Inntopia ingest and mapper | Connector contract tests, replay/DLQ simulators, schema conformance validators |
| Controls&Sec | SoD and policy hardening | OPA/Rego regression tests, control evidence extracts, authorization matrix validation |
| Data | Settlement ingest + exception routing | Stripe/bank parser fixtures, recon key validators, exception taxonomy checks |
| QA/Release | End-to-end trace + release gates | Integration pipeline runners, traceability assertions, performance gate dashboarding |

## 10. Sprint 2 Exit Gate (Authoritative)
Sprint 2 exits only if all pass:
1. Inntopia connector v1 processes >=95% valid stage events without manual rework.
2. Posting engine v1 passes immutability, reversal, and period-open tests.
3. Deferral/liability rule set v1 posts dual-book entries with Finance approvals complete.
4. Stripe settlement and bank parser adapters produce reconciliation candidates with reason-coded exceptions.
5. SoD policy checks are active and non-bypassable for posting and mapping/ruleset changes.
6. Source-event-to-journal traceability tests pass for 100% of Sprint 2 in-scope fixtures.
7. 2,000-user comfort gate passes with no material bend in scaling curve:
- p95 latency growth from 500 -> 2,000 user profile remains <=20%.
- Error rate remains <=0.5% and no idempotency invariant failures occur.
8. All Finance and Controls approvals are complete for flagged decisions.
9. CI gates are green for contracts, posting, controls, and integration suites.
