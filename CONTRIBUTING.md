# Contributing to gozo

## Prerequisites

- Rust stable (see `rust-toolchain.toml`), Go 1.24 or newer, Node 22+ and Bun 1.4 (see `packageManager`).
- `bun install` once; `cargo build` for the CLI. A husky pre-commit hook runs `ultracite fix`.

## Everyday commands

| What | Command |
| --- | --- |
| Build the CLI | `cargo build -p gozo` (or `bun run gozo -- doctor` to build and run) |
| Rust tests | `cargo test --workspace` |
| Everything CI checks | `bun run verify` (`turbo run quality lint check-types`) |
| Format Rust | `bun run fix:rust` (`cargo fmt --all`) |
| Lint + format JS | `bun run check` / `bun run fix` (ultracite) |
| Try against a Go project | `./target/debug/gozo -C path/to/project doctor` |

## Adding or changing a command

- Commands live in `crates/gozo/src/commands/<name>.rs`, one module per command, with its `clap::Args`.
- Every fact about a Go project comes from `crates/gozo-go` (typed wrappers around `go`). Never shell out to `go` from a command directly.
- Every command supports `--json` with a `schema: "gozo.<name>/v1"` document. Bump the schema when you change a field.
- Human output: stderr for progress (`ctx.ui`), stdout for the result (`ctx.out`). See `commands/doctor.rs` as the reference.
- Add a changeset: `bunx changeset`.

## Pull requests

- Title follows Conventional Commits with an uppercase subject: `feat: Add gozo env run`. Scopes are not used.
- Paste the human and `--json` output of the affected commands in the PR description.
