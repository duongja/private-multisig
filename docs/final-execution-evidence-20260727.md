# Final Execution Evidence - 2026-07-27 UTC

This note collects the current reviewer-facing execution evidence after the
Basecamp GUI threshold controls were wired through to the real on-chain execute
path.

## What Is Proven

- Basecamp GUI local workflow works end to end.
- Basecamp GUI localnet execution works end to end.
- Hosted Logos Testnet v0.2 execution works end to end.
- Hosted testnet success is now evaluated by the executed on-chain state plus
  included `create_multisig`, `propose`, and `execute_private` transactions.
  The private program deployment transaction may remain `included=false` within
  the polling window on hosted testnet and is recorded as evidence, but it is
  no longer treated as a false negative when the final proposal state is
  `Executed`.
- The GUI threshold controls are authoritative:
  - a workspace-driven threshold `1` run executes with one approval,
  - a Basecamp GUI threshold `2` run executes with two approvals,
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

## Basecamp GUI Hosted Testnet Threshold `2`

The same GUI-driven execute path was then rerun through Basecamp with threshold
`2`.

Key GUI-created proposal values:

| Field | Value |
| --- | --- |
| Multisig ID | `46bbdfa044557e3e02eea19e8b115dde46bbdfa044557e3e02eea19e8b115dde` |
| Threshold | `2` |
| Approval count | `2` |
| Aggregate hash | `f0688a181870c960d85448fb54013c015227511acd98a83b421cbffa7b277609` |

Final hosted state:

| Field | Value |
| --- | --- |
| Multisig state account | `Dub7uBS7UoFmTbenhxEme4Jh5ao7HadMm49Vh2uadrhT` |
| Proposal account | `2aT4ghteG6n2wXCrEzfFLUDbqei2EHiXxSqe1f3hPcVj` |
| Target account | `ArjMdHCb97mVFN7DtCRSyp6bbRsf7iJv4z8WHQkz6Zxo` |
| Proposal status | `Executed` |
| Approval count | `2` |
| Executed aggregate hash | `f0688a181870c960d85448fb54013c015227511acd98a83b421cbffa7b277609` |
| Target account data | `threshold-approved` |

Included transactions:

| Step | Hash |
| --- | --- |
| create multisig | `5fff881389745eaf749615651a710bd3bccb39b72e98c03fdc1dcb275bab6037` |
| propose | `91fc0526249fb27baa7d3743c789f6ab4800c9edea519bbb84a9a0673d91a0ef` |
| execute private | `19eed814a3df541a0bb94f9f0a9dfbae9431c8d8e61e9ffaf3607d6125087b5c` |

## Basecamp GUI Hosted Testnet Threshold `2` After Hosted Deploy Timeout Patch

The hosted testnet runner was patched on July 28, 2026 so a delayed private
program deployment inclusion bit no longer causes a false failure when the
actual multisig flow completes and final state validation succeeds.

Patched Basecamp GUI run:

| Field | Value |
| --- | --- |
| Multisig ID | `ec4d4b381efbeb51f2b6a0690b493689ec4d4b381efbeb51f2b6a0690b493689` |
| Threshold | `2` |
| Approval count | `2` |
| Aggregate hash | `620238ce7ca9f0e05b4fcdb6a5bfb27742bae7ee4542ddc9e17a6875515435ff` |
| Proposal status | `Executed` |
| Target account data | `threshold-approved` |

Transaction evidence:

| Step | Hash | Included |
| --- | --- | --- |
| private multisig deploy | `f19c16b16feb60f33b5558eb044151a72bcf770b0f2c79b74147607fbae498e6` | `false` |
| target deploy | `5236a12978b154703679dc9788006a108edc776d692ca544090d36fa73001a3d` | `true` |
| create multisig | `a9789651b6510c3808fc085300d61ebda808b792f4cd9c0efc39df0ab9967d17` | `true` |
| propose | `04ffd92f5212b87c15e0becdc421b1497cf55ae8b0b6bd81bf0c316793829a9d` | `true` |
| execute private | `0e48a3103cd03b1f29bbc7b917bfc89c725b6e652a53d82a5faac16151d4e40f` | `true` |

This run is intentionally kept because it demonstrates the hosted-testnet edge
case reviewers are most likely to see on their own machines: the deploy lookup
can lag, but the actual threshold-gated proposal execution still succeeds and
the final public state is authoritative.

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
