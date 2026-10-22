// Builds the `ledgerly-mcp` companion so `tauri dev` has it next to the debug
// app. `tauri build` compiles the same bin with the rest of the crate and
// bundles it next to the release app, so no staging step is needed here.
//
//   node scripts/build-mcp-sidecar.mjs [--release] [--target <triple>]

import { execFileSync } from "node:child_process";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");

const args = process.argv.slice(2);
const release = args.includes("--release");
const targetIndex = args.indexOf("--target");
const requestedTarget = targetIndex >= 0 ? args[targetIndex + 1] : undefined;

const cargoArgs = ["build", "--manifest-path", "src-tauri/Cargo.toml", "--bin", "ledgerly-mcp"];
if (release) cargoArgs.push("--release");
if (requestedTarget) cargoArgs.push("--target", requestedTarget);

execFileSync("cargo", cargoArgs, { cwd: ROOT, stdio: "inherit" });
