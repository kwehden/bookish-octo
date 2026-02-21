# Pike Rules Devil's Advocate Review

Last updated: February 21, 2026
Review basis:
- `specs/ARCHITECTURAL_GUIDANCE.md`
- `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`
- existing repo specs for alignment

Pike rules used as overriding evaluation criteria:
1. Do not guess bottlenecks.
2. Measure before tuning.
3. Fancy algorithms lose when n is small.
4. Fancy approaches are buggier; prefer simple.
5. Data dominates.

## 1. Executive Verdict
Verdict: **Approve with guardrails**.

Rationale:
1. Complexity was reduced from broad microservice-first to a small deployable topology.
2. Distributed transactions are scoped to necessary cross-service flows, not defaulted globally.
3. 2,000-user scale is now explicitly measurable with no-bend acceptance criteria.
4. Data-model determinism and provenance are now first-class requirements.

## 2. Findings and Disposition

### Critical Findings Addressed
1. Conflicting sponsor decisions across drafts.
- Status: Resolved.
- Action: Locked decisions centralized in `specs/ARCHITECTURAL_GUIDANCE.md` section 1.

2. Distributed saga complexity as default behavior.
- Status: Resolved.
- Action: Local ACID + outbox/inbox is default; saga only where required.

3. Incomplete deterministic replay invariants.
- Status: Resolved.
- Action: Mandatory posting provenance and replay invariants required in epic spec.

### High Findings Addressed
1. DuckDB-first without objective migration trigger.
- Status: Resolved.
- Action: Explicit soft and hard migration triggers added.

2. Too many abstractions too early.
- Status: Resolved.
- Action: One production implementation per adapter seam for MVP; others contract-tested.

3. Pilot scope overload risk.
- Status: Partially resolved.
- Action: US first + CA shadow recommendation is documented; execution governance must enforce it.

### Remaining Risks (Open)
1. Actor hotspotting under skewed tenant traffic.
- Mitigation: shard rebalancing + mailbox depth alerts.
2. Storage contention under close windows.
- Mitigation: load gates and migration triggers.
3. Country-pack compliance drift.
- Mitigation: quarterly rule-pack review and regression baseline.

## 3. Rule-by-Rule Scorecard

| Pike Rule | Result | Notes |
|---|---|---|
| Do not guess bottlenecks | Pass | Bottleneck assumptions replaced by test gates and observability thresholds. |
| Measure before tuning | Pass | Scale SLOs and no-bend criteria are explicit release gates. |
| Fancy loses for small n | Pass | Pilot architecture simplified to fewer deployables and selective complexity. |
| Fancy is buggier | Pass | Saga and adapter complexity constrained to required surfaces. |
| Data dominates | Pass | Deterministic data invariants and provenance are mandatory. |

## 4. Decision Review (Accept/Reject)

| Decision | Result | Reason |
|---|---|---|
| Rust | Accept | Strong type safety + single language focus. |
| Ractor | Accept with limits | Use for bounded actor domains; avoid unnecessary actor proliferation. |
| Distributed transactions | Accept with limits | Use outbox/inbox default, selective saga orchestration only. |
| DuckDB-first | Accept with exit criteria | Allowed for pilot profile with strict migration triggers. |
| DB abstraction | Accept | Required for portability, kept narrow and contract-driven. |
| Dual-book from MVP | Accept | Needed for sponsor requirements and accounting correctness. |
| ERP strategy (ERPNext first, Odoo compatible) | Accept | ERPNext implementation + Odoo contract tests in MVP. |
| Stripe-first gateway abstraction | Accept | Practical and future-proof if provider-neutral contracts are preserved. |
| US/CA pilot scope | Accept with sequencing | US production first, CA shadow/go-live after gates pass. |

## 5. Mandatory Governance Gates
1. No new service boundary without measured bottleneck evidence.
2. No optimization without benchmark and rollback criteria.
3. No production release without 2,000-user no-bend scale gate pass.
4. No close signoff without dual-book parity checks.

