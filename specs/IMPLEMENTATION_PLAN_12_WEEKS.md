# Implementation Plan (12 Weeks)

Last updated: February 21, 2026

## 1. Objective
Deliver a production-ready MVP for a federated, open-source accounting platform that supports:
- Multi-entity accounting with consolidation (2-3 pilot entities)
- Core POS/ticketing/payments ingestion (Inntopia + Square as Wave 1 MVP connectors)
- Revenue deferral/liability controls and close controls
- Audit/compliance baseline (PCI scope-reduction design, immutable evidence trail)
- Comfortable operation at 2,000 active users with linear scaling behavior and explicit no-bend gates

## 2. Assumptions
- Accounting basis for MVP: dual-book (`US_GAAP` + `IFRS`) from day one.
- ERP UX base: ERPNext first with Odoo-compatible adapter contract.
- Pilot scope is fixed to ski-resort operations, 2-3 legal entities, US + Canada, 8-12 locations.
- Connector API access and test tenants for Inntopia and Square are available by Sprint 1.
- Stripe is the first payment gateway for reconciliation flows, via a pluggable gateway abstraction.
- Canonical event schema in master spec is the contract of record.
- Security model uses Keycloak + OPA; no direct PAN storage in platform.

## 3. Team Topology (Who Builds What)
| Team | FTE | Ownership |
|---|---:|---|
| Platform Core Squad | 3 | Ledger posting API, idempotency, journal immutability, period lock service, event bus foundations |
| Finance Domain Squad | 3 | Rev rec rules (ASC 606), deferred revenue schedules, breakage/refund liability logic, consolidation/elimination rules |
| Integrations Squad | 3 | Connector SDK, Inntopia connector, Square connector, canonical mapping, replay/DLQ tooling |
| Controls & Security Squad | 2 | Keycloak/OPA policies, SoD enforcement, tamper-evident audit trail, PCI control matrix implementation |
| Data/Reconciliation Squad | 2 | Payout/bank ingestion, matching engine, exception queues, close cockpit KPIs |
| QA/Release | 2 | End-to-end test automation, performance/regression, release gates, UAT coordination |
| Program Leads | 2 | Product + architecture decisions, dependency clearing, scope/change control |

## 4. Global Dependencies
### External
- Inntopia and Square sandbox/prod credentials, webhook config approval.
- Stripe payout and settlement export access.
- Pilot legal-entity master data and chart-of-accounts approval.
- Finance policy sign-off for rev rec assumptions (breakage/refund estimates).

### Internal
- Environments: dev/stage with observability baseline by Sprint 1.
- CI/CD + migration workflow + secrets management by Sprint 1.
- Canonical schema + posting rules versioning strategy frozen in Sprint 1.

## 5. Entry and Exit Criteria (Program-Level)
### Program Entry (Week 0)
- Architecture and ERP base approved.
- Named owners assigned for every squad and integration.
- Pilot entities/connectors confirmed.

### Program Exit (End of Week 12)
- Daily transaction ingestion from Inntopia + Square into canonical events and posted journals.
- Multi-entity close run completed for pilot entities with elimination entries.
- Reconciliation workflow operational (auto-match + exception queue + dispute states).
- Audit evidence export available for end-to-end trace (source event -> journal -> close artifact).
- Security/compliance controls active (SoD, immutable logs, PCI scope documentation).

## 6. Sprint-by-Sprint Plan (6 x 2 Weeks)

### Sprint 1 (Weeks 1-2): Foundation and Contract Freeze
**Goal:** Establish technical baseline and lock contracts.

**Build assignments**
- Platform Core: Implement posting API skeleton with idempotency keys, journal schema, and immutable write path.
- Integrations: Deliver connector SDK scaffold (auth/retry/webhook verification) and canonical payload validators.
- Finance Domain: Define chart/dimensions and posting rule templates for POS/ticketing/payments flows.
- Controls & Security: Stand up Keycloak realms/roles and initial OPA policy set for entity boundaries.
- Data/Reconciliation: Define reconciliation data model and exception taxonomy.
- QA/Release: Build CI test harness and contract-test pipeline.

**Deliverables**
- Versioned canonical event schema v1.0.
- Posting API v0 with idempotent behavior tests.
- Connector SDK v0 with replay-safe contract tests.
- Access model v0 (RBAC + ABAC policy packs).

**Dependencies**
- ERP base selected; pilot entity master data available.

**Entry Criteria**
- Program entry criteria met; environments available.

**Exit Criteria**
- Schema and posting contracts signed off by architecture + finance leads.
- CI pipeline passing for schema, posting, and policy tests.

### Sprint 2 (Weeks 3-4): Core Posting and Ingestion MVP
**Goal:** Run first end-to-end ingest-to-post path in stage.

**Build assignments**
- Platform Core: Complete journal immutability/reversal, period-open checks, and posting rule engine v1.
- Integrations: Build Inntopia connector v1 and mapper to canonical model.
- Finance Domain: Implement deferral model for reservations/passes and liability account mappings.
- Controls & Security: Add SoD checks for journal posting and mapping rule changes.
- Data/Reconciliation: Build payment/settlement ingestion adapters (Stripe export + bank file parser v0).
- QA/Release: Deliver integration test suite for ingest -> post traceability.

**Deliverables**
- Inntopia events posted to GL journals in stage.
- Deferred revenue schedule job v1.
- Immutable audit trail v1 with entity-scoped query.

**Dependencies**
- Inntopia API access and sample payload coverage.

**Entry Criteria**
- Sprint 1 contracts frozen.

**Exit Criteria**
- 95%+ valid Inntopia events processed in stage without manual rework.
- Source-event-to-journal trace verified in automated tests.

### Sprint 3 (Weeks 5-6): Payments and Reconciliation Control Loop
**Goal:** Close the cash loop with matching and exception handling.

**Build assignments**
- Platform Core: Extend posting rules for fees, chargebacks, and payout clearing.
- Integrations: Build Square connector v1 (sales, refunds, tenders) and normalize payouts/tenders.
- Finance Domain: Implement refund liability remeasurement and breakage assumption versioning.
- Controls & Security: Add policy gates for estimate changes and break-glass logging.
- Data/Reconciliation: Deliver matching engine v1 (order/payment/payout) and exception queue UI/API.
- QA/Release: Run reconciliation accuracy tests with seeded mismatch cases.

**Deliverables**
- Square connector live in stage.
- Reconciliation engine v1 with deterministic exception routing.
- Chargeback/dispute lifecycle accounting states v1.

**Dependencies**
- Square webhook/callback setup and payout report access.

**Entry Criteria**
- Inntopia ingestion stable in stage.

**Exit Criteria**
- >=70% auto-match in stage test dataset.
- All unmatched items routed with reason codes and owner assignment.

### Sprint 4 (Weeks 7-8): Multi-Entity and Consolidation MVP
**Goal:** Enable federated entity operation and first consolidation close.

**Build assignments**
- Platform Core: Enforce posting boundary by legal entity and intercompany subledger primitives.
- Finance Domain: Implement due-to/due-from logic, elimination rules v1, FX translation v1.
- Integrations: Add legal_entity/location routing enrichment in connector pipelines.
- Controls & Security: Harden OPA for non-bypassable SoD across posting, approvals, and master data changes.
- Data/Reconciliation: Add entity-level close checklist service and dependency statuses.
- QA/Release: Create multi-entity close simulation suite.

**Deliverables**
- Consolidation pipeline v1 (2-3 entities).
- Intercompany posting + elimination journal generation.
- Close cockpit v1 with period lock hooks.

**Dependencies**
- Intercompany policy and transfer-pricing metadata agreed with finance.

**Entry Criteria**
- Reconciliation loop operational for pilot flows.

**Exit Criteria**
- Dry-run close completed for pilot entities including elimination and FX translation.
- Period lock/approval flow blocks unauthorized postings.

### Sprint 5 (Weeks 9-10): Compliance Hardening and UAT
**Goal:** Make controls auditable and prepare production readiness.

**Build assignments**
- Controls & Security: Implement tamper-evident log sealing, access-review reports, PCI scope/control ownership matrix.
- Platform Core: Add retention/legal hold controls and immutable adjustment workflow.
- Finance Domain: Finalize rev rec disclosures and rollforward outputs.
- Integrations: Harden replay/backfill tooling and run connector resiliency tests.
- Data/Reconciliation: Add dispute aging and unresolved exception SLA metrics.
- QA/Release: Execute full regression, performance, and UAT scripts.

**Deliverables**
- Compliance evidence pack v1 (audit export, SoD logs, access reviews).
- Performance baseline report (posting + ingestion + reconciliation).
- UAT sign-off checklist with defects triaged.

**Dependencies**
- Security/compliance reviewers available for control walkthrough.

**Entry Criteria**
- Consolidation dry-run successful.

**Exit Criteria**
- Critical/high defects at zero for MVP scope.
- Control evidence accepted by internal audit/compliance stakeholders.

### Sprint 6 (Weeks 11-12): Pilot Go-Live and Hypercare
**Goal:** Launch pilot entities and prove month-end execution.

**Build assignments**
- Platform Core: Production cutover support, migration scripts, rollback playbooks.
- Integrations: Enable production connectors for Inntopia + Square with monitoring/alerts.
- Finance Domain: Run first live deferral/reclassification jobs and close support.
- Controls & Security: Enforce production SoD policies, break-glass drills, final access attestations.
- Data/Reconciliation: Operate daily reconciliation and exception triage rotations.
- QA/Release: Validate release gates and post-release smoke/perf checks.

**Deliverables**
- Pilot production go-live.
- First period close package for pilot entities.
- Hypercare dashboard with defect/incident tracking.

**Dependencies**
- Change-approval board sign-off and production credential issuance.

**Entry Criteria**
- Sprint 5 UAT and compliance gates passed.

**Exit Criteria**
- Pilot close completed within KPI targets.
- Operational handoff complete with runbooks/on-call rotation.

## 7. Key Risks and Mitigations
| Risk | Impact | Mitigation | Owner |
|---|---|---|---|
| Partner API instability (Inntopia/Square) | Delays ingestion, data gaps | Contract tests, replay tooling, webhook retry backoff, fallback batch pulls | Integrations Lead |
| Source data quality issues | Reconciliation noise, close delays | Strict schema validation, reason-coded exceptions, daily triage SLA | Data/Reconciliation Lead |
| Scope creep in ERP customization | Delivery slowdown, brittle architecture | Keep ERP as operational UX only; core accounting logic in posting domain services | Architecture Lead |
| Compliance control gaps found late | Go-live risk | Shift-left control testing in Sprints 2-5, weekly control evidence reviews | Security Lead |
| Intercompany policy ambiguity | Consolidation errors | Freeze intercompany contracts/rules by Sprint 4 entry | Finance Lead |

## 8. Measurable Success KPIs (By End of Week 12)
| KPI | Target |
|---|---|
| Canonical event ingestion success rate (valid events) | >=98% |
| Journal posting idempotency error rate | <=0.1% |
| Source-to-journal trace completeness | 100% for in-scope transactions |
| Auto-match rate for payment reconciliation | >=85% |
| Exception queue SLA (first triage) | <24 hours |
| Pilot close cycle time | <=5 business days |
| Unauthorized posting/control bypass incidents | 0 |
| P1/P2 production defects in first 2 weeks post go-live | <=3 total |
| Connector replay/backfill recovery time objective | <4 hours |

## 9. Deliverables Summary
- Production-capable core posting ledger service with immutability and controls.
- Inntopia + Square connectors mapped to canonical event schema.
- Deferred revenue, liability, and intercompany/consolidation workflows for pilot entities.
- Reconciliation engine with exception operations and dispute accounting.
- Compliance/audit evidence framework (SoD, immutable logs, PCI scope controls).
- Go-live runbooks, on-call model, and close-operating procedures.
