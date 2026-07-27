#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_ROOT="${1:-$ROOT/.local/resumable-approval/latest}"
BIN=(cargo run -q -p private_multisig_cli --)

rm -rf "$RUN_ROOT"
mkdir -p "$RUN_ROOT"

MULTISIG_ID="2222222222222222222222222222222222222222222222222222222222222222"
PROGRAM_ID="8,7,6,5,4,3,2,1"

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
  --instruction-words "700,800" \
  --target-account-count 1 \
  --out "$RUN_ROOT/proposal.json" >/dev/null

"${BIN[@]}" approve \
  --member "$RUN_ROOT/member-a.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --out "$RUN_ROOT/approval-a.json" >/dev/null

set +e
RUST_BACKTRACE=0 RUST_LIB_BACKTRACE=0 "${BIN[@]}" aggregate \
  --config "$RUN_ROOT/config.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --member "$RUN_ROOT/member-a.json" \
  --member "$RUN_ROOT/member-b.json" \
  --member "$RUN_ROOT/member-c.json" \
  --approval "$RUN_ROOT/approval-a.json" \
  --out "$RUN_ROOT/aggregate-one-approval.json" \
  >"$RUN_ROOT/below-threshold.stdout" \
  2>"$RUN_ROOT/below-threshold.stderr"
below_status=$?
set -e
if [ "$below_status" -eq 0 ]; then
  echo "single approval unexpectedly satisfied threshold" >&2
  exit 1
fi
if ! grep -qi "below threshold" "$RUN_ROOT/below-threshold.stderr"; then
  echo "single approval failed for unexpected reason" >&2
  cat "$RUN_ROOT/below-threshold.stderr" >&2
  exit 1
fi

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
  --aggregate "$RUN_ROOT/aggregate.json" >"$RUN_ROOT/verify.json"

set +e
RUST_BACKTRACE=0 RUST_LIB_BACKTRACE=0 "${BIN[@]}" aggregate \
  --config "$RUN_ROOT/config.json" \
  --proposal "$RUN_ROOT/proposal.json" \
  --member "$RUN_ROOT/member-a.json" \
  --member "$RUN_ROOT/member-b.json" \
  --member "$RUN_ROOT/member-c.json" \
  --approval "$RUN_ROOT/approval-a.json" \
  --approval "$RUN_ROOT/approval-a.json" \
  --out "$RUN_ROOT/aggregate-duplicate.json" \
  >"$RUN_ROOT/duplicate-nullifier.stdout" \
  2>"$RUN_ROOT/duplicate-nullifier.stderr"
duplicate_status=$?
set -e
if [ "$duplicate_status" -eq 0 ]; then
  echo "duplicate approval unexpectedly succeeded" >&2
  exit 1
fi
if ! grep -qi "duplicate proposal nullifier" "$RUN_ROOT/duplicate-nullifier.stderr"; then
  echo "duplicate approval failed for unexpected reason" >&2
  cat "$RUN_ROOT/duplicate-nullifier.stderr" >&2
  exit 1
fi

python3 - "$RUN_ROOT" <<'PY'
import json
import sys
from pathlib import Path

run = Path(sys.argv[1])
aggregate = json.loads((run / "aggregate.json").read_text())
verify = json.loads((run / "verify.json").read_text())
summary = {
    "ok": True,
    "run_root": str(run),
    "partial_approval_file": str(run / "approval-a.json"),
    "resumed_approval_file": str(run / "approval-c.json"),
    "single_approval_rejected": True,
    "duplicate_nullifier_rejected": True,
    "threshold_aggregate_verified": verify.get("ok") is True,
    "approval_count": aggregate["approval_count"],
    "aggregate_hash": aggregate["aggregate_hash"],
}
(run / "resumable-approval-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
PY
