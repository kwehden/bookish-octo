# Sprint 7 Squad Agent Execution Plan

Last updated: February 23, 2026  
Scope: Sprint 7 remediation sprint for remaining design/implementation gaps (items 2-4 from post-Sprint 6 review)

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

## 2. Sprint 7 Objective
Close the remaining in-repo items:
1. Persistent actor model expansion beyond idempotency/audit-seal to additional authoritative runtime state.
2. Sprint 4 dry-run partial closure: integrated elimination/FX output coupling evidence.
3. Finance/governance backfill debt in Sprint 1-3 checklist/signoff/gate/completion artifacts.

## 3. Build Assignments by Squad
| Squad | Epic focus (Sprint 7) | Build assignments |
|---|---|---|
| Platform | Runtime durability | Add eventual-consistent write-behind persistence for journals + periods with restart reload and flush barriers; wire `AppState::with_persistence_dir` to persistent repos |
| Integration | Consolidation coupling evidence | Provide deterministic elimination/FX coupling evidence path in close dry-run flow |
| Data/Reconciliation | Close simulation correctness | Enforce elimination/FX coupled-output validation in dry-run model and add deterministic tests |
| Finance | Governance debt closure | Backfill baseline + Sprint 2/3 finance checklist evidence with explicit references |
| Controls&Sec | Signoff closure | Backfill Sprint 1/2/3 signoff packets with in-repo closure statements and impact references |
| QA/Release | Gate artifacts | Backfill Sprint 1/2/3 gate/completion artifacts and publish Sprint 7 gate status |

## 4. Per-Member Work Plan
### Platform
- `PL-L`: Own persistence architecture and restart semantics for journal + period state.
- `PL-W1`: Implement write-behind + flush/reload for `InMemoryJournalRepository`.
- `PL-W2`: Implement write-behind + flush/reload for `InMemoryPeriodRepository` and `AppState` wiring.

### Integration
- `IN-L`: Own coupled-output validation criteria for elimination + FX artifacts.
- `IN-W1`: Define elimination output validity requirements for dry-run coupling.
- `IN-W2`: Define FX translation output validity requirements for dry-run coupling.

### Data/Reconciliation
- `DA-L`: Own deterministic dry-run outcome model updates.
- `DA-W1`: Implement coupling validators and blocker taxonomy.
- `DA-W2`: Add deterministic tests and ordering checks.

### Finance
- `FI-L`: Own checklist backfill closure and evidence references.
- `FI-W1`: Backfill baseline finance checklist with explicit in-repo references.
- `FI-W2`: Backfill Sprint 2/3 finance closure references.

### Controls&Sec
- `CS-L`: Own early sprint signoff debt closure policy.
- `CS-W1`: Backfill Sprint 1/2/3 signoff packets with explicit closure records.
- `CS-W2`: Validate decision-ledger/control references remain consistent.

### QA/Release
- `QA-L`: Own Sprint 7 remediation gate status.
- `QA-W1`: Backfill Sprint 1/2/3 gate artifacts for in-repo closure semantics.
- `QA-W2`: Publish Sprint 1/2/3 completion reports and run full gate command.

## 5. Decision Impact Workflow (Finance + Controls)
1. Any decision marked `financial_impact=yes` or `control_impact=yes` is blocked for merge.
2. Finance and Controls signoff is required before remediation closure is considered complete.
3. External UAT/performance attestations remain open and are not backfilled by Sprint 7.

## 6. Pike’s 5 Rules Applied
1. Do not guess bottlenecks: persistence changes target identified authoritative-state gaps only.
2. Measure before optimize: persistence and dry-run coupling are validated with deterministic tests.
3. Prefer simple common-path behavior: in-memory remains fast path; write-behind adds eventual durability.
4. Keep implementation simple: explicit artifact-chain backfills replace implicit status assumptions.
5. Data model quality dominates: dry-run now requires explicit elimination/FX output validity.

## 7. Major Tool-Use Summary (Major Activities Only)
| Squad | Major activities | Tool use summary |
|---|---|---|
| Platform | Repository persistence expansion | Rust tests (`ledger-posting`, `posting-api`), write-behind/reload validation |
| Integration/Data | Dry-run coupling closure | `reconciliation-model` tests and contract-aligned blocker checks |
| Finance/Controls | Artifact backfill closure | Checklist/signoff diff updates and decision-reference validation |
| QA/Release | Gate verification | `./scripts/run_contract_gates.sh` and Sprint gate artifact updates |

## 8. Sprint 7 Exit Gate (Authoritative)
Sprint 7 exits only if all pass:
1. Journal repository supports write-behind persistence + restart reload + flush tests.
2. Period repository supports write-behind persistence + restart reload + flush tests.
3. `AppState::with_persistence_dir` uses persistent journal + period repositories.
4. Multi-entity close dry-run enforces elimination/FX coupling requirements with deterministic tests.
5. Sprint 4 criterion #6 is fully closed in Sprint 4 completion/gate artifacts.
6. Finance baseline and Sprint 2/3 checklist backfill items are closed with explicit references.
7. Sprint 1/2/3 signoff and gate artifacts are backfilled to in-repo closure state.
8. CI gate command remains green (`./scripts/run_contract_gates.sh`).

## 9. Sprint 7 Artifact Set
1. `specs/SPRINT7_SQUAD_AGENT_EXECUTION_PLAN.md`
2. `governance/sprint7_signoff_packet.md`
3. `governance/SPRINT7_PLAN_COMPLETION_REPORT.md`
4. `qa/SPRINT7_GATE_REPORT.md`
5. Updated Sprint 1/2/3 finance/governance/qa artifacts
6. Updated runtime durability code in `ledger-posting` and `posting-api`
