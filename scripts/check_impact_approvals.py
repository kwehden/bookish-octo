#!/usr/bin/env python3
import json
import sys
from pathlib import Path

path = Path("governance/impact_decisions.json")
if not path.exists():
    print("missing governance/impact_decisions.json", file=sys.stderr)
    sys.exit(1)

payload = json.loads(path.read_text())
failures = []

for decision in payload.get("decisions", []):
    fid = decision.get("id", "unknown")
    if decision.get("financial_impact") and not decision.get("finance_approved"):
        failures.append(f"{fid}: finance approval missing")
    if decision.get("control_impact") and not decision.get("controls_approved"):
        failures.append(f"{fid}: controls approval missing")

if failures:
    print("impact approval check failed:")
    for failure in failures:
        print(f"- {failure}")
    sys.exit(1)

print("impact approval check passed")
