# ADR-002: Distributed Transaction Standard (Outbox/Inbox Default, Selective Saga)

- Status: Proposed
- Date: 2026-02-21
- Owners: Architecture Team
- Related: `specs/ARCHITECTURAL_GUIDANCE.md`, `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`

## Context
Cross-service workflows are required, but broad saga orchestration by default introduces complexity and failure risk at pilot stage.

## Decision
Adopt this transaction standard:
1. Default: local ACID transaction + outbox write.
2. Consumer side: inbox dedupe with idempotency key.
3. Use orchestration-based saga only for flows that cross service boundaries and require compensation.
4. Compensation must be business-reversal events/journals (no history deletion).

## Consequences
### Positive
- Strong correctness with lower complexity.
- Replay-safe behavior and predictable failure handling.

### Negative
- Some cross-service flows require explicit compensation design effort.

## Alternatives Considered
1. Global XA/2PC.
2. Saga everywhere.

## Validation
- No orphan outbox events.
- Compensation tests pass for all saga-enabled flows.
- Deterministic replay remains stable.

## Follow-ups
- Publish per-flow saga registry with step-level compensation plans.
