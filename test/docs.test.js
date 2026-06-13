import { readdir, readFile } from "node:fs/promises";
import test from "node:test";
import assert from "node:assert/strict";
import { checkReadability } from "../src/readability.js";

test("project docs stay below grade 7", async () => {
  const docs = await readdir("docs");
  const files = ["README.md", "LICENSE", ...docs.filter((file) => file.endsWith(".md")).map((file) => `docs/${file}`)];
  for (const file of files) {
    const markdown = await readFile(file, "utf8");
    const result = checkReadability(markdown);
    assert.equal(result.ok, true, `${file} grade ${result.grade} is above ${result.maxGrade}`);
  }
});
