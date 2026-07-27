#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


PUBLIC_EXECUTION_CYCLE_LIMIT = 1024 * 1024 * 32


def read_json(path: Path):
    return json.loads(path.read_text())


def find_latest(root: Path, pattern: str) -> Path:
    matches = [path for path in root.glob(pattern) if path.is_file()]
    if not matches:
        raise SystemExit(f"no files match {root / pattern}")
    return max(matches, key=lambda path: path.stat().st_mtime)


def markdown(summary: dict) -> str:
    proof = summary["private_threshold_proof"]
    lines = [
        "# Cost Evidence Summary",
        "",
        f"- Generated from: `{summary['inputs']['proof_stats']}`",
        f"- Localnet evidence: `{summary['inputs']['localnet_evidence']}`",
        f"- `RISC0_DEV_MODE`: `{summary['risc0_dev_mode']}`",
        "",
        "## Private Threshold Proof",
        "",
        "| Field | Value |",
        "| --- | ---: |",
        f"| Total cycles | {proof['total_cycles']} |",
        f"| User cycles | {proof['user_cycles']} |",
        f"| Paging cycles | {proof['paging_cycles']} |",
        f"| Segments | {proof['segments']} |",
        f"| Prove seconds | {proof['prove_seconds']:.2f} |",
        "",
        "## LEZ Transactions",
        "",
        "| Operation | Transaction hash | Included | Official CU |",
        "| --- | --- | --- | --- |",
    ]
    for tx in summary["transactions"]:
        lines.append(
            f"| {tx['operation']} | `{tx['hash']}` | `{str(tx['included']).lower()}` | {tx['official_cu']} |"
        )
    lines.extend(
        [
            "",
            "## Notes",
            "",
            f"- LEZ public program execution sets a session limit of `{summary['lez_public_execution_cycle_limit']}` cycles.",
            "- The sequencer RPC used by this project exposes transaction lookup but not a receipt field containing per-transaction CU.",
            "- Treat the RISC0 cycle fields above as measured proof cost, not an official testnet CU bill.",
            "- Fill official CU once the hosted testnet explorer/RPC exposes it or the Logos team confirms the accepted mapping.",
        ]
    )
    return "\n".join(lines) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description="Summarize LP-0002 cost evidence.")
    parser.add_argument(
        "--proof-stats",
        type=Path,
        help="Path to proof-stats.json. Defaults to newest .local/smoke/*/proof/proof-stats.json.",
    )
    parser.add_argument(
        "--localnet-evidence",
        type=Path,
        help="Path to localnet-evidence.json. Defaults to newest .local/localnet-evidence/*/localnet-evidence.json.",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path(".local/cost-evidence/latest"),
        help="Output directory for cost-summary.json and cost-summary.md.",
    )
    args = parser.parse_args()

    root = Path.cwd()
    proof_stats = args.proof_stats or find_latest(root / ".local/smoke", "*/proof/proof-stats.json")
    localnet_evidence = args.localnet_evidence or find_latest(
        root / ".local/localnet-evidence", "*/localnet-evidence.json"
    )

    proof = read_json(proof_stats)
    evidence = read_json(localnet_evidence)
    txs = evidence["txs"]
    transactions = [
        {
            "operation": operation,
            "hash": tx["hash"],
            "included": tx["included"],
            "official_cu": "not exposed by current RPC",
        }
        for operation, tx in txs.items()
    ]

    summary = {
        "ok": True,
        "inputs": {
            "proof_stats": str(proof_stats),
            "localnet_evidence": str(localnet_evidence),
        },
        "risc0_dev_mode": "0",
        "lez_public_execution_cycle_limit": PUBLIC_EXECUTION_CYCLE_LIMIT,
        "official_transaction_cu_available": False,
        "official_transaction_cu_source": None,
        "private_threshold_proof": {
            "total_cycles": proof["total_cycles"],
            "user_cycles": proof["user_cycles"],
            "paging_cycles": proof["paging_cycles"],
            "segments": proof["segments"],
            "prove_seconds": proof["prove_seconds"],
            "image_id": proof["image_id"],
        },
        "transactions": transactions,
    }

    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "cost-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    (args.out_dir / "cost-summary.md").write_text(markdown(summary))
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
