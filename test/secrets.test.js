import test from "node:test";
import assert from "node:assert/strict";
import { findSecrets, redactSecrets } from "../src/secrets.js";

test("findSecrets catches common token forms", () => {
  const text = "api_key = abcdefghijklmnopqrstuvwxyz123456";
  const findings = findSecrets(text);
  assert.equal(findings.length, 1);
  assert.equal(findings[0].name, "named secret");
});

test("redactSecrets removes common token forms", () => {
  const text = "Use Bearer abcdefghijklmnopqrstuvwxyz1234567890";
  assert.equal(redactSecrets(text), "Use [redacted secret]");
});
