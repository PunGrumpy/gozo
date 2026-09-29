<p align="center">
  <strong>gozo</strong><br/>
  The missing developer experience layer for Go.
</p>

`gozo` is a single binary that sits around the `go` command and gives Go projects the workflow of Vercel CLI: `dev`, `env`, `link`, `deploy`, `logs`, `rollback`, plus project health (`doctor`, `check`, `test`, `build`) and a JSON mode on every command so coding agents can use the same tool you do.

It never re-implements Go tooling. Every fact comes from `go env -json`, `go mod edit -json`, `go work edit -json`, `go list -json`, `go test -json`. What it adds is the glue Go projects usually assemble by hand.

```sh
curl -fsSL https://raw.githubusercontent.com/pungrumpy/gozo/main/install.sh | sh
# or
npm i -g gozo   # or: bun add -g gozo
# or
cargo install gozo
```

```text
$ gozo
Detected Go project
  name        api
  module      github.com/acme/api
  go          1.27.1
  layout      workspace (3 modules)
  linked      kubernetes (api)

? What do you want to do?
❯ Start development
  Run tests
  Check project
  Doctor
  Deploy
  Show logs
  Link project
```

## Commands

| Phase | Command | What it does |
| --- | --- | --- |
| Develop | `gozo init [path]` | Create `gozo.toml` (and `go.mod` + `cmd/<name>/main.go` when missing) |
|  | `gozo dev` | Build and run the app with `.env*` layered, restart on change, start compose services |
|  | `gozo env ls\|add\|rm\|update\|pull\|run` | Per-environment variables (development / preview / production), `pull` writes `.env.local` |
|  | `gozo run <task>` | Run tasks from `gozo.toml` with deps and env |
|  | `gozo tool ls\|add\|rm\|run\|outdated\|update` | Go 1.24 `tool` directives without remembering the flags |
| Validate | `gozo doctor` | Toolchain, go.mod / go.work consistency, tidy, outdated deps, replace directives, CGO |
|  | `gozo check` | `go vet`, `gofmt`, `golangci-lint` across every module, one report |
|  | `gozo test` | `go test -json` across every module with a unified summary and failure output |
|  | `gozo generate [--check]` | `go generate` everywhere; `--check` fails when generated code is stale |
|  | `gozo build` | Every main package, `-trimpath`, version/commit/date injected, cross targets |
| Deploy | `gozo link` | Bind the directory to a target (docker or kubernetes), stored in `.gozo/` |
|  | `gozo deploy [--prod]` | Build image, run container or roll out the Deployment, record history |
| Operate | `gozo ls` | Deployment history |
|  | `gozo status` | Health of the linked target |
|  | `gozo logs [-f]` | Stream logs from the linked target |
|  | `gozo rollback [id]` | Roll back to the previous or a chosen deployment |
| Agent | `gozo api <resource>` | Project state as JSON: project, config, link, env, deployments, packages, modules, go-env, tools, doctor |
|  | `gozo update` | Self-update from GitHub releases |

Every command accepts `--json` and returns one document with a `schema` field (`gozo.doctor/v1`, `gozo.test/v1`, ...). Errors in JSON mode are JSON too.

### Conventions borrowed from Vercel CLI

- stdout is the result, stderr is the chatter. `gozo deploy > url.txt` works.
- `--yes` accepts defaults; `--non-interactive` (implied in CI and under coding agents) fails instead of prompting.
- `--cwd`, `--debug`, `--no-color` / `NO_COLOR`, `--project` / `GOZO_PROJECT`.
- `.gozo/` is the machine-local link directory, like `.vercel/`; it is git-ignored automatically.
- Exit codes: `0` ok, `1` the thing you checked is bad, `2` gozo could not do the job.

## Configuration

`gozo.toml` is committed and shared; `.gozo/project.json` is local and written by `gozo link`.

```toml
[project]
name = "api"

[dev]
cmd = "./cmd/api"
port = 8080
services = "docker-compose.yml"

[build]
targets = ["linux/amd64", "darwin/arm64"]

[tasks]
lint = "golangci-lint run ./..."
migrate = { cmd = "go run ./cmd/migrate up", deps = ["build"] }

[deploy]
target = "kubernetes"

[deploy.docker]
image = "ghcr.io/acme/api"
platform = "linux/amd64"
push = true

[deploy.kubernetes]
context = "prod"
namespace = "shop"
deployment = "api"
```

## Repository

Layout follows [vercel/turborepo](https://github.com/vercel/turborepo).

```text
crates/gozo          CLI (clap)
crates/gozo-go       typed wrapper around the `go` command
crates/gozo-core     gozo.toml, .gozo/ link, env store, deployment history
crates/gozo-doctor   health checks
crates/gozo-deploy   docker and kubernetes adapters
packages/gozo        npm launcher; platform packages are generated at publish time
scripts/             release helpers (version sync, tagging, platform packaging)
.github/             turborepo-style workflows and composite actions
```

- Build system: [Turborepo](https://turborepo.dev). `bun run verify` runs what CI runs.
- Lint and format for JS: [Ultracite](https://www.ultracite.ai) (`bun run check` / `bun run fix`).
- Versioning: [changesets](https://github.com/changesets/changesets). See `RELEASE.md`.
- Contributing: `CONTRIBUTING.md`. Agents: `AGENTS.md`.

```sh
bun install
cargo build -p gozo
./target/debug/gozo -C path/to/go/project doctor
```

## Roadmap

- v0.1 (this): every command above, docker and kubernetes targets, JSON everywhere.
- v0.2: deploy adapters as plugins (`gozo-deploy-<target>` on PATH), `gozo shell`.
- v0.3: MCP server (`gozo mcp`) exposing the `api` resources and commands to agents.
- v0.4: remote env sources (Kubernetes secrets, SSM) behind `gozo env pull`.

## License

MIT
