#!/usr/bin/env bash
set -euo pipefail

python3 scripts/validate_canonical_schema.py
python3 scripts/check_contract_freeze.py
python3 scripts/check_impact_approvals.py
opa test policies/opa
cargo test --workspace
