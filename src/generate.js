import { CREDIT_URL, SPEC_URL, SPEC_VERSION } from "./constants.js";
import { redactSecrets } from "./secrets.js";

const MAX_ACTIONS = 8;

export function generateSkillsMarkdown(scan) {
  const lines = [
    "# Skills",
    "",
    `This file tells agents how to use ${scan.name}.`,
    "",
    "## Purpose",
    "",
    sentence(scan.purpose),
    "",
    "## API",
    "",
    `- Base path: \`${scan.apiBase || "/"}\``,
    `- Source: ${sourceText(scan)}`,
    "",
    "## Auth",
    "",
    `- ${sentence(scan.auth)}`,
    "- Do not put secrets, tokens, or private keys in this file.",
    "",
    "## Safe Actions",
    "",
    ...listOrDefault(scan.safeActions, "Use read-only calls when auth rules allow them."),
    "",
    "## Risky Actions",
    "",
    ...listOrDefault(scan.riskyActions, "Ask before write, delete, email, payment, or admin work."),
    "",
    "## Limits",
    "",
    `- ${sentence(scan.limits)}`,
    "- Stop after repeated errors.",
    "",
    "## More Info",
    "",
    ...moreInfo(scan),
    "",
    "## Credits",
    "",
    `- Spec: ${SPEC_URL}`,
    `- Spec version: ${SPEC_VERSION}`,
    `- Credit: ${CREDIT_URL}`,
    "- License: CC-BY-4.0"
  ];

  return `${redactSecrets(lines.join("\n")).replace(/\n{3,}/g, "\n\n").trim()}\n`;
}

function sourceText(scan) {
  const languages = scan.languages?.length ? scan.languages.join(", ") : "project files";
  const routes = scan.routes?.length ?? 0;
  if (routes === 0) return "No public HTTP routes found";
  if (routes === 1) return `1 route from ${languages}`;
  return `${routes} routes from ${languages}`;
}

function listOrDefault(items, fallback) {
  const clean = [...new Set(items ?? [])]
    .filter(Boolean)
    .slice(0, MAX_ACTIONS)
    .map((item) => `- ${sentence(item)}`);

  if (clean.length === 0) return [`- ${fallback}`];
  return clean;
}

function moreInfo(scan) {
  const lines = [];
  for (const doc of (scan.docs ?? []).slice(0, 6)) {
    lines.push(`- Docs: ${doc}`);
  }
  if (scan.support) lines.push(`- Support: ${scan.support}`);
  if (scan.notes?.length) {
    for (const note of scan.notes.slice(0, 4)) lines.push(`- Note: ${sentence(note)}`);
  }
  if (lines.length === 0) lines.push("- Add docs in `autoskill.config.json`.");
  return lines;
}

function sentence(value) {
  const clean = String(value ?? "")
    .replace(/\s+/g, " ")
    .trim();
  if (!clean) return "";
  if (/[.!?`]$/.test(clean)) return clean;
  return `${clean}.`;
}
