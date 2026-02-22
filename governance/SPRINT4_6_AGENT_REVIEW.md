# Sprint 4-6 Squad Agent Review

Date: 2026-02-22
Method: Parallel squad-agent review aligned to sprint assignments (Platform, Finance, Integration, Controls&Sec, Data, QA/Release).
Scope: Sprint 4, Sprint 5, Sprint 6 artifacts and implementation.

## Findings (Severity Ordered)

1. High - Program release remains NO-GO due missing external attestations in Sprint 5/6.
- Missing business-owner UAT attestation and 2,000-user no-bend performance certification remain explicit blockers.
- References: `governance/sprint5_signoff_packet.md`, `qa/SPRINT5_GATE_REPORT.md`, `governance/sprint6_signoff_packet.md`, `qa/SPRINT6_GATE_REPORT.md`, `governance/SPRINT6_PLAN_COMPLETION_REPORT.md`
- Status: Open

2. Resolved - Sprint 4 Finance/Controls approvals and evidence gap is closed in-repo.
- Sprint 4 signoff/checklist/gate/completion artifacts now record completed in-repo Finance+Controls evidence and approvals; remaining blockers are explicitly external Sprint 5/6 attestations.
- References: `finance/US_CA_FINANCE_EVIDENCE_CHECKLIST_V1.md`, `governance/SPRINT4_PLAN_COMPLETION_REPORT.md`, `governance/sprint4_signoff_packet.md`, `qa/SPRINT4_GATE_REPORT.md`
- Status: Closed

3. High - Integration plan includes Stripe in Sprint 6 cutover, but Stripe adapter/replay coverage is missing.
- Connector implementation and tests currently cover Inntopia and Square; Stripe replay/cutover path is not implemented.
- References: `specs/SPRINT6_SQUAD_AGENT_EXECUTION_PLAN.md`, `crates/connector-sdk/src/lib.rs`, `crates/connector-sdk/src/inntopia.rs`, `crates/connector-sdk/src/square.rs`
- Status: Open

4. High - Canonical event contract mismatch risk in connector output.
- The canonical schema requires fields not represented in connector `CanonicalEvent` output shape, creating contract drift risk.
- References: `contracts/canonical_event_v1.schema.json`, `crates/connector-sdk/src/lib.rs`
- Status: Open

5. High - Platform idempotency and audit-seal storage are in-memory and non-durable.
- Restart/multi-instance behavior can lose replay history and audit-chain continuity.
- References: `crates/platform-core/src/lib.rs`
- Status: Open

6. Medium - Platform global lock patterns and unbounded idempotency cache threaten 2,000-user scaling goal.
- Shared `Mutex` hot paths and non-evicted idempotency result cache can bend latency/memory curves.
- References: `crates/posting-api/src/lib.rs`
- Status: Open

7. Medium - Data/reconciliation zero-denominator metric behavior may overstate success.
- If denominator is zero, metric conversion can report 100%, masking no-data scenarios.
- References: `crates/reconciliation-model/src/lib.rs`
- Status: Open

8. Medium - Reconciliation index cloning pattern may introduce avoidable memory/CPU pressure.
- Per-order cloned vectors from indexes can inflate memory and reduce throughput at scale.
- References: `crates/reconciliation-model/src/lib.rs`
- Status: Open

## Pike 5-Rules Review Notes

- Rule 1 (small teams): Findings emphasize reducing brittle central bottlenecks and clarifying ownership of external attestations.
- Rule 2 (prototype quickly): Sprint velocity is strong, but external evidence closure lag is now the primary bottleneck.
- Rule 3 (systems language): Rust foundation remains appropriate; current risks are architecture/state durability and operations evidence, not language fit.
- Rule 4 (distribution): Current in-memory control points need durable/distributed backing for confident multi-instance rollout.
- Rule 5 (data): Gate outputs are improving, but metric semantics and evidence completeness still need hardening.

## Great Refactor Decisions/Actions (Finance+Controls, Pike's 5)

| Pike rule framing | Finance+Controls decision | Action status |
|---|---|---|
| 1. Do not guess bottlenecks | Treat missing attestations as explicit blockers, not inferred completion. | External UAT/performance attestations remain open and explicitly tracked in Sprint 5/6 artifacts. |
| 2. Measure before tuning | Use scored completion rubrics and gate snapshots for approval state. | Sprint 4 completion and gate reports now carry quantified, in-repo closure evidence. |
| 3. Fancy loses when n is small | Keep approval workflow simple: packet/checklist/gate/completion alignment before broader automation. | Sprint 4 Finance+Controls closure normalized across the four Sprint 4 artifacts. |
| 4. Fancy is buggier; prefer simple | Keep one deterministic approval evidence chain in-repo. | Signoff/checklist/gate/completion references now point to one consistent evidence path. |
| 5. Data dominates | Make decision and evidence metadata the source of truth. | Approval state is documented via linked artifacts, while external attestations are tracked as explicit out-of-repo TODOs. |

## Recommended Next Actions

1. Close external UAT/performance attestations and attach immutable references in Sprint 5/6 signoff packets.
2. Keep Sprint 4 Finance/Controls closure immutable and isolated from external attestation dependencies.
3. Implement Stripe adapter plus replay/cutover tests to match declared Sprint 6 scope.
4. Resolve canonical-event/schema drift and add explicit conformance tests.
5. Migrate idempotency/audit persistence to durable store and add cache eviction/TTL controls.
6. Fix zero-denominator metric semantics and reduce reconciliation cloning overhead.
