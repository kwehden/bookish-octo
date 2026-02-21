#!/usr/bin/env python3
import json
import sys
from pathlib import Path

required_fields = {
    "event_id",
    "event_type",
    "schema_version",
    "source_system",
    "source_event_id",
    "occurred_at",
    "business_date",
    "tenant_id",
    "legal_entity_id",
    "idempotency_key",
    "payload",
}
required_event_types = {"order.captured.v1", "payment.settled.v1", "refund.v1"}

path = Path("contracts/canonical_event_v1.schema.json")
if not path.exists():
    print("canonical schema missing", file=sys.stderr)
    sys.exit(1)

schema = json.loads(path.read_text())
required = set(schema.get("required", []))
missing = required_fields - required
if missing:
    print(f"schema required fields missing: {sorted(missing)}", file=sys.stderr)
    sys.exit(1)

enum_values = set(schema.get("properties", {}).get("event_type", {}).get("enum", []))
if enum_values != required_event_types:
    print(
        f"event_type enum mismatch: expected {sorted(required_event_types)}, got {sorted(enum_values)}",
        file=sys.stderr,
    )
    sys.exit(1)

print("canonical schema check passed")
