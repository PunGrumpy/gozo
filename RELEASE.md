# Release documentation

gozo is versioned with [changesets](https://changesets.dev) and published in the same shape as `turbo`: a launcher package `gozo` on npm plus one `gozo-<os>-<arch>` package per platform, a GitHub release with tarballs, and `cargo install gozo` from source.

The automation follows [changesets.dev/guide/automating](https://changesets.dev/guide/automating#how-do-i-run-the-version-and-publish-commands) with the `changesets/action@v2` sub-actions and npm trusted publishing.

## Flow

1. **Every PR with a user-facing change adds a changeset** (`bunx changeset`). `changeset-status.yml` comments on the PR whether one is present (non-blocking).
2. **On push to `main`**, `release.yml` runs `changesets/action/select-mode`:
   - `version` (changesets pending): `changesets/action/version` runs `bun run version-packages` and opens or updates the "chore: Version packages" PR. That script is `changeset version` + `scripts/sync-version.mjs` (copies the version into `Cargo.toml` and refreshes `Cargo.lock`) + `bun install --lockfile-only`.
   - `publish` (that PR was merged): the `build` matrix compiles `gozo` for linux/darwin/windows on x86_64 and aarch64, then `publish`:
     1. `scripts/package-native.mjs` turns the binaries into `gozo-<os>-<arch>` packages,
     2. `packages/gozo/bump-version.mjs` pins the launcher's `optionalDependencies`,
     3. `changesets/action/publish` runs `bun run release:publish` = `scripts/publish-native.mjs` (platform packages) then `changeset publish` (`gozo`, the `gozo@<version>` git tag, the GitHub release with the changelog),
     4. `gh release upload` attaches `gozo-<os>-<arch>.tar.gz` + `.sha256` to that release, which `install.sh` and `gozo update` consume.
   - `none`: nothing to do.

## Manual dry run

Run the **Release** workflow from the Actions tab with `dry_run` checked. It builds and packages everything but skips npm publish and the GitHub release.

## Single source of truth

`packages/gozo/package.json` owns the version. `Cargo.toml` is derived from it by `scripts/sync-version.mjs`; the publish job refuses to run if they differ.

## One-time setup

- Repository settings > Actions > General: enable "Allow GitHub Actions to create and approve pull requests".
- npm: configure [trusted publishing](https://docs.npmjs.com/trusted-publishers) for `gozo` and each `gozo-<os>-<arch>` package, pointing at `.github/workflows/release.yml`. If a package cannot use trusted publishing, write `NPM_TOKEN` into `~/.npmrc` in the publish job (the job also upgrades npm to 11.5+, which trusted publishing requires).
- Optionally install the [changeset bot](https://github.com/apps/changeset-bot).
