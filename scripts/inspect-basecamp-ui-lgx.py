#!/usr/bin/env python3
import hashlib
import json
import tarfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_LGX_LINK = ROOT / ".local/basecamp-ui-lgx/logos-private_multisig_ui-module.lgx"


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main():
    lgx = DEFAULT_LGX_LINK
    require(lgx.exists(), f"missing LGX package: {lgx}")

    with tarfile.open(lgx, "r:gz") as archive:
        names = sorted(archive.getnames())
        require("manifest.json" in names, "missing manifest.json")
        require("variants/linux-amd64-dev/Main.qml" in names, "missing linux-amd64-dev/Main.qml")
        require(
            "variants/linux-amd64-dev/metadata.json" in names,
            "missing linux-amd64-dev/metadata.json",
        )
        require(
            "variants/linux-amd64-dev/private_multisig_ui_plugin.so" in names,
            "missing backend plugin",
        )
        require(
            "variants/linux-amd64-dev/private_multisig_ui_replica_factory.so" in names,
            "missing backend replica factory",
        )
        manifest = json.load(archive.extractfile("manifest.json"))

    require(manifest.get("name") == "private_multisig_ui", "unexpected manifest module name")
    require(manifest.get("type") == "ui_qml", "manifest type must be ui_qml")
    require(manifest.get("view") == "Main.qml", "manifest view must be Main.qml")
    require(
        "private_multisig_ui_plugin" in json.dumps(manifest.get("main", {})),
        "manifest main must reference backend plugin",
    )
    require(
        "variants/linux-amd64-dev" in manifest.get("hashes", {}),
        "missing linux-amd64-dev variant hash",
    )

    print(
        json.dumps(
            {
                "ok": True,
                "path": str(lgx),
                "bytes": lgx.stat().st_size,
                "sha256": sha256(lgx),
                "module": manifest["name"],
                "type": manifest["type"],
                "view": manifest["view"],
                "main": manifest.get("main", {}),
                "manifest_version": manifest["manifestVersion"],
                "variant": "linux-amd64-dev",
                "entries": names,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
