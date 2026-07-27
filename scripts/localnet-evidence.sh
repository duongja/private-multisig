#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_ROOT="${1:-$ROOT/.local/localnet-evidence/latest}"
SCAFFOLD="${SCAFFOLD:-/home/agate/Projects/logos/scaffold/target/release/logos-scaffold}"
LEZ_REPO="${LEZ_REPO:-/home/agate/Projects/logos/logos-execution-zone-v0.2.0-testnet}"
TARGET_PROGRAM_BINARY="${TARGET_PROGRAM_BINARY:-$LEZ_REPO/target/riscv32im-risc0-zkvm-elf/docker/hello_world.bin}"
LOCALNET_TIMEOUT_SEC="${LOCALNET_TIMEOUT_SEC:-90}"

mkdir -p "$RUN_ROOT"

cd "$ROOT"

export RISC0_DEV_MODE="${RISC0_DEV_MODE:-0}"
export BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/include/x86_64-linux-gnu -I/usr/include}"
RUNNER="${PRIVATE_MULTISIG_RUNNER:-$ROOT/target/debug/private_multisig_runner}"
WORKSPACE="${PRIVATE_MULTISIG_GUI_WORKSPACE:-}"

for name in sequencer wallet storage common mempool configs indexer explorer_service testnet_initial_state keycard_wallet wallet-ffi; do
  if [ ! -e "$LEZ_REPO/$name" ] && [ -e "$LEZ_REPO/lez/$name" ]; then
    ln -s "lez/$name" "$LEZ_REPO/$name"
  fi
done

cargo build --release --manifest-path "$LEZ_REPO/Cargo.toml" --features standalone -p sequencer_service

"$SCAFFOLD" localnet start --timeout-sec "$LOCALNET_TIMEOUT_SEC" > "$RUN_ROOT/localnet-start.log"

if [ "${KEEP_LOCALNET:-0}" != "1" ]; then
  trap '"$SCAFFOLD" localnet stop >/dev/null 2>&1 || true' EXIT
fi

runner_args=(
  localnet-evidence
  --out-dir "$RUN_ROOT"
  --target-program-binary "$TARGET_PROGRAM_BINARY"
  --poll-seconds 45
)

if [ -n "$WORKSPACE" ] && [ -f "$WORKSPACE/config.json" ] && [ -f "$WORKSPACE/proposal.json" ] && [ -f "$WORKSPACE/aggregate.json" ]; then
  runner_args+=(
    --config "$WORKSPACE/config.json"
    --proposal "$WORKSPACE/proposal.json"
    --aggregate "$WORKSPACE/aggregate.json"
  )
fi

if [ -x "$RUNNER" ]; then
  "$RUNNER" "${runner_args[@]}" 2>&1 | tee "$RUN_ROOT/runner-output.log"
else
  cargo run -q -p private_multisig_runner -- "${runner_args[@]}" 2>&1 | tee "$RUN_ROOT/runner-output.log"
fi

cat <<JSON
{
  "ok": true,
  "run_root": "$RUN_ROOT",
  "evidence": "$RUN_ROOT/localnet-evidence.json"
}
JSON
