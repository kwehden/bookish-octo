package acctcore.authz

import rego.v1

valid_break_glass := {
  "enabled": true,
  "ticket_id": "BG-2026-0001",
  "reason": "Emergency close support",
  "approved_by": "CS-L",
  "audit_ref": "AUD-2026-02-21-01",
  "log_entry_id": "BGLOG-2026-02-21-01",
  "approved_at_ns": 1700000100000000000,
  "logged_at_ns": 1700000200000000000,
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

# Allowed: approver can execute estimate change in same entity
test_allow_estimate_change_approver if {
  allow with input as {
    "action": "estimate_change",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"}
  }
}

# Denied: finance operator cannot perform estimate changes
test_deny_sod_estimate_change if {
  not allow with input as {
    "action": "estimate_change",
    "subject": {"role": "finance_operator", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"}
  }
}

# Allowed: approver can execute dispute approval in same entity
test_allow_dispute_approval_approver if {
  allow with input as {
    "action": "dispute_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"}
  }
}

# Denied: finance operator cannot perform dispute approvals
test_deny_sod_dispute_approval if {
  not allow with input as {
    "action": "dispute_approval",
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

# Denied: break-glass cannot bypass SoD on estimate changes
test_deny_break_glass_on_estimate_change if {
  not allow with input as {
    "action": "estimate_change",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"},
    "request_time_ns": 1700003600000000000,
    "break_glass": valid_break_glass
  }
}

# Denied: break-glass cannot bypass SoD on dispute approvals
test_deny_break_glass_on_dispute_approval if {
  not allow with input as {
    "action": "dispute_approval",
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
      "log_entry_id": "BGLOG-2026-02-21-02",
      "approved_at_ns": 1700000100000000000,
      "logged_at_ns": 1700000200000000000,
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
      "log_entry_id": "BGLOG-2026-02-21-03",
      "approved_at_ns": 1700000100000000000,
      "logged_at_ns": 1700000200000000000,
      "activated_at_ns": 1700000000000000000,
      "expires_at_ns": 1700007200000000000
    }
  }
}

# Denied: break-glass requires immutable log entry id
test_deny_break_glass_missing_log_entry_id if {
  not allow with input as {
    "action": "period_lock",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"},
    "request_time_ns": 1700003600000000000,
    "break_glass": {
      "enabled": true,
      "ticket_id": "BG-2026-0004",
      "reason": "Emergency close support",
      "approved_by": "CS-L",
      "audit_ref": "AUD-2026-02-21-04",
      "log_entry_id": "",
      "approved_at_ns": 1700000100000000000,
      "logged_at_ns": 1700000200000000000,
      "activated_at_ns": 1700000000000000000,
      "expires_at_ns": 1700007200000000000
    }
  }
}

# Denied: break-glass log timestamps must be valid and not exceed request time
test_deny_break_glass_invalid_log_timestamps if {
  not allow with input as {
    "action": "period_lock",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {"legal_entity_id": "US_CO_01"},
    "request_time_ns": 1700003600000000000,
    "break_glass": {
      "enabled": true,
      "ticket_id": "BG-2026-0005",
      "reason": "Emergency close support",
      "approved_by": "CS-L",
      "audit_ref": "AUD-2026-02-21-05",
      "log_entry_id": "BGLOG-2026-02-21-05",
      "approved_at_ns": 1700000100000000000,
      "logged_at_ns": 1700005600000000000,
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

# Allowed: posting approval requires explicit batch/ticket metadata in same entity
test_allow_posting_approval_with_required_fields if {
  allow with input as {
    "action": "posting_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {
      "legal_entity_id": "US_CO_01",
      "posting_batch_id": "PB-2026-02-21-001",
      "approval_ticket_id": "APRV-2026-02-21-001"
    }
  }
}

# Denied: posting approval missing required metadata
test_deny_posting_approval_missing_fields if {
  not allow with input as {
    "action": "posting_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {
      "legal_entity_id": "US_CO_01",
      "posting_batch_id": "PB-2026-02-21-002",
      "approval_ticket_id": ""
    }
  }
}

# Allowed: intercompany posting approval permits scoped multi-entity pair
test_allow_intercompany_posting_approval if {
  allow with input as {
    "action": "intercompany_posting_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {
      "from_legal_entity_id": "US_CO_01",
      "to_legal_entity_id": "CA_BC_01",
      "intercompany_contract_id": "IC-CNTR-2026-0007",
      "journal_batch_id": "IC-JRN-2026-0020"
    }
  }
}

# Denied: intercompany posting approval must include non-empty counterparty pair + contract metadata
test_deny_intercompany_posting_approval_missing_contract if {
  not allow with input as {
    "action": "intercompany_posting_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {
      "from_legal_entity_id": "US_CO_01",
      "to_legal_entity_id": "CA_BC_01",
      "intercompany_contract_id": "",
      "journal_batch_id": "IC-JRN-2026-0021"
    }
  }
}

# Denied: intercompany posting approval requires distinct entity pair
test_deny_intercompany_posting_approval_same_entity_pair if {
  not allow with input as {
    "action": "intercompany_posting_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {
      "from_legal_entity_id": "US_CO_01",
      "to_legal_entity_id": "US_CO_01",
      "intercompany_contract_id": "IC-CNTR-2026-0008",
      "journal_batch_id": "IC-JRN-2026-0022"
    }
  }
}

# Denied: approver must belong to one entity in the intercompany pair
test_deny_intercompany_posting_approval_subject_outside_pair if {
  not allow with input as {
    "action": "intercompany_posting_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_NY_01"},
    "resource": {
      "from_legal_entity_id": "US_CO_01",
      "to_legal_entity_id": "CA_BC_01",
      "intercompany_contract_id": "IC-CNTR-2026-0009",
      "journal_batch_id": "IC-JRN-2026-0023"
    }
  }
}

# Denied: finance operator cannot perform intercompany posting approval
test_deny_intercompany_posting_approval_finance_operator if {
  not allow with input as {
    "action": "intercompany_posting_approval",
    "subject": {"role": "finance_operator", "legal_entity_id": "US_CO_01"},
    "resource": {
      "from_legal_entity_id": "US_CO_01",
      "to_legal_entity_id": "CA_BC_01",
      "intercompany_contract_id": "IC-CNTR-2026-0010",
      "journal_batch_id": "IC-JRN-2026-0024"
    }
  }
}

# Denied: break-glass cannot bypass SoD on intercompany posting approval
test_deny_break_glass_on_intercompany_posting_approval if {
  not allow with input as {
    "action": "intercompany_posting_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {
      "from_legal_entity_id": "US_CO_01",
      "to_legal_entity_id": "CA_BC_01",
      "intercompany_contract_id": "IC-CNTR-2026-0011",
      "journal_batch_id": "IC-JRN-2026-0025"
    },
    "request_time_ns": 1700003600000000000,
    "break_glass": valid_break_glass
  }
}

# Allowed: close approval in same entity with close identifiers
test_allow_close_approval_same_entity if {
  allow with input as {
    "action": "close_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "CA_BC_01"},
    "resource": {
      "legal_entity_id": "CA_BC_01",
      "close_period_id": "2026-01",
      "close_checklist_id": "CHK-CA-2026-01"
    }
  }
}

# Denied: close approval requires close period + checklist metadata
test_deny_close_approval_missing_fields if {
  not allow with input as {
    "action": "close_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "CA_BC_01"},
    "resource": {
      "legal_entity_id": "CA_BC_01",
      "close_period_id": "2026-01",
      "close_checklist_id": ""
    }
  }
}

# Denied: finance operator cannot perform close approvals
test_deny_close_approval_finance_operator if {
  not allow with input as {
    "action": "close_approval",
    "subject": {"role": "finance_operator", "legal_entity_id": "CA_BC_01"},
    "resource": {
      "legal_entity_id": "CA_BC_01",
      "close_period_id": "2026-01",
      "close_checklist_id": "CHK-CA-2026-01"
    }
  }
}

# Denied: break-glass cannot bypass SoD on close approvals
test_deny_break_glass_on_close_approval if {
  not allow with input as {
    "action": "close_approval",
    "subject": {"role": "finance_approver", "legal_entity_id": "CA_BC_01"},
    "resource": {
      "legal_entity_id": "CA_BC_01",
      "close_period_id": "2026-01",
      "close_checklist_id": "CHK-CA-2026-01"
    },
    "request_time_ns": 1700003600000000000,
    "break_glass": valid_break_glass
  }
}

# Allowed: master-data change is allowed for approver in-scope with full evidence
test_allow_master_data_change_approver if {
  allow with input as {
    "action": "master_data_change",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {
      "legal_entity_id": "US_CO_01",
      "master_data_domain": "chart_of_accounts",
      "change_set_id": "MD-CHGSET-2026-0001",
      "change_request_id": "MD-CR-2026-0011"
    }
  }
}

# Denied: finance operator cannot perform master-data changes
test_deny_master_data_change_finance_operator if {
  not allow with input as {
    "action": "master_data_change",
    "subject": {"role": "finance_operator", "legal_entity_id": "US_CO_01"},
    "resource": {
      "legal_entity_id": "US_CO_01",
      "master_data_domain": "entity_master",
      "change_set_id": "MD-CHGSET-2026-0002",
      "change_request_id": "MD-CR-2026-0012"
    }
  }
}

# Denied: break-glass cannot bypass SoD on master-data changes
test_deny_break_glass_on_master_data_change if {
  not allow with input as {
    "action": "master_data_change",
    "subject": {"role": "finance_approver", "legal_entity_id": "US_CO_01"},
    "resource": {
      "legal_entity_id": "US_CO_01",
      "master_data_domain": "entity_master",
      "change_set_id": "MD-CHGSET-2026-0003",
      "change_request_id": "MD-CR-2026-0013"
    },
    "request_time_ns": 1700003600000000000,
    "break_glass": valid_break_glass
  }
}
