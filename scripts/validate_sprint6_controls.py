#!/usr/bin/env python3
import sys
from pathlib import Path


REQUIRED_SNIPPETS = {
    "controls/RELEASE_CONTROL_CHECKLIST_V1.md": [
        "Status: PASS",
        "## Sprint 6 Release Control Signoff",
        "## Pass/Fail Gate Results",
        "control_gate_result: PASS",
    ],
    "controls/INCIDENT_RESPONSE_DRILL_REPORT_V1.md": [
        "Status: PASS",
        "## Drill Scenario",
        "## Pass/Fail Outcome",
        "drill_gate_result: PASS",
    ],
    "controls/CONTROL_GATES_REGISTER_V1.md": [
        "## Sprint 6 Status",
        "release_control_signoff",
    ],
}


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


for path, snippets in REQUIRED_SNIPPETS.items():
    file_path = Path(path)
    if not file_path.exists():
        fail(f"missing required control artifact: {path}")
    content = file_path.read_text()
    if "Status: FAIL" in content:
        fail(f"explicit FAIL status detected in {path}")
    for snippet in snippets:
        if snippet not in content:
            fail(f"missing required snippet '{snippet}' in {path}")

print("sprint6 controls evidence check passed")
