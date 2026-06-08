---
name: use-arkouda
description: Find prior decisions and record new ones in a repo's ADR (Architecture Decision Record) collection using the arkouda CLI. Invoke any time you're about to make a non-trivial design, architecture, library, schema, or convention decision — check what was already decided before deciding, and capture the outcome afterwards.
license: MIT
---

# Using arkouda

In repositories that record decisions as ADRs (Architecture Decision Records — Markdown files with YAML frontmatter, conventionally under `docs/adr/`), `arkouda` is the CLI for finding, reading, validating, and scaffolding them. **Before you decide, check what's already been decided. After you decide, capture it.**

If a repo has no ADR directory yet but the `arkouda` binary is installed, this skill is also the right one to reach for: `arkouda new` enforces the schema from the first file.

## When to use

Reach for this skill any time you're about to make a non-trivial decision. Concretely:

- Before writing code that picks a library, framework, datastore, encoding, transport, or other "we now depend on X" commitment.
- Before changing a public interface, file layout, schema, naming convention, or directory structure.
- Before refactoring away from a pattern you didn't introduce — you may be about to undo a deliberate decision.
- When the user asks "did we ever decide on X?", "why is it done this way?", or otherwise touches motivation.
- When the user asks for a new ADR or to mark one superseded.
- Whenever you land in an unfamiliar repo with an ADR directory (commonly `docs/adr/`, but not always — see below).

A 5-second `arkouda list | xargs rg -i <topic>` is cheaper than redoing a debate that's already in the file. If a relevant ADR exists, build on it, propose superseding it, or notice you don't need to decide at all.

## Philosophy

Two principles shape arkouda's behaviour, and explain why some defaults look minimal:

- **Defer to Unix tools.** Arkouda earns subcommands only where standard shell tools (`rg`, `grep`, `cat`, `awk`, `xargs`) cannot. Content search, full-file printing, counting, and slicing are left to the shell — the CLI emits structured output you compose with the rest of your toolbox. Hence: no `search`, no full-file `show`.
- **Decision-centric defaults.** `arkouda list` prints one ADR path per line (no header, no padding) so it pipes cleanly. The body of an ADR, for arkouda's purposes, is its `## Decision` section, so `arkouda decision <id>` defaults to that section's body — supporting sections (`context`, `consequences`, `status`, custom) are opt-in via `--section`.

The source rationale lives in arkouda's own repo, in the ADRs [`defer-to-unix-tools`](https://github.com/manuelmauro/arkouda/blob/main/docs/adr/defer-to-unix-tools.md) and [`ls-style-list-and-decision`](https://github.com/manuelmauro/arkouda/blob/main/docs/adr/ls-style-list-and-decision.md).

## Where ADRs live

The location varies between repos. Don't hardcode `docs/adr/` in pipelines — ask arkouda. Run **`arkouda list`** to get the actual ADR paths for the repo you're in.

Resolution order, in case you need to set or override the location:

1. `--dir <path>` flag (one-shot override, single directory).
2. `ADR_DIR=<path>` environment variable (session override, single directory).
3. `.arkoudarc.toml` at the repo root (or any ancestor of the cwd) with a `dirs = [...]` list — supports multiple directories, useful for monorepos:
   ```toml
   dirs = ["docs/adr", "services/billing/docs/adr"]
   ```
   Relative paths resolve against the config file's directory. `arkouda list`, `check`, and `decision` aggregate across all listed dirs; `arkouda new` writes into the first one.
4. Default: `docs/adr/`.

Filenames are the ADR id with a `.md` suffix (e.g. `use-postgres.md`), but the directory varies — let `arkouda list` tell you.

## Commands

Four subcommands, each doing something the shell can't:

- **`arkouda list [--sort id|date|status] [-l]`** — one ADR path per line. Pipe straight into `xargs`/`rg`/`cat`/`wc`. With `-l`, headerless `ID STATUS DATE PATH TITLE` table for human skimming.
- **`arkouda decision <id> [--section <name>]`** — body of that ADR's `## Decision` section. `--section <name>` picks any other heading (`context`, `consequences`, `status`, or custom). Errors if the section is missing. For the full file, resolve the path through `arkouda list` and `cat` it.
- **`arkouda check`** — validates frontmatter, filenames, and required Markdown sections across the collection. Exit 0 clean, 1 on any error. Each error carries a code (E000–E010) and a fix hint.
- **`arkouda new "<title>" [--id <slug>] [--status proposed|accepted|superseded|deprecated|rejected] [--abstract "<one-line summary of the decision>"]`** — scaffold a new ADR with today's date. Default id is a slug from the title. The abstract should summarize *what was decided*, not just the topic.

Global flags: `--dir <path>` (also `ADR_DIR`), `-q/--quiet`. Run `arkouda --help` or `arkouda <subcommand> --help` for the authoritative surface.

There is intentionally no `search` subcommand and no full-file `show` — `rg`/`grep` and `cat` already do those.

## One-liners

`arkouda list` is the path source — it's where the ADRs *actually* are in this repo.

```sh
# Orient in an unfamiliar repo
arkouda list -l && arkouda check

# Paths of all ADRs (for piping)
arkouda list

# Search ADRs for a topic — let list provide the search roots
arkouda list | xargs rg -i <topic>

# Read the decision of a specific ADR
arkouda decision use-postgres

# Read another section instead
arkouda decision use-postgres --section consequences

# Read the whole ADR — resolve the path through list
cat "$(arkouda list | grep -F /use-postgres.md)"

# Paths of accepted ADRs only
arkouda list -l | awk '$2=="accepted" {print $4}'

# Count ADRs by status
arkouda list -l | awk '{print $2}' | sort | uniq -c

# Most recent N decisions
arkouda list -l --sort date | tail -10

# Stream every Decision section in the collection
arkouda list | while read f; do
  id=$(basename "$f" .md)
  printf '## %s\n\n' "$id"
  arkouda decision "$id"
  printf '\n'
done

# Scaffold a new decision and validate it
arkouda new "Adopt Tracing" --abstract "Use OpenTelemetry across services."
arkouda check
```

## Workflows

**Before deciding** — search what's already there:

```sh
arkouda list | xargs rg -i <topic>         # content search across all ADRs
arkouda list -l | awk '$2=="accepted"'     # accepted decisions only
arkouda decision <id>                      # read the meat of a hit
```

**After deciding** — capture it:

```sh
arkouda new "<Title>" --abstract "<one-line summary of what was decided>"
# arkouda new prints the path it created — open that file and fill in
# Context, Decision, Consequences
arkouda check
```

**Supersede an existing decision**

1. Resolve the path: `path=$(arkouda list | grep -F /<old-id>.md)`.
2. `cat "$path"` to see the current frontmatter, then edit: change `status: "superseded"` and add `superseded_by: "<new-id>"`.
3. `arkouda new "<New Title>"` for the replacement.
4. `arkouda check` to confirm both files still validate.

## ADR shape (what `check` enforces)

```markdown
---
id: "use-postgres"            # lowercase slug, must match filename stem
title: "Use Postgres"
abstract: "One-line summary of the decision (what was decided)."
status: "proposed"            # proposed | accepted | superseded | deprecated | rejected
date: "2026-05-06"            # ISO YYYY-MM-DD, must be a real date
deciders: []                  # optional
tags: []                      # optional
---

# Use Postgres                # H1 must equal title

## Status

Proposed

## Context

Why we are deciding this.

## Decision

What we decided.

## Consequences

What follows from the decision.
```

Required keys: `id`, `title`, `abstract`, `status`, `date`. Required body sections (case-insensitive H2): `Status`, `Context`, `Decision`, `Consequences` — from [Michael Nygard's ADR template](https://github.com/joelparkerhenderson/architecture-decision-record/tree/main/locales/en/templates/decision-record-template-by-michael-nygard). Filename stem must equal the frontmatter `id`.

## When `check` reports errors

Each diagnostic has a code; the hint usually tells you the exact fix.

- **E001/E002** missing or empty required field → add the field with a real value.
- **E003** invalid status → use one of the five valid values.
- **E004** id is not a lowercase slug → letters, digits, single hyphens.
- **E005** filename does not match id → rename to `<id>.md`.
- **E006** invalid date → ISO `YYYY-MM-DD` for a real calendar day.
- **E007/E008** missing or wrong H1 → first heading must be `# <title>`.
- **E009** missing required section → add the named `## Section` heading.
- **E010** duplicate id across files → make ids unique.

## What not to do

- Don't make a non-trivial decision without first checking existing ADRs.
- Don't hardcode `docs/adr/` in pipelines — different repos put ADRs elsewhere via `.arkoudarc.toml`. Use `arkouda list` to discover the actual paths.
- Don't write or edit ADR files freehand without running `arkouda check` afterwards — the schema is strict.
- Don't invent statuses outside the five valid values; downstream tooling depends on them.
- Don't change a published ADR's `id` after creation; create a new ADR and mark the old one `superseded` instead.
- Don't commit ADRs whose `arkouda check` fails — CI is likely to enforce it.
