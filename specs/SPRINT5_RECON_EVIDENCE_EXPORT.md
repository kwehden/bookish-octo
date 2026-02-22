# Sprint 5 Reconciliation Evidence Export Contract

Last updated: February 22, 2026  
Scope: Sprint 5 dispute aging + unresolved SLA metrics for compliance evidence

## 1. Output Model

`ReconComplianceEvidenceExport` fields:
- `run_id`: reconciliation run identifier.
- `generated_at`: UTC timestamp of evidence generation.
- `dispute_aging`: fixed buckets: `0-4h`, `4-8h`, `8-24h`, `24h+`.
- `unresolved_sla`:
  - `total_open`
  - `overdue`
  - `on_track`
  - `overdue_rate_bps`
- `owner_queue_counts`: deterministic owner-queue count map.

Implementation reference:
- `crates/reconciliation-model/src/lib.rs`

## 2. Deterministic Rules

1. Aging bucket selection uses `max(0, (as_of - opened_at).num_hours())`.
2. Bucket order is fixed and must always include all four buckets (even when `count=0`).
3. `overdue` is true when `as_of > sla_due_at` (strictly greater).
4. `overdue_rate_bps` uses integer basis-point ratio (`ratio_to_bps`).
5. `owner_queue_counts` is serialized from `BTreeMap` for stable ordering.

## 3. Validation Tests

- `dispute_aging_buckets_are_computed_for_exception_queue`
- `unresolved_sla_metrics_flag_overdue_items`
- `compliance_evidence_export_contains_owner_queue_counts`

## 4. QA/Gate Usage

- Consumed by: `qa/SPRINT5_GATE_REPORT.md`
- Gate alignment:
  - Sprint 5 Exit Gate #6 (aging + SLA metrics availability)
  - Sprint 5 Exit Gate #9 (CI evidence and deterministic regression coverage)
