# Control Gates Register v1

- G1: Entity boundary gate
- G2: SoD baseline gate
- G3: Policy regression gate
- G4: Evidence chain gate
- G5: Stage control activation gate

All gates are release/merge blockers in Sprint 1 when impacted.

## Sprint 2 Status

- G1: Entity boundary gate - `Active` (validated via same-entity policy checks)
- G2: SoD baseline gate - `Expanded` (non-bypassable on `posting`, `mapping_change`, `ruleset_change`)
- G3: Policy regression gate - `Active` (OPA test coverage extended for Sprint 2 controls)
- G4: Evidence chain gate - `Expanded` (Sprint 2 signoff and finance checklist linked)
- G5: Stage control activation gate - `Active` (gate script enforces Sprint 2 artifact checks)

## Sprint 3 Status

- G1: Entity boundary gate - `Active` (no change; Sprint 3 regression coverage retained)
- G2: SoD baseline gate - `Expanded` (non-bypassable actions now include `estimate_change` and `dispute_approval`)
- G3: Policy regression gate - `Expanded` (OPA tests added for Sprint 3 SoD + break-glass logging completeness scenarios)
- G4: Evidence chain gate - `Expanded` (Sprint 3 signoff packet + completion report added to governance evidence set)
- G5: Stage control activation gate - `Expanded` (gate script validates Sprint 3 artifact and section presence before execution)

## Sprint 4 Status

- G1: Entity boundary gate - `Expanded` (entity scope model hardened for multi-entity approval paths with explicit intercompany pair checks)
- G2: SoD baseline gate - `Expanded` (non-bypassable actions now include `posting_approval`, `intercompany_posting_approval`, `close_approval`, and `master_data_change`)
- G3: Policy regression gate - `Expanded` (OPA tests added for intercompany posting approval constraints, close approval constraints, and master-data control paths)
- G4: Evidence chain gate - `Expanded` (Sprint 4 signoff packet + Sprint 4 completion rubric linked as required evidence)
- G5: Stage control activation gate - `Expanded` (gate script now enforces Sprint 4 governance artifacts and Sprint 4 section checks)
