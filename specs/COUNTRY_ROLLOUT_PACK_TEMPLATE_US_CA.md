# Country Rollout Pack Template (US + CA)

Last updated: `<YYYY-MM-DD>`
Owner: `<PROGRAM_OWNER>`
Version: `<VERSION>`

## How to Use
- Duplicate the country section for each new jurisdiction.
- Keep all checkboxes as evidence-based gates; do not mark complete without linked artifacts.
- Replace all placeholders in `<ANGLE_BRACKETS>`.

---

## Country Rollout Template - United States (US)

### 1. Legal Entities
- [ ] Legal entity name: `<US_LEGAL_ENTITY_NAME>`
- [ ] Entity type confirmed (e.g., Corp/LLC): `<US_ENTITY_TYPE>`
- [ ] EIN validated: `<US_EIN>`
- [ ] Secretary of State registration verified: `<US_STATE_REGISTRATION_ID>`
- [ ] Banking entity matches legal entity: `<US_BANK_ACCOUNT_OWNER>`
- [ ] Intercompany relationships documented: `<US_INTERCOMPANY_MATRIX_LINK>`

### 2. Tax Model
- [ ] Tax nexus states documented: `<US_NEXUS_STATES_LIST>`
- [ ] Sales tax engine configured: `<US_TAX_ENGINE>`
- [ ] Product/service taxability mappings approved: `<US_TAXABILITY_MATRIX_LINK>`
- [ ] Jurisdiction-level rates and overrides validated: `<US_RATE_VALIDATION_EVIDENCE>`
- [ ] Exemption certificate handling configured: `<US_EXEMPTION_PROCESS_LINK>`
- [ ] Filing calendar and owners assigned: `<US_FILING_CALENDAR_LINK>`

### 3. Invoicing / E-Receipt Requirements
- [ ] Invoice numbering pattern approved: `<US_INVOICE_SEQUENCE_PATTERN>`
- [ ] Mandatory invoice fields configured (seller, buyer, tax breakdown, totals)
- [ ] Credit memo and void workflows validated: `<US_CREDIT_MEMO_TEST_EVIDENCE>`
- [ ] E-receipt content and delivery channel approved: `<US_ERECEIPT_CHANNELS>`
- [ ] Retention period configured: `<US_INVOICE_RETENTION_POLICY>`

### 4. Payment / Compliance Controls
- [ ] Supported rails enabled (cards/ACH/wallets): `<US_PAYMENT_RAILS>`
- [ ] PCI scope and SAQ type confirmed: `<US_PCI_SCOPE_AND_SAQ>`
- [ ] Chargeback/dispute accounting flow tested: `<US_CHARGEBACK_UAT_LINK>`
- [ ] Sanctions/AML screening requirements documented: `<US_AML_CONTROL_LINK>`
- [ ] SoD matrix for payment approvals enforced: `<US_SOD_MATRIX_LINK>`

### 5. Data Residency / Privacy Checks
- [ ] Data hosting location documented: `<US_HOSTING_REGION>`
- [ ] US privacy obligations assessed (state-level laws): `<US_PRIVACY_ASSESSMENT_LINK>`
- [ ] PII data map completed: `<US_PII_DATA_MAP_LINK>`
- [ ] Data retention + deletion policy tested: `<US_RETENTION_DELETION_EVIDENCE>`
- [ ] DPA and vendor subprocessors approved: `<US_DPA_LINK>`

### 6. Chart / Dimension Localization
- [ ] US chart mapping completed: `<US_COA_MAPPING_LINK>`
- [ ] Required dimensions enabled (`legal_entity`, `location`, `channel`, `product`, `currency`, `intercompany`, `segment`)
- [ ] Local reporting dimensions added (if any): `<US_LOCAL_DIMENSIONS>`
- [ ] Dual-book revenue recognition rules (`US_GAAP` + `IFRS`) configured: `<US_REVREC_POLICY_LINK>`
- [ ] Dual-book policy package/version references captured: `<US_BOOK_POLICY_EVIDENCE_LINK>`
- [ ] Local account descriptions reviewed by finance: `<US_FINANCE_APPROVAL_LINK>`

### 7. Close / Reporting Pack
- [ ] Period close calendar approved: `<US_CLOSE_CALENDAR_LINK>`
- [ ] Reconciliation checklist complete (cash, AR, AP, tax, deferred revenue)
- [ ] Trial balance and P&L format signed off: `<US_REPORT_PACK_LINK>`
- [ ] US dual-book tie-out approved (`US_GAAP` vs `IFRS` deltas documented): `<US_DUAL_BOOK_TIEOUT_EVIDENCE>`
- [ ] Sales tax liability reporting validated: `<US_TAX_LIABILITY_REPORT_EVIDENCE>`
- [ ] Audit trail and journal immutability test complete: `<US_AUDIT_CONTROL_EVIDENCE>`

### 8. UAT Checklist
- [ ] End-to-end order-to-cash scenario passed: `<US_UAT_OTC_CASES_LINK>`
- [ ] Refund/void/reversal scenarios passed: `<US_UAT_REFUND_CASES_LINK>`
- [ ] Tax calculation regression set passed: `<US_UAT_TAX_REGRESSION_LINK>`
- [ ] Ski-domain scenarios passed (season pass, lessons, rentals, weather-closure refunds): `<US_UAT_SKI_DOMAIN_CASES_LINK>`
- [ ] Posting to GL and subledger tie-out passed: `<US_UAT_TIEOUT_EVIDENCE>`
- [ ] Critical defects resolved (Severity 1/2): `<US_DEFECT_LOG_LINK>`
- [ ] Business owner UAT approval recorded: `<US_UAT_SIGNOFF_LINK>`

### 9. Cutover Checklist
- [ ] Cutover plan approved: `<US_CUTOVER_PLAN_LINK>`
- [ ] Legacy/opening balances reconciled and loaded: `<US_OPENING_BALANCE_EVIDENCE>`
- [ ] Master data freeze window communicated: `<US_FREEZE_COMMS_LINK>`
- [ ] Gateway abstraction conformance test (Stripe and alternate provider stub) passed: `<US_GATEWAY_SWITCH_EVIDENCE>`
- [ ] Backout plan documented and tested: `<US_BACKOUT_PLAN_LINK>`
- [ ] Hypercare staffing roster confirmed: `<US_HYPERCARE_ROSTER_LINK>`
- [ ] Go/no-go meeting completed: `<US_GONOGO_MINUTES_LINK>`

### 10. Go-Live Signoffs
- [ ] Country Finance Lead: `<US_FINANCE_LEAD_NAME>` | Date: `<YYYY-MM-DD>`
- [ ] Tax Lead: `<US_TAX_LEAD_NAME>` | Date: `<YYYY-MM-DD>`
- [ ] Compliance/Security Lead: `<US_COMPLIANCE_LEAD_NAME>` | Date: `<YYYY-MM-DD>`
- [ ] Product/Engineering Lead: `<US_ENGINEERING_LEAD_NAME>` | Date: `<YYYY-MM-DD>`
- [ ] Program Director final approval: `<US_PROGRAM_DIRECTOR_NAME>` | Date: `<YYYY-MM-DD>`

---

## Country Rollout Template - Canada (CA)

### 1. Legal Entities
- [ ] Legal entity name: `<CA_LEGAL_ENTITY_NAME>`
- [ ] Entity type confirmed (e.g., Corporation/ULC/LP): `<CA_ENTITY_TYPE>`
- [ ] Business Number (BN) validated: `<CA_BUSINESS_NUMBER>`
- [ ] Provincial registration verified: `<CA_PROVINCIAL_REGISTRATION_ID>`
- [ ] GST/HST account number validated: `<CA_GST_HST_ACCOUNT_NUMBER>`
- [ ] Intercompany relationships documented: `<CA_INTERCOMPANY_MATRIX_LINK>`

### 2. Tax Model
- [ ] GST/HST registration footprint by province documented: `<CA_GST_HST_PROVINCE_MATRIX_LINK>`
- [ ] PST/QST applicability assessed and configured: `<CA_PST_QST_ASSESSMENT_LINK>`
- [ ] Province-specific rates configured and validated: `<CA_RATE_VALIDATION_EVIDENCE>`
- [ ] Place-of-supply rules configured: `<CA_PLACE_OF_SUPPLY_RULES_LINK>`
- [ ] Zero-rated/exempt mappings approved: `<CA_TAXABILITY_MATRIX_LINK>`
- [ ] Filing calendar and owners assigned: `<CA_FILING_CALENDAR_LINK>`

### 3. Invoicing / E-Receipt Requirements
- [ ] Invoice numbering pattern approved: `<CA_INVOICE_SEQUENCE_PATTERN>`
- [ ] Mandatory invoice fields configured (BN, tax type/rate/amount, totals)
- [ ] Credit note and void workflows validated: `<CA_CREDIT_NOTE_TEST_EVIDENCE>`
- [ ] Bilingual labeling requirements assessed (including Quebec where applicable): `<CA_BILINGUAL_LABEL_ASSESSMENT_LINK>`
- [ ] Retention period configured: `<CA_INVOICE_RETENTION_POLICY>`

### 4. Payment / Compliance Controls
- [ ] Supported rails enabled (cards/Interac EFT/wallets): `<CA_PAYMENT_RAILS>`
- [ ] PCI scope and SAQ type confirmed: `<CA_PCI_SCOPE_AND_SAQ>`
- [ ] Chargeback/dispute accounting flow tested: `<CA_CHARGEBACK_UAT_LINK>`
- [ ] AML/sanctions controls assessed (including FINTRAC obligations): `<CA_AML_CONTROL_LINK>`
- [ ] SoD matrix for payment approvals enforced: `<CA_SOD_MATRIX_LINK>`

### 5. Data Residency / Privacy Checks
- [ ] Data hosting location documented: `<CA_HOSTING_REGION>`
- [ ] PIPEDA obligations assessed: `<CA_PIPEDA_ASSESSMENT_LINK>`
- [ ] Provincial privacy obligations assessed (including Quebec Law 25 where applicable): `<CA_PROVINCIAL_PRIVACY_ASSESSMENT_LINK>`
- [ ] Cross-border transfer controls documented: `<CA_TRANSFER_CONTROL_LINK>`
- [ ] Data retention + deletion policy tested: `<CA_RETENTION_DELETION_EVIDENCE>`

### 6. Chart / Dimension Localization
- [ ] CA chart mapping completed: `<CA_COA_MAPPING_LINK>`
- [ ] Required dimensions enabled (`legal_entity`, `location`, `channel`, `product`, `currency`, `intercompany`, `segment`)
- [ ] Local reporting dimensions added (if any): `<CA_LOCAL_DIMENSIONS>`
- [ ] Dual-book revenue recognition rules (`US_GAAP` + `IFRS`) aligned with group policy and local reporting requirements: `<CA_REVREC_POLICY_LINK>`
- [ ] Dual-book policy package/version references captured: `<CA_BOOK_POLICY_EVIDENCE_LINK>`
- [ ] Bilingual labels implemented where required: `<CA_LABEL_LOCALIZATION_LINK>`

### 7. Close / Reporting Pack
- [ ] Period close calendar approved: `<CA_CLOSE_CALENDAR_LINK>`
- [ ] Reconciliation checklist complete (cash, AR, AP, GST/HST, PST/QST, deferred revenue)
- [ ] Trial balance and management/statutory report format signed off: `<CA_REPORT_PACK_LINK>`
- [ ] CA dual-book tie-out approved (`US_GAAP` vs `IFRS` deltas documented): `<CA_DUAL_BOOK_TIEOUT_EVIDENCE>`
- [ ] GST/HST return support pack validated: `<CA_GST_HST_SUPPORT_EVIDENCE>`
- [ ] Provincial tax return support packs validated (PST/QST where applicable): `<CA_PROVINCIAL_TAX_SUPPORT_EVIDENCE>`

### 8. UAT Checklist
- [ ] End-to-end order-to-cash scenario passed: `<CA_UAT_OTC_CASES_LINK>`
- [ ] Refund/void/reversal scenarios passed: `<CA_UAT_REFUND_CASES_LINK>`
- [ ] GST/HST/PST/QST calculation regression set passed: `<CA_UAT_TAX_REGRESSION_LINK>`
- [ ] Ski-domain scenarios passed (season pass, lessons, rentals, weather-closure refunds): `<CA_UAT_SKI_DOMAIN_CASES_LINK>`
- [ ] Posting to GL and subledger tie-out passed: `<CA_UAT_TIEOUT_EVIDENCE>`
- [ ] Critical defects resolved (Severity 1/2): `<CA_DEFECT_LOG_LINK>`
- [ ] Business owner UAT approval recorded: `<CA_UAT_SIGNOFF_LINK>`

### 9. Cutover Checklist
- [ ] Cutover plan approved: `<CA_CUTOVER_PLAN_LINK>`
- [ ] Legacy/opening balances reconciled and loaded: `<CA_OPENING_BALANCE_EVIDENCE>`
- [ ] Master data freeze window communicated: `<CA_FREEZE_COMMS_LINK>`
- [ ] Gateway abstraction conformance test (Stripe and alternate provider stub) passed: `<CA_GATEWAY_SWITCH_EVIDENCE>`
- [ ] Backout plan documented and tested: `<CA_BACKOUT_PLAN_LINK>`
- [ ] Hypercare staffing roster confirmed: `<CA_HYPERCARE_ROSTER_LINK>`
- [ ] Go/no-go meeting completed: `<CA_GONOGO_MINUTES_LINK>`

### 10. Go-Live Signoffs
- [ ] Country Finance Lead: `<CA_FINANCE_LEAD_NAME>` | Date: `<YYYY-MM-DD>`
- [ ] Tax Lead: `<CA_TAX_LEAD_NAME>` | Date: `<YYYY-MM-DD>`
- [ ] Compliance/Security Lead: `<CA_COMPLIANCE_LEAD_NAME>` | Date: `<YYYY-MM-DD>`
- [ ] Product/Engineering Lead: `<CA_ENGINEERING_LEAD_NAME>` | Date: `<YYYY-MM-DD>`
- [ ] Program Director final approval: `<CA_PROGRAM_DIRECTOR_NAME>` | Date: `<YYYY-MM-DD>`

---

## Shared Evidence Index (Optional)
- Architecture decisions: `<ADR_LINK>`
- Control matrix: `<CONTROL_MATRIX_LINK>`
- Test evidence repository: `<TEST_EVIDENCE_REPO_LINK>`
- Cutover runbook: `<CUTOVER_RUNBOOK_LINK>`
- Risk register: `<RISK_REGISTER_LINK>`
