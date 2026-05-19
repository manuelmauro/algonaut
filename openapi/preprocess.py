#!/usr/bin/env python3
"""Preprocess the Algorand OpenAPI specs before openapi-generator runs.

openapi-generator's mustache templates can branch on a property flag
(`isByteArray`) but not emit a *file-level* `use` import conditionally. So
this step sets a schema-level `x-has-bytes` vendor extension on every schema
that has at least one `format: byte` property; the custom `model.mustache`
reads it to emit `use algonaut_encoding::Bytes;`.

Reads openapi/specs/*.oas3.json, writes the derived specs to
openapi/generated/_specs/ (git-ignored). See
docs/adr/openapi-client-regeneration.md.
"""

import json
import pathlib

SRC = pathlib.Path("openapi/specs")
OUT = pathlib.Path("openapi/generated/_specs")


def has_byte_format(prop: dict) -> bool:
    """True if a property (or its array items) is `format: byte`."""
    if prop.get("format") == "byte":
        return True
    items = prop.get("items")
    return isinstance(items, dict) and items.get("format") == "byte"


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for name in ("algod", "indexer"):
        spec = json.loads((SRC / f"{name}.oas3.json").read_text())
        schemas = spec.get("components", {}).get("schemas", {})
        flagged = 0
        for schema in schemas.values():
            props = schema.get("properties") or {}
            if any(has_byte_format(p) for p in props.values()):
                schema["x-has-bytes"] = True
                flagged += 1
        (OUT / f"{name}.oas3.json").write_text(json.dumps(spec, indent=2))
        print(f"{name}: flagged {flagged} schemas with x-has-bytes")


if __name__ == "__main__":
    main()
