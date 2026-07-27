#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEZ_REPO="${LEZ_REPO:-/home/agate/Projects/logos/logos-execution-zone-v0.2.0-testnet}"
LEZ_REMOTE="${LEZ_REMOTE:-https://github.com/logos-blockchain/logos-execution-zone.git}"
LEZ_PIN="${LEZ_PIN:-a58fbce2ff48c58b7bb5001b1a27e64b9596ee3a}"

if git -C "$LEZ_REPO" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  :
elif [ ! -e "$LEZ_REPO" ] || [ -z "$(find "$LEZ_REPO" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]; then
  mkdir -p "$(dirname "$LEZ_REPO")"
  git clone "$LEZ_REMOTE" "$LEZ_REPO"
else
  echo "LEZ_REPO exists but is not a git checkout: $LEZ_REPO" >&2
  exit 1
fi

current_head="$(git -C "$LEZ_REPO" rev-parse HEAD)"
if [ "$current_head" != "$LEZ_PIN" ]; then
  git -C "$LEZ_REPO" fetch --depth 1 origin "$LEZ_PIN" >/dev/null 2>&1 || git -C "$LEZ_REPO" fetch origin "$LEZ_PIN"
  git -C "$LEZ_REPO" checkout --quiet --detach "$LEZ_PIN"
fi

# The v0.2 repository nests LEZ services under `lez/`.
# Scaffold still looks for a flat layout, so create compatibility symlinks.
for name in sequencer wallet storage common mempool configs indexer explorer_service testnet_initial_state keycard_wallet wallet-ffi; do
  if [ ! -e "$LEZ_REPO/$name" ] && [ -e "$LEZ_REPO/lez/$name" ]; then
    ln -s "lez/$name" "$LEZ_REPO/$name"
  fi
done

cat <<JSON
{
  "ok": true,
  "root": "$ROOT",
  "lez_repo": "$LEZ_REPO",
  "lez_pin": "$LEZ_PIN"
}
JSON
