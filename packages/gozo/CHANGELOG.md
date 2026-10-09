# gozo-cli

## 0.1.0

### Minor Changes

- 5e2b880: Initial release: `init`, `link`, `dev`, `env`, `doctor`, `check`, `test`, `build`, `generate`, `run`, `tool`, `deploy`, `ls`, `status`, `logs`, `rollback`, `api` and `update`, with `--json` on every command, docker and kubernetes deploy targets, and the npm launcher package.

### Patch Changes

- d190bc0: `gozo build --json` now reports `name` without the platform suffix or `.exe` on every OS and adds `file` with the actual file name written.
- 52e8106: Fix parsing of `go mod edit -json`, `go work edit -json` and `go list -json` output from Go 1.25 and older, which print `null` for empty lists and made every project look like it had an invalid go.mod.
- 87a6304: The npm launcher now runs the binary from `target/` when executed inside a source checkout and no longer fails with "Cannot find module 'gozo/package.json'" when it has to install the platform package.
