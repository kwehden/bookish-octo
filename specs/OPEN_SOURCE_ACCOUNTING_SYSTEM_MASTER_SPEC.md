# Open Source Federated Accounting Platform - Master Spec

Last updated: February 21, 2026

## 1. Purpose and Scope
Build an open-source accounting platform for a federated business operating across countries and channels:
- Retail: online and in-person POS
- Ticketing and reservations
- Promotions and discounting
- Multi-entity, multi-currency, multi-country operations

Target operators include networks like ski resorts, golf groups, and event-center portfolios.

## 2. Outcome Targets
- Single accounting truth across channels and entities
- Faster month-end close with stronger controls and auditability
- Integration-first model with major POS/ticketing/payment systems
- Country-ready localization without forking the core platform

## 3. Product Strategy (Recommended)
Use a hybrid composable architecture:
- Operational accounting UX: Odoo Community (LGPLv3) or ERPNext (GPLv3)
- Core posting/ledger service: TigerBeetle (Apache-2.0) or equivalent ledger service
- Orchestration and controls: Temporal + OPA + Keycloak
- Analytics and BI: Superset (or Metabase with license review)
- Integration backbone: Apache Camel + Debezium

Rationale: fastest MVP while preserving long-term control and extensibility.

## 4. Requirements (MoSCoW)

### Must Have
1. Multi-entity accounting isolation with consolidation layer above entity books.
2. Global chart-of-accounts template with local overrides and mandatory dimensions:
   `legal_entity`, `location`, `channel`, `product`, `currency`, `intercompany`, `segment`.
3. Channel-specific subledgers: POS sales, e-commerce, ticketing/reservations, memberships/passes, stored value, loyalty, AR/AP, cash clearing, intercompany.
4. ASC 606 / IFRS 15 revenue framework:
   performance obligations, allocation, point-in-time vs over-time recognition.
5. Deferred revenue and contract liability tracking for reservations, passes, memberships, prepaid bundles.
6. Breakage accounting for unused entitlements with governed estimate changes.
7. Refund/return liability and remeasurement support.
8. Bundle/package accounting with standalone selling price allocation.
9. Principal-vs-agent and commission split handling (gross vs net presentation).
10. Gift card/stored value classification with compliance flags.
11. Multi-currency support: transaction rates, remeasurement, translation.
12. Intercompany subledger: due-to/due-from, transfer pricing metadata, elimination-ready postings.
13. Consolidation engine: elimination rules, FX translation, NCI support.
14. Period close controls: locks, approvals, immutable journals, traceable adjustments.
15. Payment reconciliation and dispute lifecycle accounting.
16. Full audit trail and tamper-evident logging.

### Should Have
1. Dual-book policy support (US GAAP + IFRS mode).
2. Automated reconciliation for POS batches, gateways, and bank payouts.
3. Country-specific tax and e-invoicing adapters as pluggable modules.
4. Contract asset/liability rollforwards and disclosure-ready reporting.
5. Rules governance for estimate assumptions (refunds, breakage, reserves).
6. KPI suite from accounting truth: deferred revenue waterfall, margin by channel/site, chargeback rate, intercompany aging.

### Could Have
1. Forecasting for chargebacks, refunds, and breakage.
2. Soft-close snapshots and near-real-time group view.
3. Simulation sandbox for promo/discount policy impacts on revenue timing.

### Won't Have in Phase 1
1. Native PAN storage or direct card processing in core platform.
2. Global tax filing automation for all countries on day one.
3. Broad ERP modules unrelated to accounting evidence and controls.

## 5. Target Integration Portfolio

### Wave 1 (First-class connectors)
1. Inntopia (resort commerce/reservations)
2. Lightspeed Golf (Chronogolf + Lightspeed APIs)
3. Oracle MICROS Simphony (F&B/event POS)
4. Ticketmaster (ticketing, partner API where available)
5. Square (SMB-mid market POS and payments)

### Wave 2
1. accesso Paradox / Siriusware
2. Clubessential / foreUP

### Wave 3
1. Cvent
2. FareHarbor
3. SeatGeek
4. AXS

### Canonical connector payload model (minimum)
- `business_date`, `location_id`, `legal_entity_id`
- `order_or_check_id`, `reservation_or_ticket_id`, `event_id`
- `line_items`, `discounts`, `service_charges`, `tax_lines`
- `tenders`, `payments`, `refunds`, `voids`
- `fees`, `commissions`, `payout_batch_id`
- `gift_card_activity`, `loyalty_activity`
- `customer_or_member_id`

## 6. Payments and Compliance Baseline

### Card and payment architecture
- Card-present: EMV, contactless, terminal-tokenized flows
- Card-not-present: EMV 3DS 2.x
- E-commerce PSD2/SCA handling where required (EU/EEA)
- Strong tokenization strategy and scope-reduction pattern for PCI DSS
- P2PE usage where feasible for in-person environments

### PCI and security controls
- Design for SAQ A eligibility when feasible for e-commerce flows
- If outside SAQ A scope, plan for SAQ A-EP/SAQ D controls including script/payment page governance
- Immutable audit logging, access controls, key management, and change controls

### Gateway strategy (initial)
1. Stripe (fast unified online + in-person launch)
2. Adyen (international acquiring and enterprise reconciliation depth)
3. Checkout.com (additional global coverage and settlement flexibility)

## 7. Federated Multi-Country Architecture

### Core design principles
1. Legal entity isolation at posting boundary.
2. Consolidation as a separate service layer.
3. Country pack framework for localization (tax schemas, e-invoice formats, statutory exports).
4. Data residency zoning and cross-border replication controls.
5. Intercompany governance with contract metadata and elimination orchestration.

### Control model
- Global control council: standards, policy, release approvals
- Regional operations: exception triage and compliance checks
- Local finance teams: statutory execution and regulator-facing workflows

### Security and permissioning
- RBAC + ABAC on entity, country, function, and approval limits
- Non-bypassable SoD for journal posting, master data changes, payment approvals, tax submissions
- Break-glass with expiry and mandatory review log

## 8. Non-Functional Requirements
1. Idempotent posting APIs and exactly-once posting semantics.
2. Full traceability from source transaction to GL journal lines.
3. Reconciliation tolerance rules with deterministic exception queues.
4. 99.9%+ availability target for posting and close-critical workflows.
5. Explicit data retention and legal hold policy by jurisdiction.
6. Backfill/replay-safe connector architecture with event versioning.
7. Multi-tenant isolation model aligned to legal entity boundaries.

## 9. Delivery Plan (Phased)

### Phase 0 - Discovery and Controls (4-8 weeks)
1. Confirm legal entity map and jurisdiction list.
2. Select initial accounting basis (US GAAP-only vs dual-book).
3. Freeze canonical event and journal schemas.
4. Choose Wave 1 connector sequence and test tenants.
5. Produce compliance control matrix (PCI, privacy, audit).

Exit criteria:
- Signed target architecture
- Approved control framework
- Connector priority and scope locked

### Phase 1 - MVP (0-4 months)
1. Core ledger posting service and journal model.
2. Odoo/ERPNext integration for AP/AR and reporting workflows.
3. Revenue deferral engine (reservations/passes/gift cards).
4. Wave 1 connectors: at least 2 production-ready (recommend Inntopia + Square or Simphony based on pilot partner).
5. Daily settlement reconciliation and exception handling.
6. Basic consolidation for 2-3 entities.

Exit criteria:
- Month-end close from integrated operational data
- External-audit-friendly transaction trail for pilot entities

### Phase 2 - Scale and Localization (4-10 months)
1. Expand to full Wave 1 integrations.
2. Intercompany automation and elimination workflows.
3. Country packs for first non-US jurisdictions.
4. Advanced dispute/chargeback accounting workflows.
5. KPI and disclosure dashboards.

Exit criteria:
- 5+ entities, 2+ countries live
- Automated intercompany and reconciliation controls in routine use

### Phase 3 - Enterprise Hardening (10-18 months)
1. DR/BCP hardening and control automation.
2. Continuous control monitoring and policy-as-code enforcement.
3. Additional country packs and e-invoicing adapters.
4. Plugin SDK for certified connectors and local tax extensions.

Exit criteria:
- Group-level close and compliance ops stable at scale
- Country rollout pattern repeatable with low rework

## 10. Prioritized Task Backlog (Initial)

### Epic A - Accounting Core
1. Define universal chart-of-accounts template and dimension taxonomy.
2. Build journal posting API with idempotency keys.
3. Implement journal immutability and reversal policy.
4. Build period locks and approval workflow.
5. Create subledger-to-GL posting rules engine.

### Epic B - Revenue and Liability Logic
1. Build performance-obligation model for bundled offerings.
2. Implement deferred revenue schedules.
3. Implement breakage estimation and versioned assumption logs.
4. Add refund liability and reserve remeasurement jobs.

### Epic C - Connectors and Ingestion
1. Define connector SDK (auth, retry, backfill, webhook signature validation).
2. Build normalized event ingestion service.
3. Build mapping engine from external payloads to canonical schema.
4. Build dead-letter queue + replay tooling.

### Epic D - Payments and Reconciliation
1. Build payout settlement ingestion (gateway + bank statements).
2. Implement payment-to-order matching and exception routing.
3. Implement chargeback/dispute state model and accounting impacts.
4. Build gateway abstraction layer for Stripe/Adyen/Checkout.

### Epic E - Federated Governance
1. Implement entity and country boundary controls.
2. Implement intercompany contract registry and counterparty validation.
3. Build elimination rule orchestration and consolidation pipelines.
4. Build SoD policy enforcement with OPA.

### Epic F - Compliance and Audit
1. Build immutable audit log and evidence exports.
2. Implement retention and legal hold by jurisdiction.
3. Produce PCI scope documentation and control ownership matrix.
4. Add privacy/data transfer checks on replication jobs.

### Epic G - Reporting and Close
1. Build close cockpit: checklist, dependencies, and status board.
2. Create deferred revenue, margin, chargeback, and intercompany KPIs.
3. Add statutory export packs for first countries.
4. Add disclosure packs for management and auditors.

## 11. Key Risks and Mitigations
1. License contamination from AGPL/source-available dependencies.
   Mitigation: legal review gate and approved dependency policy.
2. Connector fragility due partner-gated APIs.
   Mitigation: certification process + robust retry/replay architecture.
3. Tax localization complexity across countries.
   Mitigation: country-pack pattern and local advisory validation.
4. Reconciliation noise from source data quality issues.
   Mitigation: canonical schema contracts and strict observability.
5. Customization creep in ERP layer.
   Mitigation: hard boundary between operational UI and core posting domain.

## 12. Research Sources (Primary)

### Accounting and standards
- IFRS 15: https://www.ifrs.org/issued-standards/list-of-standards/ifrs-15-revenue-from-contracts-with-customers/
- IASB IFRS 15 post-implementation update (Sep 30, 2024): https://www.ifrs.org/news-and-events/news/2024/09/iasb-concludes-revenue-standard-working/
- FASB Topic 606 ASU 2014-09: https://storage.fasb.org/ASU%202014-09_Section%20A.pdf
- IFRS 10: https://www.ifrs.org/issued-standards/list-of-standards/ifrs-10-consolidated-financial-statements/
- IAS 21: https://www.ifrs.org/issued-standards/list-of-standards/ias-21-the-effects-of-changes-in-foreign-exchange-rates/
- IAS 24: https://www.ifrs.org/issued-standards/list-of-standards/ias-24-related-party-disclosures/

### Payments and compliance
- PCI DSS: https://www.pcisecuritystandards.org/standards/pci-dss/
- SAQ A updates (Jan 30, 2025): https://blog.pcisecuritystandards.org/important-updates-announced-for-merchants-validating-to-self-assessment-questionnaire-a
- EMV 3DS: https://www.emvco.com/emv-technologies/3-d-secure/
- PSD2 directive: https://eur-lex.europa.eu/eli/dir/2015/2366/2015-12-23/eng
- Stripe global availability: https://stripe.com/global
- Adyen global payments: https://www.adyen.com/global-payment-processing
- Checkout.com international coverage: https://www.checkout.com/solutions/international-coverage

### POS, ticketing, reservations
- Inntopia integrations: https://corp.inntopia.com/integrations/
- Lightspeed Golf: https://www.lightspeedhq.com/golf/
- Oracle Simphony APIs: https://docs.oracle.com/en/industries/food-beverage/simphony/omsstsg2api/index.html
- Ticketmaster API docs: https://developer.ticketmaster.com/products-and-docs/apis/getting-started/
- Square developers: https://squareup.com/us/en/developers
- accesso Paradox: https://accesso.com/capabilities/products/paradox/
- Cvent developer/integration updates: https://community.cvent.com/solution-evolution/integration-updates/integration-documentation

### Federation, tax, localization
- OECD transfer pricing guidance: https://www.oecd.org/en/publications/oecd-transfer-pricing-guidelines-for-multinational-enterprises-and-tax-administrations-2022_0e655865-en.html
- OECD VAT/GST guidance: https://www.oecd.org/en/publications/2017/04/international-vat-gst-guidelines_g1g75db4.html
- EU VAT Directive: https://eur-lex.europa.eu/eli/dir/2006/112/2025-04-14
- ViDA adoption update (Mar 11, 2025): https://taxation-customs.ec.europa.eu/news/adoption-vat-digital-age-package-2025-03-11_en

### Open-source stack references
- Odoo license: https://raw.githubusercontent.com/odoo/odoo/19.0/LICENSE
- ERPNext repo: https://github.com/frappe/erpnext
- TigerBeetle repo: https://github.com/tigerbeetle/tigerbeetle
- Temporal repo: https://github.com/temporalio/temporal
- Keycloak repo: https://github.com/keycloak/keycloak
- OPA repo: https://github.com/open-policy-agent/opa
- Apache Camel: https://github.com/apache/camel
- Debezium: https://github.com/debezium/debezium
- Apache Superset: https://github.com/apache/superset

## 13. Confirmed Program Sponsor Decisions
1. Primary accounting basis at launch: dual-book (`US_GAAP` + `IFRS`) from MVP.
2. ERP base: ERPNext for operational finance UI, with Odoo-compatible adapter contract.
3. First pilot segment: ski resorts.
4. Initial countries: United States and Canada.
5. Payment gateway sequence: Stripe first, with configurable provider abstraction for others.
