# LP-0002 Basecamp UI

This is a Logos Basecamp `ui_qml` module with a C++ backend for the LP-0002
private multisig member workflow.

It provides a reviewer-facing workflow for:

- creating a 2-of-3 shielded member configuration;
- creating a proposal;
- producing member approvals;
- rejecting duplicate approvals through proposal-scoped nullifiers;
- aggregating once the threshold is met;
- exporting a JSON evidence preview.

The backend calls the real `private_multisig_cli` through `QProcess`. Set
`PRIVATE_MULTISIG_CLI=/path/to/private_multisig_cli` before launching Basecamp,
or build the CLI at the repository default path:

```bash
cargo build -p private_multisig_cli
```

The LEZ submission paths still live in the Rust CLI/SDK and localnet scripts.

Build the LGX package:

```bash
cd basecamp-ui
nix build .#lgx -L
```

Install through scaffold from the repository root:

```bash
logos-scaffold basecamp setup
logos-scaffold basecamp install
logos-scaffold basecamp launch alice
```

The root `scaffold.toml` registers this module as `private_multisig_ui`.
