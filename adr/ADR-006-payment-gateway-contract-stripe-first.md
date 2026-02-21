# ADR-006: Payment Gateway Contract with Stripe-First Implementation

- Status: Proposed
- Date: 2026-02-21
- Owners: Payments Team
- Related: `specs/ARCHITECTURAL_GUIDANCE.md`, `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`

## Context
Sponsor decision is Stripe as initial payment partner while ensuring configurability for future providers.

## Decision
1. Define provider-neutral `PaymentGateway` interface.
2. Implement Stripe as first production provider.
3. Normalize settlement/dispute/payment lifecycle into canonical model.
4. Keep provider-specific fields in extension metadata only.

## Consequences
### Positive
- Fast Stripe launch with minimal lock-in.
- Easier provider expansion later.

### Negative
- Requires strict conformance testing to avoid provider-specific drift.

## Alternatives Considered
1. Stripe-specific direct integration only.
2. Multi-provider production build in MVP.

## Validation
- Stripe end-to-end scenarios pass.
- Alternate provider stub passes gateway conformance tests.

## Follow-ups
- Add second provider adapter after pilot stabilization based on business priority.
