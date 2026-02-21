# Spec Alignment and Required Deltas

Last updated: February 21, 2026
Purpose: Evaluate new locked decisions against existing specs and identify required updates.

## 1. Locked Baseline
1. Dual-book from MVP (`US_GAAP` + `IFRS`).
2. ERPNext-first with Odoo-compatible adapter contract.
3. Ski-resort pilot.
4. US + Canada initial countries.
5. Stripe-first gateway with provider abstraction.
6. Rust + Ractor runtime.
7. Distributed transactions via outbox/inbox + selective saga.
8. DuckDB initial storage profile with pluggable DB support.
9. Scale gate: comfortable 2,000 active users with no meaningful bend in scaling curve.

## 2. Compatibility Matrix
Legend: `C` compatible, `P` partial, `X` conflict.

| Decision | `OPEN_SOURCE_ACCOUNTING_SYSTEM_MASTER_SPEC.md` | `IMPLEMENTATION_PLAN_12_WEEKS.md` | `CANONICAL_DATA_MODEL_SPEC.md` | `COUNTRY_ROLLOUT_PACK_TEMPLATE_US_CA.md` |
|---|---|---|---|---|
| Dual-book MVP | P | X | P | X |
| ERPNext-first + Odoo-compatible | P | P | C | P |
| Ski pilot | C | P | P | P |
| US/CA initial | P | P | P | C |
| Stripe-first pluggable | C | P | P | P |
| Rust + Ractor | X | X | C | C |
| DuckDB-first + pluggable DB | P | X | P | P |
| 2,000-user no-bend scale gate | X | X | X | X |

## 3. Highest Priority Conflicts
1. `specs/IMPLEMENTATION_PLAN_12_WEEKS.md`
- US-GAAP-only assumption conflicts with dual-book MVP.
2. `specs/OPEN_SOURCE_ACCOUNTING_SYSTEM_MASTER_SPEC.md`
- Sponsor decisions are still listed as open in section 13.
3. `specs/COUNTRY_ROLLOUT_PACK_TEMPLATE_US_CA.md`
- No explicit dual-book close evidence requirements.
4. `specs/CANONICAL_DATA_MODEL_SPEC.md`
- Missing required posting provenance fields for deterministic dual-book replay.
5. Scale requirement is not codified in legacy specs.

## 4. Required Updates by File

### 4.1 `specs/OPEN_SOURCE_ACCOUNTING_SYSTEM_MASTER_SPEC.md`
1. Convert “Decisions Needed” to “Decisions Confirmed”.
2. Move dual-book to Must-have.
3. Add runtime/storage execution guidance reference to:
- `specs/ARCHITECTURAL_GUIDANCE.md`
- `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`

### 4.2 `specs/IMPLEMENTATION_PLAN_12_WEEKS.md`
1. Replace US-GAAP-only MVP assumption.
2. Add explicit Rust/Ractor + DuckDB profile milestones.
3. Add scale test gates and no-bend criteria.
4. Add outbox/inbox and compensation testing milestones.

### 4.3 `specs/CANONICAL_DATA_MODEL_SPEC.md`
1. Add required provenance fields:
- `book_policy_id`
- `policy_version`
- `fx_rate_set_id`
- `ruleset_version`
- `workflow_id`
2. Add deterministic ordering constraints across storage engines.
3. Add Stripe-default pilot examples while retaining provider-neutral schema.

### 4.4 `specs/COUNTRY_ROLLOUT_PACK_TEMPLATE_US_CA.md`
1. Add dual-book checklist items per country.
2. Add ski-domain UAT evidence lines.
3. Add gateway-switch conformance evidence line.

## 5. Priority Plan

### P0 (before Sprint 1 contract freeze)
1. Update locked decisions across legacy docs.
2. Add dual-book and provenance requirements in canonical model.
3. Add performance and scale acceptance gates.

### P1 (Sprint 1-3)
1. Implement runtime and storage profile contracts.
2. Add provider and ERP adapter contract conformance suites.
3. Add country-pack dual-book evidence and ski-domain UAT packs.

### P2 (Sprint 4+)
1. Run periodic conformance audits between canonical model and adapters.
2. Validate DB migration playbook under load and close-window conditions.

