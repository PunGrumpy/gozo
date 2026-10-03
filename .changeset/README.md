# Changesets

Every user-facing change ships with a changeset. Run `bunx changeset` from the repository root, pick `gozo-cli`, choose patch, minor or major, and describe the change for the changelog. Commit the generated file in this directory with your PR. The “Changeset status” workflow comments on each PR whether one is present.

On merge to `main`, the Release workflow follows the [changesets automation guide](https://changesets.dev/guide/automating#how-do-i-run-the-version-and-publish-commands):

1. `changesets/action/select-mode` decides between `version`, `publish` and `none`
2. In `version` mode, `changesets/action/version` runs `bun run changeset:version`, which bumps `packages/gozo/package.json`, syncs `Cargo.toml` and writes `CHANGELOG.md`, then opens or updates the “chore: Version packages” pull request
3. In `publish` mode, after that PR merged, binaries are built for every platform. `changesets/action/publish` then runs `bun run changeset:publish`, which publishes the platform packages and `gozo-cli`, pushes the `gozo-cli@<version>` tag and creates the GitHub release with the binaries attached

[RELEASE.md](../RELEASE.md) has the details.
