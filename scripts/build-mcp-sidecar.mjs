// Builds the `ledgerly-mcp` companion and copies it under the name Tauri expects
// for a sidecar, so `tauri build` can bundle it with the app.
//
//   node scripts/build-mcp-sidecar.mjs [--release] [--target <triple>]
//
// Tauri looks for `binaries/<name>-<target-triple>[.exe]`; this puts it there.

import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");

const args = process.argv.slice(2);
const release = args.includes("--release");
const targetIndex = args.indexOf("--target");
const requestedTarget = targetIndex >= 0 ? args[targetIndex + 1] : undefined;

const hostTarget = execFileSync("rustc", ["-vV"], { encoding: "utf8" })
  .split("\n")
  .find((line) => line.startsWith("host:"))
  .slice("host:".length)
  .trim();

const target = requestedTarget ?? hostTarget;
const exe = target.includes("windows") ? ".exe" : "";

const cargoArgs = ["build", "--manifest-path", "src-tauri/Cargo.toml", "--bin", "ledgerly-mcp"];
if (release) cargoArgs.push("--release");
if (requestedTarget) cargoArgs.push("--target", requestedTarget);

// Building this bin also runs the app's `tauri-build` script, which checks that
// every `bundle.externalBin` already exists -- i.e. the very file we are about
// to produce. Point it at an empty list for this bootstrap build only.
const env = { ...process.env, TAURI_CONFIG: JSON.stringify({ bundle: { externalBin: [] } }) };
execFileSync("cargo", cargoArgs, { cwd: ROOT, stdio: "inherit", env });

const profile = release ? "release" : "debug";
const built = requestedTarget
  ? join(ROOT, "src-tauri", "target", target, profile, `ledgerly-mcp${exe}`)
  : join(ROOT, "src-tauri", "target", profile, `ledgerly-mcp${exe}`);

const destinationDir = join(ROOT, "src-tauri", "binaries");
mkdirSync(destinationDir, { recursive: true });
const destination = join(destinationDir, `ledgerly-mcp-${target}${exe}`);

copyFileSync(built, destination);
console.log(`Sidecar ready: ${destination}`);
