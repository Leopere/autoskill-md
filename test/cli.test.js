import { execFile } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import test from "node:test";
import assert from "node:assert/strict";

const run = promisify(execFile);
const bin = path.resolve("bin/autoskill-md.js");

test("CLI generate writes skills.md and check passes", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "autoskill-cli-"));
  await mkdir(path.join(root, "src"), { recursive: true });
  await writeFile(
    path.join(root, "package.json"),
    JSON.stringify({
      name: "cli-api",
      description: "This API lets agents read status."
    })
  );
  await writeFile(path.join(root, "src", "app.js"), 'app.get("/api/status", handler);\n');

  await run(process.execPath, [bin, "generate", "--root", root, "--quiet"]);
  const markdown = await readFile(path.join(root, ".well-known", "skills.md"), "utf8");

  assert.match(markdown, /# Skills/);
  assert.match(markdown, /https:\/\/colinknapp\.com\/specs\/mcp-discovery\.html/);

  const { stdout } = await run(process.execPath, [bin, "check", "--root", root, "--strict", "--json"]);
  const result = JSON.parse(stdout);
  assert.equal(result.ok, true);
});

test("CLI check --strict fails when output is stale", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "autoskill-stale-"));
  await writeFile(
    path.join(root, "package.json"),
    JSON.stringify({
      name: "stale-api",
      description: "This API lets agents read status."
    })
  );

  await run(process.execPath, [bin, "generate", "--root", root, "--quiet"]);
  await writeFile(path.join(root, ".well-known", "skills.md"), "# Skills\n\nOld text.\n");

  await assert.rejects(
    run(process.execPath, [bin, "check", "--root", root, "--strict", "--json"]),
    (error) => {
      const result = JSON.parse(error.stdout);
      assert.equal(result.ok, false);
      assert.ok(result.problems.some((problem) => problem.includes("stale")));
      return true;
    }
  );
});
