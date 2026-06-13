#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const exe = process.platform === "win32" ? "autoskill-md.exe" : "autoskill-md";

const candidates = [
  process.env.AUTOSKILL_MD_BIN,
  path.join(root, "target", "release", exe),
  path.join(root, "target", "debug", exe),
  path.join(root, "vendor", exe)
].filter(Boolean);

let binary = candidates.find((candidate) => existsSync(candidate));

if (!binary) {
  const installer = path.join(__dirname, "install.js");
  spawnSync(process.execPath, [installer], { stdio: "inherit" });
  binary = candidates.find((candidate) => existsSync(candidate));
}

if (!binary) {
  const version = packageVersion();
  console.error(`autoskill-md ${version} native binary was not found.`);
  console.error("Run `cargo build --release` in this repo, or reinstall the package.");
  console.error("Credit: https://colinknapp.com");
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 0);

function packageVersion() {
  try {
    return JSON.parse(readFileSync(path.join(root, "package.json"), "utf8")).version;
  } catch {
    return "unknown";
  }
}
