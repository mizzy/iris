# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Iris is a CLI tool that adds syntax highlighting to `git diff` and `git log -p` output. It reads unified diff from stdin, detects the language of each changed file, and outputs highlighted diff to stdout. Intended for use as a git pager (`core.pager` or `pager.diff`/`pager.log`/`pager.show`).

## Build & Run

```bash
cargo build            # dev build
cargo run              # run from source (reads stdin)
cargo install --path .  # install to ~/.cargo/bin
```

Manual test: `git diff | cargo run` or `git log -p | cargo run`

## Architecture

- **src/main.rs** — Entry point. Reads stdin line-by-line, delegates to diff parser and highlighter, writes to stdout.
- **src/diff.rs** — Unified diff parser. Parses `diff --git` blocks into structured data (headers, hunks, added/removed/context lines). Must also pass through non-diff text (e.g. commit messages from `git log -p`).
- **src/highlight.rs** — Syntax highlighting via [syntect](https://docs.rs/syntect). Detects language from filename, highlights code lines, and preserves diff markers (`+`/`-`/`@@`).

The codebase is currently scaffolding with TODOs — implementation is pending.

## Key Design Decisions

- Uses **syntect** (Sublime Text syntax definitions) for highlighting — same engine as `bat`.
- Non-diff input must be passed through unmodified so iris works as `core.pager` for all git subcommands.
