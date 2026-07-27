# Basecamp GUI Evidence - 2026-06-26

This evidence covers the LP-0002 Basecamp GUI package.

## Commands

```bash
python3 scripts/basecamp-ui-smoke.py
cargo build -p private_multisig_cli
./scripts/basecamp-backend-flow-smoke.sh
nix flake show ./basecamp-ui --no-write-lock-file
nix build ./basecamp-ui#lgx -L --out-link .local/basecamp-ui-lgx
python3 scripts/inspect-basecamp-ui-lgx.py
```

## Result

The Basecamp UI is packaged as a Logos `ui_qml` module with a C++ backend:

| Field | Value |
| --- | --- |
| Module | `private_multisig_ui` |
| Type | `ui_qml` |
| View | `Main.qml` |
| Backend main | `private_multisig_ui_plugin.so` |
| Manifest version | `0.2.0` |
| Variant | `linux-amd64-dev` |
| Dependencies | none |

The built LGX artifact was:

```text
.local/basecamp-ui-lgx/logos-private_multisig_ui-module.lgx
```

Local build output resolved to:

```text
/nix/store/bnyjqfl8bhbs390i87jvxz0dqxqw1z7r-logos-private_multisig_ui-module-lgx-0.1.0/logos-private_multisig_ui-module.lgx
```

Artifact metadata:

| Field | Value |
| --- | --- |
| Bytes | `647465` |
| SHA-256 | `f0c6913071d63268a83c15384e6d861def10c6de50f928a522212a87765540b2` |

Package contents:

```text
manifest.json
variants/linux-amd64-dev/Main.qml
variants/linux-amd64-dev/metadata.json
variants/linux-amd64-dev/private_multisig_ui_plugin.so
variants/linux-amd64-dev/private_multisig_ui_replica_factory.so
```

The manifest reports:

```json
{
  "name": "private_multisig_ui",
  "type": "ui_qml",
  "view": "Main.qml",
  "manifestVersion": "0.2.0",
  "dependencies": [],
  "main": {
    "linux-amd64-dev": "private_multisig_ui_plugin.so"
  }
}
```

The backend exposes a Qt Remote Objects interface with:

```text
health()
runDemoFlow(threshold, proposalId, targetProgramId, instructionWords, targetAccountCount)
resetWorkspace()
```

The backend delegates the workflow to `private_multisig_cli`. Launch Basecamp
with `PRIVATE_MULTISIG_CLI=/path/to/private_multisig_cli` when the CLI is not
available from the backend process working directory.

## AppImage Install Evidence

The installed Basecamp AppImage profile requires the portable package variant:

```bash
./scripts/install-basecamp-appimage-ui.sh
```

The helper builds:

```bash
nix build ./basecamp-ui#lgx-portable -L --out-link .local/basecamp-ui-lgx-portable
```

It installs the flattened UI plugin into:

```text
/home/agate/.local/share/Logos/LogosBasecamp/plugins/private_multisig_ui
```

and verifies the package-manager scanner resolves the backend:

```json
{
  "mainFilePath": "/home/agate/.local/share/Logos/LogosBasecamp/plugins/private_multisig_ui/private_multisig_ui_plugin.so"
}
```

This verification matters because if `mainFilePath` is empty, Basecamp treats
the page as QML-only and the GUI reports `Backend unavailable`.

`logos-scaffold basecamp doctor --json` currently reports the module row as
passing and one non-blocking drift warning. The warning compares scaffold's
auto-discovered absolute flake ref to the portable relative ref in
`scaffold.toml`:

```text
discovered `path:/home/agate/Projects/logos/lp-0002-private-multisig/basecamp-ui#lgx`
but not captured in basecamp.state
```

The project keeps the relative `path:./basecamp-ui#lgx` entry so the repository
remains portable when cloned elsewhere.
