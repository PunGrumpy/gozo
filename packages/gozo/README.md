# gozo

The missing developer experience layer for Go. One binary wraps the `go` command with the workflow of Vercel CLI (`dev`, `env`, `link`, `deploy`, `logs`, `rollback`) and project health (`doctor`, `check`, `test`, `build`). Every command has JSON output for coding agents.

```sh
npx gozo-cli        # interactive menu in a Go project
npm i -g gozo-cli   # or install globally
gozo doctor --json
```

This npm package is a thin launcher. The native binary for your platform installs as an optional dependency such as `gozo-cli-linux-64` or `gozo-cli-darwin-arm64`. You can also install with `curl -fsSL https://raw.githubusercontent.com/PunGrumpy/gozo/main/install.sh | sh` or `cargo install gozo`.

Source and documentation live in the [gozo repository](https://github.com/PunGrumpy/gozo).
