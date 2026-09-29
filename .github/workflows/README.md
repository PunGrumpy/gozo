## Workflows

| Workflow | Trigger | What it does |
| --- | --- | --- |
| `test.yml` | PR, push to main | Rust tests on Linux/macOS/Windows, CLI smoke test against a real Go project, npm launcher test |
| `lint.yml` | PR | `cargo fmt`, `cargo clippy -D warnings`, `bun run verify` (turbo: ultracite + Rust gates + package lint/types) |
| `lint-pr-title.yml` | PR | Conventional-commit PR titles (`feat: Add ...`) |
| `changeset-status.yml` | PR | Comments whether the PR carries a changeset (non-blocking) |
| `release.yml` | push to main, manual dry run | changesets/action v2: `select-mode` → `version` PR, or `build` binaries → `publish` (npm trusted publishing, `gozo@<version>` tag, GitHub release with tarballs) |

### Composite actions

- `setup-rust`: toolchain from `rust-toolchain.toml` + `Swatinem/rust-cache` (saved only on main).
- `setup-bun`: Bun from `packageManager` + Node from `engines` + bun cache + `bun install --frozen-lockfile`.
- `setup-go`: Go for the smoke tests.

### Secrets

- None required for npm when trusted publishing is configured for `gozo` and `gozo-*` (`id-token: write` on the publish job). See RELEASE.md for the token fallback.
- `GITHUB_TOKEN` (automatic): release PR, tags, GitHub release.

Action versions are pinned to commit SHAs where the upstream turborepo repo pins them; bump them together.
