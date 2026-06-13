import test from "node:test";
import assert from "node:assert/strict";
import { generateSkillsMarkdown } from "../src/generate.js";
import { SPEC_URL, SPEC_VERSION, CREDIT_URL } from "../src/constants.js";
import { findSecrets } from "../src/secrets.js";
import { checkReadability } from "../src/readability.js";

test("generateSkillsMarkdown writes spec-shaped Markdown with credit", () => {
  const markdown = generateSkillsMarkdown({
    name: "tickets-api",
    purpose: "This API lets agents read ticket status.",
    apiBase: "/api/v1",
    auth: "Public reads need no auth.",
    limits: "Use a slow pace.",
    languages: ["node"],
    routes: [{ method: "GET", path: "/api/v1/tickets/:id" }],
    safeActions: ["GET /api/v1/tickets/:id"],
    riskyActions: ["Ask before ticket changes"],
    docs: ["https://example.com/docs"],
    support: "https://example.com/support",
    notes: []
  });

  assert.match(markdown, /^# Skills/);
  assert.match(markdown, /## Purpose/);
  assert.match(markdown, /## Auth/);
  assert.match(markdown, /## Safe Actions/);
  assert.match(markdown, /## Risky Actions/);
  assert.match(markdown, new RegExp(escapeRegex(SPEC_URL)));
  assert.match(markdown, new RegExp(SPEC_VERSION));
  assert.match(markdown, new RegExp(escapeRegex(CREDIT_URL)));
  assert.match(markdown, /CC-BY-4\.0/);
  assert.equal(findSecrets(markdown).length, 0);
  assert.equal(checkReadability(markdown).ok, true);
});

function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
