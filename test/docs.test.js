import { readFile } from "node:fs/promises";
import test from "node:test";
import assert from "node:assert/strict";
import { checkReadability } from "../src/readability.js";

test("project docs stay below grade 7", async () => {
  for (const file of ["README.md", "LICENSE"]) {
    const markdown = await readFile(file, "utf8");
    const result = checkReadability(markdown);
    assert.equal(result.ok, true, `${file} grade ${result.grade} is above ${result.maxGrade}`);
  }
});
