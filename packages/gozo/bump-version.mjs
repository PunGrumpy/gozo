#!/usr/bin/env node
// Run at publish time. The committed package.json has no optionalDependencies
// so the workspace installs before any platform package exists on npm.
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const JSON_INDENT = 2;
const file = path.join(import.meta.dirname, "package.json");
const pkg = JSON.parse(readFileSync(file, "utf-8"));

const platformPackages = [
  "gozo-darwin-64",
  "gozo-darwin-arm64",
  "gozo-linux-64",
  "gozo-linux-arm64",
  "gozo-windows-64",
];

pkg.optionalDependencies = Object.fromEntries(
  platformPackages.toSorted().map((name) => [name, pkg.version])
);

writeFileSync(file, `${JSON.stringify(pkg, null, JSON_INDENT)}\n`);
console.log(
  `pinned ${platformPackages.length} platform packages to ${pkg.version}`
);
