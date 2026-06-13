import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { DEFAULT_CONFIG, DEFAULT_OUT } from "./constants.js";
import { loadConfig } from "./config.js";
import { generateSkillsMarkdown } from "./generate.js";
import { checkReadability } from "./readability.js";
import { scanProject } from "./scan.js";
import { findSecrets } from "./secrets.js";

export async function runCli(argv = []) {
  const { command, options } = parseArgs(argv);

  if (options.help || !command) {
    printHelp();
    return;
  }

  if (command === "init") {
    await initConfig(options);
    return;
  }

  if (command === "generate") {
    await generate(options);
    return;
  }

  if (command === "check") {
    await check(options);
    return;
  }

  throw new Error(`Unknown command: ${command}`);
}

async function generate(options) {
  const root = path.resolve(options.root ?? process.cwd());
  const out = path.resolve(root, options.out ?? DEFAULT_OUT);
  const config = await loadConfig(root, options.config ?? DEFAULT_CONFIG);
  const scan = await scanProject(root, config);
  const markdown = generateSkillsMarkdown(scan);
  const result = validate(markdown);

  await mkdir(path.dirname(out), { recursive: true });
  await writeFile(out, markdown, "utf8");

  if (!options.quiet) {
    printText(`Wrote ${path.relative(root, out)}`);
    printText(`Reading grade: ${result.readability.grade}`);
    printWarnings(scan.warnings);
    printProblems(result.problems);
  }

  if (options.strict && result.problems.length > 0) process.exitCode = 1;
}

async function check(options) {
  const root = path.resolve(options.root ?? process.cwd());
  const out = path.resolve(root, options.out ?? DEFAULT_OUT);
  const config = await loadConfig(root, options.config ?? DEFAULT_CONFIG);
  const scan = await scanProject(root, config);
  const expected = generateSkillsMarkdown(scan);
  const result = validate(expected);
  const problems = [...result.problems];

  let current = "";
  try {
    current = await readFile(out, "utf8");
  } catch {
    problems.push(`Missing ${path.relative(root, out)}.`);
  }

  if (current && normalizeNewlines(current) !== normalizeNewlines(expected)) {
    problems.push(`${path.relative(root, out)} is stale. Run autoskill-md generate.`);
  }

  if (options.json) {
    printJson({
      ok: problems.length === 0,
      strict: Boolean(options.strict),
      output: path.relative(root, out),
      readability: result.readability,
      problems,
      warnings: scan.warnings
    });
  } else if (!options.quiet) {
    if (problems.length === 0) printText("skills.md is up to date.");
    printText(`Reading grade: ${result.readability.grade}`);
    printWarnings(scan.warnings);
    printProblems(problems);
  }

  if (options.strict && problems.length > 0) process.exitCode = 1;
}

async function initConfig(options) {
  const root = path.resolve(options.root ?? process.cwd());
  const configPath = path.resolve(root, options.config ?? DEFAULT_CONFIG);
  const sample = {
    name: path.basename(root),
    purpose: "This API helps agents use this project.",
    apiBase: "/api",
    auth: "Use the auth rules in the API docs.",
    safeActions: ["GET health and status data"],
    riskyActions: ["Ask before write or delete calls"],
    docs: [],
    support: "",
    limits: "Use a slow pace.",
    ignore: []
  };

  await writeFile(configPath, `${JSON.stringify(sample, null, 2)}\n`, { flag: "wx" });
  if (!options.quiet) printText(`Wrote ${path.relative(root, configPath)}`);
}

function validate(markdown) {
  const readability = checkReadability(markdown);
  const secrets = findSecrets(markdown);
  const problems = [];
  if (!readability.ok) {
    problems.push(`Reading grade ${readability.grade} is above ${readability.maxGrade}.`);
  }
  for (const secret of secrets) {
    problems.push(`Found ${secret.name}: ${secret.sample}`);
  }
  return { readability, secrets, problems };
}

function parseArgs(argv) {
  const args = [...argv];
  const command = args.shift();
  const options = {};

  while (args.length > 0) {
    const arg = args.shift();
    if (arg === "--help" || arg === "-h") options.help = true;
    else if (arg === "--root") options.root = args.shift();
    else if (arg === "--out") options.out = args.shift();
    else if (arg === "--config") options.config = args.shift();
    else if (arg === "--strict") options.strict = true;
    else if (arg === "--quiet") options.quiet = true;
    else if (arg === "--json") options.json = true;
    else throw new Error(`Unknown flag: ${arg}`);
  }

  return { command, options };
}

function printHelp() {
  printText(`autoskill-md

Usage:
  autoskill-md init [--root path]
  autoskill-md generate [--root path] [--out .well-known/skills.md] [--strict]
  autoskill-md check [--root path] [--out .well-known/skills.md] [--strict] [--json]

Defaults:
  --root current directory
  --out .well-known/skills.md
`);
}

function printWarnings(warnings) {
  for (const warning of warnings) printText(`Warning: ${warning}`);
}

function printProblems(problems) {
  for (const problem of problems) printText(`Problem: ${problem}`);
}

function printText(value) {
  console.log(value);
}

function printJson(value) {
  console.log(JSON.stringify(value, null, 2));
}

function normalizeNewlines(value) {
  return value.replace(/\r\n/g, "\n");
}
