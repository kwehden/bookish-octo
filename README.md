# AcctCore

Open-source accounting core for federated, multi-entity operations across multichannel businesses (online retail, in-person POS, ticketing, reservations, discounts, and promotions).

Current pilot focus:
- Industry: ski resorts
- Countries: United States and Canada
- Finance basis: dual-book (`US_GAAP` + `IFRS`)
- Finance UI adapters: ERPNext first, Odoo-compatible contract
- Payment partner (initial): Stripe, with pluggable gateway design

## Scope and Goals

AcctCore is designed to provide:
- Canonical event ingestion from operational systems
- Deterministic, idempotent journal posting
- Immutable auditability and controls-first operation
- Reconciliation pipelines with reason-coded exception routing
- Clear scaling gates up to a comfortable 2,000 active users

## Architecture (Current Baseline)

- Language/runtime: Rust workspace with modular crates
- Event/contract focus: canonical schema + contract freeze gates
- Posting domain:
  - idempotency enforcement
  - posting rule engine v1
  - period-open checks
  - journal reversal endpoint
- Integration domain:
  - connector SDK
  - Inntopia normalization adapter
- Data/reconciliation domain:
  - settlement ingest (`Stripe` CSV, bank CSV v0)
  - deterministic exception reason-code routing
- Controls/security domain:
  - OPA policy pack with non-bypassable SoD checks
  - break-glass TTL and audit metadata constraints

## Repository Layout

- `crates/` Rust workspace crates (`posting-api`, `ledger-posting`, `connector-sdk`, `reconciliation-model`, `settlement-ingest`, `platform-core`)
- `contracts/` canonical schema and recon/exception contracts
- `policies/opa/` authorization and SoD policy tests
- `finance/` policy packages, COA dimensions, evidence checklists
- `controls/` control gate register and access model artifacts
- `specs/` master architecture/specification documents and sprint plans
- `qa/` sprint gate reports
- `perf/` load/performance harnesses (`k6`)
- `governance/` signoff packets, architect updates, impact records
- `scripts/` contract and quality gate scripts

## Local Setup (Ubuntu)

Required:
- `rustc` and `cargo` (Rust toolchain)
- `python3`
- `opa`

Recommended:
- `k6` (performance gates)
- `gh` (PR workflow)

## Build and Test

From repository root:

```bash
cargo fmt --all
cargo test --workspace
opa test policies/opa
./scripts/run_contract_gates.sh
```

## Run Posting API

```bash
cargo run -p posting-api
```

Default endpoint:
- `POST /v1/posting/events`
- `POST /v1/ledger/journals/:journal_id/reverse`
- `GET /v1/ops/slo`
- `GET /v1/ops/capacity`

## Specs and Planning Entry Points

- `specs/OPEN_SOURCE_ACCOUNTING_SYSTEM_MASTER_SPEC.md`
- `specs/ARCHITECTURAL_GUIDANCE.md`
- `specs/TECHNICAL_IMPLEMENTATION_SPEC_BY_EPIC.md`
- `specs/IMPLEMENTATION_PLAN_12_WEEKS.md`
- `specs/SPRINT1_SQUAD_AGENT_EXECUTION_PLAN.md`
- `specs/SPRINT2_SQUAD_AGENT_EXECUTION_PLAN.md`

## Contributor Onboarding

1. Read the architecture and sprint plan specs above.
2. Run local gates and verify a clean baseline.
3. Pick one scoped change on a dedicated branch.
4. Open a new PR for every change batch (no direct pushes to `main`).
5. Attach evidence (tests/gates/decision impacts) in PR description.

## License

MIT. See `LICENSE`.
