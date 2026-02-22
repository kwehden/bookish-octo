#!/usr/bin/env python3
import sys
from pathlib import Path


REQUIRED_SNIPPETS = {
    "controls/TAMPER_SEALING_VERIFICATION_V1.md": [
        "Status: PASS",
        "seal_chain_verified: true",
    ],
    "controls/ACCESS_REVIEW_REPORTING_V1.md": [
        "Status: PASS",
        "review_attestation_complete: true",
    ],
    "controls/PCI_SCOPE_CONTROL_OWNERSHIP_MATRIX_V1.md": [
        "Status: PASS",
        "ownership_signoff_complete: true",
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
    for snippet in snippets:
        if snippet not in content:
            fail(f"missing required snippet '{snippet}' in {path}")

print("sprint5 controls evidence check passed")
