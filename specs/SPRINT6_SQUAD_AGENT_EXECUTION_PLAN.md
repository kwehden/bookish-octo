# Sprint 6 Squad Agent Execution Plan

Last updated: February 22, 2026  
Scope: Sprint 6 (Weeks 11-12), Pilot Cutover and Production Readiness (US + Canada ski-resort pilot)

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

## 2. Sprint 6 Build Assignments by Squad
Verified Sprint 5 baseline input:
- Compliance hardening baseline is implemented with expanded controls and evidence artifacts.
- Sprint 5 engineering baseline is `GO`; external UAT and performance certification remain open for production exit.
- Gate runner now enforces Sprint 5 artifacts and section checks.

| Squad | Epic focus (Sprint 6) | Build assignments |
|---|---|---|
| Platform | A/B/H/K | Production hardening for posting/rev-rec paths, operational runbooks, and no-bend scaling instrumentation for 2,000-user comfort targets |
| Finance | C/J | Final Finance board closeout, dual-book pilot close package, and go-live cutover reconciliation pack |
| Integration | D/E/I | Pilot cutover readiness for Inntopia/Square + Stripe, rollback runbooks, and controlled replay backlogs |
| Controls&Sec | H | Final control attestations, incident-response control drills, and release-control signoff enforcement |
| Data/Reconciliation | F/G | Pilot close monitoring dashboards, exception-burn-down controls, and post-cutover evidence exports |
| QA/Release | Cross-epic release gates | Final UAT/performance certification, release rehearsal, and go/no-go recommendation |

## 3. Per-Member Work Plan
### Platform
- `PL-L`: Own production hardening architecture and scale/no-bend signoff.
- `PL-W1`: Implement operational runbooks and readiness probes for posting/rev-rec/audit endpoints.
- `PL-W2`: Implement 2,000-user comfort instrumentation baselines and trend reporting hooks.

### Finance
- `FI-L`: Own Finance cutover board and dual-book close signoff.
- `FI-W1`: Produce pilot close reconciliation pack (US + CA entities).
- `FI-W2`: Finalize rev-rec disclosure acceptance and tie-out signoff package.

### Integration
- `IN-L`: Own cutover dependency board for connectors and payment provider integrations.
- `IN-W1`: Implement controlled replay backlog execution and rollback checkpoints for Inntopia.
- `IN-W2`: Implement controlled replay backlog execution and rollback checkpoints for Square/Stripe.

### Controls&Sec
- `CS-L`: Own production control attestation and release-control signoff.
- `CS-W1`: Execute incident-response control drills for tamper/access/PCI scenarios.
- `CS-W2`: Implement final release-control checklist automation and evidence bundle exports.

### Data/Reconciliation
- `DA-L`: Own pilot close monitoring and exception-burn-down governance.
- `DA-W1`: Implement post-cutover dispute/SLA trend monitors and escalation thresholds.
- `DA-W2`: Implement daily compliance export packaging for cutover week.

### QA/Release
- `QA-L`: Own final go/no-go recommendation.
- `QA-W1`: Execute production-like UAT scripts and defect zeroing workflow.
- `QA-W2`: Execute performance certification suite and release rehearsal evidence capture.

## 4. Architect Coordination and Updates
Cadence:
1. Daily squad standups (internal, 15 min).
2. Daily Architect sync (all leaders, 20 min).
3. Tue/Thu cutover dependency board (30 min).
4. Friday release-control review with Finance + Controls mandatory (45 min).

Architect update template:
```md
Squad: <...>
RAG: <Green|Amber|Red>
Completed (24h):
Next (24h):
Dependencies/Blockers:
Decisions Needed: <decision> | financial_impact:<yes/no> | control_impact:<yes/no>
Exit-Criteria Progress: uat_attestation:% | perf_cert:% | cutover_dry_run:% | release_controls:% | critical_defects_open:#
```

Architect Update 1 (current):
- Sprint 6 scope fixed to pilot cutover and production-readiness closure.
- Sprint 5 external UAT/performance closures remain mandatory entry dependencies.
- Pike-rule simplification retained: production readiness must be evidenced through measurable gates before launch.

## 5. Decision Impact Workflow (Finance + Controls)
1. Any decision marked `financial_impact=yes` or `control_impact=yes` is blocked for merge.
2. Required approvals:
- Finance Impact Board approval (`FI-L` + delegate reviewers).
- Controls Impact Board approval (`CS-L` + delegate reviewers).
3. QA enforces merge gate: no merge without both approvals when flagged.
4. Architect resolves technical direction only after both checks complete.
5. Decisions affecting cutover sequencing, disclosure signoff, control attestations, or release-control overrides are always flagged.

## 6. Finance Impact Board (Required Decisions)
Required financial approval before merge:
1. Final dual-book close package for pilot entities.
2. Cutover reconciliation thresholds and variance handling.
3. Revenue disclosure acceptance for pilot launch week.
4. Defect severity policy for finance-critical UAT cases.
5. Release readiness criteria for finance operations handoff.
6. Post-cutover daily reconciliation evidence requirements.

Minimum evidence:
- Signed dual-book close package.
- Cutover tie-out report with approved thresholds.
- Finance UAT completion and defect disposition report.

## 7. Controls Impact Board (Required Decisions)
Required controls approval before merge:
1. Release-control override policy and required approvals.
2. Incident-response drill completion for tamper/access/PCI controls.
3. Final access-review and PCI ownership attestations for go-live.
4. Control evidence retention and chain-of-custody process for cutover week.
5. CI merge-policy requirements for production launch gate.

Minimum evidence:
- Signed release-control checklist.
- Incident drill report with pass/fail outcomes.
- Final control attestation bundle linked in governance artifacts.

## 8. Pike’s 5 Rules Review Outcome (Sprint 6)
Accepted corrections for Sprint 6:
1. Ship one complete cutover path first, then expand post-launch scope.
2. Keep release gates objective and measurable; avoid discretionary hand-waving.
3. Resolve critical defects before optimization or feature expansion.
4. Preserve immutable evidence for launch decisions and control attestations.
5. Block launch unless UAT, performance, and control certifications are all green.

## 9. Major Tool-Use Summary by Squad (Major Activities Only)
| Squad | Major activities | Tool use summary |
|---|---|---|
| Platform | Production hardening + no-bend instrumentation | Rust regression suites, runtime metrics checks, runbook validation |
| Finance | Dual-book cutover package | Tie-out comparators, signoff matrices, disclosure acceptance evidence |
| Integration | Connector/payment cutover and rollback readiness | Replay/backfill rehearsal tools, recovery validation suites |
| Controls&Sec | Final control attestations | OPA suites, incident drill evidence validators, release checklist checks |
| Data | Post-cutover monitoring + exports | Reconciliation trend jobs, SLA dashboards, compliance export validators |
| QA/Release | Final UAT/perf/release rehearsal | End-to-end suites, perf certification runs, release gate dashboards |

## 10. Sprint 6 Exit Gate (Authoritative)
Sprint 6 exits only if all pass:
1. Final UAT attestation is complete with critical/high defects at zero for MVP launch scope.
2. Performance certification validates 2,000-user comfort target with no bend in scaling curve.
3. Cutover rehearsal completes with validated rollback and replay checkpoints.
4. Finance dual-book close package and disclosure acceptance are fully approved.
5. Controls attestation bundle (tamper/access/PCI/release controls) is complete and approved.
6. Post-cutover reconciliation/export monitoring is active and validated.
7. CI gates remain green for contracts, code, controls, and release evidence checks.
8. Finance and Controls approvals are complete for all flagged Sprint 6 decisions.
9. Final architect go/no-go packet is published and signed.
