// GENERATED FILE — do not edit.
//
// Owned by tools/agent-guide/repo-check.mjs in the GaugeWright repository and
// rendered into each active repository by tools/agent-guide.mjs. Edit it there
// and re-render; a local edit fails this repository's gate.
//
// This check is deliberately self-contained: it verifies the shared agent-guide
// block against a digest carried in the file itself, so a public repository can
// run it without access to the private company repository. Whether that digest
// is still the current one is checked from GaugeWright, which owns it.

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";

const EXPECTED_DIGEST = "e569e544faca09a8ba38c163405c6c4127a22ec5ff405927e9227b166833262b";
const BEGIN = "<!-- BEGIN GAUGEWRIGHT SHARED AGENT GUIDE";
const END = "<!-- END GAUGEWRIGHT SHARED AGENT GUIDE -->";

const root = path.resolve(process.argv[2] ?? process.cwd());
const failures = [];
const fail = (message) => failures.push(message);

const guidePath = path.join(root, "AGENTS.md");
const aliasPath = path.join(root, "CLAUDE.md");

if (!fs.existsSync(guidePath)) {
  fail("AGENTS.md is missing. Every active repository carries one agent guide at that path.");
} else {
  const guide = fs.readFileSync(guidePath, "utf8");
  const begins = guide.split(BEGIN).length - 1;
  const ends = guide.split(END).length - 1;

  if (begins !== 1 || ends !== 1) {
    fail(
      `AGENTS.md must contain exactly one shared agent-guide block; found ${begins} begin and ${ends} end markers.`,
    );
  } else {
    const startIndex = guide.indexOf(BEGIN);
    const markerEnd = guide.indexOf("-->", startIndex);
    const endIndex = guide.indexOf(END);

    if (markerEnd === -1 || endIndex < startIndex) {
      fail("The shared agent-guide block markers in AGENTS.md are malformed.");
    } else {
      const marker = guide.slice(startIndex, markerEnd);
      const declared = /sha256:([0-9a-f]{64})/.exec(marker)?.[1];
      const body = guide.slice(markerEnd + 3, endIndex).replace(/\r\n/g, "\n").trim();
      const actual = crypto.createHash("sha256").update(body, "utf8").digest("hex");

      if (!declared) {
        fail("The shared agent-guide block does not declare a sha256 digest in its begin marker.");
      } else if (declared !== actual) {
        fail(
          "The shared agent-guide block in AGENTS.md was edited locally: its content does not match the " +
            `digest it declares (declared ${declared.slice(0, 12)}…, actual ${actual.slice(0, 12)}…). ` +
            "Shared guidance is owned by the GaugeWright repository; change it there and re-render.",
        );
      } else if (declared !== EXPECTED_DIGEST) {
        fail(
          `The shared agent-guide block is at ${declared.slice(0, 12)}… but this checker expects ` +
            `${EXPECTED_DIGEST.slice(0, 12)}…. Re-render both from the GaugeWright repository.`,
        );
      }
    }
  }
}

if (!fs.existsSync(aliasPath)) {
  fail("CLAUDE.md is missing. It must be a symbolic link to AGENTS.md so every tool reads one file.");
} else {
  let stat;
  try {
    stat = fs.lstatSync(aliasPath);
  } catch {
    stat = null;
  }

  if (!stat?.isSymbolicLink()) {
    fail(
      "CLAUDE.md is a regular file. It must be a symbolic link to AGENTS.md — a copy is exactly how the " +
        "two guides drifted apart before.",
    );
  } else {
    const target = fs.readlinkSync(aliasPath);
    if (target !== "AGENTS.md") {
      fail(`CLAUDE.md links to ${target} rather than AGENTS.md.`);
    }
  }
}

// One guide, one place. A nested AGENTS.md or CLAUDE.md would give tools
// operating in that subtree separate guidance, which is the drift this check
// exists to prevent, so any guide filename outside the root pair is rejected.
const GUIDE_NAMES = new Set(["AGENTS.MD", "CLAUDE.MD"]);
const IGNORED_DIRS = new Set([".git", "node_modules", "target", "dist", "build"]);

const trackedPaths = () => {
  // git ls-files also keeps submodule contents out of scope, which is what we
  // want: a consumed repository carries its own guide and is not this one's.
  const listed = spawnSync("git", ["-C", root, "ls-files", "-z"], { encoding: "utf8" });
  if (listed.status === 0) {
    return listed.stdout.split("\0").filter(Boolean);
  }

  // Not a git checkout (an exported tree, say). Walk it instead.
  const found = [];
  const walk = (dir, prefix) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        if (!IGNORED_DIRS.has(entry.name)) walk(path.join(dir, entry.name), relative);
      } else {
        found.push(relative);
      }
    }
  };
  walk(root, "");
  return found;
};

for (const relative of trackedPaths()) {
  const name = path.posix.basename(relative);
  if (!GUIDE_NAMES.has(name.toUpperCase())) continue;
  if (relative === "AGENTS.md" || relative === "CLAUDE.md") continue;
  fail(
    `${relative} is a second agent guide. This repository carries one guide, AGENTS.md at its root, ` +
      "aliased only by the root CLAUDE.md symlink; guidance for a subtree belongs in that one file.",
  );
}

if (failures.length > 0) {
  console.error("agent guide check FAILED:");
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}

console.log("agent guide check passed");
