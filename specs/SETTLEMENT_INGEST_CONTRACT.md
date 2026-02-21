# Settlement Ingest Contract (Sprint 2)

Last updated: February 21, 2026  
Owner: Data/Reconciliation

## 1. Scope
Sprint 2 contract for payment-settlement ingestion inputs used by reconciliation:
- Stripe settlement CSV parser (`v0`)
- Bank statement CSV parser (`v0`)
- Deterministic reconciliation key handoff and exception routing prerequisites

## 2. Stripe Settlement CSV (`v0`)
Expected header row (exact field names):

```csv
payout_id,balance_transaction_id,source_id,available_on,currency,gross,fee,net,type
```

Field rules:
- `payout_id`: required, non-empty.
- `balance_transaction_id`: required, non-empty.
- `source_id`: required, non-empty. Treated as `gateway_transaction_id` for recon keys.
- `available_on`: required; ISO date (`YYYY-MM-DD`) or `MM/DD/YYYY`.
- `currency`: required; normalized to uppercase.
- `gross`: required signed decimal, max 2 fractional digits.
- `fee`: required signed decimal, max 2 fractional digits.
- `net`: required signed decimal, max 2 fractional digits.
- `type`: required, non-empty (example: `charge`, `refund`, `adjustment`).

Output object (`StripeSettlementLine`):
- `payout_id`
- `balance_transaction_id`
- `gateway_transaction_id`
- `available_on`
- `currency`
- `gross_minor`
- `fee_minor`
- `net_minor`
- `transaction_type`

## 3. Bank Statement CSV (`v0`)
Expected header row (exact field names):

```csv
statement_id,value_date,bank_reference,description,currency,amount
```

Field rules:
- `statement_id`: required, non-empty.
- `value_date`: required; ISO date (`YYYY-MM-DD`) or `MM/DD/YYYY`.
- `bank_reference`: required, non-empty.
- `description`: required, non-empty.
- `currency`: required; normalized to uppercase.
- `amount`: required signed decimal, max 2 fractional digits.

Output object (`BankStatementLine`):
- `statement_id`
- `value_date`
- `bank_reference`
- `description`
- `currency`
- `amount_minor`

## 4. Validation and Error Contract
Parser failures MUST return one of:
- `Csv { line, message }`
- `MissingField { line, field }`
- `InvalidDate { line, field, value }`
- `InvalidAmount { line, field, value }`

Behavioral constraints:
- Minor-unit conversion is deterministic and does not use floating-point math.
- Input row order is preserved in parsed output.
- Line numbers in errors are 1-based CSV line numbers (including header).

## 5. Reconciliation Key Handoff
Settlement ingest outputs are expected to populate canonical recon keys:
- `gateway_transaction_id` <- Stripe `source_id`
- `payout_batch_id` <- Stripe `payout_id`
- `bank_reference` <- bank `bank_reference`
- `amount` <- Stripe `net_minor` and bank `amount_minor`
- `currency` <- normalized currency fields
- `value_date` <- Stripe `available_on` and bank `value_date`

## 6. Deterministic Reason-Code Routing Link
`reconciliation-model` routing helper (`route_exception`) consumes:
- `ReconException.exception_type`
- `ReconException.severity`
- `MatchOutcome`

Outputs deterministic:
- `reason_code`
- `owner_queue`

Reason codes are taxonomy-backed and non-random by design, supporting Sprint 2 gate requirement for reason-coded exceptions.

## 7. Non-Goals (Sprint 2)
Out of scope for `v0`:
- XLS/XLSX parser support
- Multi-currency normalization against FX reference tables
- Auto-fuzzy matching across descriptor text
- Provider-specific schema auto-detection
