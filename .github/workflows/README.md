## Workflows

| Workflow | Trigger | What it does |
| --- | --- | --- |
| `test.yml` | PR, push to main | Rust tests on Linux, macOS and Windows, a CLI smoke test against a real Go project, and the npm launcher test |
| `lint.yml` | PR | `cargo fmt`, `cargo clippy -D warnings` and `bun run verify` (ultracite, Rust gates, package lint and types through turbo) |
| `lint-pr-title.yml` | PR | Conventional Commits PR titles, such as `feat: Add ...` |
| `changeset-status.yml` | PR | Comments whether the PR carries a changeset, without blocking |
| `release.yml` | push to main, manual dry run | changesets/action v2: `select-mode`, then either the `version` PR or `build` binaries and `publish` (npm trusted publishing, `gozo@<version>` tag, GitHub release with tarballs) |

### Composite actions

- `setup-rust`: toolchain from `rust-toolchain.toml` plus `Swatinem/rust-cache`, saved only on main
- `setup-bun`: Bun from `packageManager`, Node from `engines`, the bun cache and `bun install --frozen-lockfile`
- `setup-go`: Go for the smoke tests

### Secrets

- npm needs none when trusted publishing is configured for `gozo` and `gozo-*`; the publish job has `id-token: write`. [RELEASE.md](../../RELEASE.md) describes the token fallback
- `GITHUB_TOKEN` is automatic and covers the release PR, tags and the GitHub release

Action versions are pinned to the commit SHAs the upstream turborepo repository pins. Bump them together.
