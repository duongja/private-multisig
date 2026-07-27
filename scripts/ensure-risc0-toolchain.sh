#!/usr/bin/env bash
set -euo pipefail

RZUP_VERSION="${RZUP_VERSION:-0.5.0}"
RISC0_RUST_VERSION="${RISC0_RUST_VERSION:-1.94.1}"
RISC0_R0VM_VERSION="${RISC0_R0VM_VERSION:-3.0.5}"

strip_ansi() {
  sed 's/\x1b\[[0-9;]*m//g'
}

if ! command -v rzup >/dev/null 2>&1; then
  echo "installing rzup ${RZUP_VERSION}"
  cargo install --locked rzup --version "${RZUP_VERSION}"
fi

installed_rust_version="$(
  rzup show 2>/dev/null | strip_ansi | awk '
    $1 == "rust" { getline; gsub(/^[*[:space:]]+/, "", $0); print $0; exit }
  '
)"

installed_r0vm_version="$(
  rzup show 2>/dev/null | strip_ansi | awk '
    $1 == "r0vm" { getline; gsub(/^[*[:space:]]+/, "", $0); print $0; exit }
  '
)"

if [ "${installed_rust_version}" != "${RISC0_RUST_VERSION}" ]; then
  echo "installing Risc Zero rust ${RISC0_RUST_VERSION}"
  rzup install rust "${RISC0_RUST_VERSION}"
fi

if [ "${installed_r0vm_version}" != "${RISC0_R0VM_VERSION}" ]; then
  echo "installing Risc Zero r0vm ${RISC0_R0VM_VERSION}"
  rzup install r0vm "${RISC0_R0VM_VERSION}"
fi

rzup show | strip_ansi
