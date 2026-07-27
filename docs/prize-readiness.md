# Prize Readiness

This document maps the current repository to LP-0002 success criteria.

## Implemented

- Private membership commitment model for shielded LEZ account holders.
- Proposal-scoped nullifier design to prevent double approval on one proposal.
- Aggregate threshold approval payload that does not reveal member identities.
- RISC0 guest proving threshold membership and nullifier uniqueness.
- CLI commands for member generation, config creation, proposal creation,
  approval, aggregation, verification, and proving.
- LEZ v0.2 `lee_core` handlers for `create_multisig`, `propose`, and
  `execute_private`.
- Raw LEZ v0.2 RISC0 program guest for executing those handlers on-chain.
- Reusable Rust SDK facade in `private_multisig_core::sdk` for Logos modules
  and Basecamp helpers.
- Deterministic program error codes for invalid thresholds, bad proposal index,
  invalid aggregate proof, wrong account shape, decode failures, and replayed
  proposal status.
- Local standalone LEZ v0.2 evidence flow with `RISC0_DEV_MODE=0`.
- Hosted LEZ v0.2 testnet evidence with included transactions for program
  deployment, multisig creation, proposal creation, and private execution.
- CI workflow and preflight script for reproducible checks.
- Resumable approval smoke proving persisted partial approvals can be resumed
  and duplicate approvals are rejected.
- Basecamp QML + C++ backend GUI package for member setup, proposal creation,
  approval collection, duplicate nullifier rejection, threshold aggregation
  state, CLI-backed verification, and evidence preview.
- Cost evidence summarizer for RISC0 proof cycles and localnet transaction
  inclusion, following the Logos-recommended `lez-signature-bench` measurement
  style.

## Proven Locally

Run:

```bash
./scripts/preflight.sh
```

This checks shell scripts, formats this repository's Rust sources, runs
core/program tests, checks CLI/runner crates, and executes RISC0 method tests.

Run:

```bash
./scripts/resumable-approval-smoke.sh
```

This writes a single approval to disk, confirms it is below threshold, adds a
second approval later, verifies the aggregate, and confirms duplicate approval
reuse fails through the nullifier check.

The latest captured successful run is documented in
`docs/resumable-approval-evidence-20260626.md`.

Run:

```bash
export RISC0_DEV_MODE=0
./scripts/local-threshold-smoke.sh
```

This creates a local 2-of-3 multisig, generates two approvals, aggregates them,
and produces a RISC0 proof receipt for the aggregate threshold statement.

Run:

```bash
export RISC0_DEV_MODE=0
./scripts/localnet-evidence.sh
```

This starts a standalone LEZ v0.2 sequencer, deploys the private multisig
program, deploys a target program, creates a multisig, creates a proposal, and
executes the target action after private threshold approval.

The latest captured successful localnet run is documented in
`docs/localnet-evidence-20260625.md`.

Run:

```bash
export RISC0_DEV_MODE=0
./scripts/testnet-evidence.sh .local/testnet-evidence/latest
```

This connects to hosted LEZ testnet at `https://testnet.lez.logos.co/`,
deploys the private multisig program, deploys a target program, creates a
multisig, creates a proposal, and executes the proposal after private threshold
approval.

The latest captured successful hosted testnet run is documented in
`docs/testnet-evidence-20260626.md`.

Run:

```bash
python3 scripts/basecamp-ui-smoke.py
```

This validates the Basecamp `ui_qml` module metadata and the expected QML
workflow markers. The module lives in `basecamp-ui/`; see
`docs/basecamp-gui.md`.

Run:

```bash
./scripts/summarize-cost-evidence.py --out-dir .local/cost-evidence/latest
```

This writes proof-cycle and localnet transaction cost evidence. The captured
summary is documented in `docs/cost-evidence-20260626.md`.

## Pending For Final Prize Submission

- Official hosted-testnet CU values for deployment, multisig creation,
  proposal creation, private execution, and target call once the explorer/RPC
  exposes them. Current proof-cycle evidence is documented.
- v0.2-compatible SPEL-generated IDL or evaluator-approved replacement for the
  current manual SPEL-shaped IDL.
- Final Basecamp GUI recording after building/loading `basecamp-ui` in the
  current recommended Basecamp release.
- Final narrated demo video showing proof generation with `RISC0_DEV_MODE=0`,
  localnet/testnet execution, transaction hashes, and the user flow.

## Current Limitations

- The current IDL is manual because the available local SPEL checkout targets an
  older `nssa_core` API while LEZ v0.2 programs use `lee_core`. It is validated
  by `scripts/validate-idl.py` until a v0.2-compatible SPEL generator is
  available.
- The localnet flow uses scaffold's standalone sequencer mode. Hosted v0.2
  testnet evidence is captured separately in
  `docs/testnet-evidence-20260626.md`.
- The proof circuit currently proves aggregate threshold membership and
  proposal-scoped nullifier uniqueness. It does not hide proposal content; that
  is explicitly out of scope for LP-0002.
- The current default reference action uses LEZ `hello_world.bin` as the
  threshold-gated target program. It claims one public account and writes the
  configured greeting into that account. A final demo can still switch to a more
  meaningful treasury transfer or parameter-change target if the testnet exposes
  stable supporting programs.
- `private_multisig_core::sdk` is a Rust integration surface, not a separate
  package registry release yet. A downstream Logos module can depend on the
  workspace crate directly.
- `basecamp-ui` calls the real Rust CLI from its C++ backend for the
  member/proposal/approval/aggregate/verify flow. RISC0 proof generation and
  LEZ submission remain in the Rust CLI/SDK/localnet scripts until the deeper
  custom module binding is stabilized.
- The current sequencer RPC exposed to this project exposes transaction lookup
  but not a receipt field with official per-transaction CU. The cost evidence
  records measured RISC0 proof cycles and leaves official CU explicitly pending.

## Reviewer Commands

From a clean checkout with Rust, RISC0 tooling, and Logos circuits available:

```bash
./scripts/preflight.sh
```

For full local evidence:

```bash
export RISC0_DEV_MODE=0
./scripts/preflight.sh --with-localnet
```

Useful output files:

- `.local/smoke/latest/proof/proof-stats.json`
- `.local/smoke/latest/proof/journal.json`
- `.local/resumable-approval/latest/resumable-approval-summary.json`
- `docs/resumable-approval-evidence-20260626.md`
- `docs/sdk.md`
- `docs/cost-evidence-20260626.md`
- `.local/localnet-evidence/latest/localnet-evidence.json`
- `.local/testnet-evidence/latest/testnet-evidence.json`
