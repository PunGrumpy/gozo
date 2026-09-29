# Contributing to gozo

## Prerequisites

- Rust stable (see `rust-toolchain.toml`), Go 1.24 or newer, Node 22 or newer, and Bun 1.4 (see `packageManager`)
- `bun install` once, then `cargo build` for the CLI. A husky pre-commit hook runs `ultracite fix`

## Everyday commands

| What | Command |
| --- | --- |
| Build the CLI | `cargo build -p gozo`, or `bun run gozo -- doctor` to build and run |
| Rust tests | `cargo test --workspace` |
| Everything CI checks | `bun run verify` (`turbo run quality lint check-types`) |
| Format Rust | `bun run fix:rust` (`cargo fmt --all`) |
| Lint and format JS | `bun run check` and `bun run fix` (ultracite) |
| Try against a Go project | `./target/debug/gozo -C path/to/project doctor` |
| Try the npm launcher | `./packages/gozo/bin/gozo --help`, which uses `target/` in a checkout. `GOZO_BINARY_PATH` overrides it |

## Repository layout

The layout follows [vercel/turborepo](https://github.com/vercel/turborepo):

```text
crates/gozo          CLI (clap)
crates/gozo-go       typed wrapper around the `go` command
crates/gozo-core     gozo.toml, .gozo/ link, env store, deployment history
crates/gozo-doctor   health checks
crates/gozo-deploy   docker and kubernetes adapters
packages/gozo        npm launcher; platform packages are generated at publish time
scripts/             release helpers (version sync, platform packaging, publishing)
.github/             turborepo-style workflows and composite actions
```

Turborepo orchestrates tasks and Bun installs packages. Ultracite lints and formats JavaScript. Changesets drive versioning, described in [RELEASE.md](RELEASE.md).

## Adding or changing a command

- Commands live in `crates/gozo/src/commands/<name>.rs`, one module per command, with its `clap::Args`
- Every fact about a Go project comes from `crates/gozo-go`, the typed wrapper around `go`. Never shell out to `go` from a command directly
- Every command supports `--json` with a `schema: "gozo.<name>/v1"` document. Bump the schema when you change a field
- Progress goes to stderr through `ctx.ui` and the result goes to stdout through `ctx.out`. `commands/doctor.rs` is the reference
- Add a changeset with `bunx changeset`

## Pull requests

- The title follows Conventional Commits with an uppercase subject and no scope: `feat: Add gozo env run`
- Paste the human output and the `--json` output of the affected commands in the description
