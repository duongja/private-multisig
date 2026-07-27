# Hosted Testnet v0.2 Check - 2026-07-27 UTC

This note records the LP-0002 retarget, failure analysis, and successful rerun
against the current hosted LEZ v0.2 stack after the June 30, 2026 testnet
rollout.

## Workspace Retarget

The LP-0002 workspace was updated from the stale local checkout path
`logos-execution-zone-v0.2.0-rc5-testnet` to the current local checkout
`logos-execution-zone-v0.2.0-testnet`.

Updated areas:

- `Cargo.toml` path dependencies
- `scaffold.toml`
- `scripts/localnet-evidence.sh`
- `scripts/testnet-evidence.sh`
- supporting docs that still referenced `rc5` and the removed `noop.bin`
  artifact

The evidence runner now targets the shipped hello-world program binary:

`/home/agate/Projects/logos/logos-execution-zone-v0.2.0-testnet/target/riscv32im-risc0-zkvm-elf/docker/hello_world.bin`

## Build Check

The retargeted code builds against the current local v0.2 tree:

```bash
cargo metadata --no-deps --format-version 1
cargo check -p private_multisig_runner
```

## Root Cause Found

The first hosted rerun reached:

- private multisig deployment,
- target program deployment,
- `create_multisig`,
- `propose`,

but the final `execute_private` transaction never appeared on-chain.

The issue was in the LP-0002 runner, not in the general v0.2 compatibility:

- the proposal targeted an arbitrary public account,
- `execute_private` marked that chained-call target as `is_authorized = true`,
- but LEZ v0.2 only accepts chained public authorization when it is backed by:
  - a signer-authorized account inherited from the caller, or
  - a caller-derived PDA delegated through `ChainedCall.pda_seeds`.

In other words, the old runner tried to authorize an arbitrary account in a
public chained call. The v0.2 state machine rejects that model.

## Fix

The runner was updated so the chained target account is a proper public PDA of
the private multisig program:

- derive `target_account = AccountId::for_public_pda(private_multisig_program_id, seed)`
- store the same seed in `proposal.pda_seeds`
- keep `authorized_indices = [0]`

This matches the v0.2 public authorization model enforced by
`validated_state_diff.rs`.

## Hosted Testnet Success Run

Command:

```bash
RISC0_DEV_MODE=0 ./scripts/testnet-evidence.sh .local/testnet-evidence/20260727T141431Z
```

Hosted sequencer:

`https://testnet.lez.logos.co/`

## Result

The fixed runner succeeded end-to-end on hosted v0.2 testnet.

| Step | Hash | Included |
| --- | --- | --- |
| private multisig deploy | `52cdb5dfb95522c3273aca74f45acfd5683fcc478edae18450bd0a5ac8f63a80` | yes |
| target deploy | `5236a12978b154703679dc9788006a108edc776d692ca544090d36fa73001a3d` | yes |
| create multisig | `ad80bd338e07f62da0dfa6bbb0800e4e636ce4a99b7b15fefa16c356fe4823ec` | yes |
| propose | `7e4773107b4447b5f706546c9a1b0a212c517a478c21e0b3e025b2b94abe06b9` | yes |
| execute private | `39c0eb1aec3500911bfeabcf89493af12e74891449398f765651efd058061dfa` | yes |

Final public state from the run:

| Field | Value |
| --- | --- |
| Multisig state account | `5FbNvDQ56qoWsWwUcV7ydTge3AuLWLYTEMyiFuuvrHZU` |
| Proposal account | `Gi2NRz3PsDrZ1QkWFTRM4rZiaXM2JAxzhCSrexGDTcUD` |
| Target PDA account | `9LnAxVCigAhWNKQr7Uowqem6ATDTQGhPEbeeaXdtY8A5` |
| Proposal status | `Executed` |
| Approval count | `2` |
| Target account owner | `0df885a022528320b3652d13af8285e644e408ec1908253d5ede250d8b3d7406` |
| Target account data | `threshold-approved` |

This proves the hosted v0.2 stack now works end-to-end for:

- program deployment,
- multisig creation,
- proposal creation,
- threshold approval aggregation,
- private execution,
- chained public state mutation through a caller-authorized PDA.

## Evidence Paths

- run root:
  `.local/testnet-evidence/20260727T141431Z`
- runner log:
  `.local/testnet-evidence/20260727T141431Z/full.log`
- JSON evidence:
  `.local/testnet-evidence/20260727T141431Z/testnet-evidence.json`
