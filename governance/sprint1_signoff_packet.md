# Sprint 1 Signoff Packet

## Required approvals
- [ ] Architecture signoff (`ARCH-L`)
- [ ] Finance signoff (`FI-L`)
- [ ] Controls signoff (`CS-L`)
- [ ] QA/Release gate signoff (`QA-L`)

## Impact-board references
- Finance impact decisions: `governance/impact_decisions.json`
- Controls policy/tests: `policies/opa/access.rego`, `policies/opa/access_test.rego`

## Evidence links
- Schema: `contracts/canonical_event_v1.schema.json`
- Posting: `crates/posting-api/src/lib.rs`, `crates/ledger-posting/src/lib.rs`
- Integration contracts: `crates/connector-sdk/src/lib.rs`
- Reconciliation model: `contracts/reconciliation_model_v0.json`, `contracts/exception_taxonomy_v0.json`
- QA gate output: `qa/SPRINT1_GATE_REPORT.md`

## Sprint 2 Reference
- Follow-on signoff packet: `governance/sprint2_signoff_packet.md`
