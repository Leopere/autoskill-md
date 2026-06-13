import { access, readFile } from "node:fs/promises";
import path from "node:path";
import { DEFAULT_CONFIG } from "./constants.js";

export const defaultConfig = Object.freeze({
  name: "",
  purpose: "",
  apiBase: "",
  auth: "",
  safeActions: [],
  riskyActions: [],
  docs: [],
  support: "",
  limits: "",
  ignore: []
});

export async function loadConfig(root, configPath = DEFAULT_CONFIG) {
  const file = path.resolve(root, configPath);
  try {
    await access(file);
  } catch {
    return { ...defaultConfig };
  }

  let parsed;
  try {
    parsed = JSON.parse(await readFile(file, "utf8"));
  } catch (error) {
    throw new Error(`Could not read ${configPath}: ${error.message}`);
  }

  return {
    ...defaultConfig,
    ...parsed,
    safeActions: asArray(parsed.safeActions),
    riskyActions: asArray(parsed.riskyActions),
    docs: asArray(parsed.docs),
    ignore: asArray(parsed.ignore)
  };
}

function asArray(value) {
  if (Array.isArray(value)) return value.filter(Boolean).map(String);
  if (typeof value === "string" && value.trim()) return [value.trim()];
  return [];
}
