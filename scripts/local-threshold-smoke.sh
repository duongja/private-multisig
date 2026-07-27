#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_ROOT="${1:-$ROOT/.local/smoke/latest}"
BIN=(cargo run -q -p private_multisig_cli --)
PROVE_BIN=(cargo run -q -p private_multisig_cli --features prove --)

rm -rf "$RUN_ROOT"
mkdir -p "$RUN_ROOT"

MULTISIG_ID="1111111111111111111111111111111111111111111111111111111111111111"
PROGRAM_ID="1,2,3,4,5,6,7,8"

cd "$ROOT"

"${BIN[@]}" generate-member --multisig-id "$MULTISIG_ID" --out "$RUN_ROOT/member-a.json" >/dev/null
"${BIN[@]}" generate-member --multisig-id "$MULTISIG_ID" --out "$RUN_ROOT/member-b.json" >/dev/null
"${BIN[@]}" generate-member --multisig-id "$MULTISIG_ID" --out "$RUN_ROOT/member-c.json" >/dev/null

"${BIN[@]}" create-config \
  --multisig-id "$MULTISIG_ID" \
  --threshold 2 \
  --member "$RUN_ROOT/member-a.json" \
  --member "$RUN_ROOT/member-b.json" \
  --member "$RUN_ROOT/member-c.json" \
  --out "$RUN_ROOT/config.json" >/dev/null

"${BIN[@]}" create-proposal \
  --multisig-id "$MULTISIG_ID" \
  --proposal-id 1 \
  --target-program-id "$PROGRAM_ID" \
  --instruction-words "42,100" \
  --target-account-count 2 \
  --out "$RUN_ROOT/proposal.json" >/dev/null

"${BIN[@]}" approve \
  --member "$RUN_ROOT/member-a.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --out "$RUN_ROOT/approval-a.json" >/dev/null

"${BIN[@]}" approve \
  --member "$RUN_ROOT/member-c.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --out "$RUN_ROOT/approval-c.json" >/dev/null

"${BIN[@]}" aggregate \
  --config "$RUN_ROOT/config.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --member "$RUN_ROOT/member-a.json" \
  --member "$RUN_ROOT/member-b.json" \
  --member "$RUN_ROOT/member-c.json" \
  --approval "$RUN_ROOT/approval-a.json" \
  --approval "$RUN_ROOT/approval-c.json" \
  --out "$RUN_ROOT/aggregate.json" >/dev/null

"${BIN[@]}" verify \
  --config "$RUN_ROOT/config.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --aggregate "$RUN_ROOT/aggregate.json" > "$RUN_ROOT/verify.json"

"${PROVE_BIN[@]}" prove \
  --config "$RUN_ROOT/config.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --member "$RUN_ROOT/member-a.json" \
  --member "$RUN_ROOT/member-b.json" \
  --member "$RUN_ROOT/member-c.json" \
  --approval "$RUN_ROOT/approval-a.json" \
  --approval "$RUN_ROOT/approval-c.json" \
  --out-dir "$RUN_ROOT/proof" > "$RUN_ROOT/prove.json"

cat <<JSON
{
  "ok": true,
  "run_root": "$RUN_ROOT",
  "config": "$RUN_ROOT/config.json",
  "proposal": "$RUN_ROOT/proposal.json",
  "aggregate": "$RUN_ROOT/aggregate.json",
  "verify": "$RUN_ROOT/verify.json",
  "proof": "$RUN_ROOT/proof/journal.json",
  "proof_stats": "$RUN_ROOT/proof/proof-stats.json"
}
JSON
