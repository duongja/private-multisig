#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WITH_LOCALNET=0

for arg in "$@"; do
  case "$arg" in
    --with-localnet)
      WITH_LOCALNET=1
      ;;
    --help|-h)
      cat <<'EOF'
Usage: scripts/preflight.sh [--with-localnet]

Runs LP-0002 reproducibility checks.

Default checks:
  - ensure the pinned LEZ v0.2.0 checkout is present;
  - shell syntax check;
  - manual IDL validation;
  - Basecamp QML module shape validation;
  - Basecamp backend CLI flow smoke;
  - rustfmt check for this repository's Rust sources;
  - core/program unit tests;
  - resumable approval smoke;
  - runner and CLI cargo checks;
  - RISC0 method guest tests.

Optional:
  --with-localnet also runs scripts/localnet-evidence.sh with RISC0_DEV_MODE=0.
EOF
      exit 0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

cd "$ROOT"

export RISC0_DEV_MODE="${RISC0_DEV_MODE:-0}"
export BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/include/x86_64-linux-gnu -I/usr/include}"

echo "== Ensure pinned LEZ v0.2.0 checkout =="
"$ROOT/scripts/ensure-lez-v0.2.sh"

echo "== Shell syntax =="
bash -n scripts/*.sh

echo "== IDL validation =="
python3 scripts/validate-idl.py
python3 -m py_compile scripts/validate-idl.py scripts/summarize-cost-evidence.py scripts/basecamp-ui-smoke.py scripts/inspect-basecamp-ui-lgx.py

echo "== Basecamp UI smoke =="
python3 scripts/basecamp-ui-smoke.py
"$ROOT/scripts/basecamp-backend-flow-smoke.sh" "$ROOT/.local/basecamp-backend-flow-smoke/preflight"

echo "== Rust formatting =="
mapfile -t rust_sources < <(
  find crates methods \
    -path '*/target' -prune -o \
    -name '*.rs' -print | sort
)
rustfmt --edition 2021 --check "${rust_sources[@]}"

echo "== Core and program tests =="
cargo test -p private_multisig_core -p private_multisig_program

echo "== Resumable approval smoke =="
"$ROOT/scripts/resumable-approval-smoke.sh" "$ROOT/.local/resumable-approval/preflight"

echo "== CLI and runner checks =="
cargo check -p private_multisig_cli
cargo check -p private_multisig_cli --features prove
cargo check -p private_multisig_runner

echo "== RISC0 method tests =="
cargo test -p private_multisig_methods

if [ "$WITH_LOCALNET" = "1" ]; then
  echo "== Localnet evidence =="
  "$ROOT/scripts/localnet-evidence.sh" "$ROOT/.local/localnet-evidence/preflight"
fi

echo "preflight completed"
