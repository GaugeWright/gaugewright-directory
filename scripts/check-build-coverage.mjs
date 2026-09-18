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
// WHERE THE ANSWER COMES FROM
//
// The obligations come from the tree: every tracked workspace manifest, every
// tracked lockfile. Nothing can be dropped from that list by forgetting to
// mention it, which is the whole point.
//
// The coverage comes from the build graph — the `covers` attribute of the
// targets in the repository's BUCK files (GaugeWright BUILD.md, DR-0124,
// stage 3). Until stage 3 it came from the text of scripts/check.sh, matched
// against the shell idioms seven repositories happened to use, and the cost of
// that is recorded in this file's own history: the moment gaugewright-cloud put
// `--no-fetch` behind a variable on `cargo audit`, both of its real audits
// became invisible here, and "audited by nothing" for a lockfile audited on the
// very next line is the worst kind of false report, because the obvious way to
// clear it is to declare an exception that is a lie. Trees built through shell
// variables could not be matched at all, which is why "is this npm tree built?"
// was never an obligation.
//
// A declaration has no idioms. `covers = ["rust-audit:Cargo.lock"]` says one
// thing, and Buck2 refuses to analyse a target whose `covers` names a file the
// target does not also declare in `srcs` — so a claim is about something the
// step actually opens. What this check adds is the other direction: that every
// obligation in the tree is claimed by some target.
//
// It reads the BUCK files as text rather than asking Buck2, deliberately. This
// check runs in every green bar, including on a CI runner and in a lone
// checkout, and a lone checkout is not a Buck2 project — the project root is the
// workspace that contains every repository. Making Buck2 a prerequisite of the
// bar to read a declaration out of a declarative file would buy nothing and cost
// the constraint that no stage introduces a hard host prerequisite.
//
// WHAT IT DELIBERATELY DOES NOT ENFORCE
//
// "Is this npm tree built or tested?" is not here, and neither is "is this
// target actually invoked by check.sh?" — the first because the obligation was
// never portable, the second because a target's reachability is a question
// about the script, which each repository's own gate answers.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SOURCE_DIGEST = "71b826bd12d78df786b2ba424f58ad62b493e8d76bc8ed8c14b91309d0a9d0db";
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

/**
 * The obligations the build graph claims: every string in a `covers = [...]`
 * attribute, across the repository's BUCK files.
 *
 * A list of string literals in a declarative file, and nothing else is read
 * from it. A target whose `covers` is computed rather than written out would be
 * invisible here — which is why the rule that consumes this attribute takes a
 * plain list of strings and Buck2 checks each one against `srcs`.
 */
export function declaredCoverage(buck) {
  const covered = new Set();
  for (const [, body] of buck.matchAll(/\bcovers\s*=\s*\[([^\]]*)\]/g)) {
    for (const [, entry] of body.matchAll(/["']([^"']+)["']/g)) covered.add(entry);
  }
  return covered;
}

export function analyze({ buck, rustLocks, npmLocks, workspaces, exceptions = {} }) {
  const declared = declaredCoverage(buck);
  const findings = [];
  const covered = new Set();
  const claim = (kind, path) => {
    const key = `${kind}:${path}`;
    if (declared.has(key)) covered.add(key);
    else if (!exceptions[key]) findings.push({ kind, path });
  };

  for (const lock of rustLocks) claim("rust-audit", lock);
  for (const manifest of workspaces) {
    claim("rust-build", dirname(manifest) === "." ? "." : dirname(manifest));
  }
  for (const lock of npmLocks) {
    claim("npm-audit", dirname(lock) === "." ? "." : dirname(lock));
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
    else if (covered.has(key)) stale.push({ key, why: "is covered by a target in BUCK" });
  }
  // A claim about an obligation the tree does not have is the same failure in
  // the other direction: a target declaring coverage of a lockfile that was
  // deleted reads as coverage until someone looks.
  for (const key of declared) {
    if (!universe.has(key)) stale.push({ key, why: "is claimed by a target in BUCK but names nothing in the tree" });
  }
  return { findings, stale, obligations: universe.size };
}

const DESCRIBE = {
  "rust-audit": (p) =>
    `${p} is audited by nothing (expected: a target in BUCK with covers = ["rust-audit:${p}"])`,
  "rust-build": (p) =>
    `the cargo workspace at ${p} is compiled by nothing `
    + `(expected: a target in BUCK with covers = ["rust-build:${p}"])`,
  "npm-audit": (p) =>
    `${p}/package-lock.json is audited by nothing `
    + `(expected: a target in BUCK with covers = ["npm-audit:${p}"])`,
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
  const buckFiles = tracked(root, "BUCK", "*/BUCK");
  const { findings, stale, obligations } = analyze({
    buck: buckFiles.map((f) => readFileSync(resolve(root, f), "utf8")).join("\n"),
    rustLocks: tracked(root, "*Cargo.lock"),
    npmLocks: tracked(root, "*package-lock.json"),
    workspaces,
    exceptions,
  });

  for (const { kind, path } of findings) console.error(`FAIL  ${DESCRIBE[kind](path)}`);
  for (const { key, why } of stale) {
    console.error(`FAIL  the declared coverage or exception ${key} ${why}; remove it`);
  }
  if (findings.length || stale.length) {
    console.error(`\nCover it with a target in BUCK, or declare it in ${POLICY_PATH} with the lane that does.`);
    process.exitCode = 1;
    return;
  }

  const declaredExceptions = Object.keys(exceptions).length;
  console.log(
    `build coverage check passed: ${obligations} obligation${obligations === 1 ? "" : "s"} `
      + `across ${buckFiles.length} BUCK file${buckFiles.length === 1 ? "" : "s"}, `
      + `${declaredExceptions} declared exception${declaredExceptions === 1 ? "" : "s"}.`,
  );
}

const invoked = process.argv[1]
  && import.meta.url === pathToFileURL(resolve(process.argv[1])).href;
if (invoked) await main();
