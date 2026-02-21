# ADR-007: Scale SLO and No-Bend Acceptance Gates (2,000 Active Users)

- Status: Proposed
- Date: 2026-02-21
- Owners: Platform + SRE
- Related: `specs/ARCHITECTURAL_GUIDANCE.md`, `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`

## Context
The platform must comfortably support 2,000 active users without a meaningful bend in scaling behavior.

## Decision
Adopt release-blocking scale gates:
1. Target tests at 2,000 active users with defined traffic mix and peak profile.
2. No-bend criteria:
- Scaling efficiency `E(N) >= 0.80` for N=2..8 replicas.
- p95 latency step degradation <=10%.
- Error-rate drift <=0.05% absolute.
3. Mandatory pass for target + soak + spike tests before pilot go-live.

## Consequences
### Positive
- Objective performance decision-making.
- Early detection of architecture bottlenecks.

### Negative
- Requires dedicated performance test infrastructure and dataset management.

## Alternatives Considered
1. Informal performance testing without release gates.
2. Throughput-only gating without scaling efficiency checks.

## Validation
- All scale test scenarios pass SLOs.
- Ledger invariants hold under peak and soak workloads.

## Follow-ups
- Publish weekly SLO burn and capacity trend reports during pilot phase.
