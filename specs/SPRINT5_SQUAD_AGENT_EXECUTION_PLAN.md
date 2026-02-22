# Sprint 5 Squad Agent Execution Plan

Last updated: February 21, 2026
Scope: Sprint 5 (Weeks 9-10), Compliance Hardening and UAT (US + Canada ski-resort pilot)

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

## 2. Sprint 5 Build Assignments by Squad
Verified Sprint 4 baseline input:
- Multi-entity boundary, intercompany, and close-checklist baseline artifacts are implemented.
- Sprint 4 full program exit remains conditional on approvals and stage evidence closure.
- Controls gate runner now enforces Sprint 4 artifacts and section checks.

| Squad | Epic focus (Sprint 5) | Build assignments |
|---|---|---|
| Platform | A/B/H | Implement retention and legal-hold controls, immutable adjustment workflow, and audit-export-safe data interfaces |
| Finance | C/J | Finalize rev-rec disclosures, rollforward outputs, and consolidation-ready finance evidence packages |
| Integration | D/E/I | Harden replay/backfill tooling and execute connector resiliency tests for Inntopia/Square flows |
| Controls&Sec | H + cross-epic gates | Implement tamper-evident log sealing, access-review reporting, and PCI scope/control ownership matrix |
| Data/Reconciliation | F/G | Add dispute aging, unresolved exception SLA metrics, and compliance-facing reconciliation evidence exports |
| QA/Release | Cross-epic quality gates | Execute full regression + performance + UAT suite and publish readiness recommendation |

## 3. Per-Member Work Plan
### Platform
- `PL-L`: Own retention/legal-hold architecture and immutable adjustment signoff package.
- `PL-W1`: Implement data-retention and legal-hold policy enforcement controls in posting/recon data flows.
- `PL-W2`: Implement immutable adjustment workflow with provenance and audit trail validation tests.

### Finance
- `FI-L`: Own finance disclosure/rollforward completion and signoff routing for UAT evidence.
- `FI-W1`: Implement rev-rec disclosure output pack and disclosure validation fixtures.
- `FI-W2`: Implement rollforward output generation and dual-book reconciliation evidence tables.

### Integration
- `IN-L`: Coordinate connector resiliency scope and replay/backfill readiness burn-down.
- `IN-W1`: Harden Inntopia replay/backfill workflows with failure-injection and recovery tests.
- `IN-W2`: Harden Square replay/backfill workflows and resiliency telemetry exports.

### Controls&Sec
- `CS-L`: Own tamper-evident control baseline and PCI ownership matrix signoff.
- `CS-W1`: Implement log-sealing controls and verification checks.
- `CS-W2`: Implement access-review reporting pipeline and PCI scope/control ownership matrix artifacts.

### Data/Reconciliation
- `DA-L`: Own dispute-aging and unresolved SLA metrics scope for close readiness.
- `DA-W1`: Implement dispute aging buckets and trend metrics for exception queues.
- `DA-W2`: Implement unresolved exception SLA metrics and compliance-facing evidence export views.

### QA/Release
- `QA-L`: Own Sprint 5 go/no-go recommendation for UAT and compliance readiness.
- `QA-W1`: Execute full regression suite and defect triage matrix.
- `QA-W2`: Execute performance + UAT scripts and capture gate evidence with release recommendations.

## 4. Architect Coordination and Updates
Cadence:
1. Daily squad standups (internal, 15 min).
2. Daily Architect sync (all leaders, 20 min).
3. Tue/Thu compliance dependency board (30 min).
4. Friday sprint control review with Finance + Controls mandatory (45 min).

Architect update template:
```md
Squad: <...>
RAG: <Green|Amber|Red>
Completed (24h):
Next (24h):
Dependencies/Blockers:
Decisions Needed: <decision> | financial_impact:<yes/no> | control_impact:<yes/no>
Exit-Criteria Progress: tamper_evidence_pass:% | access_review_pack:% | resiliency_recovery_pass:% | uat_script_pass:% | critical_defects_open:#
```

Architect Update 1 (current):
- Sprint 5 scope fixed to compliance hardening and UAT readiness evidence.
- Sprint 4 carry-forward approvals and dry-run findings remain explicit blockers where impacted.
- Pike-rule simplification retained: implement minimal strict controls with objective evidence before optimization work.

## 5. Decision Impact Workflow (Finance + Controls)
1. Any decision marked `financial_impact=yes` or `control_impact=yes` is blocked for merge.
2. Required approvals:
- Finance Impact Board approval (`FI-L` + delegate reviewers).
- Controls Impact Board approval (`CS-L` + delegate reviewers).
3. QA enforces merge gate: no merge without both approvals when flagged.
4. Architect resolves technical direction only after both checks complete.
5. Decisions affecting disclosure outputs, retention/legal-hold, PCI scope ownership, or access-review evidence are always flagged.

## 6. Finance Impact Board (Required Decisions)
Required financial approval before merge:
1. Rev-rec disclosure output definitions and required controls for publication.
2. Rollforward output logic and tie-out thresholds for dual-book reporting.
3. Adjustment workflow constraints impacting period close and disclosure timing.
4. Exception-aging and unresolved-SLA treatment for financial close readiness.
5. UAT acceptance criteria for finance-critical user journeys.
6. Evidence packaging format for internal audit/compliance review.

Minimum evidence:
- Signed disclosure output matrix and approval records.
- Rollforward tie-out fixture pack with expected outputs.
- Finance UAT acceptance report with defect disposition and approvals.

## 7. Controls Impact Board (Required Decisions)
Required controls approval before merge:
1. Tamper-evident log sealing implementation and verification controls.
2. Access-review report scope, frequency, and evidence-retention policy.
3. PCI scope/control ownership mapping and responsibility matrix enforcement.
4. Retention/legal-hold controls and override governance.
5. CI merge-policy requirements for compliance and UAT evidence gates.

Minimum evidence:
- Log-sealing verification report with integrity checks.
- Access-review evidence extracts and review attestations.
- PCI scope/control ownership matrix with accountable owners.

## 8. Pike’s 5 Rules Review Outcome (Sprint 5)
Accepted corrections for Sprint 5:
1. Build one complete auditable compliance evidence path first before broad workflow expansion.
2. Keep control implementations explicit and testable with deterministic pass/fail outputs.
3. Prioritize defect elimination in finance/control-critical UAT scenarios before optimization.
4. Preserve immutable provenance for adjustments, disclosures, and evidence exports.
5. Use quantified release gates (defect counts, evidence completeness, UAT pass rates) as hard blockers.

## 9. Major Tool-Use Summary by Squad (Major Activities Only)
| Squad | Major activities | Tool use summary |
|---|---|---|
| Platform | Retention/legal-hold + immutable adjustments | Rust regression suites, retention policy tests, adjustment provenance checks |
| Finance | Disclosure/rollforward completion | Fixture packs, tie-out comparators, finance approval evidence artifacts |
| Integration | Replay/backfill resiliency hardening | Failure-injection harnesses, recovery-time checks, replay contract tests |
| Controls&Sec | Tamper/access/PCI controls | OPA/Rego suites, log-sealing validators, access-review export checks |
| Data | Aging/SLA evidence metrics | Reconciliation metric pipelines, SLA dashboards, compliance export validators |
| QA/Release | Regression/perf/UAT release gates | End-to-end runners, defect dashboards, UAT evidence packaging |

## 10. Sprint 5 Exit Gate (Authoritative)
Sprint 5 exits only if all pass:
1. Tamper-evident log sealing and verification controls pass with evidence artifacts attached.
2. Access-review reports and PCI scope/control ownership matrix are complete and approved.
3. Retention/legal-hold controls and immutable adjustment workflows pass regression tests.
4. Connector replay/backfill resiliency tests pass recovery objectives for in-scope failure scenarios.
5. Rev-rec disclosures and rollforward outputs are complete and Finance-approved.
6. Dispute-aging and unresolved exception SLA metrics are available and validated in QA evidence.
7. Full regression + performance + UAT suites pass with critical/high defects at zero for MVP scope.
8. Finance and Controls approvals are complete for all flagged Sprint 5 decisions.
9. CI gates are green for compliance controls, integrations, platform, data, finance outputs, and QA/UAT suites.
