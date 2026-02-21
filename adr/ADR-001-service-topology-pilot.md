# ADR-001: Service Topology for Pilot (Macroservice with Strong Module Boundaries)

- Status: Proposed
- Date: 2026-02-21
- Owners: Architecture Team
- Related: `specs/ARCHITECTURAL_GUIDANCE.md`, `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`

## Context
The pilot must deliver deterministic accounting controls quickly while minimizing operational complexity and failure modes. Sponsor constraints require Rust/Ractor, dual-book accounting, Stripe-first gateway abstraction, ERPNext-first integration, and US/CA rollout.

## Decision
Use a reduced deployable topology for pilot:
1. `acct-core-svc`
2. `acct-integration-svc`
3. `acct-erp-bridge-svc`
4. `acct-query-svc`

Maintain strict internal module boundaries in `acct-core-svc` rather than splitting immediately into many microservices.

## Consequences
### Positive
- Reduced distributed complexity and easier debugging.
- Faster delivery and lower operational overhead.
- Better alignment with Pike rules for pilot scale.

### Negative
- Some future decomposition work will be needed if scale hotspots emerge.

## Alternatives Considered
1. Full microservice split by bounded context at MVP.
2. Monolith without module boundaries.

## Validation
- 2,000-user scale gate passes with no-bend criteria.
- No unresolved service-coupling blockers for close-critical workflows.

## Follow-ups
- Revisit decomposition after pilot close and performance profiling.
