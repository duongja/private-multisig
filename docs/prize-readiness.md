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
- Hosted LEZ v0.2 testnet evidence with included transactions for multisig
  creation, proposal creation, and private execution, with deployment
  inclusion recorded separately because the hosted deploy lookup can lag behind
  final state.
- CI workflow and preflight script for reproducible checks.
- Resumable approval smoke proving persisted partial approvals can be resumed
  and duplicate approvals are rejected.
- Basecamp QML + C++ backend GUI package for member setup, proposal creation,
  approval collection, duplicate nullifier rejection, threshold aggregation
  state, CLI-backed verification, evidence preview, and real localnet/testnet
  execution.
- Cost evidence summarizer for RISC0 proof cycles and localnet transaction
  inclusion, following the Logos-recommended `lez-signature-bench` measurement
  style.
- Workspace-driven execute path where Basecamp GUI threshold and proposal
  artifacts now drive the actual hosted testnet execution.

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

The latest reviewer-facing localnet execution summary is documented in
`docs/final-execution-evidence-20260727.md`.

Run:

```bash
export RISC0_DEV_MODE=0
./scripts/testnet-evidence.sh .local/testnet-evidence/latest
```

This connects to hosted LEZ testnet at `https://testnet.lez.logos.co/`,
deploys the private multisig program, deploys a target program, creates a
multisig, creates a proposal, and executes the proposal after private threshold
approval. On the current hosted testnet, the private program deployment lookup
may remain `included=false` within the polling window even when
`create_multisig`, `propose`, and `execute_private` are included and the final
proposal state is `Executed`. The runner now records that deployment bit as
evidence but does not treat it as a false failure when the real workflow and
final state validate.

The latest hosted v0.2 execution summaries are documented in:

- `docs/testnet-v0.2-check-20260727.md`
- `docs/final-execution-evidence-20260727.md`

Run:

```bash
python3 scripts/basecamp-ui-smoke.py
```

This validates the Basecamp `ui_qml` module metadata and the expected QML
workflow markers. The module lives in `basecamp-ui/`; see
`docs/basecamp-gui.md`.

The current GUI-backed execution evidence is summarized in
`docs/final-execution-evidence-20260727.md`.

Run:

```bash
./scripts/summarize-cost-evidence.py --out-dir .local/cost-evidence/latest
```

This writes proof-cycle and localnet transaction cost evidence. The current
reviewer-facing cost note is documented in `docs/cost-evidence-20260727.md`.

## Pending For Final Prize Submission

- Official hosted-testnet CU values for deployment, multisig creation,
  proposal creation, private execution, and target call once the explorer/RPC
  exposes them. Current proof-cycle evidence is documented.
- Final Basecamp GUI screenshots and recording after building/loading
  `basecamp-ui` in the current recommended Basecamp release.
- Final narrated demo video showing proof generation with `RISC0_DEV_MODE=0`,
  localnet/testnet execution, transaction hashes, threshold `1`, threshold `2`,
  threshold `3`, and the user flow.
- Final reviewer pass over README/evidence docs after any evaluator feedback.

## Current Limitations

- The published IDL is now generated from a thin SPEL wrapper source, but the
  runtime handler crate still executes directly on `lee_core` rather than
  through upstream SPEL runtime macros. That keeps the working v0.2 execution
  path intact while the upstream runtime remains `nssa_core`-oriented.
- The localnet flow uses scaffold's standalone sequencer mode. Hosted v0.2
  testnet evidence is captured separately in
  `docs/final-execution-evidence-20260727.md`.
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
  member/proposal/approval/aggregate/verify/prove flow and calls the repository
  execution path for real LEZ submission. The proposal form is now wired to the
  hosted-compatible execution template, but arbitrary user-defined chained-call
  composition is still not exposed as a generalized advanced editor.
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
- `docs/cost-evidence-20260727.md`
- `docs/final-execution-evidence-20260727.md`
- `.local/localnet-evidence/latest/localnet-evidence.json`
- `.local/testnet-evidence/latest/testnet-evidence.json`
