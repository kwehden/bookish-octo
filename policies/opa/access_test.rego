package acctcore.authz

import rego.v1

# Allowed: same entity + approver role
test_allow_same_entity_approver if {
  allow with input as {
    "action": "posting",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"}
  }
}

# Denied: cross-entity
test_deny_cross_entity if {
  not allow with input as {
    "action": "posting",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "CA_BC_01"}
  }
}

# Denied: finance operator cannot perform critical actions alone
test_deny_sod_violation if {
  not allow with input as {
    "action": "policy_change",
    "subject": {"role": "finance_operator", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"}
  }
}
