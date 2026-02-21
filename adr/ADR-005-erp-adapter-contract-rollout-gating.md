# ADR-005: ERP Adapter Contract and Rollout Gating

- Status: Proposed
- Date: 2026-02-21
- Owners: ERP Integration Team
- Related: `specs/ARCHITECTURAL_GUIDANCE.md`, `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`

## Context
Sponsor direction is ERPNext-first while preserving compatibility with Odoo and future ERP systems.

## Decision
1. Define a stable `ErpAdapter` contract in core integration layer.
2. Implement ERPNext adapter for MVP production.
3. Implement Odoo adapter as contract-conformance stub during MVP.
4. Gate Odoo production adapter rollout until first successful pilot close.

## Consequences
### Positive
- Prevents ERP-specific leakage into core accounting logic.
- Enables controlled future ERP support.

### Negative
- Adds contract-testing overhead early.

## Alternatives Considered
1. Direct ERPNext coupling without adapter boundary.
2. Full ERPNext + Odoo production adapters at MVP.

## Validation
- Contract tests pass for ERPNext and Odoo stub.
- Core services contain zero ERP-specific conditionals.

## Follow-ups
- Define adapter certification checklist for future ERP implementations.
