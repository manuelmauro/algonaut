#!/usr/bin/env python3
"""Preprocess the Algorand OpenAPI specs before openapi-generator runs.

The custom mustache templates can branch on a property flag, but cannot on
their own emit a file-level `use` import, nor pick the algonaut domain type
for a field the spec types as a plain string/array. So this step injects
vendor extensions the templates consume:

  * `x-rust-imports` on a schema   -> the `use` lines its model file needs;
  * `x-rust-type` on a property    -> the exact Rust type to emit;
  * `x-rust-serde` on a property   -> extra `#[serde(..)]` attributes.

`format: byte` properties pull in `use algonaut_encoding::Bytes;`; everything
else comes from the per-field table in openapi/type-overrides.json.

Reads openapi/specs/*.oas3.json, writes the derived specs to
openapi/generated/_specs/. See docs/adr/openapi-client-regeneration.md.
"""

import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "openapi" / "specs"
OUT = ROOT / "openapi" / "generated" / "_specs"
TABLE = ROOT / "openapi" / "type-overrides.json"

BYTES_IMPORT = "use algonaut_encoding::Bytes;"


def is_byte_format(prop: dict) -> bool:
    """True if a property (or its array items) is `format: byte`."""
    if prop.get("format") == "byte":
        return True
    items = prop.get("items")
    return isinstance(items, dict) and items.get("format") == "byte"


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    table = json.loads(TABLE.read_text())["overrides"]

    for name in ("algod", "indexer"):
        spec = json.loads((SRC / f"{name}.oas3.json").read_text())
        schemas = spec.get("components", {}).get("schemas", {})
        # overrides for this crate, keyed by (schema name, spec property name)
        overrides = {(o["schema"], o["field"]): o for o in table if o["crate"] == name}
        applied = 0

        for schema_name, schema in schemas.items():
            props = schema.get("properties") or {}
            imports: set[str] = set()
            for prop_name, prop in props.items():
                override = overrides.get((schema_name, prop_name))
                if override:
                    prop["x-rust-type"] = override["type"]
                    if override.get("serde"):
                        prop["x-rust-serde"] = override["serde"]
                    if override.get("import"):
                        imports.add(override["import"])
                    applied += 1
                elif is_byte_format(prop):
                    # a byte field with no override regenerates as `Bytes`
                    imports.add(BYTES_IMPORT)
            if imports:
                schema["x-rust-imports"] = sorted(imports)

        (OUT / f"{name}.oas3.json").write_text(json.dumps(spec, indent=2))
        with_imports = sum("x-rust-imports" in s for s in schemas.values())
        expected = sum(1 for o in table if o["crate"] == name)
        print(
            f"{name}: applied {applied}/{expected} overrides, "
            f"{with_imports} schemas need imports"
        )
        if applied != expected:
            raise SystemExit(
                f"{name}: {expected - applied} override(s) in type-overrides.json "
                f"did not match a schema/property — the spec may have changed."
            )


if __name__ == "__main__":
    main()
