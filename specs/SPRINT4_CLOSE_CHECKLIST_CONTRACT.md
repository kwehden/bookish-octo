# Sprint 4 Close Checklist Contract (v1)

Last updated: February 23, 2026  
Owner: Data/Reconciliation

## 1. Scope
Contract for Sprint 4 entity close-checklist behavior in `reconciliation-model`:
- Entity-level dependency status tracking for close prerequisites.
- Deterministic checklist progression decisions with blocker enforcement.
- Multi-entity dry-run close simulation for 2-3 pilot entities.
- Coupled elimination + FX translation output validation in the dry-run path.

## 2. Entity Checklist Data Contract
Primary model: `EntityCloseChecklist`
- `checklist_id`: deterministic checklist identifier.
- `legal_entity_id`: legal entity scope key.
- `period_id`: accounting period key.
- `status`: checklist lifecycle status (`InProgress`, `Blocked`, `ReadyToClose`, `Closed`).
- `dependencies`: list of `CloseChecklistDependency`.
- `updated_at`: UTC timestamp for last mutation.

Dependency model: `CloseChecklistDependency`
- `dependency_id`: dependency identifier.
- `description`: human-readable dependency label.
- `required_for_close`: whether dependency is required to reach `ReadyToClose`.
- `status`: dependency state (`Pending`, `InProgress`, `Satisfied`, `Blocked`).

## 3. Dependency Transition Contract
Transition primitive: `transition_close_dependency_status`
- Inputs: checklist, dependency id, next dependency status, mutation timestamp.
- Output: updated checklist with recomputed checklist status.
- Deterministic errors:
  - `DependencyNotFound` for unknown dependency IDs.
  - `InvalidDependencyTransition` for unsupported state regressions.

Allowed transition matrix:
- `Pending` -> `InProgress` | `Satisfied` | `Blocked`
- `InProgress` -> `Satisfied` | `Blocked`
- `Blocked` -> `InProgress` | `Satisfied`
- Idempotent transition to same state is allowed.
- `Satisfied` is terminal (except idempotent self-transition).

## 4. Progression Decision Contract
Evaluation primitives:
- `evaluate_entity_close_checklist`
- `evaluate_entity_close_checklist_for_actor` (authorization-neutral wrapper)

Output: `CloseChecklistProgression`
- `can_progress=false` when one or more dependencies are `Blocked`.
- `unresolved_blockers` returns dependency IDs in `Blocked` status.
- Checklist status derivation:
  - Any unresolved blockers -> `Blocked`
  - No blockers + all `required_for_close` dependencies `Satisfied` -> `ReadyToClose`
  - No blockers + unresolved required dependencies -> `InProgress`
  - Existing `Closed` remains `Closed` when no blockers exist

Authorization-neutral behavior:
- Actor context (`CloseChecklistActorContext`) is accepted for traceability but does not alter progression outcomes in this crate-level contract.

## 5. Multi-Entity Dry-Run Contract
Dry-run primitives:
- `simulate_multi_entity_close_dry_run`
- `simulate_multi_entity_close_dry_run_for_actor`

Input: `MultiEntityCloseDryRunInput`
- `run_id`
- `run_started_at`
- `checklists` (must contain 2-3 entities)
- `consolidation_outputs` (entity-scoped elimination and FX journal lines for dry-run coupling checks)

Output: `MultiEntityCloseDryRunResult`
- Deterministic entity sorting by `legal_entity_id`, then `checklist_id`.
- Per-entity outputs in `EntityCloseDryRunResult`:
  - `status`, `can_progress`, `close_ready`, `elimination_output_valid`, `fx_translation_output_valid`, `unresolved_blockers`
- Overall outcome:
  - `passed=true` only when all entities are progression-eligible, close-ready, and have valid elimination + FX translation outputs.
  - `failed_entities` lists legal entities that fail gating.

Validation rule:
- Entity count outside 2-3 returns `UnsupportedEntityCount`.
- Missing, unbalanced, or account-incomplete elimination/FX outputs produce deterministic blockers and fail dry-run pass criteria.

## 6. Sprint 4 Exit-Gate Mapping
Mapped to `specs/SPRINT4_SQUAD_AGENT_EXECUTION_PLAN.md`:
- Gate #5: checklist dependency-state visibility and blocker-based progression control.
- Gate #6: multi-entity (2-3) dry-run close simulation pass/fail signal.
- Gate #9 (owned scope): model-level CI regression evidence from crate test suite.

## 7. Test Evidence in Crate
Implemented in `crates/reconciliation-model/src/lib.rs`:
- `dependency_state_transitions_promote_entity_to_ready_to_close`
- `dependency_state_transition_rejects_regression_from_satisfied`
- `unresolved_blockers_block_close_progression`
- `checklist_evaluation_is_authorization_neutral`
- `multi_entity_close_dry_run_passes_for_two_ready_entities`
- `multi_entity_close_dry_run_fails_when_one_entity_has_blocker`
- `multi_entity_close_dry_run_fails_when_fx_translation_output_is_missing`
- `multi_entity_close_dry_run_coupling_is_deterministic_for_unsorted_inputs`
- `multi_entity_close_dry_run_requires_two_to_three_entities`

## 8. Sprint 4 In-Repo Approval/Evidence Status
- [x] Contract artifact published and referenced by Sprint 4 gate report.
- [x] Sprint 4 gate report evidence published for checklist and dry-run behavior.
- [x] Sprint 4 signoff packet updated with completed in-repo approvals.
- [x] Sprint 4 completion report updated to separate in-repo closure from external attestations.

## 9. Remaining External Items (Carried in Sprint 5/6)
- Business-owner UAT attestation remains pending outside this repository.
- 2,000-user no-bend performance certification remains pending outside this repository.
