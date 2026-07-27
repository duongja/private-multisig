# Basecamp GUI

The Basecamp integration lives in `basecamp-ui/` as a Logos `ui_qml` module
with a C++ backend. `metadata.json` declares `main =
private_multisig_ui_plugin` and `view = Main.qml`.

## What It Demonstrates

- configurable threshold member setup with shielded-account commitments.
- Proposal creation for a threshold-gated action.
- Member approvals that reveal only commitments/nullifiers in the exported
  evidence view.
- Duplicate approval rejection through proposal-scoped nullifiers.
- Threshold aggregation state before execution.
- Backend execution of the same workflow by calling `private_multisig_cli`
  from the Basecamp backend process.
- Real localnet and hosted-testnet execution from inside Basecamp.
- Threshold-driven execution where the GUI's current `config.json`,
  `proposal.json`, and `aggregate.json` drive the actual on-chain call.

The GUI is a Basecamp surface for the workflow. The backend binding delegates to
the Rust CLI for config/proposal/approval/aggregate/verify/prove, and now also
delegates to the repository evidence paths for real localnet and hosted-testnet
execution from inside Basecamp.

## Build And Load

From this repository:

```bash
python3 scripts/basecamp-ui-smoke.py
cargo build -p private_multisig_cli
./scripts/basecamp-backend-flow-smoke.sh
cd basecamp-ui
nix build .#lgx -L
```

From the repository root, the same build with a stable output link is:

```bash
nix build ./basecamp-ui#lgx -L --out-link .local/basecamp-ui-lgx
python3 scripts/inspect-basecamp-ui-lgx.py
```

Then install and launch through scaffold's managed Basecamp profile:

```bash
cd ..
export PRIVATE_MULTISIG_CLI="$PWD/target/debug/private_multisig_cli"
logos-scaffold basecamp setup
logos-scaffold basecamp install
logos-scaffold basecamp launch alice
```

The `scaffold.toml` module entry points scaffold at `path:./basecamp-ui#lgx`.
The latest package evidence is documented in
`docs/basecamp-gui-evidence-20260626.md`.

## AppImage Basecamp Install

For the installed `LogosBasecamp` AppImage profile, use the portable package:

```bash
./scripts/install-basecamp-appimage-ui.sh
```

This installs into:

```text
$HOME/.local/share/Logos/LogosBasecamp/plugins/private_multisig_ui
```

The helper verifies that Basecamp's package scanner resolves:

```text
mainFilePath = .../private_multisig_ui_plugin.so
```

If `mainFilePath` is empty, Basecamp loads the QML page without its C++
backend and the page reports `Backend unavailable`.

After install, restart Basecamp and open `private_multisig_ui`.

## GUI Test Steps

1. Open `private_multisig_ui` from Basecamp.
2. Click `Health`.
   - Expected: JSON with `"ok": true`, `backend:
     "private_multisig_ui"`, and the path to `private_multisig_cli`.
3. Click `Run Backend Flow`.
   - Expected: JSON with `"ok": true`.
   - The command list should include `generate_alice`, `generate_bob`,
     `generate_carol`, `create_config`, `create_proposal`, `approve_alice`,
     `approve_bob`, `duplicate_alice`, `aggregate`, and `verify`.
   - `duplicate_alice` is expected to fail with a non-zero exit code because it
     proves duplicate approvals are rejected.
4. Use the step-by-step buttons:
   - `Generate`
   - `Create Config`
   - `Create Proposal`
   - `Approve`
   - `Aggregate`
   - `Verify`
   - `Prove`
   - `Execute Localnet`
   - `Execute Testnet`
5. Click `Reset Backend` to clear the GUI backend workspace if you want to run
   the flow again.

## Current Boundary

The Basecamp backend now calls the real CLI for the threshold approval flow and
can trigger the real repository localnet/testnet execution paths from inside
Basecamp using the current GUI workspace artifacts.

That means the current GUI threshold, proposal, and aggregate state now shape
the real execute path. This is enough to prove threshold `1`, threshold `2`,
and threshold `3` hosted-testnet execution from the same UI workflow.

Current limitation: the proposal editor is still tuned to the project's
hosted-compatible target-program template. It is not yet a general advanced
editor for arbitrary target-program/account composition.

A later deeper binding can link the Rust SDK directly into the C++ backend or a
Logos Core module once the preferred custom-module API is stable.
