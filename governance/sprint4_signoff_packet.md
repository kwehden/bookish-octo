# Sprint 4 Signoff Packet

## Scope
- Sprint 4 controls/finance/governance baseline for multi-entity and consolidation authorization gates.
- OPA hardening for non-bypassable actions across posting approvals, intercompany approvals, close approvals, and master-data changes.
- Sprint 4 evidence-chain and gate-script enforcement for required governance/control artifacts.

## Required approvals
- [ ] Architecture signoff (`ARCH-L`)
- [ ] Finance signoff (`FI-L`)
- [ ] Controls signoff (`CS-L`)
- [ ] QA/Release gate signoff (`QA-L`)
- [ ] Sponsor impact signoff (`SP-L`)

## Sponsor-Impact Summary
- Finance: intercompany posting approvals now require explicit contract and journal trace fields, reducing consolidation posting ambiguity.
- Controls/SOX: close approval and master-data changes are now explicit non-bypassable SoD actions with deterministic authorization metadata requirements.
- Governance: Sprint 4 completion evidence is now standardized through an exit-gate rubric aligned to the authoritative Sprint 4 plan.
- Delivery: gate runner fails fast if Sprint 4 artifact set or required Sprint 4 document sections are missing.

## Impact-board references
- Sprint 4 plan source: `specs/SPRINT4_SQUAD_AGENT_EXECUTION_PLAN.md`
- Architecture status updates: `governance/architect_updates.md` (Update 5)

## Evidence links
- OPA policy: `policies/opa/access.rego`
- OPA tests: `policies/opa/access_test.rego`
- Gate runner: `scripts/run_contract_gates.sh`
- Controls register update: `controls/CONTROL_GATES_REGISTER_V1.md`
- Finance checklist update: `finance/US_CA_FINANCE_EVIDENCE_CHECKLIST_V1.md`
- Sprint 4 completion report: `governance/SPRINT4_PLAN_COMPLETION_REPORT.md`

## Execution evidence
- OPA policy test command: `opa test policies/opa`
- Full gate command: `scripts/run_contract_gates.sh`

## Signoff record
- Target signoff date: `2026-02-21`
- Decision ID range reserved for Sprint 4 controls: `DEC-S4-001` through `DEC-S4-006`
