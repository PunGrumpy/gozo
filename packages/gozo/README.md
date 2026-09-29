# gozo

The missing developer experience layer for Go: one binary that wraps the `go` command with the workflow of Vercel CLI (`dev`, `env`, `link`, `deploy`, `logs`, `rollback`) plus project health (`doctor`, `check`, `test`, `build`), and JSON output on every command for coding agents.

```sh
npx gozo            # interactive menu in a Go project
npm i -g gozo       # or install globally
gozo doctor --json
```

This npm package is a thin launcher. The native binary for your platform is installed as an optional dependency (`gozo-linux-64`, `gozo-darwin-arm64`, ...). Other install options: `curl -fsSL https://raw.githubusercontent.com/pungrumpy/gozo/main/install.sh | sh` or `cargo install gozo`.

Source and docs: https://github.com/pungrumpy/gozo
