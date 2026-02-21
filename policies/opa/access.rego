package acctcore.authz

import rego.v1

default allow := false

same_entity if {
  input.subject.legal_entity_id == input.resource.legal_entity_id
}

posting_action if { input.action == "posting" }
period_lock_action if { input.action == "period_lock" }
policy_change_action if { input.action == "policy_change" }

critical_action if posting_action
critical_action if period_lock_action
critical_action if policy_change_action

sod_block if {
  critical_action
  input.subject.role == "finance_operator"
}

allow if {
  same_entity
  not sod_block
}
