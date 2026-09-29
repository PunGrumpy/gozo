#!/usr/bin/env node
// Syncs the changesets-owned version into Cargo.toml and Cargo.lock without
// cargo, so it works on a bare CI runner with no toolchain or registry cache.
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
const toml = readFileSync(cargoToml, "utf-8");
const match = WORKSPACE_VERSION.exec(toml);
if (!match?.groups) {
  console.error(
    "sync-version: could not find [workspace.package] version in Cargo.toml"
  );
  process.exit(1);
}
const current = match.groups.version;
if (current === version) {
  console.log(`sync-version: Cargo.toml already at ${version}`);
  process.exit(0);
}
writeFileSync(
  cargoToml,
  toml.replace(WORKSPACE_VERSION, `$<head>${version}$<tail>`)
);

// Only workspace members lack a `source` line in Cargo.lock.
const cargoLock = path.join(root, "Cargo.lock");
const lock = readFileSync(cargoLock, "utf-8");
const blocks = lock.split("\n[[package]]\n");
let bumped = 0;
const patched = blocks
  .map((block, index) => {
    if (index === 0 || /^source = /mu.test(block)) {
      return block;
    }
    const versionLine = new RegExp(
      `^version = "${current.replaceAll(".", "\\.")}"$`,
      "mu"
    );
    if (!versionLine.test(block)) {
      return block;
    }
    bumped += 1;
    return block.replace(versionLine, `version = "${version}"`);
  })
  .join("\n[[package]]\n");
if (bumped === 0) {
  console.error(
    `sync-version: no workspace crates at ${current} found in Cargo.lock`
  );
  process.exit(1);
}
writeFileSync(cargoLock, patched);
console.log(
  `sync-version: Cargo.toml and ${bumped} Cargo.lock entries -> ${version}`
);
