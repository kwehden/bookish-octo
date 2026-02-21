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
