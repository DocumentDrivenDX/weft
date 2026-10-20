#!/usr/bin/env python3
"""Generate the checked-in UMF 0.10 core schema bundle from owner-pinned sources."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SOURCE_MANIFEST = ROOT / "spec/upstream/actions-plan-010/owner-pins.json"
BUNDLE_PATH = ROOT / "spec/upstream/umf-0.10.0-bundled.schema.json"
PROVENANCE_PATH = ROOT / "spec/upstream/umf-0.10.0-bundled.source.json"
SEMANTIC_URI = "urn:umf:core:semantic-type-reference:1.0.0"
SEMANTIC_LOCAL = "#/$defs/semanticTypeReference"

def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()

def load(path: Path) -> tuple[bytes, object]:
    raw = path.read_bytes()
    return raw, json.loads(raw)

def replace_refs(value: object) -> None:
    if isinstance(value, dict):
        if value.get("$ref") == SEMANTIC_URI:
            value["$ref"] = SEMANTIC_LOCAL
        else:
            for child in value.values():
                replace_refs(child)
    elif isinstance(value, list):
        for child in value:
            replace_refs(child)

def main() -> None:
    pins_raw, pins = load(SOURCE_MANIFEST)
    if pins.get("owner") != "UMF":
        raise SystemExit("upstream-owner")
    resources = {row["file"]: row for row in pins.get("resources", [])}
    core_row = resources.get("core.schema.json")
    semantic_row = resources.get("semantic-type-reference.schema.json")
    if not core_row or not semantic_row:
        raise SystemExit("upstream-resources")
    core_path = SOURCE_MANIFEST.parent / core_row["file"]
    semantic_path = SOURCE_MANIFEST.parent / semantic_row["file"]
    core_raw, core = load(core_path)
    semantic_raw, semantic = load(semantic_path)
    if sha(core_raw) != core_row["sha256"] or sha(semantic_raw) != semantic_row["sha256"]:
        raise SystemExit("upstream-source-pin")
    if core.get("$id") != "urn:umf:core:0.10.0" or semantic.get("$id") != SEMANTIC_URI:
        raise SystemExit("upstream-schema-identity")
    if core != load(ROOT / "spec/upstream/umf-0.10.0.schema.json")[1]:
        raise SystemExit("core-schema-copy-differs-from-owner-pinned-source")
    replace_refs(core)
    replace_refs(semantic)
    semantic.pop("$id", None)
    semantic.pop("$schema", None)
    core.setdefault("$defs", {})["semanticTypeReference"] = semantic
    raw_bundle = (json.dumps(core, ensure_ascii=False, indent=2) + "\n").encode()
    BUNDLE_PATH.write_bytes(raw_bundle)
    provenance = {
        "format": "weft-umf-schema-bundle-source/0.1",
        "owner": pins["owner"],
        "ownerPinsPath": SOURCE_MANIFEST.relative_to(ROOT).as_posix(),
        "ownerPinsSha256": sha(pins_raw),
        "producer": pins["producer"],
        "inputs": [
            {"path": core_path.relative_to(ROOT).as_posix(), "ownerPath": core_row["ownerPath"], "sha256": sha(core_raw)},
            {"path": semantic_path.relative_to(ROOT).as_posix(), "ownerPath": semantic_row["ownerPath"], "sha256": sha(semantic_raw)},
        ],
        "bundlePath": BUNDLE_PATH.relative_to(ROOT).as_posix(),
        "bundleSha256": sha(raw_bundle),
        "generation": "Inline the owner-pinned semanticType reference resource at its exact external $ref sites; preserve all schema assertions and recursive self-reference semantics.",
    }
    PROVENANCE_PATH.write_text(json.dumps(provenance, ensure_ascii=False, indent=2) + "\n")

if __name__ == "__main__":
    main()
