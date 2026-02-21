# Sprint 3 Squad Agent Execution Plan

Last updated: February 21, 2026
Scope: Sprint 3 (Weeks 5-6), Payments and Reconciliation Control Loop (US + Canada ski-resort pilot)

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

## 2. Sprint 3 Build Assignments by Squad
Verified Sprint 2 baseline input:
- Posting runtime supports idempotency, rule-driven posting, period checks, and reversals.
- Settlement ingest adapters (Stripe + bank parser) and deterministic reason-code routing are implemented.
- SoD and break-glass baseline controls are active in OPA tests and gate runner.
- Remaining carry-forward gaps: cross-squad end-to-end ingest->post->recon gate closure and open Finance/Controls signoffs.

| Squad | Epic focus (Sprint 3) | Build assignments |
|---|---|---|
| Platform | A/B/F | Extend posting rules for fees, chargebacks, and payout clearing, add dispute lifecycle accounting states, and preserve idempotency/period/reversal invariants |
| Finance | B/C/J | Implement refund-liability remeasurement and breakage assumption versioning with dual-book tie-out evidence across US/CA entities |
| Integration | D/E/I | Deliver Square connector v1 (sales/refunds/tenders/payouts), canonical normalization, replay/backfill, and webhook verification |
| Controls&Sec | H + cross-epic gates | Add policy gates for estimate/assumption changes, chargeback approval controls, and break-glass logging completeness checks |
| Data/Reconciliation | F/G | Build matching engine v1 (order/payment/payout), exception queue API, owner routing, and SLA-bearing reconciliation traces |
| QA/Release | Cross-epic quality gates | Execute seeded mismatch reconciliation accuracy tests, validate auto-match and reason-code routing gates, and publish Sprint 3 signoff packet |

## 3. Per-Member Work Plan
### Platform
- `PL-L`: Own posting-rule/dispute-state scope and technical signoff for cash-loop accounting invariants.
- `PL-W1`: Implement fee, chargeback, and payout-clearing posting extensions with balancing and replay-safe tests.
- `PL-W2`: Implement dispute lifecycle accounting state transitions and regression coverage for reversal + period-lock behavior.

### Finance
- `FI-L`: Own Finance Impact Board decisions for breakage/remeasurement and signoff routing.
- `FI-W1`: Implement refund-liability remeasurement rules and effective-date governance.
- `FI-W2`: Implement breakage assumption versioning and dual-book tie-out fixtures for payout/chargeback scenarios.

### Integration
- `IN-L`: Coordinate Square stage readiness and dependency clearance.
- `IN-W1`: Implement Square sales/refunds/tenders canonical mapper and connector contract tests.
- `IN-W2`: Implement Square payout ingest + replay/backfill + DLQ tooling and webhook signature checks.

### Controls&Sec
- `CS-L`: Own controls decisions and non-bypassable gate scope for estimate-change approvals.
- `CS-W1`: Implement OPA policy controls for estimate changes and chargeback/dispute approval actions.
- `CS-W2`: Implement break-glass logging completeness checks and policy evidence export updates.

### Data/Reconciliation
- `DA-L`: Own matching engine v1 scope and exception queue dependency matrix.
- `DA-W1`: Implement deterministic matcher for order/payment/payout keys and auto-match scoring outputs.
- `DA-W2`: Implement exception queue API with reason codes, owner assignment, and triage SLA fields.

### QA/Release
- `QA-L`: Own Sprint 3 gate recommendation and final go/no-go packet.
- `QA-W1`: Build seeded mismatch accuracy suite and auto-match measurement harness.
- `QA-W2`: Build end-to-end routing assertions for unmatched reason-code + owner/SLA assignment gates.

## 4. Architect Coordination and Updates
Cadence:
1. Daily squad standups (internal, 15 min).
2. Daily Architect sync (all leaders, 20 min).
3. Tue/Thu dependency and controls board (30 min).
4. Friday sprint control review with Finance + Controls mandatory (45 min).

Architect update template:
```md
Squad: <...>
RAG: <Green|Amber|Red>
Completed (24h):
Next (24h):
Dependencies/Blockers:
Decisions Needed: <decision> | financial_impact:<yes/no> | control_impact:<yes/no>
Exit-Criteria Progress: square_valid_rate:% | automatch_rate:% | unmatched_reason_codes:% | estimate_policy_gate_pass:% | dispute_flow_tests:%
```

Architect Update 1 (current):
- Sprint 3 scope fixed to closing the cash-loop control path (Square ingest -> posting -> matching -> exception ownership).
- Carry-forward Sprint 2 integration/signoff gaps remain explicit merge blockers until resolved.
- Pike-rule simplification retained: deterministic matching and local ACID boundaries before broader orchestration complexity.

## 5. Decision Impact Workflow (Finance + Controls)
1. Any decision marked `financial_impact=yes` or `control_impact=yes` is blocked for merge.
2. Required approvals:
- Finance Impact Board approval (`FI-L` + delegate reviewers).
- Controls Impact Board approval (`CS-L` + delegate reviewers).
3. QA enforces merge gate: no merge without both approvals when flagged.
4. Architect resolves technical direction only after both checks complete.
5. Decisions affecting breakage assumptions, refund-liability remeasurement, dispute accounting, or estimate-change authority are always flagged.

## 6. Finance Impact Board (Required Decisions)
Required financial approval before merge:
1. Chargeback/dispute lifecycle posting states and payout-clearing mappings.
2. Refund-liability remeasurement cadence and effective-date policy.
3. Breakage assumption versioning policy and rollback semantics.
4. Square tender/refund treatment and dual-book mapping parity.
5. Auto-match tolerance bands and materiality thresholds affecting close.
6. Unmatched exception ownership/SLA design with close impact implications.

Minimum evidence:
- Signed chargeback/dispute mapping matrix by entity.
- Breakage/remeasurement policy changelog with approvers and effective dates.
- Dual-book tie-out fixture report for Square payout + dispute scenarios.

## 7. Controls Impact Board (Required Decisions)
Required controls approval before merge:
1. Non-bypassable policy gates for estimate/assumption changes.
2. Chargeback/dispute approval action controls and SoD enforcement.
3. Break-glass logging completeness (ticket, reason, approver, audit reference, TTL window).
4. Exception queue ownership integrity and SLA timestamp controls.
5. CI merge-policy requirements for reconciliation accuracy and control regressions.

Minimum evidence:
- Updated role/action/entity matrix covering estimate and dispute actions.
- OPA positive/negative test results for new critical actions.
- Exception queue trace extracts proving owner + reason-code + SLA capture.

## 8. Pike’s 5 Rules Review Outcome (Sprint 3)
Accepted corrections for Sprint 3:
1. Build one complete payments control loop first (Square sale/refund/payout to matched/unmatched outcomes) before broad connector expansion.
2. Keep transaction boundaries simple: local ACID + outbox/inbox defaults, no generalized saga framework.
3. Keep matching deterministic and reason-coded before introducing heuristic/fuzzy complexity.
4. Preserve explicit correctness invariants for chargeback/dispute postings and dual-book parity.
5. Enforce quantitative exit gates: auto-match, unmatched routing completeness, and control-policy pass rates as hard sprint blockers.

## 9. Major Tool-Use Summary by Squad (Major Activities Only)
| Squad | Major activities | Tool use summary |
|---|---|---|
| Platform | Fees/chargebacks/payout posting extensions | Rust invariant tests, dispute-state fixtures, replay/idempotency regression suites |
| Finance | Remeasurement + breakage policy packs | Dual-book tie-out fixtures, policy version artifacts, approval matrix tooling |
| Integration | Square connector + payout normalization | Connector contract tests, webhook signature simulators, replay/backfill test harnesses |
| Controls&Sec | Estimate/dispute policy gates | OPA/Rego regression tests, control evidence exports, approval-trace validations |
| Data | Matching engine + exception queue API | Seeded recon datasets, matcher accuracy runners, owner/SLA routing assertions |
| QA/Release | Reconciliation accuracy and sprint gate pack | End-to-end pipeline tests, mismatch scenario runners, gate dashboards and signoff reports |

## 10. Sprint 3 Exit Gate (Authoritative)
Sprint 3 exits only if all pass:
1. Square connector v1 processes sales/refunds/tenders/payout payloads in stage with >=95% valid-event success.
2. Posting extensions for fees, chargebacks, and payout clearing pass balancing/idempotency/period-lock regression tests.
3. Refund-liability remeasurement and breakage assumption versioning are approved and tied to dual-book evidence.
4. Matching engine v1 achieves >=70% auto-match on the Sprint 3 stage dataset.
5. 100% of unmatched items are routed with reason code, owner assignment, and SLA timestamp fields.
6. Chargeback/dispute lifecycle accounting states are exercised and validated in automated tests.
7. Controls policy gates pass for estimate-change actions, dispute approvals, and break-glass logging completeness.
8. Finance and Controls approvals are complete for all flagged Sprint 3 decisions.
9. CI gates are green for contracts, connectors, posting, controls, reconciliation accuracy, and QA suites.
