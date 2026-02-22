#!/usr/bin/env python3
import sys
from pathlib import Path

required_paths = [
    "contracts/canonical_event_v1.schema.json",
    "contracts/reconciliation_model_v0.json",
    "contracts/exception_taxonomy_v0.json",
    "finance/POSTING_RULE_TEMPLATE_PACK_V1.md",
    "finance/DUAL_BOOK_POLICY_PACKAGE_V1.md",
    "controls/ACCESS_MODEL_V0.md",
    "controls/CONTROL_GATES_REGISTER_V1.md",
    "controls/TAMPER_SEALING_VERIFICATION_V1.md",
    "controls/ACCESS_REVIEW_REPORTING_V1.md",
    "controls/PCI_SCOPE_CONTROL_OWNERSHIP_MATRIX_V1.md",
    "policies/opa/access.rego",
    "policies/opa/access_test.rego",
    "governance/impact_decisions.json",
    "specs/SPRINT5_RECON_EVIDENCE_EXPORT.md",
]

missing = [p for p in required_paths if not Path(p).exists()]
if missing:
    print("contract freeze check failed; missing files:", file=sys.stderr)
    for p in missing:
        print(f"- {p}", file=sys.stderr)
    sys.exit(1)

print("contract freeze check passed")
