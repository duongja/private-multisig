#!/usr/bin/env python3
import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
WRAPPER_PATH = ROOT / "spel" / "private_multisig_idl.rs"
OUT_PATH = ROOT / "docs" / "private-multisig-idl.json"
ALT_OUT_PATH = ROOT / "docs" / "private-multisig.idl.json"


ERRORS = [
    (2000, "InvalidAccountCount", "invalid account count"),
    (2001, "AlreadyInitialized", "account is already initialized"),
    (2002, "InvalidThreshold", "invalid threshold"),
    (2003, "DecodeState", "could not decode private multisig state"),
    (2004, "DecodeProposal", "could not decode proposal state"),
    (2005, "CreateKeyMismatch", "create_key mismatch"),
    (2006, "ProposalIndexMismatch", "proposal index mismatch"),
    (2007, "ProposalNotActive", "proposal is not active"),
    (2008, "TargetAccountCountMismatch", "target account count mismatch"),
    (2009, "InvalidAggregateProof", "invalid aggregate threshold proof"),
    (2010, "AccountDataTooLarge", "account data exceeds LEZ data limit"),
]


def run_spel_generate() -> dict:
    cmd = [
        "cargo",
        "run",
        "-p",
        "spel_idl_exporter",
        "--quiet",
        "--locked",
        "--",
        str(WRAPPER_PATH),
    ]
    proc = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if proc.returncode != 0:
        if proc.stdout:
            sys.stderr.write(proc.stdout)
        if proc.stderr:
            sys.stderr.write(proc.stderr)
        raise SystemExit(f"SPEL IDL generation failed with exit code {proc.returncode}")
    if proc.stderr:
        sys.stderr.write(proc.stderr)
    return json.loads(proc.stdout)


def discriminator(name: str) -> list[int]:
    return list(hashlib.sha256(f"global:{name}".encode()).digest()[:8])


def pascal_case(name: str) -> str:
    return "".join(part.capitalize() for part in name.split("_"))


def enrich_idl(idl: dict) -> dict:
    idl["spec"] = "spel"
    idl["instruction_type"] = "private_multisig_core::PrivateMultisigInstruction"

    account_defs = idl.get("accounts", [])
    state_accounts = []
    helper_types = idl.get("types", [])
    for account_def in account_defs:
        if account_def["name"] in {"PrivateMultisigState", "PrivateProposalState"}:
            state_accounts.append(account_def)
        else:
            helper_types.append(account_def)
    idl["accounts"] = state_accounts
    idl["types"] = helper_types

    for instruction in idl["instructions"]:
        instruction["discriminator"] = discriminator(instruction["name"])
        instruction["execution"] = {"public": True, "private_owned": True}
        instruction["variant"] = pascal_case(instruction["name"])
        for account in instruction["accounts"]:
            account["visibility"] = ["public"]

    idl["errors"] = [
        {"code": code, "name": name, "msg": msg}
        for code, name, msg in ERRORS
    ]
    return idl


def main() -> None:
    check_only = "--check" in sys.argv[1:]
    idl = enrich_idl(run_spel_generate())
    rendered = json.dumps(idl, indent=2) + "\n"

    if check_only:
        current = OUT_PATH.read_text()
        alt_current = ALT_OUT_PATH.read_text()
        if current != rendered or alt_current != rendered:
            raise SystemExit(
                "SPEL IDL is out of date. Run scripts/generate-spel-idl.py to refresh it."
            )
        print(json.dumps({"ok": True, "idl": str(OUT_PATH), "mode": "check"}))
        return

    OUT_PATH.write_text(rendered)
    ALT_OUT_PATH.write_text(rendered)
    print(json.dumps({"ok": True, "idl": str(OUT_PATH), "mode": "write"}))


if __name__ == "__main__":
    main()
