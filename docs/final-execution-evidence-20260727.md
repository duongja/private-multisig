# Final Execution Evidence - 2026-07-27 UTC

This note collects the current reviewer-facing execution evidence after the
Basecamp GUI threshold controls were wired through to the real on-chain execute
path.

## What Is Proven

- Basecamp GUI local workflow works end to end.
- Basecamp GUI localnet execution works end to end.
- Hosted Logos Testnet v0.2 execution works end to end.
- The GUI threshold controls are authoritative:
  - a workspace-driven threshold `1` run executes with one approval,
  - a Basecamp GUI threshold `3` run executes with three approvals.

## Basecamp GUI Localnet Run

The Basecamp GUI `Execute Localnet` action completed successfully and wrote:

```text
/home/agate/.local/share/Logos/ui-host/private-multisig-ui/latest/onchain/localnet/localnet-evidence.json
```

Key results:

| Field | Value |
| --- | --- |
| Proposal status | `Executed` |
| Approval count | `2` |
| Target account data | `threshold-approved` |

Included transactions:

| Step | Hash |
| --- | --- |
| create multisig | `00dd98c47dd95888b1ab0ec42e4c45fcfbb1027142fdcd6f566b02dc29b63903` |
| propose | `79c5660e7e16b30697d7c3b6efa74f441cbfc18c7f9c44ccf93c8b5dcecc86ce` |
| execute private | `9fd110549d64335868af8261ef97f5164a66c8a687d7a5e613007ccb534d06a6` |

## Hosted Testnet Workspace-Driven Threshold `1`

The threshold-wired execute path was first verified with a workspace-driven
hosted testnet run using threshold `1`.

Key results:

| Field | Value |
| --- | --- |
| Proposal status | `Executed` |
| Approval count | `1` |
| Target account data | `threshold-approved` |

Included transactions:

| Step | Hash |
| --- | --- |
| create multisig | `c50d8baf8bc84ad8cc7fc47948c02aa834759e411c40ec99ee82a7e5df461a1c` |
| propose | `817baa496677c40ea22e71d538816c571ed2112312b9cc4c6ebe3253c2be8375` |
| execute private | `b746b177988f650855eac2833d66476d1fa1d962b0ffcce849a2d2f959726ce2` |

This run proves the execute path is no longer hardcoded to `2-of-3`.

## Basecamp GUI Hosted Testnet Threshold `3`

The current Basecamp GUI now drives hosted testnet execution with the actual
GUI-created:

- `multisig_id`,
- `config.threshold`,
- `proposal`,
- `aggregate`,
- `approval_count`.

The successful threshold `3` GUI run produced:

```text
/home/agate/.local/share/Logos/ui-host/private-multisig-ui/latest/onchain/testnet/testnet-evidence.json
```

Key GUI-created proposal values:

| Field | Value |
| --- | --- |
| Multisig ID | `970b35e7e28cf7ef7587c20879982dd6970b35e7e28cf7ef7587c20879982dd6` |
| Threshold | `3` |
| Approval count | `3` |
| Aggregate hash | `6e4d4cb1b81ea90f563cb1561391d783b3e209b1a2df34b1325a9ee07d1cd6e2` |

Final hosted state:

| Field | Value |
| --- | --- |
| Multisig state account | `2jLaArKwddShVMhpf4PB35HaxsjELfJiV918uyst1nmk` |
| Proposal account | `GLnStJW6retTaCRPLANEQxsNW4sCchYT7gnWyHrNRN1C` |
| Target account | `2rBYf24CttbAvEiv3DdQ2aZzeguYoU4uw1RFugnwc5qK` |
| Proposal status | `Executed` |
| Approval count | `3` |
| Executed aggregate hash | `6e4d4cb1b81ea90f563cb1561391d783b3e209b1a2df34b1325a9ee07d1cd6e2` |
| Target account data | `threshold-approved` |

Included transactions:

| Step | Hash |
| --- | --- |
| create multisig | `017e413e31af3c883f122da9b65f4fd6a2483422db91290a1a952d780b2a4e49` |
| propose | `2ef2dac04126d9e24dcb375f6c0acb16072da2fe6ee57c4f2c88ab7baa746072` |
| execute private | `1a614c701def0ca6b25d30a7611340c7f45062a579ecf575c065a5eb4dbb7790` |

## Meaning For Reviewers

The current repository now demonstrates three separate properties:

1. the private threshold approval system works locally,
2. the LEZ v0.2 execute path works on hosted testnet,
3. the Basecamp GUI controls are not cosmetic; they now shape the actual
   proposal that gets executed on-chain.
