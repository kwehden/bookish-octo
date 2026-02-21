# Posting Rule Template Pack v1

Scope-limited event types (Sprint 1):
- `order.captured.v1`
- `payment.settled.v1`
- `refund.v1`

Rules:
1. Both-book atomicity: post `US_GAAP` and `IFRS` together or reject both.
2. Provenance required on posted journals:
- `book_policy_id`
- `policy_version`
- `fx_rate_set_id`
- `ruleset_version`
- `workflow_id`
3. Journal balancing required in tx and base currency.
