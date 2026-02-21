# Sprint 1 Squad Agent Execution Plan

Last updated: February 21, 2026
Scope: Sprint 1 (Weeks 1-2), Foundation and Contract Freeze

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

## 2. Sprint 1 Build Assignments by Squad
| Squad | Epic focus (Sprint 1) | Build assignments |
|---|---|---|
| Platform | A/B (+K prework) | Posting API skeleton, idempotency keys, immutable write-path scaffolding, journal schema, deterministic replay scaffolding, initial SLO/no-bend instrumentation stubs |
| Finance | B/C/J prep | Chart/dimensions freeze, posting rule templates for POS/ticketing/payments, dual-book policy package baseline (`US_GAAP` + `IFRS`), country finance evidence checklist seeds |
| Integration | D/E/I contracts | Connector SDK scaffold, auth/retry/webhook verification, canonical payload validators, payment/ERP contract harnesses (Stripe-first, ERPNext-first with Odoo contract stub) |
| Controls&Sec | H + cross-epic gates | Keycloak realms/roles, initial OPA entity-boundary + SoD policy pack, policy test suite in CI, control gate register |
| Data/Reconciliation | F (+G readiness) | Reconciliation data model, exception taxonomy, idempotent recon API contract baseline, consolidation-readiness provenance checks |
| QA/Release | Cross-epic quality gates | CI contract-test pipeline, schema/posting/policy merge gates, evidence retention + traceability pack for signoff |

## 3. Per-Member Work Plan
### Platform
- `PL-L`: contract freeze across A/B/K prework, dependency resolution, final signoff packet.
- `PL-W1`: idempotency endpoint + envelope validator + outbox/inbox primitives + actor restart tests.
- `PL-W2`: posting API v0 + immutable journal contract + provenance fields + baseline load harness plumbing.

### Finance
- `FI-L`: B/C/J finance contract orchestration and signoff routing.
- `FI-W1`: posting rule template pack + dimension mapping + dual-book atomicity rules.
- `FI-W2`: rev-rec dual-book policy package v1 + US/CA dual-book evidence checklist.

### Integration
- `IN-L`: D/E/I interface freeze and CI contract gates.
- `IN-W1`: connector SDK v0 + canonical validators + replay/DLQ tests.
- `IN-W2`: ERP adapter contract harness (ERPNext + Odoo stub), integration policy hooks.

### Controls&Sec
- `CS-L`: cross-epic controls gates G1-G5 and approvals.
- `CS-W1`: Keycloak realm/role baseline, break-glass role shell.
- `CS-W2`: OPA policy pack v0 + policy regression tests + audit-link requirements.

### Data/Reconciliation
- `DA-L`: recon contract freeze package and dependency matrix.
- `DA-W1`: recon entities/keys/outcomes and exception taxonomy artifacts.
- `DA-W2`: idempotency + audit-link conformance tests and G-readiness checks.

### QA/Release
- `QA-L`: quality gate framework and go/no-go recommendation.
- `QA-W1`: CI automation for schema/posting/connector contracts.
- `QA-W2`: controls/recon test coverage and evidence packaging.

## 4. Architect Coordination and Updates
Cadence:
1. Daily squad standups (internal, 15 min).
2. Daily Architect sync (all leaders, 20 min).
3. Tue/Thu contract board (30 min).
4. Friday sprint control review with Finance + Controls mandatory (45 min).

Architect update template:
```md
Squad: <...>
RAG: <Green|Amber|Red>
Completed (24h):
Next (24h):
Dependencies/Blockers:
Decisions Needed: <decision> | financial_impact:<yes/no> | control_impact:<yes/no>
Exit-Criteria Progress: contract_freeze:% | ci_gate_pass:%
```

Architect Update 1 (current):
- Scope locked to Sprint 1 contract freeze deliverables.
- All squads mapped to member-level assignments.
- Finance and Controls impact-board criteria loaded as merge blockers.
- Pike-rule simplifications accepted for Sprint 1 execution.

## 5. Decision Impact Workflow (Finance + Controls)
1. Any squad decision marked `financial_impact=yes` or `control_impact=yes` is blocked for merge.
2. Required approvals:
- Finance Impact Board approval (`FI-L` + delegate reviewers).
- Controls Impact Board approval (`CS-L` + delegate reviewers).
3. QA enforces merge gate: no merge without both approvals when flagged.
4. Architect resolves technical direction only after both checks complete.

## 6. Finance Impact Board (Required Decisions)
Required financial approval before merge:
1. Dual-book atomic posting behavior (both-or-neither).
2. Posting rule precedence, rounding, and policy versioning semantics.
3. Provenance requirements (`book_policy_id`, `policy_version`, `fx_rate_set_id`, `ruleset_version`, `workflow_id`).
4. Refund/breakage/liability assumption handling.
5. Stripe settlement/dispute mapping to canonical accounting outcomes.
6. Reconciliation exception severity/SLA design affecting close.

Minimum evidence:
- Rule mapping matrix signed by Finance.
- Replay checksum + dual-book tie-out report.
- CI evidence for posting invariants.

## 7. Controls Impact Board (Required Decisions)
Required controls approval before merge:
1. Non-bypassable SoD policy points (posting, period lock, policy/ruleset change, payment release).
2. Keycloak/OPA entity-boundary enforcement.
3. Outbox/inbox + compensation controls for cross-service flows.
4. Immutable audit chain from source event to journal/recon outcomes.
5. CI merge policy for mandatory control tests.

Minimum evidence:
- Role/action/entity matrix.
- OPA negative/positive authorization test results.
- Audit log trace extracts and CI gate report.

## 8. Pike’s 5 Rules Review Outcome (Sprint 1)
Accepted corrections for Sprint 1:
1. Freeze minimal posting scope only (`order.captured`, `payment.settled`, `refund`) while enforcing dual-book correctness.
2. No broad saga rollout in Sprint 1; use local ACID + outbox/inbox as default.
3. Add explicit dual-book atomicity and provenance to exit gates.
4. Keep controls simple and strict on three critical actions first:
- posting
- period lock
- policy/ruleset change
5. Add quantitative baseline measurement gate (500-user 30m baseline) without optimization work in Sprint 1.

## 9. Major Tool-Use Summary by Squad (Major Activities Only)
| Squad | Major activities | Tool use summary |
|---|---|---|
| Platform | Posting/idempotency/runtime foundation | Rust test harness, schema validators, replay fixture runner |
| Finance | Rule templates/policy packages | Rule fixture packs, dual-book diff fixtures, approval checklist artifacts |
| Integration | SDK + contract harnesses | Connector contract tests, webhook replay/signature simulator, ERP/payment contract suites |
| Controls&Sec | Identity/policy baseline | Keycloak config tooling, OPA/Rego test suite, policy evidence extracts |
| Data | Recon model/taxonomy | Schema/migration validators, seeded mismatch datasets, lineage checks |
| QA/Release | CI gates + signoff evidence | CI orchestration, contract/policy pipeline runners, gate dashboards |

## 10. Sprint 1 Exit Gate (Authoritative)
Sprint 1 exits only if all pass:
1. Canonical schema v1.0 frozen and signed by Architecture + Finance.
2. Posting API v0 idempotency + immutable write-path tests pass.
3. Connector SDK v0 contract tests pass (replay-safe).
4. Access model v0 (`RBAC + ABAC`) with policy tests pass.
5. Finance and Controls approvals complete for all flagged decisions.
6. CI gates green for schema/posting/policy/contract suites.
