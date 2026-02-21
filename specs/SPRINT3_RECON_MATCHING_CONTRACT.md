# Sprint 3 Reconciliation Matching Contract (v1)

Last updated: February 21, 2026  
Owner: Data/Reconciliation

## 1. Scope
Contract for Sprint 3 matching engine outputs in `reconciliation-model` for the control loop:
- Order -> payment -> payout deterministic matching
- Exception queue payloads for non-auto outcomes
- Gate metrics used by QA/Release exit criteria

## 2. Input Contract (`ReconRunInput`)
Required fields:
- `run_id`: non-empty string for deterministic run-level IDs.
- `run_started_at`: UTC timestamp used as the immutable baseline for SLA derivation.
- `orders`: list of `ReconOrder` with `order_id`, `payment_id`, `payout_id`, `currency`, `amount_minor`, `captured_at`.
- `payments`: list of `ReconPayment` with `payment_id`, `order_id`, `payout_id`, `currency`, `amount_minor`, `settled_at`.
- `payouts`: list of `ReconPayout` with `payout_id`, `payment_id`, `bank_reference`, `currency`, `amount_minor`, `settled_at`.
- `tolerance_minor`: integer tolerance for amount deltas in minor units.

Normalization and indexing rules:
- Currency comparisons are uppercase deterministic comparisons.
- Matching order iteration is sorted by `order_id` ascending.
- Payment and payout candidates are indexed by IDs via deterministic map ordering and candidate sorting.

## 3. Matching Contract (`reconcile_v1`)
Per order, one `ReconMatchRecord` is produced with:
- `order_id`
- `expected_payment_id`
- `expected_payout_id`
- `matched_payment_id` (nullable)
- `matched_payout_id` (nullable)
- `outcome` (`MatchedExact`, `MatchedTolerance`, `PartialMatch`, `Unmatched`, `Duplicate`, `Investigate`)
- `reason_code` (nullable for auto-matches)

Decision precedence (deterministic):
1. Missing payment candidate -> `Unmatched` + `MissingGatewayReference`.
2. Duplicate payment candidates -> `Duplicate` + `DuplicateCandidate`.
3. Missing payout candidate -> `Unmatched` + `MissingBankReference`.
4. Duplicate payout candidates -> `Duplicate` + `DuplicateCandidate`.
5. Key-link mismatch across order/payment/payout IDs -> `PartialMatch` + `PartialAllocationRequired`.
6. Currency mismatch -> `Unmatched` + `CurrencyMismatch`.
7. Amount deltas:
- both zero -> `MatchedExact`
- both within tolerance -> `MatchedTolerance`
- only one side within tolerance -> `PartialMatch` + `PartialAllocationRequired`
- both outside tolerance -> `Unmatched` + `AmountMismatch`

## 4. Exception Queue Contract
Every non-auto outcome produces one `ReconExceptionQueueItem` with:
- `exception_id`: `<NORMALIZED_RUN_ID>-EX-####` sequence, deterministic by sorted order processing.
- `order_id`, `payment_id`, `payout_id`
- `reason_code`
- `owner_queue` (from deterministic routing taxonomy)
- `opened_at` (equal to `run_started_at`)
- `sla_due_at` (deterministic offset from `run_started_at`)
- `outcome`

SLA offsets:
- `AmountMismatch`, `CurrencyMismatch`, `PartialAllocationRequired`: +4h
- `MissingGatewayReference`, `MissingBankReference`: +8h
- `ToleranceMatchReview`: +12h
- `DuplicateCandidate`, `Unclassified`: +24h
- `HighRiskInvestigate`: +2h

## 5. Metrics Contract (`ReconRunMetrics`)
Outputs:
- `total_candidates`
- `auto_matched`
- `non_auto_candidates`
- `routed_exceptions`
- `auto_match_rate_bps`
- `routed_exception_rate_bps`

Rate interpretation:
- Basis points (`bps`) where `10_000` equals `100.00%`.
- Helper accessors expose percent views for gate assertions.

## 6. Sprint 3 Exit-Gate Mapping
Mapped to `specs/SPRINT3_SQUAD_AGENT_EXECUTION_PLAN.md`:
- Exit gate #4: `auto_match_rate_bps >= 7_000` (`>=70.00%`).
- Exit gate #5: `routed_exception_rate_bps == 10_000` for all non-auto candidates.

Seeded fixture baseline in crate tests:
- `total_candidates=11`
- `auto_matched=8` (`72.72%`)
- `non_auto_candidates=3`
- `routed_exceptions=3` (`100% routed with reason/owner/SLA`)
