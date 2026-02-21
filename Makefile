.PHONY: gates test policy schema approvals freeze

gates: schema freeze approvals policy test

schema:
	python3 scripts/validate_canonical_schema.py

freeze:
	python3 scripts/check_contract_freeze.py

approvals:
	python3 scripts/check_impact_approvals.py

policy:
	opa test policies/opa

test:
	cargo test --workspace
