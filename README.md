# LP-0002 Private Multisig

Prize implementation workspace for LP-0002: a private M-of-N multisig
primitive for LEZ.

## Overview

This repository contains a working LP-0002 implementation:

- hosted Logos Testnet v0.2 execution works end to end,
- Basecamp GUI execution works end to end,
- the GUI threshold controls drive the real on-chain execution path,
- reviewer evidence and reproducible scripts are included in the repository.

The current repository is published at
`https://github.com/duongja/private-multisig`.

Narrated demo video:
`https://drive.google.com/file/d/1vHEHCbz9aWfdcXFQjfUJpXHGlw9heY_u/view?usp=sharing`

Start with these reviewer-facing artifacts:

- [docs/final-execution-evidence-20260727.md](docs/final-execution-evidence-20260727.md)
- [docs/prize-readiness.md](docs/prize-readiness.md)
- [docs/basecamp-gui.md](docs/basecamp-gui.md)
- [docs/cost-evidence-20260727.md](docs/cost-evidence-20260727.md)

The implementation includes the privacy-critical core:

- member commitments for shielded LEZ accounts,
- deterministic Merkle membership roots,
- proposal-scoped nullifiers,
- aggregate threshold approval payloads,
- RISC0 aggregate threshold proof generation,
- v0.2 `lee_core` program handlers for create/propose/execute-private,
- raw LEZ v0.2 RISC0 program guest for private multisig execution,
- reusable Rust SDK facade for Logos modules,
- local CLI tooling for test vectors and proposal approval artifacts,
- a Logos Basecamp QML UI package with a C++ backend that calls the real CLI
  for member/proposal/approval aggregation.

Official hosted-testnet CU values remain dependent on what the current testnet
RPC and explorer expose; the current cost evidence documents that limitation
directly.

## Submission Facts

- Repository:
  `https://github.com/duongja/private-multisig`
- Demo video:
  `https://drive.google.com/file/d/1vHEHCbz9aWfdcXFQjfUJpXHGlw9heY_u/view?usp=sharing`
- Current reviewed commit:
  `87f8d864bbb65223312b509ce33a7a69f9161b4a`
- Example hosted testnet `execute_private` transaction:
  `e1805cae3ab6e46239cc823cc45d79e45a78ba80e57168e8fc50e240b9e70b59`

## Reviewer Quick Path

Minimal reviewer flow:

```bash
./scripts/preflight.sh
```

Full local evidence:

```bash
export RISC0_DEV_MODE=0
./scripts/preflight.sh --with-localnet
```

Current evidence index:

- [docs/final-execution-evidence-20260727.md](docs/final-execution-evidence-20260727.md)
- [docs/prize-readiness.md](docs/prize-readiness.md)
- [docs/basecamp-gui.md](docs/basecamp-gui.md)
- [docs/cost-evidence-20260727.md](docs/cost-evidence-20260727.md)
- [docs/testnet-v0.2-check-20260727.md](docs/testnet-v0.2-check-20260727.md)
- narrated demo:
  `https://drive.google.com/file/d/1vHEHCbz9aWfdcXFQjfUJpXHGlw9heY_u/view?usp=sharing`

## Build

Run the standard non-localnet preflight:

```bash
./scripts/preflight.sh
```

The preflight includes SPEL IDL freshness and validation:

```bash
python3 scripts/generate-spel-idl.py --check
python3 scripts/validate-idl.py
python3 scripts/basecamp-ui-smoke.py
```

Run the full localnet preflight:

```bash
export RISC0_DEV_MODE=0
./scripts/preflight.sh --with-localnet
```

Individual development commands:

```bash
cargo test
cargo run -p private_multisig_cli -- --help
cargo run -p private_multisig_cli --features prove -- prove --help
```

Run the first local 2-of-3 threshold smoke:

```bash
export RISC0_DEV_MODE=0
./scripts/local-threshold-smoke.sh
cat .local/smoke/latest/aggregate.json
cat .local/smoke/latest/verify.json
cat .local/smoke/latest/proof/proof-stats.json
```

The smoke creates three local member credentials, builds a 2-of-3 multisig
config, creates a proposal, writes two approval shares, aggregates them, and
proves the aggregate threshold statement with RISC0.

Run the resumable approval smoke:

```bash
./scripts/resumable-approval-smoke.sh
cat .local/resumable-approval/latest/resumable-approval-summary.json
```

This writes one approval, proves that one approval is below threshold, adds a
second approval in a later command, verifies the aggregate, and proves duplicate
approval reuse is rejected by the nullifier check.

A captured successful run is summarized in
[docs/resumable-approval-evidence-20260626.md](docs/resumable-approval-evidence-20260626.md).

The workspace tests also execute the raw LEZ program guest locally with the
RISC0 executor:

```bash
cargo test -p private_multisig_methods
```

That covers `create_multisig`, `propose`, and `execute_private` through the
same `ProgramInput`/`ProgramOutput` path used by LEZ v0.2 programs.

Run the local LEZ v0.2 evidence script:

```bash
export RISC0_DEV_MODE=0
./scripts/localnet-evidence.sh
cat .local/localnet-evidence/latest/localnet-evidence.json
```

The script builds `sequencer_service` with `--features standalone`, starts a
local sequencer through `logos-scaffold`, deploys this private multisig program,
deploys a threshold-gated target program, creates a 2-of-3 multisig, creates a
proposal, and executes the proposal after private threshold approval. The
current target program default is LEZ's shipped `hello_world.bin`, which writes
the configured greeting into one claimed target account. A captured successful
run is summarized in
[docs/localnet-evidence-20260625.md](docs/localnet-evidence-20260625.md).

Run the hosted LEZ v0.2 testnet evidence script:

```bash
export RISC0_DEV_MODE=0
./scripts/testnet-evidence.sh .local/testnet-evidence/latest
cat .local/testnet-evidence/latest/testnet-evidence.json
```

The script connects to `https://testnet.lez.logos.co/`, deploys the private
multisig program, deploys a threshold-gated target program, creates a multisig,
creates a proposal, and executes the proposal after private threshold approval.
On current hosted testnet, the private program deployment transaction may remain
`included=false` within the polling window even when the actual
`create_multisig`, `propose`, and `execute_private` flow succeeds and the final
proposal/account state is valid. The runner now treats final executed state as
the authoritative success criterion and keeps the deploy inclusion bit as
evidence rather than a hard failure. The current hosted v0.2 reruns and
GUI-backed threshold runs are summarized in
[docs/final-execution-evidence-20260727.md](docs/final-execution-evidence-20260727.md)
and [docs/testnet-v0.2-check-20260727.md](docs/testnet-v0.2-check-20260727.md).

Summarize proof-cycle and transaction cost evidence:

```bash
./scripts/summarize-cost-evidence.py --out-dir .local/cost-evidence/latest
cat .local/cost-evidence/latest/cost-summary.md
```

The current cost/CU position is summarized in
[docs/cost-evidence-20260727.md](docs/cost-evidence-20260727.md).

Build and inspect the Basecamp UI package:

```bash
python3 scripts/basecamp-ui-smoke.py
cargo build -p private_multisig_cli
./scripts/basecamp-backend-flow-smoke.sh
nix build ./basecamp-ui#lgx -L --out-link .local/basecamp-ui-lgx
python3 scripts/inspect-basecamp-ui-lgx.py
```

The current GUI execution behavior is summarized in
[docs/basecamp-gui.md](docs/basecamp-gui.md) and
[docs/final-execution-evidence-20260727.md](docs/final-execution-evidence-20260727.md).

## Design

See [docs/cryptographic-design.md](docs/cryptographic-design.md).
See [docs/lez-program-scaffold.md](docs/lez-program-scaffold.md).
See [docs/sdk.md](docs/sdk.md).
See [docs/basecamp-gui.md](docs/basecamp-gui.md).
See [docs/prize-readiness.md](docs/prize-readiness.md).
