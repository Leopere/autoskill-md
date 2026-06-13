#!/usr/bin/env node
import { createWriteStream, existsSync, mkdirSync } from "node:fs";
import { chmod, rm } from "node:fs/promises";
import https from "node:https";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const vendor = path.join(root, "vendor");
const exe = process.platform === "win32" ? "autoskill-md.exe" : "autoskill-md";
const output = path.join(vendor, exe);

if (existsSync(output)) process.exit(0);

const version = JSON.parse(readFileSync(path.join(root, "package.json"), "utf8")).version;
const asset = assetName();

if (!asset) {
  warn("No prebuilt autoskill-md binary is known for this platform.");
  process.exit(0);
}

mkdirSync(vendor, { recursive: true });
const archive = path.join(vendor, asset);
const url = `https://github.com/Leopere/autoskill-md/releases/download/v${version}/${asset}`;

try {
  await download(url, archive);
  const result = spawnSync("tar", ["-xzf", archive, "-C", vendor], { stdio: "inherit" });
  if (result.status !== 0) throw new Error("Could not unpack downloaded archive.");
  if (process.platform !== "win32") await chmod(output, 0o755);
  await rm(archive, { force: true });
  console.log(`Installed autoskill-md ${version}. Credit: https://colinknapp.com`);
} catch (error) {
  warn(`Could not install autoskill-md binary: ${error.message}`);
  warn("The wrapper will still work if AUTOSKILL_MD_BIN points to a binary.");
  process.exit(0);
}

function assetName() {
  const table = {
    "darwin-arm64": "autoskill-md-aarch64-apple-darwin.tar.gz",
    "darwin-x64": "autoskill-md-x86_64-apple-darwin.tar.gz",
    "linux-arm64": "autoskill-md-aarch64-unknown-linux-gnu.tar.gz",
    "linux-x64": "autoskill-md-x86_64-unknown-linux-gnu.tar.gz",
    "win32-x64": "autoskill-md-x86_64-pc-windows-msvc.tar.gz"
  };
  return table[`${process.platform}-${process.arch}`];
}

function download(url, file, redirects = 0) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if ([301, 302, 303, 307, 308].includes(response.statusCode ?? 0)) {
          response.resume();
          if (!response.headers.location || redirects > 5) {
            reject(new Error("Too many redirects."));
            return;
          }
          resolve(download(response.headers.location, file, redirects + 1));
          return;
        }
        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`Download failed with HTTP ${response.statusCode}.`));
          return;
        }
        const stream = createWriteStream(file);
        response.pipe(stream);
        stream.on("finish", () => stream.close(resolve));
        stream.on("error", reject);
      })
      .on("error", reject);
  });
}

function warn(message) {
  console.warn(`autoskill-md: ${message}`);
}
