#!/usr/bin/env python3
import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
IDL_PATH = ROOT / "docs" / "private-multisig-idl.json"


def expect(condition, message):
    if not condition:
        raise SystemExit(f"IDL validation failed: {message}")


def type_json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def discriminator(name):
    return list(hashlib.sha256(f"global:{name}".encode()).digest()[:8])


def main():
    with IDL_PATH.open() as f:
        idl = json.load(f)

    expect(idl["name"] == "private_multisig", "program name mismatch")
    expect(idl["version"] == "0.1.0", "version mismatch")
    expect(idl.get("spec") == "spel", "spec must be spel")
    expect(
        idl.get("instruction_type") == "private_multisig_core::PrivateMultisigInstruction",
        "instruction_type mismatch",
    )

    instructions = {ix["name"]: ix for ix in idl["instructions"]}
    expect(
        set(instructions) == {"create_multisig", "propose", "execute_private"},
        f"unexpected instruction set: {sorted(instructions)}",
    )

    for name, ix in instructions.items():
        expect(ix.get("variant"), f"{name} missing variant")
        expect(ix.get("execution") == {"public": True, "private_owned": True}, f"{name} execution mismatch")
        expect(ix.get("discriminator") == discriminator(name), f"{name} discriminator mismatch")
        for account in ix["accounts"]:
            expect(account.get("visibility") == ["public"], f"{name}.{account['name']} visibility mismatch")

    create = instructions["create_multisig"]
    expect([a["name"] for a in create["accounts"]] == ["multisig_state"], "create accounts mismatch")
    expect(create["accounts"][0].get("init") is True, "create multisig_state must be init")
    expect(
        create["accounts"][0].get("pda", {}).get("seeds") == [{"kind": "arg", "path": "create_key"}],
        "create multisig_state PDA mismatch",
    )
    expect(
        [(arg["name"], type_json(arg["type"])) for arg in create["args"]]
        == [
            ("create_key", type_json({"array": ["u8", 32]})),
            ("threshold", type_json("u8")),
            ("member_count", type_json("u8")),
            ("member_root", type_json({"array": ["u8", 32]})),
        ],
        "create args mismatch",
    )

    propose = instructions["propose"]
    expect([a["name"] for a in propose["accounts"]] == ["multisig_state", "proposal"], "propose accounts mismatch")
    expect(propose["accounts"][0]["writable"] is True, "propose multisig_state must be writable")
    expect(propose["accounts"][1].get("init") is True, "propose proposal must be init")
    expect(
        propose["accounts"][1].get("pda", {}).get("seeds")
        == [
            {"kind": "const", "value": "private_ms_prop"},
            {"kind": "arg", "path": "create_key"},
            {"kind": "arg", "path": "proposal_index"},
        ],
        "proposal PDA mismatch",
    )
    expect(
        [(arg["name"], type_json(arg["type"])) for arg in propose["args"]]
        == [
            ("create_key", type_json({"array": ["u8", 32]})),
            ("proposal_index", type_json("u64")),
            ("target_program_id", type_json({"array": ["u32", 8]})),
            ("target_instruction_data", type_json({"vec": "u32"})),
            ("target_account_count", type_json("u8")),
            ("pda_seeds", type_json({"vec": {"array": ["u8", 32]}})),
            ("authorized_indices", type_json({"vec": "u8"})),
        ],
        "propose args mismatch",
    )

    execute = instructions["execute_private"]
    expect(
        [a["name"] for a in execute["accounts"]] == ["multisig_state", "proposal", "target_accounts"],
        "execute accounts mismatch",
    )
    expect(execute["accounts"][2].get("rest") is True, "execute target_accounts must be rest")
    expect(
        execute["accounts"][1].get("pda", {}).get("seeds")
        == [
            {"kind": "const", "value": "private_ms_prop"},
            {"kind": "arg", "path": "create_key"},
            {"kind": "arg", "path": "proposal_index"},
        ],
        "execute proposal PDA mismatch",
    )
    expect(
        [(arg["name"], type_json(arg["type"])) for arg in execute["args"]]
        == [
            ("create_key", type_json({"array": ["u8", 32]})),
            ("proposal_index", type_json("u64")),
            ("aggregate", type_json({"defined": "AggregateApproval"})),
        ],
        "execute args mismatch",
    )

    accounts = {a["name"]: a for a in idl.get("accounts", [])}
    expect(set(accounts) == {"PrivateMultisigState", "PrivateProposalState"}, "account types mismatch")
    types = {t["name"]: t for t in idl.get("types", [])}
    expect(set(types) == {"AggregateApproval", "PrivateProposalStatus"}, "defined types mismatch")

    errors = {err["code"]: err["name"] for err in idl.get("errors", [])}
    expect(
        errors
        == {
            2000: "InvalidAccountCount",
            2001: "AlreadyInitialized",
            2002: "InvalidThreshold",
            2003: "DecodeState",
            2004: "DecodeProposal",
            2005: "CreateKeyMismatch",
            2006: "ProposalIndexMismatch",
            2007: "ProposalNotActive",
            2008: "TargetAccountCountMismatch",
            2009: "InvalidAggregateProof",
            2010: "AccountDataTooLarge",
        },
        f"error table mismatch: {errors}",
    )

    print(json.dumps({"ok": True, "idl": str(IDL_PATH), "instructions": sorted(instructions)}))


if __name__ == "__main__":
    main()
