import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import test from "node:test";
import { isAbsolute, join, relative, resolve } from "node:path";

import * as publicApi from "../dist/index.js";
import { resolvePleumBinaryForRuntime } from "../dist/resolve-binary.js";

const supportedPlatforms = [
  ["darwin", "arm64", "@aaif/pleum-binary-darwin-arm64", "pleum"],
  ["darwin", "x64", "@aaif/pleum-binary-darwin-x64", "pleum"],
  ["linux", "arm64", "@aaif/pleum-binary-linux-arm64", "pleum"],
  ["linux", "x64", "@aaif/pleum-binary-linux-x64", "pleum"],
  ["win32", "x64", "@aaif/pleum-binary-win32-x64", "pleum.exe"],
];

function setPleumBinary(t, value) {
  const original = process.env.PLEUM_BINARY;
  process.env.PLEUM_BINARY = value;
  t.after(() => {
    if (original === undefined) {
      delete process.env.PLEUM_BINARY;
    } else {
      process.env.PLEUM_BINARY = original;
    }
  });
}

for (const [
  platform,
  arch,
  packageName,
  executableName,
] of supportedPlatforms) {
  test(`resolves ${platform}-${arch}`, () => {
    let resolvedSpecifier;
    let checkedPath;
    const fixturePackageRoot = join("/fixtures", packageName);

    const result = resolvePleumBinaryForRuntime(platform, arch, {
      resolvePackageJson(specifier) {
        resolvedSpecifier = specifier;
        return join(fixturePackageRoot, "package.json");
      },
      isFile(path) {
        checkedPath = path;
        return true;
      },
    });

    assert.equal(resolvedSpecifier, `${packageName}/package.json`);
    assert.equal(result, resolve(fixturePackageRoot, "bin", executableName));
    assert.equal(checkedPath, result);
    assert.equal(isAbsolute(result), true);
  });
}

test("exports only the public resolver from the package root", () => {
  assert.deepEqual(Object.keys(publicApi), ["resolvePleumBinary"]);
});

test("uses PLEUM_BINARY as an explicit override", (t) => {
  const directory = mkdtempSync(join(tmpdir(), "pleum-acp-override-"));
  const binaryPath = join(directory, "pleum");
  writeFileSync(binaryPath, "");
  setPleumBinary(t, relative(process.cwd(), binaryPath));

  t.after(() => {
    rmSync(directory, { recursive: true, force: true });
  });

  assert.equal(publicApi.resolvePleumBinary(), binaryPath);
});

test("rejects an invalid PLEUM_BINARY override", (t) => {
  setPleumBinary(t, "missing-pleum-binary");

  assert.throws(
    () => publicApi.resolvePleumBinary(),
    /PLEUM_BINARY does not point to a file/,
  );
});

test("reports unsupported platform and architecture combinations", () => {
  assert.throws(
    () =>
      resolvePleumBinaryForRuntime("freebsd", "x64", {
        resolvePackageJson() {
          throw new Error("should not resolve a package");
        },
        isFile() {
          return false;
        },
      }),
    /No Pleum npm binary is available for freebsd-x64/,
  );
});

test("reports a missing optional platform package", () => {
  assert.throws(
    () =>
      resolvePleumBinaryForRuntime("linux", "x64", {
        resolvePackageJson() {
          throw new Error("module not found");
        },
        isFile() {
          return false;
        },
      }),
    /Pleum binary package @aaif\/pleum-binary-linux-x64 is not installed/,
  );
});

test("reports a missing executable in an installed platform package", () => {
  assert.throws(
    () =>
      resolvePleumBinaryForRuntime("darwin", "arm64", {
        resolvePackageJson() {
          return join(
            "/fixtures",
            "@aaif/pleum-binary-darwin-arm64/package.json",
          );
        },
        isFile() {
          return false;
        },
      }),
    /Pleum executable from @aaif\/pleum-binary-darwin-arm64 was not found/,
  );
});
