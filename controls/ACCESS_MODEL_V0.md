# Access Model v0 (RBAC + ABAC)

Core roles:
- `finance_operator`
- `finance_approver`
- `controls_admin`
- `integration_operator`
- `recon_analyst`

ABAC constraints:
- `tenant_id` must match token claim.
- `legal_entity_id` must match token allowed set.

Critical actions requiring SoD:
- posting
- period lock
- policy/ruleset change
