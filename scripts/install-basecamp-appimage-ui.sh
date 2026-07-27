#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASECAMP_DATA_DIR="${BASECAMP_DATA_DIR:-$HOME/.local/share/Logos/LogosBasecamp}"
PLUGINS_DIR="${PLUGINS_DIR:-$BASECAMP_DATA_DIR/plugins}"
MODULE_NAME="private_multisig_ui"
LGX_LINK="${LGX_LINK:-$ROOT/.local/basecamp-ui-lgx-portable}"
INSPECT_DIR="${INSPECT_DIR:-$ROOT/.local/basecamp-ui-appimage-install/extracted}"
PLUGIN_DIR="$PLUGINS_DIR/$MODULE_NAME"
LGX="$LGX_LINK/logos-private_multisig_ui-module.lgx"

cd "$ROOT"

if [ ! -x "$ROOT/target/debug/private_multisig_cli" ]; then
  cargo build -p private_multisig_cli
fi

nix build ./basecamp-ui#lgx-portable -L --out-link "$LGX_LINK"

rm -rf "$INSPECT_DIR"
mkdir -p "$INSPECT_DIR"
tar -xzf "$LGX" -C "$INSPECT_DIR"

if [ ! -f "$INSPECT_DIR/variants/linux-amd64/private_multisig_ui_plugin.so" ]; then
  echo "portable LGX is missing private_multisig_ui_plugin.so" >&2
  exit 1
fi
if [ ! -f "$INSPECT_DIR/variants/linux-amd64/private_multisig_ui_replica_factory.so" ]; then
  echo "portable LGX is missing private_multisig_ui_replica_factory.so" >&2
  exit 1
fi

mkdir -p "$PLUGINS_DIR"
if [ -d "$PLUGIN_DIR" ]; then
  backup="$ROOT/.local/basecamp-ui-appimage-install/backups/$MODULE_NAME-$(date -u +%Y%m%dT%H%M%SZ)"
  mkdir -p "$(dirname "$backup")"
  cp -a "$PLUGIN_DIR" "$backup"
  echo "backup=$backup"
fi

rm -rf "$PLUGIN_DIR"
mkdir -p "$PLUGIN_DIR"
cp "$INSPECT_DIR/manifest.json" "$PLUGIN_DIR/manifest.json"
cp "$INSPECT_DIR/variants/linux-amd64/"* "$PLUGIN_DIR/"
printf 'linux-amd64' > "$PLUGIN_DIR/variant"
chmod +x "$PLUGIN_DIR"/*.so "$PLUGIN_DIR"/lib*.so.* 2>/dev/null || true

python3 - <<'PY' "$PLUGIN_DIR/manifest.json"
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
manifest = json.loads(path.read_text())
main = manifest.get("main")
if not isinstance(main, dict):
    main = {}
for variant in ("linux-amd64", "linux-x86_64", "linux-amd64-dev", "linux-x86_64-dev"):
    main[variant] = "private_multisig_ui_plugin.so"
manifest["main"] = main
path.write_text(json.dumps(manifest, indent=2) + "\n")
PY

lgpm="${LGPM:-}"
if [ -z "$lgpm" ] || [ ! -x "$lgpm" ]; then
  lgpm="$(command -v lgpm 2>/dev/null || true)"
fi
if { [ -z "$lgpm" ] || [ ! -x "$lgpm" ]; } && [ "${FIND_LGPM_IN_NIX_STORE:-0}" = "1" ]; then
  lgpm="$(find /nix/store -path '*/bin/lgpm' -type f 2>/dev/null | head -n 1 || true)"
fi

main_file=""
if [ -n "$lgpm" ] && [ -x "$lgpm" ]; then
  main_file="$("$lgpm" \
    --modules-dir "$BASECAMP_DATA_DIR/modules" \
    --ui-plugins-dir "$PLUGINS_DIR" \
    --json info "$MODULE_NAME" \
    2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin).get("mainFilePath",""))' || true)"
fi

if [ -z "$main_file" ]; then
  main_file="$PLUGIN_DIR/private_multisig_ui_plugin.so"
fi

if [ ! -f "$main_file" ]; then
  echo "backend mainFilePath did not resolve to an existing file: $main_file" >&2
  exit 1
fi

cat <<JSON
{
  "ok": true,
  "module": "$MODULE_NAME",
  "plugin_dir": "$PLUGIN_DIR",
  "lgx": "$LGX",
  "cli": "$ROOT/target/debug/private_multisig_cli",
  "mainFilePath": "$main_file",
  "next": "Restart Basecamp, open private_multisig_ui, click Health, then Run Backend Flow."
}
JSON
