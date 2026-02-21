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
