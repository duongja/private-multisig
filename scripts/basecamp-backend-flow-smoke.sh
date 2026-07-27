#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_ROOT="${1:-$ROOT/.local/basecamp-backend-flow-smoke/latest}"
CLI="${PRIVATE_MULTISIG_CLI:-$ROOT/target/debug/private_multisig_cli}"
MULTISIG_ID="1111111111111111111111111111111111111111111111111111111111111111"

if [ ! -x "$CLI" ]; then
  cargo build -p private_multisig_cli >/dev/null
fi

rm -rf "$RUN_ROOT"
mkdir -p "$RUN_ROOT"

"$CLI" generate-member --multisig-id "$MULTISIG_ID" --out "$RUN_ROOT/alice.json" >"$RUN_ROOT/generate-alice.json"
"$CLI" generate-member --multisig-id "$MULTISIG_ID" --out "$RUN_ROOT/bob.json" >"$RUN_ROOT/generate-bob.json"
"$CLI" generate-member --multisig-id "$MULTISIG_ID" --out "$RUN_ROOT/carol.json" >"$RUN_ROOT/generate-carol.json"

"$CLI" create-config \
  --multisig-id "$MULTISIG_ID" \
  --threshold 2 \
  --member "$RUN_ROOT/alice.json" \
  --member "$RUN_ROOT/bob.json" \
  --member "$RUN_ROOT/carol.json" \
  --out "$RUN_ROOT/config.json" \
  >"$RUN_ROOT/create-config.json"

"$CLI" create-proposal \
  --multisig-id "$MULTISIG_ID" \
  --proposal-id 1 \
  --target-program-id "1,2,3,4,5,6,7,8" \
  --instruction-words "9,10" \
  --target-account-count 1 \
  --out "$RUN_ROOT/proposal.json" \
  >"$RUN_ROOT/create-proposal.json"

"$CLI" approve \
  --member "$RUN_ROOT/alice.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --out "$RUN_ROOT/approval-alice.json" \
  >"$RUN_ROOT/approve-alice.json"

"$CLI" approve \
  --member "$RUN_ROOT/bob.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --out "$RUN_ROOT/approval-bob.json" \
  >"$RUN_ROOT/approve-bob.json"

set +e
"$CLI" aggregate \
  --config "$RUN_ROOT/config.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --member "$RUN_ROOT/alice.json" \
  --member "$RUN_ROOT/bob.json" \
  --member "$RUN_ROOT/carol.json" \
  --approval "$RUN_ROOT/approval-alice.json" \
  --approval "$RUN_ROOT/approval-alice.json" \
  --out "$RUN_ROOT/duplicate-aggregate.json" \
  >"$RUN_ROOT/duplicate-aggregate.stdout" \
  2>"$RUN_ROOT/duplicate-aggregate.stderr"
duplicate_exit=$?
set -e

if [ "$duplicate_exit" -eq 0 ]; then
  echo "duplicate approval was not rejected" >&2
  exit 1
fi

"$CLI" aggregate \
  --config "$RUN_ROOT/config.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --member "$RUN_ROOT/alice.json" \
  --member "$RUN_ROOT/bob.json" \
  --member "$RUN_ROOT/carol.json" \
  --approval "$RUN_ROOT/approval-alice.json" \
  --approval "$RUN_ROOT/approval-bob.json" \
  --out "$RUN_ROOT/aggregate.json" \
  >"$RUN_ROOT/aggregate.stdout.json"

"$CLI" verify \
  --config "$RUN_ROOT/config.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --aggregate "$RUN_ROOT/aggregate.json" \
  >"$RUN_ROOT/verify.json"

python3 - "$RUN_ROOT" "$duplicate_exit" <<'PY'
import json
import sys
from pathlib import Path

run_root = Path(sys.argv[1])
summary = {
    "ok": True,
    "run_root": str(run_root),
    "duplicate_exit": int(sys.argv[2]),
    "aggregate": json.loads((run_root / "aggregate.json").read_text()),
    "verify": json.loads((run_root / "verify.json").read_text()),
}
(run_root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
PY
