# Changesets

Every user-facing change ships with a changeset. Run `bunx changeset` from the repository root, pick `gozo`, choose patch / minor / major, and describe the change for the changelog. Commit the generated file in this directory with your PR. The "Changeset status" workflow comments on each PR whether one is present.

On merge to `main`, the Release workflow follows https://changesets.dev/guide/automating#how-do-i-run-the-version-and-publish-commands:

1. `changesets/action/select-mode` decides between `version`, `publish` and `none`.
2. `version`: `changesets/action/version` runs `bun run version-packages` (bumps `packages/gozo/package.json`, syncs `Cargo.toml`, writes `CHANGELOG.md`) and opens or updates the "chore: Version packages" pull request.
3. `publish` (that PR merged): binaries are built for every platform, then `changesets/action/publish` runs `bun run release:publish`, which publishes the platform packages and `gozo`, pushes the `gozo@<version>` tag and creates the GitHub release. The binaries are attached to that release.

See RELEASE.md for details.
