# Release documentation

gozo is versioned with [changesets](https://changesets.dev). It ships the same way as `turbo`: the `gozo-cli` launcher on npm, one `gozo-cli-<os>-<arch>` package per platform, a GitHub release with tarballs, and `cargo install gozo` from source.

The automation follows the [changesets automation guide](https://changesets.dev/guide/automating#how-do-i-run-the-version-and-publish-commands), using the `changesets/action@v2` sub-actions and npm trusted publishing.

## Flow

1. Every PR with a user-facing change adds a changeset with `bunx changeset`. The `changeset-status.yml` workflow comments on the PR whether one is present, without blocking it.
2. On push to `main`, `release.yml` runs `changesets/action/select-mode`, which picks one of three modes:
   - `version`, when changesets are pending: `changesets/action/version` runs `bun run changeset:version` and opens or updates the “chore: Version packages” PR. That script runs `changeset version`, then `scripts/sync-version.mjs` to patch the version into `Cargo.toml` and `Cargo.lock`, then `bun install --lockfile-only`
   - `publish`, after that PR merged: the `build` matrix compiles `gozo` for linux, darwin and windows on x86_64 and aarch64, then the `publish` job runs
   - `none`: nothing to do
3. The `publish` job takes four steps:
   1. `scripts/package-native.mjs` turns the binaries into `gozo-cli-<os>-<arch>` packages
   2. `packages/gozo/bump-version.mjs` pins the launcher’s `optionalDependencies`
   3. `changesets/action/publish` runs `bun run changeset:publish`: `scripts/publish-native.mjs` publishes the platform packages, then `changeset publish` publishes `gozo-cli`, pushes the `gozo-cli@<version>` tag and creates the GitHub release with the changelog
   4. `gh release upload` attaches `gozo-<os>-<arch>.tar.gz` and `.sha256` to that release, which `install.sh` and `gozo update` download

## Manual dry run

Run the **Release** workflow from the Actions tab with `dry_run` checked. It builds and packages everything but skips npm publish and the GitHub release.

## Single source of truth

`packages/gozo/package.json` owns the version. `scripts/sync-version.mjs` derives `Cargo.toml` from it, and the publish job refuses to run if they differ.

## One-time setup

- In the repository settings under Actions, enable “Allow GitHub Actions to create and approve pull requests”
- On npm, configure [trusted publishing](https://docs.npmjs.com/trusted-publishers) for `gozo-cli` and each `gozo-cli-<os>-<arch>` package, pointing at `.github/workflows/release.yml`. If a package cannot use trusted publishing, write `NPM_TOKEN` into `~/.npmrc` in the publish job. The job upgrades npm to 11.5 or newer, which trusted publishing requires
- Optionally install the [changeset bot](https://github.com/apps/changeset-bot)
