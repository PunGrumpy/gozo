```
   ____ _____  ____ _____
  / __ `/ __ \/_  // __ \       The missing developer experience layer for Go.
 / /_/ / /_/ / / // /_/ /
 \__, /\____/ /___|\____/       curl -fsSL https://raw.githubusercontent.com/PunGrumpy/gozo/main/install.sh | sh
/____/
                                ⚠ Status: Experimental. Use at your own risk.
```

gozo is a developer CLI for Go projects written in Rust: a single native binary that wraps the `go` command with the workflow of Vercel CLI. It never re-implements Go tooling; every fact comes from `go env`, `go mod`, `go work`, `go list` and `go test` in JSON, and every gozo command answers in JSON too, so people and coding agents share one tool.

## Highlights

- **Develop:** `gozo dev` builds, runs and restarts your app with `.env` files layered and compose services up; `gozo env` manages development, preview and production variables like `vercel env`
- **Validate:** `gozo doctor`, `check`, `test`, `generate --check` and `build` run across every module of a `go.work` workspace with one report
- **Deploy and operate:** `gozo link` binds a directory to Docker or Kubernetes, then `deploy`, `status`, `logs -f` and `rollback` work without remembering the underlying commands
- **Agent-native:** `--json` on every command, `gozo api <resource>` for project state, stdout reserved for results, `--non-interactive` detected automatically in CI and agents

<p>
  <a href="https://github.com/PunGrumpy/gozo/releases/latest"><img alt="gozo release" src="https://img.shields.io/github/v/release/PunGrumpy/gozo.svg?style=for-the-badge&amp;labelColor=000000&amp;label=release" height="28"></a>
  <a href="https://www.npmjs.com/package/gozo"><img alt="npm" src="https://img.shields.io/npm/v/gozo.svg?style=for-the-badge&amp;labelColor=000000" height="28"></a>
  <a href="https://github.com/PunGrumpy/gozo/blob/main/LICENSE"><img alt="License: MIT" src="https://img.shields.io/github/license/PunGrumpy/gozo.svg?style=for-the-badge&amp;labelColor=000000" height="28"></a>
</p>

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/PunGrumpy/gozo/main/install.sh | sh
```

Or `npm i -g gozo`, `bun add -g gozo`, `cargo install gozo`.

## Get started

Run gozo inside any Go project for an interactive menu, or go straight to a command:

```bash
cd your_project
gozo                # detects the project and asks what to do
gozo doctor         # toolchain, go.mod / go.work, tidy, outdated deps, CGO
gozo dev            # http://localhost:8080 with live reload
gozo link           # choose docker or kubernetes, stored in .gozo/
gozo deploy --prod  # stdout is the URL, everything else is on stderr
```

No Go project yet? `gozo init` creates `go.mod`, `cmd/<name>/main.go` and `gozo.toml`.

## Commands

| Phase    | Commands                                       |
| -------- | ---------------------------------------------- |
| Develop  | `init`, `dev`, `env`, `run`, `tool`            |
| Validate | `doctor`, `check`, `test`, `generate`, `build` |
| Deploy   | `link`, `deploy`                               |
| Operate  | `ls`, `status`, `logs`, `rollback`             |
| Agent    | `api`, `update`                                |

`gozo <command> --help` documents every flag. Global flags follow Vercel CLI: `--json`, `--yes`, `--non-interactive`, `--cwd`, `--debug`, `--no-color`, `--project`. Exit codes: `0` ok, `1` the thing you checked is bad, `2` gozo could not do the job.

## For agents

Every command returns one JSON document with a stable `schema` field (`gozo.doctor/v1`, `gozo.test/v1`, ...), and errors are JSON as well:

```bash
gozo doctor --json | jq '.findings[] | select(.status == "fail")'
gozo api project    # name, root, modules, Go version, link target
gozo api ls         # every resource
```

## Configuration

`gozo.toml` is committed and shared (dev command, tasks, build targets, deploy defaults). `.gozo/` is local: the link, env store and deployment history. `gozo init` writes a commented starter file.

## Build from source

Requires Rust stable, Go 1.24 or newer, and Bun for the JavaScript tooling:

```bash
git clone https://github.com/PunGrumpy/gozo.git
cd gozo
bun install
cargo build --release -p gozo
./target/release/gozo
```

Run everything CI runs with `bun run verify`. See [CONTRIBUTING.md](CONTRIBUTING.md) for the repository layout and release flow.

## License

[MIT](LICENSE)
