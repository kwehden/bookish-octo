# ADR-003: Storage Profile and Migration Contract (DuckDB-First, Pluggable)

- Status: Proposed
- Date: 2026-02-21
- Owners: Architecture Team
- Related: `specs/ARCHITECTURAL_GUIDANCE.md`, `specs/PIKE_DEVILS_ADVOCATE_REVIEW.md`

## Context
Sponsor direction requires DuckDB as initial backing store with future support for compatible distributed database backends.

## Decision
1. Start with `duckdb` storage profile for pilot.
2. Isolate persistence behind repository traits.
3. Maintain a second CI-tested backend profile contract.
4. Trigger migration planning and execution using explicit soft/hard thresholds in architecture guidance.

## Consequences
### Positive
- Fast start with clear migration path.
- Reduced lock-in to a single engine.

### Negative
- Requires strict portability discipline in SQL and schema management.

## Alternatives Considered
1. Start directly on distributed SQL.
2. Hard-couple domain to one engine.

## Validation
- Contract tests pass across both storage profiles.
- Migration drill passes row-count/checksum parity gates.

## Follow-ups
- Decide first alternate backend implementation and run dual-profile performance tests.
