# PCI Scope and Control Ownership Matrix v1

Date: `2026-02-22`  
Scope: Sprint 5 PCI scope/control ownership baseline for Stripe-first integration

Status: PASS

## Ownership Matrix

| Control Area | Control ID | Owner | Approver | Evidence Ref |
|---|---|---|---|---|
| Scope Definition | PCI-SCOPE-001 | `CS-L` | `ARCH-L` | `EVID-PCI-0001` |
| Payment Flow Segmentation | PCI-NET-002 | `IN-L` | `CS-L` | `EVID-PCI-0002` |
| Key Access Review | PCI-IAM-003 | `CS-L` | `FI-L` | `EVID-PCI-0003` |
| Log Retention and Integrity | PCI-LOG-004 | `PL-L` | `CS-L` | `EVID-PCI-0004` |

## Validation Signals
- `ownership_signoff_complete: true`
- `required_action`: `pci_scope_update`
- OPA required fields:
  - `pci_scope_version`
  - `control_matrix_id`
  - `owner_approval_id`

## Governance Linkage
- `governance/impact_decisions.json` (`DEC-007`)
- `governance/sprint5_signoff_packet.md`
