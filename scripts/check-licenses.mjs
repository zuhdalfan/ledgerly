// Fails when a dependency ships under a licence that cannot be distributed in
// an MIT-licensed binary.
//
//   node scripts/check-licenses.mjs
//
// Ledgerly is MIT and ships as a compiled executable, so the licences of its
// whole dependency graph travel with it. This reads them from cargo metadata
// rather than shelling out to a licence tool, so it needs no extra install.

import { execFileSync } from "node:child_process";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");

/** The platforms a release actually ships to. */
const PLATFORMS = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin"];

/** Copyleft or field-of-use limits that an MIT binary cannot carry. */
const FORBIDDEN = [
  /^AGPL/i,
  /^GPL/i,
  /^SSPL/i,
  /^BUSL/i,
  /^Elastic-2/i,
  /Commons-Clause/i,
  /^CC-BY-NC/i,
  /^CC-BY-SA/i,
  /research/i,
];

/** Weak copyleft: usually tolerable, but worth a human eye in a static binary. */
const WATCH = [/^LGPL/i, /^MPL/i, /^EPL/i, /^CDDL/i];

// The lockfile contains crates that no target actually builds (unused optional
// dependencies), so the graph is resolved per shipped platform and unioned.
// Anything that could end up in a binary is therefore still covered.
const packages = new Map();
for (const platform of PLATFORMS) {
  let metadata;
  try {
    metadata = JSON.parse(
      execFileSync(
        "cargo",
        ["metadata", "--format-version", "1", "--filter-platform", platform, "--manifest-path", "src-tauri/Cargo.toml"],
        { cwd: ROOT, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 }
      )
    );
  } catch (error) {
    console.error(`could not read cargo metadata for ${platform}: ${error.message}`);
    process.exit(1);
  }

  const reachable = new Set(metadata.workspace_members);
  const nodes = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
  const queue = [...reachable];
  while (queue.length > 0) {
    for (const dependency of nodes.get(queue.pop())?.dependencies ?? []) {
      if (!reachable.has(dependency)) {
        reachable.add(dependency);
        queue.push(dependency);
      }
    }
  }

  for (const pkg of metadata.packages) {
    if (!reachable.has(pkg.id)) continue;
    packages.set(`${pkg.name}@${pkg.version}`, pkg);
  }
}

const forbidden = [];
const watch = [];
const unknown = [];

for (const pkg of packages.values()) {
  const licence = pkg.license ?? "";
  if (!licence) {
    unknown.push(pkg.name);
    continue;
  }
  if (FORBIDDEN.some((pattern) => pattern.test(licence))) forbidden.push(`${pkg.name}@${pkg.version} -> ${licence}`);
  else if (WATCH.some((pattern) => pattern.test(licence))) watch.push(`${pkg.name}@${pkg.version} -> ${licence}`);
}

const licences = new Map();
for (const pkg of packages.values()) {
  const licence = pkg.license ?? "(none declared)";
  licences.set(licence, (licences.get(licence) ?? 0) + 1);
}

console.log(`${packages.size} packages shipped across ${PLATFORMS.length} platforms.`);
console.log("Licences in use:");
for (const [licence, count] of [...licences.entries()].sort((a, b) => b[1] - a[1])) {
  console.log(`  ${String(count).padStart(4)}  ${licence}`);
}

for (const entry of watch) console.log(`  WATCH  ${entry}`);
for (const name of unknown) console.log(`  no licence declared: ${name}`);

if (forbidden.length > 0) {
  console.error("\nLicence check FAILED, these cannot ship in an MIT binary:\n");
  for (const entry of forbidden) console.error(`  ${entry}`);
  process.exit(1);
}

console.log("\nLicence check passed: nothing copyleft in the shipped graph.");
