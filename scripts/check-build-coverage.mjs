#!/usr/bin/env node
// GENERATED. The company owns this file at tools/shared-checks/build-coverage.mjs
// in the GaugeWright repository and renders it into every active repository.
// Edit it there and re-render (`node tools/shared-checks.mjs --write`); a local
// edit fails this check, which is the point of rendering rather than copying.
//
// WHAT IT ENFORCES
//
// Every cargo workspace is compiled by something, every Cargo.lock is audited by
// something, and every package-lock.json is audited by something — or the
// repository declares the gap with a reason.
//
// Absence has no line number. A crate nothing compiles and a lockfile nothing
// audits look exactly like a crate and a lockfile: no gate fails, no diff shows
// it, and it surfaces when a release trips over it or an advisory lands on a
// path nobody was watching. Both happened in gaugedesk-src within a fortnight —
// `src-tauri` compiled by no per-change job and broken on trunk for a week, its
// lockfile carrying two high-severity advisories on the shipped desktop binary —
// and each was repaired one at a time while nothing stopped the next.
//
// The first dry run of this check across the ecosystem found the same shape
// again in gaugewright-cloud: `infra/sev-guest-quote`, in the attestation path,
// audited by nothing and compiled by nothing.
//
// WHAT IT DELIBERATELY DOES NOT ENFORCE
//
// "Is this npm tree built or tested?" is not here. It is not portable: whipplescript-src
// builds its trees through shell variables a matcher cannot resolve, and
// gaugewright-cloud builds its trees in a CI job rather than in check.sh. An
// obligation that reports false failures in two of seven repositories would be
// switched off, and a check nobody trusts is worse than no check.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SOURCE_DIGEST = "06132706503a942abe4356d15d15ff0db482aa3f42b78548ecaa9e6abfc74b01";
const PLACEHOLDER = "__EXPECTED_DIGEST__";
const POLICY_PATH = "scripts/build-coverage.policy.json";

/** Fail if this rendered copy has drifted from the source that owns it. */
function verifyRenderedCopy(selfPath) {
  const body = readFileSync(selfPath, "utf8").replace(
    `const SOURCE_DIGEST = "${SOURCE_DIGEST}";`,
    `const SOURCE_DIGEST = "${PLACEHOLDER}";`,
  );
  const actual = createHash("sha256").update(body, "utf8").digest("hex");
  if (actual === SOURCE_DIGEST) return null;
  return (
    "this file has been edited locally. It is generated from "
    + "tools/shared-checks/build-coverage.mjs in the GaugeWright repository; "
    + "change it there and re-render with `node tools/shared-checks.mjs --write`."
  );
}

function tracked(root, ...pathspecs) {
  const out = execFileSync("git", ["ls-files", "-z", "--", ...pathspecs], {
    cwd: root,
    encoding: "utf8",
  });
  return out.split("\0").filter(Boolean).sort();
}

/** `-not -path '<glob>'` exclusions on a `find` sweep, as matchers. */
function sweepExclusions(line) {
  return [...line.matchAll(/-not -path '([^']+)'/g)].map(([, glob]) => {
    const pattern = glob.replace(/[.+^${}()|[\]\\]/g, "\\$&").replaceAll("*", "[^\\0]*");
    return new RegExp(`^${pattern}$`);
  });
}

/**
 * Read coverage out of what `check.sh` actually runs, across the idioms the
 * seven repositories use. A repository audits its root lockfile with a bare
 * `cargo audit`, or names each one with `--file`; it sweeps npm trees with
 * `git ls-files` or `find`, audits a single root tree with a bare `npm audit`,
 * or names a tree with `--prefix`.
 */
export function analyze({ check, rustLocks, npmLocks, workspaces, exceptions = {} }) {
  const findings = [];
  const covered = new Set();
  const claim = (kind, path, isCovered) => {
    const key = `${kind}:${path}`;
    if (isCovered) covered.add(key);
    else if (!exceptions[key]) findings.push({ kind, path });
  };

  const bareCargoAudit = /^[ \t]*cargo audit[ \t]*$/m.test(check);
  for (const lock of rustLocks) {
    claim(
      "rust-audit",
      lock,
      check.includes(`cargo audit --file ${lock}`)
        || (bareCargoAudit && lock === "Cargo.lock"),
    );
  }

  // A root workspace is compiled by the ordinary cargo invocations; any other
  // workspace has to be named, which is exactly what `src-tauri` never was.
  const rootBuilt = /cargo (test|clippy|check)\b/.test(check);
  for (const manifest of workspaces) {
    const dir = dirname(manifest) === "." ? "." : dirname(manifest);
    claim(
      "rust-build",
      dir,
      dir === "." ? rootBuilt : check.includes(`--manifest-path ${manifest}`),
    );
  }

  const sweepLine = check
    .split("\n")
    .find((line) => /git ls-files ['"]\*package-lock\.json|find \. -name package-lock\.json/.test(line));
  const exclusions = sweepLine ? sweepExclusions(sweepLine) : [];
  const bareNpmAudit = /^[ \t]*npm audit\b/m.test(check);
  for (const lock of npmLocks) {
    const dir = dirname(lock) === "." ? "." : dirname(lock);
    const sweptIn = Boolean(sweepLine) && !exclusions.some((rx) => rx.test(`./${lock}`));
    const named = new RegExp(
      `npm --prefix ["']?${dir.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}["']? audit`,
    ).test(check);
    claim("npm-audit", dir, sweptIn || named || (bareNpmAudit && dir === "."));
  }

  // An exception for something covered, or for something that no longer exists,
  // is a claim that stopped being true with nobody rechecking.
  const universe = new Set([
    ...rustLocks.map((l) => `rust-audit:${l}`),
    ...workspaces.map((m) => `rust-build:${dirname(m) === "." ? "." : dirname(m)}`),
    ...npmLocks.map((l) => `npm-audit:${dirname(l) === "." ? "." : dirname(l)}`),
  ]);
  const stale = [];
  for (const key of Object.keys(exceptions)) {
    if (!universe.has(key)) stale.push({ key, why: "names nothing in the tree" });
    else if (covered.has(key)) stale.push({ key, why: "is covered by check.sh" });
  }
  return { findings, stale, obligations: universe.size };
}

const DESCRIBE = {
  "rust-audit": (p) => `${p} is audited by nothing (expected: cargo audit --file ${p})`,
  "rust-build": (p) =>
    `the cargo workspace at ${p} is compiled by nothing `
    + `(expected: cargo check --manifest-path ${p === "." ? "Cargo.toml" : `${p}/Cargo.toml`})`,
  "npm-audit": (p) => `${p}/package-lock.json is audited by nothing`,
};

export function readPolicy(root) {
  const path = resolve(root, POLICY_PATH);
  if (!existsSync(path)) return {};
  return JSON.parse(readFileSync(path, "utf8")).exceptions ?? {};
}

async function main() {
  const selfPath = fileURLToPath(import.meta.url);
  const root = resolve(dirname(selfPath), "..");
  const drift = verifyRenderedCopy(selfPath);
  if (drift) {
    console.error(`FAIL  ${drift}`);
    process.exitCode = 1;
    return;
  }

  const exceptions = readPolicy(root);
  const workspaces = tracked(root, "*Cargo.toml").filter((manifest) =>
    /^\[workspace\]$/m.test(readFileSync(resolve(root, manifest), "utf8")),
  );
  const { findings, stale, obligations } = analyze({
    check: readFileSync(resolve(root, "scripts/check.sh"), "utf8"),
    rustLocks: tracked(root, "*Cargo.lock"),
    npmLocks: tracked(root, "*package-lock.json"),
    workspaces,
    exceptions,
  });

  for (const { kind, path } of findings) console.error(`FAIL  ${DESCRIBE[kind](path)}`);
  for (const { key, why } of stale) {
    console.error(`FAIL  the declared exception ${key} ${why}; remove it`);
  }
  if (findings.length || stale.length) {
    console.error(`\nCover it in scripts/check.sh, or declare it in ${POLICY_PATH} with the lane that does.`);
    process.exitCode = 1;
    return;
  }

  const declared = Object.keys(exceptions).length;
  console.log(
    `build coverage check passed: ${obligations} obligations, `
      + `${declared} declared exception${declared === 1 ? "" : "s"}.`,
  );
}

const invoked = process.argv[1]
  && import.meta.url === pathToFileURL(resolve(process.argv[1])).href;
if (invoked) await main();
