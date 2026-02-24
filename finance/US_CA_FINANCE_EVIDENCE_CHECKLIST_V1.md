# US/CA Finance Evidence Checklist v1

Closure convention:
- `[x]` means in-repo artifact closure is recorded with explicit references.
- `[ ]` means evidence is still open or outside repository scope.

## Baseline Checklist

- [x] Dual-book tie-out (`US_GAAP` vs `IFRS`) baseline closure recorded in `finance/POSTING_RULE_TEMPLATE_PACK_V1.md`, `governance/sprint1_signoff_packet.md`, and `governance/SPRINT1_PLAN_COMPLETION_REPORT.md`
- [x] Posting rule precedence approval evidence recorded in `finance/POSTING_RULE_TEMPLATE_PACK_V1.md` and `governance/impact_decisions.json` (`DEC-001`, `DEC-002`)
- [x] FX rate-set provenance field validation recorded in `finance/POSTING_RULE_TEMPLATE_PACK_V1.md` and `qa/SPRINT1_GATE_REPORT.md`
- [x] Exception severity/SLA mapping evidence recorded in `contracts/exception_taxonomy_v0.json` and `qa/SPRINT1_GATE_REPORT.md`
- [x] Country evidence checklist linkage (US + CA) recorded in `finance/COA_DIMENSIONS_V1.md` and `governance/sprint1_signoff_packet.md`

## Sprint 2 Checklist

- [x] SoD evidence confirms `finance_operator` denied for `posting`, `mapping_change`, and `ruleset_change` (`policies/opa/access.rego`, `policies/opa/access_test.rego`, `qa/SPRINT2_GATE_REPORT.md`)
- [x] Sponsor-impact review attached in `governance/sprint2_signoff_packet.md`
- [x] Break-glass TTL evidence validates max window and expiry enforcement (`policies/opa/access.rego`, `policies/opa/access_test.rego`)
- [x] Break-glass audit metadata evidence includes `ticket_id`, `approved_by`, and `audit_ref` (`policies/opa/access.rego`, `policies/opa/access_test.rego`)
- [x] Sprint 2 signoff packet attached in `governance/sprint2_signoff_packet.md`
- [x] Sprint 2 completion rubric attached in `governance/SPRINT2_PLAN_COMPLETION_REPORT.md`
- [x] OPA test evidence attached for Sprint 2 controls (`opa test policies/opa`; references: `qa/SPRINT2_GATE_REPORT.md`, `governance/sprint2_signoff_packet.md`)

## Sprint 3 Checklist

- [x] SoD evidence confirms `finance_operator` denied for `estimate_change` and `dispute_approval` (`policies/opa/access.rego`, `policies/opa/access_test.rego`, `qa/SPRINT3_GATE_REPORT.md`)
- [x] Non-bypassable control evidence confirms break-glass denial for `estimate_change` and `dispute_approval` (`policies/opa/access_test.rego`)
- [x] Break-glass logging completeness evidence includes `ticket_id`, `reason`, `approved_by`, `audit_ref`, `log_entry_id` (`policies/opa/access.rego`, `policies/opa/access_test.rego`)
- [x] Break-glass logging timestamp evidence validates `approved_at_ns`, `logged_at_ns`, and `request_time_ns` ordering (`policies/opa/access.rego`, `policies/opa/access_test.rego`)
- [x] Sprint 3 signoff packet attached in `governance/sprint3_signoff_packet.md`
- [x] Sprint 3 completion rubric attached in `governance/SPRINT3_PLAN_COMPLETION_REPORT.md`
- [x] OPA test evidence attached for Sprint 3 controls (`opa test policies/opa`; references: `qa/SPRINT3_GATE_REPORT.md`, `governance/sprint3_signoff_packet.md`)

## Sprint 4 Checklist

- [x] SoD evidence confirms `finance_operator` denied for `posting_approval`, `intercompany_posting_approval`, `close_approval`, and `master_data_change`
- [x] Multi-entity intercompany approval evidence validates distinct `from_legal_entity_id`/`to_legal_entity_id` pair and in-pair approver entity membership
- [x] Intercompany governance evidence includes `intercompany_contract_id` and `journal_batch_id` for each approval trace
- [x] Close approval evidence validates required `close_period_id` and `close_checklist_id` before authorization
- [x] Master-data governance evidence validates `master_data_domain`, `change_set_id`, and `change_request_id` linkage
- [x] Non-bypassable control evidence confirms break-glass denial for Sprint 4 approval/master-data actions
- [x] Sprint 4 signoff packet attached in `governance/sprint4_signoff_packet.md`
- [x] Sprint 4 completion rubric attached in `governance/SPRINT4_PLAN_COMPLETION_REPORT.md`
- [x] OPA test evidence attached for Sprint 4 controls (`opa test policies/opa`)

## Sprint 5 Checklist

- [x] Rev-rec rollforward endpoint evidence attached (`/v1/revrec/rollforward`) with dual-book fixture validation
- [x] Rev-rec disclosures endpoint evidence attached (`/v1/revrec/disclosures`) with policy/fx metadata coverage
- [x] Legal-hold and immutable adjustment workflow evidence linked with regression tests
- [x] Tamper log seal verification evidence linked (`controls/TAMPER_SEALING_VERIFICATION_V1.md`)
- [x] Access-review export evidence linked (`controls/ACCESS_REVIEW_REPORTING_V1.md`)
- [x] PCI scope/control ownership evidence linked (`controls/PCI_SCOPE_CONTROL_OWNERSHIP_MATRIX_V1.md`)
- [x] Dispute-aging and unresolved-SLA evidence linked (`specs/SPRINT5_RECON_EVIDENCE_EXPORT.md`)
- [x] Sprint 5 signoff packet attached in `governance/sprint5_signoff_packet.md`
- [x] Sprint 5 completion rubric attached in `governance/SPRINT5_PLAN_COMPLETION_REPORT.md`
- [x] OPA test evidence attached for Sprint 5 controls (`opa test policies/opa`)

## Sprint 6 Checklist

- [x] Pilot dual-book close package and tie-out approvals recorded for US + CA entities (`DEC-009`)
- [x] Cutover reconciliation variance thresholds and escalation policy approvals recorded (`DEC-010`)
- [x] Launch-week revenue disclosure acceptance approvals recorded (`DEC-011`)
- [x] Release-control override and chain-of-custody governance approvals recorded (`DEC-012`)
- [x] Post-cutover reconciliation monitoring release threshold approvals recorded (`DEC-013`)
- [x] Final Sprint 6 go/no-go authority and escalation protocol approvals recorded (`DEC-014`)
- [x] Sprint 6 signoff packet attached in `governance/sprint6_signoff_packet.md`
- [x] Sprint 6 completion rubric attached in `governance/SPRINT6_PLAN_COMPLETION_REPORT.md`
- [x] Sprint 6 QA gate report attached in `qa/SPRINT6_GATE_REPORT.md`
- [x] Sprint 6 cutover monitoring contract attached in `specs/SPRINT6_CUTOVER_MONITORING_CONTRACT.md`
- [ ] External business-owner UAT attestation archived (outside this repository)
- [ ] External performance certification archived (outside this repository)

## Sprint 7 Checklist

- [x] Baseline finance evidence backfill completed for dual-book tie-out, posting precedence, FX provenance, and exception taxonomy references
- [x] Sprint 2 finance checklist closure backfilled with explicit policy/test/signoff references
- [x] Sprint 3 finance checklist closure backfilled with explicit policy/test/signoff references
- [x] Sprint 1/2/3 completion artifact chain linked (`signoff -> gate -> completion report`)
- [x] External attestation non-claim preserved (UAT/performance remain open outside repository)
