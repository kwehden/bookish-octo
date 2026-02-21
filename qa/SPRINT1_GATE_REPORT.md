# Sprint 1 Gate Report (Current Run)

## Gate status
- [x] Canonical schema check
- [x] Contract freeze artifact check
- [x] Impact approval check
- [x] OPA policy tests
- [x] Cargo workspace tests

## Notes
- Minimal posting scope enforced: `order.captured.v1`, `payment.settled.v1`, `refund.v1`.
- Dual-book provenance fields required in posting contracts.
- Finance/Controls impact decisions tracked in `governance/impact_decisions.json`.
