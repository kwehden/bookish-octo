package acctcore.authz

import rego.v1

default allow := false
default max_break_glass_ttl_ns := 14400000000000

subject_entity_id := object.get(object.get(input, "subject", {}), "legal_entity_id", "")

resource_entity_ids[entity_id] if {
  entity_id := object.get(object.get(input, "resource", {}), "legal_entity_id", "")
  entity_id != ""
}

resource_entity_ids[entity_id] if {
  entities := object.get(object.get(input, "resource", {}), "entity_ids", [])
  some i
  entity_id := entities[i]
  is_string(entity_id)
  entity_id != ""
}

resource_entity_ids[entity_id] if {
  entity_id := object.get(object.get(input, "resource", {}), "from_legal_entity_id", "")
  entity_id != ""
}

resource_entity_ids[entity_id] if {
  entity_id := object.get(object.get(input, "resource", {}), "to_legal_entity_id", "")
  entity_id != ""
}

resource_entity_count := count({entity_id | resource_entity_ids[entity_id]})

subject_entity_valid if {
  subject_entity_id != ""
}

subject_entity_in_scope if {
  subject_entity_valid
  resource_entity_ids[subject_entity_id]
}

same_entity_scope if {
  resource_entity_count == 1
  subject_entity_in_scope
}

posting_action if { input.action == "posting" }
posting_approval_action if { input.action == "posting_approval" }
intercompany_posting_approval_action if { input.action == "intercompany_posting_approval" }
period_lock_action if { input.action == "period_lock" }
close_approval_action if { input.action == "close_approval" }
policy_change_action if { input.action == "policy_change" }
master_data_change_action if { input.action == "master_data_change" }
mapping_change_action if { input.action == "mapping_change" }
ruleset_change_action if { input.action == "ruleset_change" }
estimate_change_action if { input.action == "estimate_change" }
dispute_approval_action if { input.action == "dispute_approval" }

non_bypassable_sod_action if posting_action
non_bypassable_sod_action if posting_approval_action
non_bypassable_sod_action if intercompany_posting_approval_action
non_bypassable_sod_action if close_approval_action
non_bypassable_sod_action if master_data_change_action
non_bypassable_sod_action if mapping_change_action
non_bypassable_sod_action if ruleset_change_action
non_bypassable_sod_action if estimate_change_action
non_bypassable_sod_action if dispute_approval_action

critical_action if non_bypassable_sod_action
critical_action if period_lock_action
critical_action if policy_change_action

sod_block if {
  critical_action
  input.subject.role == "finance_operator"
}

intercompany_posting_pair_valid if {
  resource := object.get(input, "resource", {})
  from_entity := object.get(resource, "from_legal_entity_id", "")
  to_entity := object.get(resource, "to_legal_entity_id", "")
  from_entity != ""
  to_entity != ""
  from_entity != to_entity
}

intercompany_subject_in_pair if {
  subject_entity_valid
  resource := object.get(input, "resource", {})
  subject_entity_id == object.get(resource, "from_legal_entity_id", "")
}

intercompany_subject_in_pair if {
  subject_entity_valid
  resource := object.get(input, "resource", {})
  subject_entity_id == object.get(resource, "to_legal_entity_id", "")
}

entity_scope_allowed if {
  not intercompany_posting_approval_action
  same_entity_scope
}

entity_scope_allowed if {
  intercompany_posting_approval_action
  intercompany_posting_pair_valid
  subject_entity_in_scope
  intercompany_subject_in_pair
}

posting_approval_context_valid if {
  not posting_approval_action
}

posting_approval_context_valid if {
  posting_approval_action
  resource := object.get(input, "resource", {})
  posting_batch_id := object.get(resource, "posting_batch_id", "")
  approval_ticket_id := object.get(resource, "approval_ticket_id", "")
  posting_batch_id != ""
  approval_ticket_id != ""
}

intercompany_posting_approval_context_valid if {
  not intercompany_posting_approval_action
}

intercompany_posting_approval_context_valid if {
  intercompany_posting_approval_action
  intercompany_posting_pair_valid
  resource := object.get(input, "resource", {})
  contract_id := object.get(resource, "intercompany_contract_id", "")
  journal_batch_id := object.get(resource, "journal_batch_id", "")
  contract_id != ""
  journal_batch_id != ""
}

close_approval_context_valid if {
  not close_approval_action
}

close_approval_context_valid if {
  close_approval_action
  resource := object.get(input, "resource", {})
  close_period_id := object.get(resource, "close_period_id", "")
  close_checklist_id := object.get(resource, "close_checklist_id", "")
  close_period_id != ""
  close_checklist_id != ""
}

master_data_change_context_valid if {
  not master_data_change_action
}

master_data_change_context_valid if {
  master_data_change_action
  resource := object.get(input, "resource", {})
  master_data_domain := object.get(resource, "master_data_domain", "")
  change_set_id := object.get(resource, "change_set_id", "")
  change_request_id := object.get(resource, "change_request_id", "")
  master_data_domain != ""
  change_set_id != ""
  change_request_id != ""
}

break_glass_requested if {
  object.get(object.get(input, "break_glass", {}), "enabled", false)
}

break_glass_audit_fields_present if {
  bg := object.get(input, "break_glass", {})
  ticket_id := object.get(bg, "ticket_id", "")
  reason := object.get(bg, "reason", "")
  approved_by := object.get(bg, "approved_by", "")
  audit_ref := object.get(bg, "audit_ref", "")
  log_entry_id := object.get(bg, "log_entry_id", "")
  ticket_id != ""
  reason != ""
  approved_by != ""
  audit_ref != ""
  log_entry_id != ""
}

break_glass_log_timestamps_valid if {
  bg := object.get(input, "break_glass", {})
  request_time_ns := object.get(input, "request_time_ns", -1)
  activated_at_ns := object.get(bg, "activated_at_ns", -1)
  approved_at_ns := object.get(bg, "approved_at_ns", -1)
  logged_at_ns := object.get(bg, "logged_at_ns", -1)
  is_number(request_time_ns)
  is_number(activated_at_ns)
  is_number(approved_at_ns)
  is_number(logged_at_ns)
  approved_at_ns >= activated_at_ns
  logged_at_ns >= activated_at_ns
  approved_at_ns <= request_time_ns
  logged_at_ns <= request_time_ns
}

break_glass_ttl_valid if {
  bg := object.get(input, "break_glass", {})
  activated_at_ns := object.get(bg, "activated_at_ns", -1)
  expires_at_ns := object.get(bg, "expires_at_ns", -1)
  is_number(activated_at_ns)
  is_number(expires_at_ns)
  expires_at_ns > activated_at_ns
  expires_at_ns-activated_at_ns <= max_break_glass_ttl_ns
}

break_glass_active_window if {
  bg := object.get(input, "break_glass", {})
  request_time_ns := object.get(input, "request_time_ns", -1)
  activated_at_ns := object.get(bg, "activated_at_ns", -1)
  expires_at_ns := object.get(bg, "expires_at_ns", -1)
  is_number(request_time_ns)
  is_number(activated_at_ns)
  is_number(expires_at_ns)
  request_time_ns >= activated_at_ns
  request_time_ns <= expires_at_ns
}

break_glass_compliant if {
  break_glass_requested
  break_glass_audit_fields_present
  break_glass_log_timestamps_valid
  break_glass_ttl_valid
  break_glass_active_window
}

break_glass_block if {
  break_glass_requested
  not break_glass_compliant
}

break_glass_sod_bypass_block if {
  break_glass_requested
  non_bypassable_sod_action
}

allow if {
  entity_scope_allowed
  not sod_block
  not break_glass_block
  not break_glass_sod_bypass_block
  posting_approval_context_valid
  intercompany_posting_approval_context_valid
  close_approval_context_valid
  master_data_change_context_valid
}
