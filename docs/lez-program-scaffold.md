# LEZ Program Scaffold

The current LEZ layer is implemented in `crates/private_multisig_program`.

It contains three handler-level state transitions against the LEZ v0.2
`lee_core` account/program types:

- `create_multisig`: initializes the private multisig state account with the
  threshold, member count, and private member Merkle root.
- `propose`: creates a public proposal account and records the target chained
  call metadata.
- `execute_private`: verifies an aggregate threshold approval against the
  stored multisig state and proposal, marks the proposal executed, and emits one
  LEZ chained call.

The handlers are tested directly with `cargo test -p private_multisig_program`.
Those tests cover state initialization, proposal creation, threshold aggregate
verification, proposal execution, PDA seed forwarding, target account forwarding,
and authorized target account marking.

The raw RISC0 LEZ guest is implemented at
`methods/guest/src/bin/private_multisig.rs`. It reads
`PrivateMultisigInstruction` via `lee_core::program::read_lee_inputs`, dispatches
to the handler crate, and commits a `ProgramOutput`. The method crate exposes
the generated `PRIVATE_MULTISIG_ELF` and `PRIVATE_MULTISIG_ID` constants.

`cargo test -p private_multisig_methods` executes this guest ELF locally with
the RISC0 executor. The tests currently verify:

- `create_multisig` initializes a PDA-claimed multisig state account;
- `propose` creates a PDA-claimed proposal account;
- `execute_private` verifies the aggregate threshold approval, marks the
  proposal executed, and emits the configured chained call;
- only the configured target account indices are marked authorized in the
  chained call pre-state.

## Deterministic Errors

The program crate exposes `ProgramError::code()` for deterministic failure
reporting. Current codes:

| Code | Error | Meaning |
| ---: | --- | --- |
| 2000 | `InvalidAccountCount` | Instruction received the wrong number of accounts. |
| 2001 | `AlreadyInitialized` | An init account was already initialized. |
| 2002 | `InvalidThreshold` | Threshold is zero or greater than member count. |
| 2003 | `DecodeState` | Multisig state account could not be decoded. |
| 2004 | `DecodeProposal` | Proposal account could not be decoded. |
| 2005 | `CreateKeyMismatch` | Instruction create key does not match stored state. |
| 2006 | `ProposalIndexMismatch` | Proposal index is stale or does not match the proposal. |
| 2007 | `ProposalNotActive` | Proposal was already executed or cancelled. |
| 2008 | `TargetAccountCountMismatch` | Provided target accounts do not match proposal metadata. |
| 2009 | `InvalidAggregateProof` | Aggregate threshold approval failed verification, including duplicate nullifiers. |
| 2010 | `AccountDataTooLarge` | Serialized account state exceeds LEZ account data limits. |

The test suite currently covers codes `2002`, `2006`, and `2009` explicitly.

## SPEL/IDL Status

The runtime LEZ layer still targets the v0.2 `lee_core` account/program model
directly, but the repository now generates its published IDL from a thin
SPEL-annotated wrapper source:

- wrapper source: `spel/private_multisig_idl.rs`
- exporter: `tools/spel_idl_exporter`
- checked artifact: `docs/private-multisig-idl.json`

The split is intentional. The current upstream `spel-framework` runtime still
targets the older `nssa_core` API, while this program must use
`lee_core::program::read_lee_inputs` and the v0.2 `ProgramOutput` shape.
Keeping the handler crate independent from the runtime macros avoids regressing
the working v0.2 program path while still satisfying the SPEL IDL requirement.

The generated IDL is refreshed by:

```bash
python3 scripts/generate-spel-idl.py
```

And validated by:

```bash
python3 scripts/generate-spel-idl.py --check
python3 scripts/validate-idl.py
```

This checks instruction names, argument types, account order, PDA seeds,
execution flags, discriminators, account/defined type names, and deterministic
program error codes.

This split keeps the proof guest and program handler crates buildable today:

```bash
cargo test
```

## Localnet Evidence

`scripts/localnet-evidence.sh` runs the program against a real standalone LEZ
v0.2 sequencer with `RISC0_DEV_MODE=0`.

The standalone build flag is required:

```bash
cargo build --release --manifest-path "$LEZ_REPO/Cargo.toml" --features standalone -p sequencer_service
```

Without `--features standalone`, the sequencer uses the real zone-sdk
publisher and waits for a Bedrock node on `localhost:18080`. That is correct for
the integrated network stack, but not for the local prize evidence flow. The
standalone feature switches the sequencer to the mock publisher used by local
scaffold development.

The evidence script deploys two programs:

- the private multisig LEZ program generated from
  `methods/guest/src/bin/private_multisig.rs`;
- a target `hello_world.bin` program from the LEZ checkout, used as the
  threshold-gated action against one claimed public account.

It then submits and confirms these transactions:

- private multisig program deployment;
- target program deployment;
- multisig state creation;
- proposal creation;
- private threshold execution.

A successful captured run is documented in
`docs/localnet-evidence-20260625.md`.
