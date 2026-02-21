package acctcore.authz

import rego.v1

valid_break_glass := {
  "enabled": true,
  "ticket_id": "BG-2026-0001",
  "reason": "Emergency close support",
  "approved_by": "CS-L",
  "audit_ref": "AUD-2026-02-21-01",
  "activated_at_ns": 1700000000000000000,
  "expires_at_ns": 1700007200000000000
}

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

# Denied: finance operator cannot change mapping
test_deny_sod_mapping_change if {
  not allow with input as {
    "action": "mapping_change",
    "subject": {"role": "finance_operator", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"}
  }
}

# Denied: finance operator cannot change rulesets
test_deny_sod_ruleset_change if {
  not allow with input as {
    "action": "ruleset_change",
    "subject": {"role": "finance_operator", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"}
  }
}

# Denied: break-glass cannot bypass SoD on posting
test_deny_break_glass_on_posting if {
  not allow with input as {
    "action": "posting",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"},
    "request_time_ns": 1700003600000000000,
    "break_glass": valid_break_glass
  }
}

# Allowed: break-glass can be used on period lock when compliant
test_allow_valid_break_glass_period_lock if {
  allow with input as {
    "action": "period_lock",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"},
    "request_time_ns": 1700003600000000000,
    "break_glass": valid_break_glass
  }
}

# Denied: break-glass TTL must not exceed configured max
test_deny_break_glass_ttl_exceeded if {
  not allow with input as {
    "action": "period_lock",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"},
    "request_time_ns": 1700003600000000000,
    "break_glass": {
      "enabled": true,
      "ticket_id": "BG-2026-0002",
      "reason": "Emergency close support",
      "approved_by": "CS-L",
      "audit_ref": "AUD-2026-02-21-02",
      "activated_at_ns": 1700000000000000000,
      "expires_at_ns": 1700018000000000000
    }
  }
}

# Denied: break-glass requires complete audit metadata
test_deny_break_glass_missing_audit_fields if {
  not allow with input as {
    "action": "period_lock",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"},
    "request_time_ns": 1700003600000000000,
    "break_glass": {
      "enabled": true,
      "ticket_id": "BG-2026-0003",
      "reason": "Emergency close support",
      "approved_by": "",
      "audit_ref": "AUD-2026-02-21-03",
      "activated_at_ns": 1700000000000000000,
      "expires_at_ns": 1700007200000000000
    }
  }
}

# Denied: break-glass request must be inside active window
test_deny_break_glass_outside_active_window if {
  not allow with input as {
    "action": "period_lock",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"},
    "request_time_ns": 1700008200000000000,
    "break_glass": valid_break_glass
  }
}
