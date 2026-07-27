#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_ROOT="${1:-$ROOT/.local/testnet-evidence/latest}"
SEQUENCER="${SEQUENCER:-https://testnet.lez.logos.co/}"
LEZ_REPO="${LEZ_REPO:-/home/agate/Projects/logos/logos-execution-zone-v0.2.0-testnet}"
TARGET_PROGRAM_BINARY="${TARGET_PROGRAM_BINARY:-$LEZ_REPO/target/riscv32im-risc0-zkvm-elf/docker/hello_world.bin}"
POLL_SECONDS="${POLL_SECONDS:-180}"

mkdir -p "$RUN_ROOT"

cd "$ROOT"

export RISC0_DEV_MODE="${RISC0_DEV_MODE:-0}"
export BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/include/x86_64-linux-gnu -I/usr/include}"
RUNNER="${PRIVATE_MULTISIG_RUNNER:-$ROOT/target/debug/private_multisig_runner}"
WORKSPACE="${PRIVATE_MULTISIG_GUI_WORKSPACE:-}"

runner_args=(
  localnet-evidence
  --sequencer "$SEQUENCER"
  --out-dir "$RUN_ROOT"
  --target-program-binary "$TARGET_PROGRAM_BINARY"
  --poll-seconds "$POLL_SECONDS"
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

cp "$RUN_ROOT/localnet-evidence.json" "$RUN_ROOT/testnet-evidence.json"

cat <<JSON
{
  "ok": true,
  "sequencer": "$SEQUENCER",
  "run_root": "$RUN_ROOT",
  "evidence": "$RUN_ROOT/testnet-evidence.json"
}
JSON
