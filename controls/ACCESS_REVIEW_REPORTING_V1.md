# Access Review Reporting v1

Date: `2026-02-22`  
Scope: Sprint 5 access-review export controls (US + Canada pilot)

Status: PASS

## Control Output
- `review_attestation_complete: true`
- `review_period`: `2026-Q1`
- `required_action`: `access_review_export`
- `required_fields`:
  - `review_period_id`
  - `review_owner`
  - `evidence_ref`

## Evidence
- OPA controls:
  - `test_allow_access_review_export_with_required_fields`
  - `test_deny_access_review_export_missing_evidence`
- Governance linkage:
  - `governance/sprint5_signoff_packet.md`
  - `governance/impact_decisions.json` (`DEC-006`)

## Ownership and Cadence
- Frequency: quarterly
- Primary owner: `CS-L`
- Secondary reviewer: `FI-L`
- Evidence retention: 7 years in immutable governance artifact store
