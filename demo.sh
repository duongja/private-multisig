#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export RISC0_DEV_MODE="${RISC0_DEV_MODE:-0}"

cd "$ROOT"

./scripts/preflight.sh --with-localnet
