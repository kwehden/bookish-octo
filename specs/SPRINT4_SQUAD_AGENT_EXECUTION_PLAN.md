# Sprint 4 Squad Agent Execution Plan

Last updated: February 21, 2026
Scope: Sprint 4 (Weeks 7-8), Multi-Entity and Consolidation MVP (US + Canada ski-resort pilot)

## 1. Agent Topology
Coordinator:
- `ARCH-L` Architect Coordinator

Squads (1 Leader + 2 Workers each):
- Platform: `PL-L`, `PL-W1`, `PL-W2`
- Finance: `FI-L`, `FI-W1`, `FI-W2`
- Integration: `IN-L`, `IN-W1`, `IN-W2`
- Controls&Sec: `CS-L`, `CS-W1`, `CS-W2`
- Data/Reconciliation: `DA-L`, `DA-W1`, `DA-W2`
- QA/Release: `QA-L`, `QA-W1`, `QA-W2`

## 2. Sprint 4 Build Assignments by Squad
Verified Sprint 3 baseline input:
- Square normalization, Sprint 3 posting-rule extensions, and deterministic matching baseline are implemented.
- Sprint 3 completion is partial at overall gate level; cross-squad stage metrics and approvals remain open.
- Controls baseline expanded for estimate/dispute actions and break-glass logging completeness.

| Squad | Epic focus (Sprint 4) | Build assignments |
|---|---|---|
| Platform | A/B/C | Enforce legal-entity posting boundaries, add intercompany subledger primitives, and provide consolidation-safe period-lock hooks |
| Finance | C/J | Implement due-to/due-from logic, elimination rules v1, and FX translation policy pack for US/CA consolidation |
| Integration | D/E/I | Add legal_entity + location routing enrichment for Inntopia/Square connector pipelines and consolidation context propagation |
| Controls&Sec | H + cross-epic gates | Harden OPA for non-bypassable SoD across posting, approvals, and master-data changes in multi-entity flows |
| Data/Reconciliation | F/G | Build entity-level close checklist service, dependency status engine, and consolidation trace coverage |
| QA/Release | Cross-epic quality gates | Execute multi-entity close simulation suite and enforce dry-run consolidation exit gates |

## 3. Per-Member Work Plan
### Platform
- `PL-L`: Own intercompany boundary architecture and signoff packet for consolidation-safe posting behavior.
- `PL-W1`: Implement legal-entity boundary enforcement and intercompany subledger record primitives.
- `PL-W2`: Implement consolidation hooks for period-lock and intercompany journal lifecycle controls.

### Finance
- `FI-L`: Own Finance Impact Board decisions for intercompany policy and consolidation assumptions.
- `FI-W1`: Implement due-to/due-from mapping logic and elimination rule definitions.
- `FI-W2`: Implement FX translation policy v1 and dual-book consolidation tie-out fixtures.

### Integration
- `IN-L`: Coordinate connector enrichment readiness and legal_entity/location routing dependencies.
- `IN-W1`: Implement Inntopia pipeline enrichment with legal_entity/location routing fields.
- `IN-W2`: Implement Square pipeline enrichment and reconciliation context propagation for multi-entity close.

### Controls&Sec
- `CS-L`: Own non-bypassable SoD scope for multi-entity approvals and master-data changes.
- `CS-W1`: Implement OPA policy extensions for posting approvals and intercompany/master-data governance controls.
- `CS-W2`: Implement policy evidence exports for multi-entity authorization and period-lock approval traces.

### Data/Reconciliation
- `DA-L`: Own close checklist service scope and entity dependency model.
- `DA-W1`: Implement entity-level close checklist API and dependency-state transitions.
- `DA-W2`: Implement consolidation trace checks linking source->journal->entity close artifacts.

### QA/Release
- `QA-L`: Own Sprint 4 dry-run close gate recommendation and signoff packet.
- `QA-W1`: Build multi-entity close simulation suite for 2-3 pilot entities.
- `QA-W2`: Build regression assertions for elimination, FX translation, and period-lock authorization behavior.

## 4. Architect Coordination and Updates
Cadence:
1. Daily squad standups (internal, 15 min).
2. Daily Architect sync (all leaders, 20 min).
3. Tue/Thu consolidation dependency board (30 min).
4. Friday sprint control review with Finance + Controls mandatory (45 min).

Architect update template:
```md
Squad: <...>
RAG: <Green|Amber|Red>
Completed (24h):
Next (24h):
Dependencies/Blockers:
Decisions Needed: <decision> | financial_impact:<yes/no> | control_impact:<yes/no>
Exit-Criteria Progress: entity_boundary_pass:% | elimination_fixture_pass:% | fx_translation_tests:% | close_simulation_pass:% | period_lock_authz_pass:%
```

Architect Update 1 (current):
- Sprint 4 scope fixed to multi-entity boundaries and first consolidation dry run.
- Sprint 3 carry-forward approvals and stage gates remain merge blockers where impacted.
- Pike-rule simplification retained: enforce explicit entity boundaries and deterministic elimination logic before broader optimization.

## 5. Decision Impact Workflow (Finance + Controls)
1. Any decision marked `financial_impact=yes` or `control_impact=yes` is blocked for merge.
2. Required approvals:
- Finance Impact Board approval (`FI-L` + delegate reviewers).
- Controls Impact Board approval (`CS-L` + delegate reviewers).
3. QA enforces merge gate: no merge without both approvals when flagged.
4. Architect resolves technical direction only after both checks complete.
5. Decisions affecting elimination logic, FX translation policy, intercompany mappings, or period-lock approvals are always flagged.

## 6. Finance Impact Board (Required Decisions)
Required financial approval before merge:
1. Due-to/due-from account mapping and intercompany classification standards.
2. Elimination entry sequencing and precedence rules.
3. FX translation rate sources, cutover timing, and policy versioning for consolidation.
4. Dual-book consolidation tie-out requirements by legal entity.
5. Close dependency handling for unresolved exceptions across entities.
6. Consolidation variance thresholds and escalation policy.

Minimum evidence:
- Signed intercompany mapping matrix by entity pair.
- Elimination fixture pack with expected journal outputs.
- FX translation and dual-book consolidation tie-out report.

## 7. Controls Impact Board (Required Decisions)
Required controls approval before merge:
1. Non-bypassable SoD for multi-entity posting and intercompany approvals.
2. Master-data and mapping/ruleset change controls in consolidation-critical paths.
3. Period-lock and close-approval authorization enforcement across entities.
4. Consolidation evidence-chain completeness from source through close artifacts.
5. CI merge-policy requirements for multi-entity authorization and close simulation tests.

Minimum evidence:
- Updated role/action/entity matrix covering intercompany and close approval actions.
- OPA positive/negative tests for multi-entity authorization scenarios.
- Consolidation trace extracts proving source->journal->close linkage.

## 8. Pike’s 5 Rules Review Outcome (Sprint 4)
Accepted corrections for Sprint 4:
1. Build one complete multi-entity dry-run close path first (2-3 entities) before expanding entity count.
2. Keep entity boundaries explicit and non-overloaded in posting and reconciliation flows.
3. Keep elimination and FX translation deterministic before introducing automation heuristics.
4. Preserve immutable provenance across intercompany and consolidation journals.
5. Use quantitative close-simulation gates as hard blockers for Sprint 4 exit.

## 9. Major Tool-Use Summary by Squad (Major Activities Only)
| Squad | Major activities | Tool use summary |
|---|---|---|
| Platform | Entity-boundary and intercompany primitives | Rust invariant suites, boundary authorization fixtures, intercompany posting regressions |
| Finance | Elimination + FX policy implementation | Consolidation fixture packs, dual-book tie-out reports, policy version evidence |
| Integration | Connector routing enrichment | Contract tests, entity/location routing fixtures, replay/backfill validation |
| Controls&Sec | Multi-entity SoD hardening | OPA/Rego regression suites, approval trace exports, control matrix validation |
| Data | Close checklist + dependency engine | Close-state transition tests, dependency graph validators, traceability assertions |
| QA/Release | Multi-entity dry-run close simulation | End-to-end close runners, elimination/FX assertions, gate dashboards |

## 10. Sprint 4 Exit Gate (Authoritative)
Sprint 4 exits only if all pass:
1. Legal-entity posting boundary controls pass for all in-scope connector and posting paths.
2. Intercompany subledger primitives and due-to/due-from mappings are implemented and Finance-approved.
3. Elimination rules v1 generate expected consolidation journals in simulation fixtures.
4. FX translation v1 passes policy and dual-book tie-out tests for US/CA entities.
5. Close checklist service reports dependency states and blocks progression on unresolved blockers.
6. Multi-entity dry-run close completes for 2-3 pilot entities including elimination and FX translation outputs.
7. Period lock and close-approval flows block unauthorized actions in automated authorization tests.
8. Finance and Controls approvals are complete for all flagged Sprint 4 decisions.
9. CI gates are green for contracts, posting, controls, integration, consolidation, and close simulation suites.
