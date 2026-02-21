package acctcore.authz

import rego.v1

default allow := false
default max_break_glass_ttl_ns := 14400000000000

same_entity if {
  input.subject.legal_entity_id == input.resource.legal_entity_id
}

posting_action if { input.action == "posting" }
period_lock_action if { input.action == "period_lock" }
policy_change_action if { input.action == "policy_change" }
mapping_change_action if { input.action == "mapping_change" }
ruleset_change_action if { input.action == "ruleset_change" }

non_bypassable_sod_action if posting_action
non_bypassable_sod_action if mapping_change_action
non_bypassable_sod_action if ruleset_change_action

critical_action if non_bypassable_sod_action
critical_action if period_lock_action
critical_action if policy_change_action

sod_block if {
  critical_action
  input.subject.role == "finance_operator"
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
  ticket_id != ""
  reason != ""
  approved_by != ""
  audit_ref != ""
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
  same_entity
  not sod_block
  not break_glass_block
  not break_glass_sod_bypass_block
}
