// Guards the product's core promise: Ledgerly never talks to the network.
//
//   node scripts/check-offline.mjs
//
// It fails if an HTTP/TLS client crate or browser networking API sneaks in.
// Run it in CI on every commit, and before every release.

import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");

/** Crates that would let the app open a network connection. */
const FORBIDDEN_CRATES = [
  "reqwest",
  "ureq",
  "isahc",
  "surf",
  "attohttpc",
  "minreq",
  "awc",
  "curl",
  "curl-sys",
  "hyper",
  "h2",
  "hyper-tls",
  "native-tls",
  "openssl",
  "openssl-sys",
  "rustls",
  "webpki-roots",
  "tungstenite",
  "tokio-tungstenite",
  "git2",
  "libssh2",
];

/** Browser APIs that would let the frontend reach out. */
const FORBIDDEN_JS_PATTERNS = [
  /\bfetch\s*\(/,
  /\bXMLHttpRequest\b/,
  /\bnew\s+WebSocket\b/,
  /\bnew\s+EventSource\b/,
  /\bnavigator\.sendBeacon\b/,
];

const problems = [];

function fail(message) {
  problems.push(message);
}

function walk(directory) {
  const out = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) out.push(...walk(path));
    else out.push(path);
  }
  return out;
}

// 1. The Rust dependency graph.
let tree = "";
try {
  tree = execFileSync(
    "cargo",
    ["tree", "--manifest-path", "src-tauri/Cargo.toml", "--prefix", "none", "--no-dedupe"],
    { cwd: ROOT, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }
  );
} catch (error) {
  fail(`could not read the dependency graph: ${error.message}`);
}

const crates = new Set();
for (const line of tree.split("\n")) {
  const match = line.trim().match(/^([a-z0-9_-]+) v/);
  if (match) crates.add(match[1]);
}

for (const forbidden of FORBIDDEN_CRATES) {
  if (crates.has(forbidden)) {
    fail(`the Rust dependency graph contains '${forbidden}'`);
  }
}

// 2. The frontend dependency list.
const pkg = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8"));
const jsDeps = Object.keys({ ...pkg.dependencies, ...pkg.devDependencies });
for (const name of jsDeps) {
  if (/^(axios|node-fetch|got|superagent|ky|undici|ws)$/.test(name)) {
    fail(`package.json depends on '${name}'`);
  }
}

// 3. Our own source, which is where a stray fetch() would appear.
const sources = [
  ...walk(join(ROOT, "src")),
  ...walk(join(ROOT, "src-tauri", "src")),
].filter((path) => /\.(svelte|ts|js|rs|html|css)$/.test(path));

for (const path of sources) {
  const text = readFileSync(path, "utf8");
  for (const pattern of FORBIDDEN_JS_PATTERNS) {
    if (pattern.test(text)) {
      fail(`${relative(ROOT, path)} contains ${pattern}`);
    }
  }
}

// 4. No remote URLs in *code*. Documentation links in comments are harmless,
//    but a URL in markup or script is how a CDN dependency sneaks in.
const urlPattern = /https?:\/\//;
const allowedHosts = /^https?:\/\/([a-z0-9-]+\.)*(github\.com|tauri\.app|svelte\.dev|vite\.dev|w3\.org)/i;
const isComment = (line) => /^\s*(\/\/|#|\*|\/\*|<!--)/.test(line);

for (const path of sources) {
  const text = readFileSync(path, "utf8");
  for (const [index, line] of text.split("\n").entries()) {
    if (isComment(line)) continue;
    if (urlPattern.test(line) && !allowedHosts.test(line.trim().match(/https?:\/\/\S+/)?.[0] ?? "")) {
      fail(`${relative(ROOT, path)}:${index + 1} references an external URL`);
    }
  }
}

if (problems.length > 0) {
  console.error("Offline check FAILED:\n");
  for (const problem of problems) console.error(`  - ${problem}`);
  console.error("\nLedgerly must never make a network request. Remove the above.");
  process.exit(1);
}

console.log(`Offline check passed: ${crates.size} crates, ${sources.length} source files, no network access.`);
