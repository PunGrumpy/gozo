#!/usr/bin/env node
// Turns built binaries into npm platform packages (gozo-linux-64, ...), the
// way vercel/turborepo's turbo-releaser does for `turbo`.
//
//   node scripts/package-native.mjs --version 0.2.0 --artifacts ./release-artifacts --out ./packages/gozo/npm
//
// Expects <artifacts>/gozo-<os>-<arch>/gozo[.exe] where os/arch follow the
// release asset naming (linux|darwin|windows, x86_64|aarch64).
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const EXECUTABLE = 0o755;
const JSON_INDENT = 2;

// asset name -> npm package facts. `os` follows Node's process.platform values
// npm checks against; linux binaries also run on Android (Termux).
const PLATFORMS = [
  {
    asset: "gozo-linux-x86_64",
    cpu: ["x64"],
    exe: "",
    name: "gozo-linux-64",
    os: ["linux", "android"],
  },
  {
    asset: "gozo-linux-aarch64",
    cpu: ["arm64"],
    exe: "",
    name: "gozo-linux-arm64",
    os: ["linux", "android"],
  },
  {
    asset: "gozo-darwin-x86_64",
    cpu: ["x64"],
    exe: "",
    name: "gozo-darwin-64",
    os: ["darwin"],
  },
  {
    asset: "gozo-darwin-aarch64",
    cpu: ["arm64"],
    exe: "",
    name: "gozo-darwin-arm64",
    os: ["darwin"],
  },
  {
    asset: "gozo-windows-x86_64",
    cpu: ["x64"],
    exe: ".exe",
    name: "gozo-windows-64",
    os: ["win32"],
  },
];

const parseArgs = (argv) => {
  const args = {};
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg.startsWith("--")) {
      args[arg.slice(2)] = argv[i + 1];
      i += 1;
    }
  }
  return args;
};

const args = parseArgs(process.argv.slice(2));
const { version } = args;
const artifacts = path.resolve(args.artifacts ?? "release-artifacts");
const out = path.resolve(args.out ?? "packages/gozo/npm");
if (!version) {
  console.error(
    "usage: package-native.mjs --version X.Y.Z [--artifacts DIR] [--out DIR]"
  );
  process.exit(2);
}

const template = path.join(root, "packages/gozo/template");

const generatePackage = (platform) => {
  const bin = path.join(artifacts, platform.asset, `gozo${platform.exe}`);
  if (!existsSync(bin)) {
    console.warn(`skip ${platform.name}: ${bin} not found`);
    return false;
  }
  const dir = path.join(out, platform.name);
  rmSync(dir, { force: true, recursive: true });
  mkdirSync(path.join(dir, "bin"), { recursive: true });
  const target = path.join(dir, "bin", `gozo${platform.exe}`);
  copyFileSync(bin, target);
  chmodSync(target, EXECUTABLE);
  copyFileSync(path.join(template, "README.md"), path.join(dir, "README.md"));
  copyFileSync(path.join(template, "LICENSE"), path.join(dir, "LICENSE"));
  const manifest = {
    bugs: "https://github.com/PunGrumpy/gozo/issues",
    cpu: platform.cpu,
    description: `The ${platform.asset.replace("gozo-", "")} binary for gozo, the missing developer experience layer for Go.`,
    homepage: "https://github.com/PunGrumpy/gozo",
    license: "MIT",
    name: platform.name,
    os: platform.os,
    preferUnplugged: true,
    repository: "https://github.com/PunGrumpy/gozo",
    version,
  };
  writeFileSync(
    path.join(dir, "package.json"),
    `${JSON.stringify(manifest, null, JSON_INDENT)}\n`
  );
  console.log(
    `generated ${platform.name}@${version} -> ${path.relative(root, dir)}`
  );
  return true;
};

let generated = 0;
for (const platform of PLATFORMS) {
  if (generatePackage(platform)) {
    generated += 1;
  }
}
if (generated === 0) {
  console.error("no platform packages generated");
  process.exit(1);
}
