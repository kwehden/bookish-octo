# Sprint 2 Signoff Packet

## Scope
- Non-bypassable SoD checks for `posting`, `mapping_change`, and `ruleset_change`.
- Break-glass governance controls: TTL maximum, active-window validation, and mandatory audit metadata.
- Sprint 2 gate wiring in `scripts/run_contract_gates.sh` for artifact integrity checks.

## Required approvals
- [ ] Architecture signoff (`ARCH-L`)
- [ ] Finance signoff (`FI-L`)
- [ ] Controls signoff (`CS-L`)
- [ ] QA/Release gate signoff (`QA-L`)
- [ ] Sponsor impact signoff (`SP-L`)

## Sponsor-Impact Summary
- Finance: reduces unauthorized posting and configuration-change exposure by enforcing role-based SoD.
- Controls/SOX: ensures emergency access is time-bounded and audit-linked.
- Operations: allows controlled `period_lock` emergency action without allowing posting/ruleset bypass.
- Delivery: gate runner now fails fast when Sprint 2 governance artifacts are missing.

## Impact-board references
- Finance/controls decision source: `governance/impact_decisions.json`
- Architecture status updates: `governance/architect_updates.md` (Update 3)

## Evidence links
- OPA policy: `policies/opa/access.rego`
- OPA tests: `policies/opa/access_test.rego`
- Gate runner: `scripts/run_contract_gates.sh`
- Controls register update: `controls/CONTROL_GATES_REGISTER_V1.md`
- Finance checklist update: `finance/US_CA_FINANCE_EVIDENCE_CHECKLIST_V1.md`

## Execution evidence
- OPA policy test command: `opa test policies/opa`
- Full gate command: `scripts/run_contract_gates.sh`

## Signoff record
- Target signoff date: `2026-02-21`
- Decision ID range reserved for Sprint 2 controls: `DEC-S2-001` through `DEC-S2-003`
