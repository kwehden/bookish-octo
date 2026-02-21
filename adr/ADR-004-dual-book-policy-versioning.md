# ADR-004: Dual-Book Policy Package Versioning and Provenance

- Status: Proposed
- Date: 2026-02-21
- Owners: Finance Architecture + Platform
- Related: `specs/CANONICAL_DATA_MODEL_SPEC.md`, `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`

## Context
MVP requires both `US_GAAP` and `IFRS` from day one. Deterministic replay and auditability require policy provenance on posted journals.

## Decision
Require policy provenance on posted journals:
1. `book_policy_id`
2. `policy_version`
3. `fx_rate_set_id`
4. `ruleset_version`
5. `workflow_id` (when distributed workflow applies)

Both books must post as an atomic business outcome (both-or-neither for in-scope transactions).

## Consequences
### Positive
- Strong audit traceability.
- Reliable replay across policy revisions.

### Negative
- Additional operational governance for policy package release management.

## Alternatives Considered
1. Single policy marker per run.
2. No explicit policy provenance fields.

## Validation
- Book parity reports reconcile with documented policy deltas.
- Replay checksums stable under fixed policy/FX sets.

## Follow-ups
- Define policy release and rollback runbook with approval controls.
