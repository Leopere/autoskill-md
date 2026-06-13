import test from "node:test";
import assert from "node:assert/strict";
import { checkReadability } from "../src/readability.js";

test("checkReadability accepts simple docs", () => {
  const result = checkReadability("This API reads tickets. Ask before changes. Use a slow pace.");
  assert.equal(result.ok, true);
  assert.ok(result.grade < 7);
});

test("checkReadability rejects hard docs", () => {
  const result = checkReadability(
    "This implementation operationalizes interdependent authorization mechanisms through heterogeneous infrastructure abstractions."
  );
  assert.equal(result.ok, false);
});
