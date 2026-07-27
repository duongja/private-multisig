#!/usr/bin/env python3
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
UI = ROOT / "basecamp-ui"


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def main():
    metadata_path = UI / "metadata.json"
    qml_path = UI / "Main.qml"
    flake_path = UI / "flake.nix"
    cmake_path = UI / "CMakeLists.txt"
    backend_cpp_path = UI / "src/private_multisig_ui_plugin.cpp"
    rep_path = UI / "src/private_multisig_ui.rep"

    require(metadata_path.exists(), "missing basecamp-ui/metadata.json")
    require(qml_path.exists(), "missing basecamp-ui/Main.qml")
    require(flake_path.exists(), "missing basecamp-ui/flake.nix")
    require(cmake_path.exists(), "missing basecamp-ui/CMakeLists.txt")
    require(backend_cpp_path.exists(), "missing Basecamp backend plugin")
    require(rep_path.exists(), "missing Basecamp backend .rep file")

    metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    require(metadata.get("name") == "private_multisig_ui", "unexpected module name")
    require(metadata.get("type") == "ui_qml", "module must be ui_qml")
    require(metadata.get("view") == "Main.qml", "module view must be Main.qml")
    require(metadata.get("main") == "private_multisig_ui_plugin", "backend main must be declared")
    require(isinstance(metadata.get("dependencies"), list), "dependencies must be a list")

    qml = qml_path.read_text(encoding="utf-8")
    required_qml_markers = [
        "PrivateMultisigBasecampUi",
        'logos.module("private_multisig_ui")',
        "runBackendFlow",
        "function generateMembers()",
        "function createProposal()",
        "function approveSelectedMember()",
        "function aggregateBackendApprovals()",
        "function verifyBackendAggregate()",
        "function proveBackendAggregate()",
        "function executeLocalnetBackend()",
        "function executeHostedTestnetBackend()",
        "function testDuplicateSelected()",
        "function refreshWorkspaceState()",
        "duplicateRejected",
        "memberRoot",
        "evidenceJson",
        "StatusCard",
    ]
    for marker in required_qml_markers:
        require(marker in qml, f"missing QML marker: {marker}")

    flake = flake_path.read_text(encoding="utf-8")
    require("mkLogosQmlModule" in flake, "flake must use mkLogosQmlModule")
    require("configFile = ./metadata.json" in flake, "flake must use metadata.json")

    backend_cpp = backend_cpp_path.read_text(encoding="utf-8")
    require("QProcess" in backend_cpp, "backend must invoke CLI through QProcess")
    require("PRIVATE_MULTISIG_CLI" in backend_cpp, "backend must support PRIVATE_MULTISIG_CLI")
    require("runDemoFlow" in backend_cpp, "backend must expose runDemoFlow")
    require("generateMembers" in backend_cpp, "backend must expose generateMembers")
    require("approveMember" in backend_cpp, "backend must expose approveMember")
    require("aggregateApprovals" in backend_cpp, "backend must expose aggregateApprovals")
    require("verifyAggregate" in backend_cpp, "backend must expose verifyAggregate")
    require("proveAggregate" in backend_cpp, "backend must expose proveAggregate")
    require("runLocalnetExecution" in backend_cpp, "backend must expose runLocalnetExecution")
    require("runHostedTestnetExecution" in backend_cpp, "backend must expose runHostedTestnetExecution")

    rep = rep_path.read_text(encoding="utf-8")
    require("SLOT(QString runDemoFlow" in rep, "rep must expose runDemoFlow")
    require("SLOT(QString loadWorkspaceState())" in rep, "rep must expose loadWorkspaceState")
    require("SLOT(QString generateMembers(QString multisigId))" in rep, "rep must expose generateMembers")
    require("SLOT(QString approveMember(QString memberId))" in rep, "rep must expose approveMember")
    require("SLOT(QString aggregateApprovals())" in rep, "rep must expose aggregateApprovals")
    require("SLOT(QString verifyAggregate())" in rep, "rep must expose verifyAggregate")
    require("SLOT(QString runLocalnetExecution())" in rep, "rep must expose runLocalnetExecution")
    require("SLOT(QString runHostedTestnetExecution())" in rep, "rep must expose runHostedTestnetExecution")

    print(
        json.dumps(
            {
                "ok": True,
                "module": metadata["name"],
                "type": metadata["type"],
                "view": metadata["view"],
                "main": metadata["main"],
                "qml_bytes": qml_path.stat().st_size,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
