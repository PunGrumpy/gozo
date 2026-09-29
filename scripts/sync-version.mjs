#!/usr/bin/env node
// Copies packages/gozo/package.json's version (owned by changesets) into the
// Cargo workspace so `gozo --version`, crates.io and npm always agree.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u;
const WORKSPACE_VERSION =
  /(?<head>\[workspace\.package\][^[]*?\nversion\s*=\s*")(?<version>[^"]+)(?<tail>")/u;

const pkg = JSON.parse(
  readFileSync(path.join(root, "packages/gozo/package.json"), "utf-8")
);
const { version } = pkg;
if (!SEMVER.test(version)) {
  console.error(
    `sync-version: refusing to write invalid version ${JSON.stringify(version)}`
  );
  process.exit(1);
}

const cargoToml = path.join(root, "Cargo.toml");
const before = readFileSync(cargoToml, "utf-8");
const match = WORKSPACE_VERSION.exec(before);
if (!match?.groups) {
  console.error(
    "sync-version: could not find [workspace.package] version in Cargo.toml"
  );
  process.exit(1);
}
if (match.groups.version === version) {
  console.log(`sync-version: Cargo.toml already at ${version}`);
  process.exit(0);
}

writeFileSync(
  cargoToml,
  before.replace(WORKSPACE_VERSION, `$<head>${version}$<tail>`)
);
// Refresh Cargo.lock entries for the workspace members without touching dependencies.
// oxlint-disable-next-line sonarjs/no-os-command-from-path -- cargo comes from the developer's toolchain
execFileSync("cargo", ["update", "--workspace", "--offline"], {
  cwd: root,
  stdio: "inherit",
});
console.log(`sync-version: Cargo.toml -> ${version}`);
