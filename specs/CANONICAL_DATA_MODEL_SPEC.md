# Canonical Data Model Specification

Last updated: February 21, 2026
Status: Draft v1.0

## 1. Purpose
Define a canonical, implementation-neutral accounting data model for a federated, multi-entity, multi-country open-source accounting platform. This model standardizes ingestion, posting, reconciliation, and audit outputs across heterogeneous source systems (POS, ticketing, reservations, e-commerce, gateways, and banks).

This specification is normative for:
- Connector normalization contracts
- Event ingestion APIs
- Posting engine input/output
- Reconciliation workflows
- Downstream reporting and audit traceability

## 2. Design Goals
1. Deterministic accounting outcomes from equivalent economic events.
2. Legal-entity isolation at posting boundaries with controlled consolidation above entity ledgers.
3. Full traceability from source payload to canonical event to journal line.
4. Exactly-once posting semantics on an at-least-once transport.
5. Backfill and replay safety through explicit versioning and idempotency keys.
6. Multi-currency and multi-book support without duplicating source transactions.
7. Extensible taxonomy for channels, products, and country-specific localization packs.
8. Reconciliation-first design for settlements, fees, refunds, chargebacks, and bank cash movement.
9. Immutable accounting records with explicit reversal/correction flows.
10. Open-source portability across storage and ledger implementations.

## 3. Modeling Principles
1. Canonical entities represent business semantics, not source-system table structures.
2. Events are append-only and immutable after acceptance.
3. Corrections are represented as new events and new journals, never in-place mutation.
4. Journal lines are atomic accounting entries and MUST balance per journal and per currency.
5. Dimension values are first-class references, not free-text attributes.
6. Monetary fields use decimal strings with explicit currency codes.
7. Time is ISO 8601 UTC unless local/business date semantics are explicitly required.

## 4. Canonical Entities

### 4.1 Core Context Entities
- `Tenant`: top-level federation boundary.
- `LegalEntity`: legal accounting boundary; owns books and compliance obligations.
- `LedgerBook`: accounting basis or reporting book (for example `US_GAAP`, `IFRS`).
- `Location`: physical or virtual operating unit (store, venue, mountain, property, online storefront).
- `Channel`: sales origin (`POS`, `ECOM`, `TICKETING`, `RESERVATIONS`, `MEMBERSHIP`, `OTHER`).
- `Currency`: ISO 4217 currency reference.
- `FiscalCalendar`: period definitions, close windows, lock states.

### 4.2 Commercial Entities
- `Order`: commercial agreement at transaction-time (sale, reservation, ticket purchase).
- `OrderLine`: sellable line item with quantity, price, taxability, and fulfillment attributes.
- `Discount`: line or order-level discount allocation.
- `TaxLine`: calculated tax component.
- `ServiceCharge`: mandatory or optional service/add-on charge.
- `PerformanceObligation`: revenue recognition unit for ASC 606 / IFRS 15.
- `FulfillmentEvent`: point-in-time or over-time satisfaction evidence.

### 4.3 Payments Entities
- `PaymentIntent`: customer payment authorization request.
- `PaymentTransaction`: authorization, capture, sale, void, refund, adjustment, chargeback.
- `Tender`: payment instrument abstracted from gateway specifics.
- `PayoutBatch`: gateway settlement batch identifier and aggregate.
- `SettlementLine`: settled amount component (gross, fee, net, reserve).
- `DisputeCase`: chargeback/retrieval/pre-arbitration lifecycle object.

### 4.4 Accounting Entities
- `Account`: chart-of-accounts node with type and normal balance.
- `Journal`: accounting document header.
- `JournalLine`: balanced debit/credit entry in a single currency for one account and dimension set.
- `PostingRun`: deterministic execution record of rule evaluation and journal generation.
- `PostingRule`: declarative mapping from canonical business events to journal templates.
- `ExchangeRate`: transaction, remeasurement, translation rates.

### 4.5 Reconciliation Entities
- `BankStatement`: imported bank feed or statement.
- `BankStatementLine`: atomic cash movement record.
- `MatchGroup`: group of records evaluated together for reconciliation.
- `MatchResult`: match decision and tolerance details.
- `ReconException`: unresolved break requiring triage.
- `ClearingBalanceSnapshot`: point-in-time balance for clearing and suspense accounts.

### 4.6 Control and Audit Entities
- `IdempotencyRecord`: accepted request/event key with payload hash and result pointer.
- `SchemaVersion`: supported canonical schema versions.
- `AuditLink`: immutable references between source event, canonical event, posting run, journal, and reconciliation records.

## 5. Canonical Event Schema

### 5.1 Event Envelope (Required)
| Field | Type | Required | Description |
|---|---|---|---|
| `event_id` | string (UUID/ULID) | Yes | Global unique event identifier. |
| `event_type` | string | Yes | Semantic type, for example `order.captured.v1`. |
| `schema_version` | string | Yes | Canonical event schema version. |
| `source_system` | string | Yes | Connector/system of origin (`square`, `simphony`, `adyen`). |
| `source_event_id` | string | Yes | Native source identifier used for dedupe and traceability. |
| `occurred_at` | string (ISO 8601) | Yes | Business occurrence timestamp from source. |
| `ingested_at` | string (ISO 8601) | Yes | Platform ingestion timestamp. |
| `business_date` | string (YYYY-MM-DD) | Yes | Operational accounting date at location/legal-entity context. |
| `tenant_id` | string | Yes | Tenant boundary. |
| `legal_entity_id` | string | Yes | Posting boundary legal entity. |
| `location_id` | string | Yes | Operational location. |
| `channel` | enum | Yes | Canonical channel. |
| `idempotency_key` | string | Yes | Request/event dedupe key. |
| `correlation_id` | string | Yes | End-to-end workflow correlation. |
| `causation_id` | string | No | Prior event that caused this event. |
| `payload_hash` | string | Yes | Deterministic hash of normalized payload. |
| `trace` | object | No | Transport metadata for diagnostics. |
| `data` | object | Yes | Event-type-specific payload body. |

### 5.2 Event Data Rules
1. `data` MUST conform to the JSON schema for (`event_type`, `schema_version`).
2. Unknown top-level envelope fields MUST be ignored by consumers and preserved by storage.
3. Unknown `data` fields MAY be retained as extension fields under `data.extensions`.
4. Monetary values in `data` MUST use `{ "amount": "123.45", "currency": "USD" }`.
5. Arrays representing components (`line_items`, `tax_lines`, `fees`) MUST be stable ordered using source line sequence where available.

## 6. Canonical Journal Schema

### 6.1 Journal Header
| Field | Type | Required | Description |
|---|---|---|---|
| `journal_id` | string | Yes | Immutable journal identifier. |
| `journal_number` | string | Yes | Human-readable sequence per legal entity/book. |
| `journal_type` | enum | Yes | `SUBLEDGER`, `ADJUSTMENT`, `REVERSAL`, `REMEASUREMENT`, `ELIMINATION`. |
| `status` | enum | Yes | `DRAFT`, `POSTED`, `REVERSED`, `VOID`. |
| `tenant_id` | string | Yes | Tenant boundary. |
| `legal_entity_id` | string | Yes | Legal entity book owner. |
| `ledger_book` | string | Yes | Accounting basis/book. |
| `accounting_date` | string (YYYY-MM-DD) | Yes | Date used for period assignment. |
| `posted_at` | string (ISO 8601) | No | Timestamp when status became `POSTED`. |
| `source_event_ids` | array[string] | Yes | Canonical events that produced this journal. |
| `posting_run_id` | string | Yes | Rule execution reference. |
| `book_policy_id` | string | Yes | Policy package identifier used for this book. |
| `policy_version` | string | Yes | Version of accounting policy package used. |
| `fx_rate_set_id` | string | Yes | FX rate set identifier used in conversion and translation. |
| `ruleset_version` | string | Yes | Posting ruleset version used by engine. |
| `workflow_id` | string | No | Distributed workflow/saga trace ID when applicable. |
| `description` | string | No | Journal narrative. |
| `reversal_of_journal_id` | string | No | Prior journal reference for reversal entries. |

### 6.2 Journal Line
| Field | Type | Required | Description |
|---|---|---|---|
| `line_id` | string | Yes | Immutable line identifier. |
| `line_number` | integer | Yes | Sequence within journal. |
| `account_id` | string | Yes | Chart-of-accounts reference. |
| `entry_side` | enum | Yes | `DEBIT` or `CREDIT`. |
| `amount` | string (decimal) | Yes | Positive decimal string. |
| `currency` | string (ISO 4217) | Yes | Transaction currency. |
| `base_amount` | string (decimal) | Yes | Functional/base currency amount. |
| `base_currency` | string | Yes | Functional/base currency code. |
| `fx_rate_id` | string | No | Rate used for conversion. |
| `dimensions` | object | Yes | Dimension key/value set (see section 8). |
| `memo` | string | No | Supplemental line narrative. |
| `source_ref` | object | Yes | Source line/order/payment references. |

### 6.3 Journal Constraints
1. Total debits MUST equal total credits by `currency` within each `journal_id`.
2. Total debits MUST equal total credits by `base_currency` within each `journal_id`.
3. `line_id` MUST be unique globally; (`journal_id`, `line_number`) MUST be unique.
4. `POSTED` journals are immutable.
5. Corrections MUST use a reversal journal plus replacement journal (if needed).
6. Posting provenance fields (`book_policy_id`, `policy_version`, `fx_rate_set_id`, `ruleset_version`) MUST be present on all `POSTED` journals.

## 7. Posting Rules Model

### 7.1 Rule Object
| Field | Type | Required | Description |
|---|---|---|---|
| `rule_id` | string | Yes | Stable rule identifier. |
| `version` | integer | Yes | Rule version for reproducibility. |
| `active_from` | string (ISO 8601) | Yes | Activation timestamp. |
| `active_to` | string (ISO 8601) | No | End timestamp. |
| `priority` | integer | Yes | Evaluation order within scope. |
| `scope` | object | Yes | Applies by tenant/entity/channel/product/country/book. |
| `when` | expression | Yes | Predicate over canonical event fields. |
| `derive` | array | No | Derived field computations before template fill. |
| `entries` | array | Yes | Debit/credit template lines with account resolution. |
| `validation` | array | No | Preconditions and balancing checks. |
| `on_error` | enum | Yes | `REJECT_EVENT`, `ROUTE_EXCEPTION`, `HALT_POSTING_RUN`. |
| `rule_hash` | string | Yes | Integrity hash for audit repeatability. |

### 7.2 Rule Evaluation Semantics
1. Rules evaluate in ascending `priority`.
2. Matching strategy is `FIRST_MATCH` or `ALL_MATCH` per event type configuration.
3. Account resolution supports:
- Direct account mapping.
- Lookup tables keyed by dimension combinations.
- Country pack overrides.
4. Posting output MUST include `rule_id` and `version` on every journal line provenance.
5. Rule changes are non-retroactive unless explicit replay is initiated.

### 7.3 Posting Run Output
Each posting run returns:
- `posting_run_id`
- Evaluated event IDs
- Applied rules list
- Generated journal IDs
- Rejected events with reason codes
- Deterministic checksum over outputs

## 8. Dimensions Taxonomy

### 8.1 Required Dimensions (All Journal Lines)
| Dimension | Example | Notes |
|---|---|---|
| `legal_entity` | `US_CO_01` | Posting boundary. |
| `location` | `VAIL_BASE_LODGE` | Operational site. |
| `channel` | `POS` | Source channel taxonomy. |
| `product` | `LIFT_TICKET_DAY` | Canonical product/service class. |
| `currency` | `USD` | Transaction currency context. |
| `intercompany` | `NONE` or partner entity | Required even if `NONE`. |
| `segment` | `MOUNTAIN_OPS` | Management reporting segment. |

### 8.2 Optional/Conditional Dimensions
- `customer`
- `contract`
- `performance_obligation`
- `event_id` (operational event such as concert or tee time)
- `member_plan`
- `promotion`
- `tax_jurisdiction`
- `salesperson`
- `device_or_terminal`

### 8.3 Dimension Governance
1. Dimension values are mastered and versioned; orphan values are not allowed.
2. New dimension types require governance approval and backward compatibility review.
3. Historical journals retain original dimension values even if master data is renamed.

## 9. Reconciliation Data Model

### 9.1 Primary Reconciliation Objects
| Entity | Purpose |
|---|---|
| `SettlementFile` | Gateway-provided settlement import artifact. |
| `SettlementLine` | Atomic settlement movement (gross/fee/net/reserve). |
| `PayoutBatch` | Group of settlement lines paid together. |
| `BankStatement` | Bank feed/statement import artifact. |
| `BankStatementLine` | Atomic bank movement used for cash matching. |
| `MatchGroup` | Candidate records grouped for reconciliation. |
| `MatchResult` | Outcome with tolerance and confidence metadata. |
| `ReconException` | Unresolved break requiring action. |

### 9.2 Reconciliation Keys
- `gateway_transaction_id`
- `gateway_capture_id`
- `payout_batch_id`
- `bank_reference`
- `amount`
- `currency`
- `value_date`
- `legal_entity_id`

### 9.3 Reconciliation Outcomes
- `MATCHED_EXACT`
- `MATCHED_TOLERANCE`
- `PARTIAL_MATCH`
- `UNMATCHED`
- `DUPLICATE`
- `INVESTIGATE`

### 9.4 Exception Lifecycle Fields
- `exception_id`
- `exception_type`
- `severity`
- `opened_at`
- `owner`
- `sla_due_at`
- `resolution_code`
- `resolved_at`

## 10. Idempotency and Versioning Rules

### 10.1 Idempotency (Write Operations)
1. Every ingest/posting/reconciliation write MUST include `idempotency_key`.
2. Deduplication scope is (`tenant_id`, `operation_type`, `idempotency_key`).
3. Accepted duplicate with same `payload_hash` MUST return original result.
4. Duplicate with mismatched `payload_hash` MUST return conflict (`IDEMPOTENCY_PAYLOAD_MISMATCH`).
5. Idempotency records MUST retain:
- request envelope
- payload hash
- first-seen timestamp
- terminal result pointer

### 10.2 Event and Schema Versioning
1. `schema_version` uses semantic versions (`MAJOR.MINOR.PATCH`).
2. Backward-compatible additions increment `MINOR`.
3. Breaking changes increment `MAJOR` and require dual-read migration period.
4. `event_type` is version-suffixed at major boundary (`order.captured.v1`, `order.captured.v2`).
5. Consumers MUST reject unsupported major versions with explicit error code.

### 10.3 Rule and Journal Reproducibility
1. Posting run captures exact rule set versions and hashes.
2. Replay mode MUST reproduce identical journals for same inputs unless configuration explicitly differs.
3. Any non-deterministic dependency (for example FX source) MUST be materialized by reference ID.

## 11. Status and State Models

### 11.1 Event Ingestion State
- `RECEIVED`: transport accepted, not yet validated.
- `VALIDATED`: schema and reference checks passed.
- `NORMALIZED`: source mapped to canonical.
- `ACCEPTED`: persisted and eligible for posting.
- `REJECTED`: terminal validation failure.
- `REPLAYED`: reprocessed from historical input.

### 11.2 Order Lifecycle State
- `OPEN`
- `CAPTURED`
- `FULFILLED_PARTIAL`
- `FULFILLED_COMPLETE`
- `CANCELED`
- `REFUNDED_PARTIAL`
- `REFUNDED_FULL`

### 11.3 Payment Transaction State
- `INITIATED`
- `AUTHORIZED`
- `CAPTURED`
- `SETTLED`
- `PARTIALLY_REFUNDED`
- `REFUNDED`
- `VOIDED`
- `CHARGEBACK_OPEN`
- `CHARGEBACK_WON`
- `CHARGEBACK_LOST`

### 11.4 Journal Lifecycle State
- `DRAFT`: generated, not posted.
- `POSTED`: immutable, impacts balances.
- `REVERSED`: reversed by linked reversal journal.
- `VOID`: invalid before posting; no balance impact.

### 11.5 Reconciliation Lifecycle State
- `PENDING`
- `AUTO_MATCHED`
- `MANUAL_REVIEW`
- `EXCEPTION_OPEN`
- `EXCEPTION_RESOLVED`
- `CLOSED`

## 12. Example Payloads (JSON)

### 12.1 Order Capture Event (`order.captured.v1`)
```json
{
  "event_id": "01JMX4Q2EXN6QKBH4P7HEN96XH",
  "event_type": "order.captured.v1",
  "schema_version": "1.0.0",
  "source_system": "square",
  "source_event_id": "sq_ord_482991",
  "occurred_at": "2026-02-21T16:07:12Z",
  "ingested_at": "2026-02-21T16:07:13Z",
  "business_date": "2026-02-21",
  "tenant_id": "tenant_federation_001",
  "legal_entity_id": "US_CO_01",
  "location_id": "BRECK_BASE_AREA",
  "channel": "POS",
  "idempotency_key": "square:sq_ord_482991:captured",
  "correlation_id": "corr_3d6f4bc8",
  "payload_hash": "sha256:4d4a6c6749d2f8f8f7abf9dbf5f89f4584182c87e5f9d8e08a8b66f4cde2bda8",
  "data": {
    "order_id": "ORD-20260221-000482991",
    "order_status": "CAPTURED",
    "customer_id": "CUST-7821",
    "currency": "USD",
    "totals": {
      "subtotal": { "amount": "180.00", "currency": "USD" },
      "discount_total": { "amount": "20.00", "currency": "USD" },
      "tax_total": { "amount": "11.20", "currency": "USD" },
      "grand_total": { "amount": "171.20", "currency": "USD" }
    },
    "line_items": [
      {
        "line_id": "1",
        "product_id": "LIFT_TICKET_DAY",
        "description": "Adult Lift Ticket - 1 Day",
        "quantity": "2",
        "unit_price": { "amount": "90.00", "currency": "USD" },
        "line_subtotal": { "amount": "180.00", "currency": "USD" },
        "performance_obligation_id": "POB-LIFT-DAY"
      }
    ],
    "discounts": [
      {
        "discount_id": "DISC-PROMO-10PCT",
        "scope": "ORDER",
        "amount": { "amount": "20.00", "currency": "USD" }
      }
    ],
    "tax_lines": [
      {
        "tax_id": "CO_SALES_TAX",
        "jurisdiction": "US-CO",
        "amount": { "amount": "11.20", "currency": "USD" }
      }
    ],
    "tenders": [
      {
        "tender_id": "TND-1",
        "type": "CARD",
        "payment_transaction_id": "PAY-9A2F"
      }
    ]
  }
}
```

### 12.2 Payment Settlement Event (`payment.settled.v1`)
```json
{
  "event_id": "01JMX5J9HE00W5Q5Y8KM2R9EJG",
  "event_type": "payment.settled.v1",
  "schema_version": "1.0.0",
  "source_system": "adyen",
  "source_event_id": "ady_settle_72418110",
  "occurred_at": "2026-02-22T03:15:00Z",
  "ingested_at": "2026-02-22T03:15:02Z",
  "business_date": "2026-02-21",
  "tenant_id": "tenant_federation_001",
  "legal_entity_id": "US_CO_01",
  "location_id": "BRECK_BASE_AREA",
  "channel": "POS",
  "idempotency_key": "adyen:ady_settle_72418110",
  "correlation_id": "corr_3d6f4bc8",
  "payload_hash": "sha256:545f25546cbdbf3607bf4ef3ed7487b3f11e5ba36f7f9fbf4e1276c315f947e4",
  "data": {
    "payment_transaction_id": "PAY-9A2F",
    "order_id": "ORD-20260221-000482991",
    "gateway_transaction_id": "psp_8836182710",
    "payout_batch_id": "PO-20260222-10017",
    "settlement_date": "2026-02-22",
    "currency": "USD",
    "gross_amount": { "amount": "171.20", "currency": "USD" },
    "fee_amount": { "amount": "4.28", "currency": "USD" },
    "net_amount": { "amount": "166.92", "currency": "USD" },
    "settlement_lines": [
      {
        "line_type": "GROSS",
        "amount": { "amount": "171.20", "currency": "USD" }
      },
      {
        "line_type": "PROCESSING_FEE",
        "amount": { "amount": "4.28", "currency": "USD" }
      },
      {
        "line_type": "NET_PAYOUT",
        "amount": { "amount": "166.92", "currency": "USD" }
      }
    ]
  }
}
```

### 12.3 Posting Output (`posting.result.v1`)
```json
{
  "posting_run_id": "PR-20260222-000031",
  "run_started_at": "2026-02-22T03:16:10Z",
  "run_completed_at": "2026-02-22T03:16:11Z",
  "tenant_id": "tenant_federation_001",
  "legal_entity_id": "US_CO_01",
  "ledger_book": "US_GAAP",
  "input_event_ids": [
    "01JMX4Q2EXN6QKBH4P7HEN96XH",
    "01JMX5J9HE00W5Q5Y8KM2R9EJG"
  ],
  "applied_rules": [
    { "rule_id": "RULE-SALE-CASH-CLEARING", "version": 3 },
    { "rule_id": "RULE-SETTLEMENT-FEE", "version": 2 }
  ],
  "journals": [
    {
      "journal_id": "JRN-20260222-001282",
      "journal_number": "USCO01-2026-0001282",
      "journal_type": "SUBLEDGER",
      "status": "POSTED",
      "accounting_date": "2026-02-21",
      "posted_at": "2026-02-22T03:16:11Z",
      "source_event_ids": ["01JMX4Q2EXN6QKBH4P7HEN96XH"],
      "posting_run_id": "PR-20260222-000031",
      "lines": [
        {
          "line_id": "JRN-20260222-001282-1",
          "line_number": 1,
          "account_id": "1105-CASH-CLEARING",
          "entry_side": "DEBIT",
          "amount": "171.20",
          "currency": "USD",
          "base_amount": "171.20",
          "base_currency": "USD",
          "dimensions": {
            "legal_entity": "US_CO_01",
            "location": "BRECK_BASE_AREA",
            "channel": "POS",
            "product": "LIFT_TICKET_DAY",
            "currency": "USD",
            "intercompany": "NONE",
            "segment": "MOUNTAIN_OPS"
          },
          "source_ref": {
            "order_id": "ORD-20260221-000482991",
            "payment_transaction_id": "PAY-9A2F"
          }
        },
        {
          "line_id": "JRN-20260222-001282-2",
          "line_number": 2,
          "account_id": "4000-REVENUE-LIFT-TICKETS",
          "entry_side": "CREDIT",
          "amount": "160.00",
          "currency": "USD",
          "base_amount": "160.00",
          "base_currency": "USD",
          "dimensions": {
            "legal_entity": "US_CO_01",
            "location": "BRECK_BASE_AREA",
            "channel": "POS",
            "product": "LIFT_TICKET_DAY",
            "currency": "USD",
            "intercompany": "NONE",
            "segment": "MOUNTAIN_OPS"
          },
          "source_ref": {
            "order_id": "ORD-20260221-000482991",
            "line_id": "1"
          }
        },
        {
          "line_id": "JRN-20260222-001282-3",
          "line_number": 3,
          "account_id": "2105-SALES-TAX-PAYABLE",
          "entry_side": "CREDIT",
          "amount": "11.20",
          "currency": "USD",
          "base_amount": "11.20",
          "base_currency": "USD",
          "dimensions": {
            "legal_entity": "US_CO_01",
            "location": "BRECK_BASE_AREA",
            "channel": "POS",
            "product": "LIFT_TICKET_DAY",
            "currency": "USD",
            "intercompany": "NONE",
            "segment": "MOUNTAIN_OPS",
            "tax_jurisdiction": "US-CO"
          },
          "source_ref": {
            "order_id": "ORD-20260221-000482991"
          }
        }
      ]
    },
    {
      "journal_id": "JRN-20260222-001283",
      "journal_number": "USCO01-2026-0001283",
      "journal_type": "SUBLEDGER",
      "status": "POSTED",
      "accounting_date": "2026-02-22",
      "posted_at": "2026-02-22T03:16:11Z",
      "source_event_ids": ["01JMX5J9HE00W5Q5Y8KM2R9EJG"],
      "posting_run_id": "PR-20260222-000031",
      "lines": [
        {
          "line_id": "JRN-20260222-001283-1",
          "line_number": 1,
          "account_id": "1010-BANK-OPERATING",
          "entry_side": "DEBIT",
          "amount": "166.92",
          "currency": "USD",
          "base_amount": "166.92",
          "base_currency": "USD",
          "dimensions": {
            "legal_entity": "US_CO_01",
            "location": "BRECK_BASE_AREA",
            "channel": "POS",
            "product": "LIFT_TICKET_DAY",
            "currency": "USD",
            "intercompany": "NONE",
            "segment": "MOUNTAIN_OPS"
          },
          "source_ref": {
            "payout_batch_id": "PO-20260222-10017",
            "payment_transaction_id": "PAY-9A2F"
          }
        },
        {
          "line_id": "JRN-20260222-001283-2",
          "line_number": 2,
          "account_id": "6100-PAYMENT-PROCESSING-FEES",
          "entry_side": "DEBIT",
          "amount": "4.28",
          "currency": "USD",
          "base_amount": "4.28",
          "base_currency": "USD",
          "dimensions": {
            "legal_entity": "US_CO_01",
            "location": "BRECK_BASE_AREA",
            "channel": "POS",
            "product": "LIFT_TICKET_DAY",
            "currency": "USD",
            "intercompany": "NONE",
            "segment": "MOUNTAIN_OPS"
          },
          "source_ref": {
            "payout_batch_id": "PO-20260222-10017"
          }
        },
        {
          "line_id": "JRN-20260222-001283-3",
          "line_number": 3,
          "account_id": "1105-CASH-CLEARING",
          "entry_side": "CREDIT",
          "amount": "171.20",
          "currency": "USD",
          "base_amount": "171.20",
          "base_currency": "USD",
          "dimensions": {
            "legal_entity": "US_CO_01",
            "location": "BRECK_BASE_AREA",
            "channel": "POS",
            "product": "LIFT_TICKET_DAY",
            "currency": "USD",
            "intercompany": "NONE",
            "segment": "MOUNTAIN_OPS"
          },
          "source_ref": {
            "payment_transaction_id": "PAY-9A2F"
          }
        }
      ]
    }
  ],
  "rejections": [],
  "checksum": "sha256:43c32f41491a79d4e219665f67efd2b3562272ba45a1ef7387d7afe5d4f6ce71"
}
```

## 13. Validation Checklist (Implementation Minimum)
1. Schema validation for all event and journal payloads.
2. Referential integrity for required dimensions and accounts.
3. Balance validation by currency and base currency.
4. Idempotency conflict detection using payload hash.
5. Immutable audit links from source to posting and reconciliation outputs.
6. Deterministic replay test suite for representative event sets.
