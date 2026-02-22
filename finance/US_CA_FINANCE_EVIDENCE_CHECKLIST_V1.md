# US/CA Finance Evidence Checklist v1

- [ ] Dual-book tie-out (`US_GAAP` vs `IFRS`) complete
- [ ] Posting rule precedence approved
- [ ] FX rate-set provenance validated
- [ ] Exception severity/SLA mapping approved
- [ ] Country evidence checklist linked (US + CA)

## Sprint 2 Checklist

- [ ] SoD evidence confirms `finance_operator` denied for `posting`, `mapping_change`, and `ruleset_change`
- [ ] Sponsor-impact review attached in `governance/sprint2_signoff_packet.md`
- [ ] Break-glass TTL evidence validates max window and expiry enforcement
- [ ] Break-glass audit metadata evidence includes `ticket_id`, `approved_by`, and `audit_ref`
- [ ] OPA test evidence attached for Sprint 2 controls (`opa test policies/opa`)

## Sprint 3 Checklist

- [ ] SoD evidence confirms `finance_operator` denied for `estimate_change` and `dispute_approval`
- [ ] Non-bypassable control evidence confirms break-glass denial for `estimate_change` and `dispute_approval`
- [ ] Break-glass logging completeness evidence includes `ticket_id`, `reason`, `approved_by`, `audit_ref`, `log_entry_id`
- [ ] Break-glass logging timestamp evidence validates `approved_at_ns`, `logged_at_ns`, and `request_time_ns` ordering
- [ ] Sprint 3 signoff packet attached in `governance/sprint3_signoff_packet.md`
- [ ] Sprint 3 completion rubric attached in `governance/SPRINT3_PLAN_COMPLETION_REPORT.md`
- [ ] OPA test evidence attached for Sprint 3 controls (`opa test policies/opa`)

## Sprint 4 Checklist

- [ ] SoD evidence confirms `finance_operator` denied for `posting_approval`, `intercompany_posting_approval`, `close_approval`, and `master_data_change`
- [ ] Multi-entity intercompany approval evidence validates distinct `from_legal_entity_id`/`to_legal_entity_id` pair and in-pair approver entity membership
- [ ] Intercompany governance evidence includes `intercompany_contract_id` and `journal_batch_id` for each approval trace
- [ ] Close approval evidence validates required `close_period_id` and `close_checklist_id` before authorization
- [ ] Master-data governance evidence validates `master_data_domain`, `change_set_id`, and `change_request_id` linkage
- [ ] Non-bypassable control evidence confirms break-glass denial for Sprint 4 approval/master-data actions
- [ ] Sprint 4 signoff packet attached in `governance/sprint4_signoff_packet.md`
- [ ] Sprint 4 completion rubric attached in `governance/SPRINT4_PLAN_COMPLETION_REPORT.md`
- [ ] OPA test evidence attached for Sprint 4 controls (`opa test policies/opa`)

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
