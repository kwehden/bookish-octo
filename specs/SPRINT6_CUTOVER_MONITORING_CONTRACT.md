# Sprint 6 Cutover Monitoring Contract

Last updated: February 22, 2026  
Scope: Sprint 6 pilot cutover release-readiness monitoring for US + CA launch entities

## 1. Output Model

Primary reconciliation monitoring output:
- `PostCutoverMonitoringSnapshot` (implementation: `crates/reconciliation-model/src/lib.rs`)

Fields:
- `run_id`: reconciliation run identifier for traceability.
- `generated_at`: UTC timestamp for the monitoring snapshot.
- `unresolved_overdue_bps`: unresolved overdue exception rate in basis points.
- `open_critical_defects`: count of open critical defects in launch scope.
- `uat_attested`: external UAT attestation status flag.
- `perf_certified`: external performance certification status flag.
- `release_ready`: deterministic release gate decision.

Supporting cutover-rehearsal output:
- `CutoverRehearsalResult` (implementation: `crates/connector-sdk/src/lib.rs`)
- `checkpoint_results` must include replay-recovery and rollback checkpoints.

## 2. Deterministic Rules

1. `unresolved_overdue_bps` is derived from `unresolved_exception_sla_metrics(...).overdue_rate_bps`.
2. `release_ready=true` only when all are true:
- `open_critical_defects == 0`
- `uat_attested == true`
- `perf_certified == true`
- `unresolved_overdue_bps <= 1000`
3. `unresolved_overdue_bps <= 1000` is an inclusive threshold and must be evaluated in integer basis points.
4. `generated_at` is taken directly from the provided `as_of` timestamp; no local-time transformations are permitted.
5. Cutover rehearsal is treated as failed when any checkpoint has `passed=false`; `passed` is the all-checkpoints conjunction.
6. Owner queue aggregations used by monitoring evidence must be serialized from `BTreeMap` to keep deterministic key ordering.

## 3. Validation Tests

Reconciliation monitoring tests:
- `unresolved_sla_metrics_flag_overdue_items`
- `post_cutover_monitoring_is_release_ready_when_gates_are_green`
- `post_cutover_monitoring_blocks_release_on_critical_defects`

Cutover rehearsal tests:
- `cutover_rehearsal_passes_when_all_checkpoints_pass`
- `cutover_rehearsal_fails_when_checkpoint_fails`

## 4. Gate Usage and Evidence Chain

- Consumed by: `qa/SPRINT6_GATE_REPORT.md`
- Rubric linkage: `governance/SPRINT6_PLAN_COMPLETION_REPORT.md`
- Plan linkage: `specs/SPRINT6_SQUAD_AGENT_EXECUTION_PLAN.md` (Exit Gate #3 and #6)

This contract binds Sprint 6 release gating to deterministic reconciliation and cutover monitoring outputs rather than discretionary review notes.
