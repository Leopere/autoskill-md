import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { DEFAULT_IGNORES } from "./constants.js";
import { redactSecrets } from "./secrets.js";

const CODE_EXTENSIONS = new Set([
  ".go",
  ".rs",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".ts",
  ".tsx",
  ".py"
]);

const MANIFESTS = new Set([
  "package.json",
  "go.mod",
  "Cargo.toml",
  "pyproject.toml"
]);

const MAX_FILE_BYTES = 512 * 1024;
const MAX_FILES = 1200;

export async function scanProject(root, config = {}) {
  const absoluteRoot = path.resolve(root);
  const state = {
    root: absoluteRoot,
    name: "",
    purpose: "",
    apiBase: "",
    auth: "",
    docs: new Set(),
    support: "",
    limits: "",
    safeActions: new Set(),
    riskyActions: new Set(),
    notes: new Set(),
    routes: [],
    languages: new Set(),
    filesScanned: 0,
    warnings: []
  };

  const ignore = new Set([...DEFAULT_IGNORES, ...(config.ignore ?? [])]);
  const files = [];
  await collectFiles(absoluteRoot, absoluteRoot, ignore, files, state);

  for (const file of sortFiles(files)) {
    if (state.filesScanned >= MAX_FILES) {
      state.warnings.push(`Stopped after ${MAX_FILES} files.`);
      break;
    }
    await scanFile(file, state);
  }

  applyConfig(state, config);
  return toResult(state);
}

function sortFiles(files) {
  return files.sort((a, b) => filePriority(a) - filePriority(b) || a.localeCompare(b));
}

function filePriority(file) {
  const name = path.basename(file);
  if (name === "package.json") return 0;
  if (name === "go.mod") return 1;
  if (name === "Cargo.toml") return 2;
  if (name === "pyproject.toml") return 3;
  if (/^readme/i.test(name)) return 4;
  return 10;
}

async function collectFiles(root, dir, ignore, files, state) {
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch (error) {
    state.warnings.push(`Could not read ${relative(root, dir)}: ${error.message}`);
    return;
  }

  for (const entry of entries) {
    if (ignore.has(entry.name)) continue;
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      await collectFiles(root, fullPath, ignore, files, state);
      continue;
    }
    if (!entry.isFile()) continue;

    const ext = path.extname(entry.name);
    if (CODE_EXTENSIONS.has(ext) || MANIFESTS.has(entry.name) || /^readme/i.test(entry.name)) {
      files.push(fullPath);
    }
  }
}

async function scanFile(file, state) {
  const rel = relative(state.root, file);
  let text;
  try {
    text = await readFile(file, "utf8");
  } catch (error) {
    state.warnings.push(`Could not read ${rel}: ${error.message}`);
    return;
  }

  if (Buffer.byteLength(text, "utf8") > MAX_FILE_BYTES) {
    state.warnings.push(`Skipped large file ${rel}.`);
    return;
  }

  state.filesScanned += 1;
  const cleanText = redactSecrets(text);
  const name = path.basename(file);
  const ext = path.extname(file);

  if (name === "package.json") scanPackageJson(cleanText, state);
  if (name === "go.mod") scanGoMod(cleanText, state);
  if (name === "Cargo.toml") scanCargoToml(cleanText, state);
  if (name === "pyproject.toml") scanPyproject(cleanText, state);
  if (/^readme/i.test(name)) scanReadme(cleanText, state);

  if (CODE_EXTENSIONS.has(ext)) {
    state.languages.add(languageForExtension(ext));
    scanHints(cleanText, state);
    scanRoutes(cleanText, rel, ext, state);
  }
}

function scanPackageJson(text, state) {
  try {
    const pkg = JSON.parse(text);
    state.name ||= stringField(pkg.name);
    state.purpose ||= stringField(pkg.description);
    addUrl(state.docs, pkg.homepage);
    if (typeof pkg.repository === "string") addUrl(state.docs, pkg.repository);
    if (pkg.repository && typeof pkg.repository.url === "string") addUrl(state.docs, pkg.repository.url);
  } catch {
    state.warnings.push("Could not parse package.json.");
  }
}

function scanGoMod(text, state) {
  const match = text.match(/^module\s+(.+)$/m);
  if (match) state.name ||= match[1].trim().split("/").pop();
}

function scanCargoToml(text, state) {
  state.name ||= tomlString(text, "name");
  state.purpose ||= tomlString(text, "description");
  addUrl(state.docs, tomlString(text, "homepage"));
  addUrl(state.docs, tomlString(text, "repository"));
}

function scanPyproject(text, state) {
  state.name ||= tomlString(text, "name");
  state.purpose ||= tomlString(text, "description");
}

function scanReadme(text, state) {
  const title = text.match(/^#\s+(.+)$/m);
  if (title) state.name ||= title[1].trim();

  const firstPlainLine = text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find((line) => line && !line.startsWith("#") && !line.startsWith("[!") && !line.startsWith("<"));
  if (firstPlainLine && firstPlainLine.length < 180) state.purpose ||= trimSentence(firstPlainLine);
}

function scanHints(text, state) {
  const patterns = [
    /^\s*(?:\/\/|#)\s*(?:autoskill|skill)\s*:\s*(.+)$/gim,
    /\/\*\*?([\s\S]*?)\*\//g
  ];

  for (const pattern of patterns) {
    pattern.lastIndex = 0;
    for (const match of text.matchAll(pattern)) {
      const body = cleanComment(match[1]);
      for (const line of body.split(/\r?\n/)) {
        applyHint(line.trim(), state);
      }
    }
  }
}

function applyHint(line, state) {
  if (!line) return;
  const direct = line.match(/^(?:autoskill|skill)\s*:\s*(.+)$/i);
  const body = direct ? direct[1].trim() : line;
  const match = body.match(/^([a-z][a-z -]{1,24})\s*:\s*(.+)$/i);
  if (!match) {
    if (/^(use|read|ask|do|do not|prefer|stop)\b/i.test(body)) state.notes.add(trimSentence(body));
    return;
  }

  const key = match[1].toLowerCase().replace(/\s+/g, "");
  const value = trimSentence(match[2]);
  if (!value) return;

  if (key === "purpose") state.purpose ||= value;
  else if (key === "api" || key === "base" || key === "apibase") state.apiBase ||= value;
  else if (key === "auth") state.auth ||= value;
  else if (key === "safe" || key === "safeaction") state.safeActions.add(value);
  else if (key === "risky" || key === "write" || key === "riskyaction") state.riskyActions.add(value);
  else if (key === "docs" || key === "doc") addUrl(state.docs, value);
  else if (key === "support") state.support ||= value;
  else if (key === "limits" || key === "rate") state.limits ||= value;
  else state.notes.add(`${match[1]}: ${value}`);
}

function scanRoutes(text, file, ext, state) {
  const routePatterns = [
    {
      language: "node",
      pattern: /\b(?:app|router|server|fastify)\s*\.\s*(get|post|put|patch|delete|head|options)\s*\(\s*["'`]([^"'`]+)["'`]/gi
    },
    {
      language: "go",
      pattern: /\b(?:Handle|HandleFunc)\s*\(\s*["`]([^"`]+)["`]/g,
      method: "GET"
    },
    {
      language: "go",
      pattern: /\b(?:GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s*\(\s*["`]([^"`]+)["`]/g
    },
    {
      language: "rust",
      pattern: /#\[\s*(get|post|put|patch|delete|head|options)\s*\(\s*"([^"]+)"/gi
    },
    {
      language: "rust",
      pattern: /\.route\s*\(\s*"([^"]+)"\s*,\s*(get|post|put|patch|delete|head|options)\s*\(/gi,
      reverse: true
    },
    {
      language: "python",
      pattern: /@\w+(?:\.\w+)?\s*\.\s*(get|post|put|patch|delete|head|options|route)\s*\(\s*["']([^"']+)["']/gi
    }
  ];

  for (const routePattern of routePatterns) {
    if (!matchesLanguage(ext, routePattern.language)) continue;
    routePattern.pattern.lastIndex = 0;
    for (const match of text.matchAll(routePattern.pattern)) {
      let method = routePattern.method;
      let routePath;
      if (routePattern.reverse) {
        routePath = match[1];
        method = match[2].toUpperCase();
      } else if (routePattern.method) {
        routePath = match[1];
      } else {
        method = match[1].toUpperCase();
        routePath = match[2];
      }
      if (method === "ROUTE") method = "GET";
      addRoute(state, method, routePath, file);
    }
  }

  scanNextRoute(file, state);
}

function scanNextRoute(file, state) {
  const normalized = file.replaceAll(path.sep, "/");
  const apiIndex = normalized.indexOf("/api/");
  if (apiIndex === -1 && !normalized.startsWith("api/")) return;
  if (!/\.(js|jsx|ts|tsx|mjs|cjs)$/.test(normalized)) return;

  const apiPart = normalized
    .replace(/^src\//, "")
    .replace(/^pages\/api\//, "/api/")
    .replace(/^app\/api\//, "/api/")
    .replace(/\/route\.(js|jsx|ts|tsx|mjs|cjs)$/, "")
    .replace(/\.(js|jsx|ts|tsx|mjs|cjs)$/, "")
    .replace(/\[([^\]]+)\]/g, ":$1");

  if (apiPart.startsWith("/api/")) addRoute(state, "GET", apiPart, file);
}

function addRoute(state, method, routePath, file) {
  if (!routePath || !routePath.startsWith("/")) return;
  const route = {
    method: method.toUpperCase(),
    path: normalizeRoutePath(routePath),
    file
  };
  const key = `${route.method} ${route.path}`;
  if (state.routes.some((item) => `${item.method} ${item.path}` === key)) return;
  state.routes.push(route);

  const action = `${route.method} ${route.path}`;
  if (isSafeMethod(route.method)) state.safeActions.add(action);
  else state.riskyActions.add(action);
}

function applyConfig(state, config) {
  state.name = config.name || state.name || path.basename(state.root);
  state.purpose = config.purpose || state.purpose || "This project exposes code and docs for agents.";
  state.apiBase = config.apiBase || state.apiBase || guessApiBase(state.routes);
  state.auth = config.auth || state.auth || "Auth rules are not set. Check the API docs before you call it.";
  state.support = config.support || state.support;
  state.limits = config.limits || state.limits || "No rate limit was found. Use a slow pace.";

  for (const action of config.safeActions ?? []) state.safeActions.add(action);
  for (const action of config.riskyActions ?? []) state.riskyActions.add(action);
  for (const doc of config.docs ?? []) addUrl(state.docs, doc);
}

function toResult(state) {
  return {
    root: state.root,
    name: state.name,
    purpose: state.purpose,
    apiBase: state.apiBase,
    auth: state.auth,
    docs: [...state.docs],
    support: state.support,
    limits: state.limits,
    safeActions: [...state.safeActions],
    riskyActions: [...state.riskyActions],
    notes: [...state.notes],
    routes: state.routes.sort((a, b) => `${a.path} ${a.method}`.localeCompare(`${b.path} ${b.method}`)),
    languages: [...state.languages].filter(Boolean).sort(),
    filesScanned: state.filesScanned,
    warnings: state.warnings
  };
}

function guessApiBase(routes) {
  if (routes.some((route) => route.path.startsWith("/api/v1"))) return "/api/v1";
  if (routes.some((route) => route.path.startsWith("/api"))) return "/api";
  return "/";
}

function cleanComment(text) {
  return text
    .split(/\r?\n/)
    .map((line) => line.replace(/^\s*\*\s?/, "").replace(/^\s*(?:\/\/|#)\s?/, ""))
    .join("\n")
    .trim();
}

function matchesLanguage(ext, language) {
  if (language === "node") return [".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx"].includes(ext);
  if (language === "go") return ext === ".go";
  if (language === "rust") return ext === ".rs";
  if (language === "python") return ext === ".py";
  return false;
}

function languageForExtension(ext) {
  if (ext === ".go") return "go";
  if (ext === ".rs") return "rust";
  if (ext === ".py") return "python";
  if ([".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx"].includes(ext)) return "node";
  return "";
}

function isSafeMethod(method) {
  return ["GET", "HEAD", "OPTIONS"].includes(method.toUpperCase());
}

function normalizeRoutePath(routePath) {
  return routePath.replace(/\/+/g, "/").replace(/\/$/, "") || "/";
}

function tomlString(text, key) {
  const match = text.match(new RegExp(`^${key}\\s*=\\s*["']([^"']+)["']`, "m"));
  return match ? match[1].trim() : "";
}

function stringField(value) {
  return typeof value === "string" ? value.trim() : "";
}

function addUrl(set, value) {
  if (typeof value !== "string") return;
  const clean = value.trim().replace(/^git\+/, "").replace(/\.git$/, "");
  if (!clean) return;
  if (/^(https?:\/\/|\/)/.test(clean)) set.add(clean);
}

function trimSentence(value) {
  return value.replace(/\s+/g, " ").trim().replace(/\s*[.;]\s*$/, "");
}

function relative(root, file) {
  return path.relative(root, file) || ".";
}
