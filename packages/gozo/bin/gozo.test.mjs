// Launcher test: with GOZO_BINARY_PATH set to any executable, the launcher must
// exec it and propagate the exit code. Run with `node --test bin/gozo.test.mjs`.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, before, test } from "node:test";

const EXECUTABLE = 0o755;
const FAKE_EXIT_CODE = 3;
const launcher = path.join(import.meta.dirname, "gozo");
const isWindows = process.platform === "win32";

let dir = "";
let fake = "";

before(() => {
  dir = mkdtempSync(path.join(os.tmpdir(), "gozo-launcher-"));
  fake = path.join(dir, isWindows ? "fake.cmd" : "fake.sh");
  const script = isWindows
    ? `@echo fake %*\r\n@exit /b ${FAKE_EXIT_CODE}\r\n`
    : `#!/bin/sh\necho fake "$@"\nexit ${FAKE_EXIT_CODE}\n`;
  writeFileSync(fake, script, { mode: EXECUTABLE });
});

after(() => {
  rmSync(dir, { force: true, recursive: true });
});

test("execs GOZO_BINARY_PATH and propagates the exit code", () => {
  const result = spawnSync(process.execPath, [launcher, "doctor", "--json"], {
    encoding: "utf-8",
    env: { ...process.env, GOZO_BINARY_PATH: fake },
  });
  assert.equal(result.status, FAKE_EXIT_CODE, result.stderr);
  assert.match(result.stdout, /fake doctor --json/u);
});

test("fails clearly when GOZO_BINARY_PATH points nowhere", () => {
  const result = spawnSync(process.execPath, [launcher], {
    encoding: "utf-8",
    env: { ...process.env, GOZO_BINARY_PATH: path.join(dir, "nope") },
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /GOZO_BINARY_PATH/u);
});

test("uses the locally built binary in a source checkout", () => {
  const repoRoot = path.join(import.meta.dirname, "..", "..", "..");
  const built = ["release", "debug"]
    .map((profile) =>
      path.join(repoRoot, "target", profile, isWindows ? "gozo.exe" : "gozo")
    )
    .find((candidate) => existsSync(candidate));
  if (!built) {
    // Nothing built yet; CI covers this after `cargo build`.
    return;
  }
  const env = { ...process.env };
  delete env.GOZO_BINARY_PATH;
  const result = spawnSync(process.execPath, [launcher, "--version"], {
    encoding: "utf-8",
    env,
  });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /^gozo \d+\.\d+\.\d+/u);
});
