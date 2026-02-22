#!/usr/bin/env bash
set -euo pipefail

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "Missing required artifact: $path" >&2
    exit 1
  fi
}

require_heading() {
  local path="$1"
  local heading="$2"
  if ! grep -q "$heading" "$path"; then
    echo "Missing required heading '$heading' in $path" >&2
    exit 1
  fi
}

python3 scripts/validate_canonical_schema.py
python3 scripts/check_contract_freeze.py
python3 scripts/validate_sprint5_controls.py
python3 scripts/check_impact_approvals.py

require_file "governance/sprint2_signoff_packet.md"
require_file "governance/sprint3_signoff_packet.md"
require_file "governance/sprint4_signoff_packet.md"
require_file "governance/sprint5_signoff_packet.md"
require_file "governance/SPRINT3_PLAN_COMPLETION_REPORT.md"
require_file "governance/SPRINT4_PLAN_COMPLETION_REPORT.md"
require_file "governance/SPRINT5_PLAN_COMPLETION_REPORT.md"
require_file "qa/SPRINT5_GATE_REPORT.md"
require_file "controls/TAMPER_SEALING_VERIFICATION_V1.md"
require_file "controls/ACCESS_REVIEW_REPORTING_V1.md"
require_file "controls/PCI_SCOPE_CONTROL_OWNERSHIP_MATRIX_V1.md"
require_heading "governance/architect_updates.md" "## Update 3"
require_heading "governance/architect_updates.md" "## Update 4"
require_heading "governance/architect_updates.md" "## Update 5"
require_heading "governance/architect_updates.md" "## Update 7"
require_heading "finance/US_CA_FINANCE_EVIDENCE_CHECKLIST_V1.md" "## Sprint 2"
require_heading "finance/US_CA_FINANCE_EVIDENCE_CHECKLIST_V1.md" "## Sprint 3 Checklist"
require_heading "finance/US_CA_FINANCE_EVIDENCE_CHECKLIST_V1.md" "## Sprint 4 Checklist"
require_heading "finance/US_CA_FINANCE_EVIDENCE_CHECKLIST_V1.md" "## Sprint 5 Checklist"
require_heading "controls/CONTROL_GATES_REGISTER_V1.md" "## Sprint 2 Status"
require_heading "controls/CONTROL_GATES_REGISTER_V1.md" "## Sprint 3 Status"
require_heading "controls/CONTROL_GATES_REGISTER_V1.md" "## Sprint 4 Status"
require_heading "controls/CONTROL_GATES_REGISTER_V1.md" "## Sprint 5 Status"
require_heading "governance/SPRINT3_PLAN_COMPLETION_REPORT.md" "## Quantitative Completion Rubric"
require_heading "governance/SPRINT4_PLAN_COMPLETION_REPORT.md" "## Quantitative Completion Rubric"
require_heading "governance/SPRINT5_PLAN_COMPLETION_REPORT.md" "## Quantitative Completion Rubric"

opa test policies/opa
cargo test --workspace
