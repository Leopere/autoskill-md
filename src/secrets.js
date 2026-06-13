const SECRET_PATTERNS = [
  {
    name: "private key",
    pattern: /-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----/g
  },
  {
    name: "aws access key",
    pattern: /\bAKIA[0-9A-Z]{16}\b/g
  },
  {
    name: "github token",
    pattern: /\bgh[pousr]_[A-Za-z0-9_]{36,}\b/g
  },
  {
    name: "slack token",
    pattern: /\bxox[baprs]-[A-Za-z0-9-]{20,}\b/g
  },
  {
    name: "named secret",
    pattern: /\b(?:api[_-]?key|secret|token|password|passwd|pwd|session[_-]?id)\b\s*[:=]\s*["']?[A-Za-z0-9_./+=-]{16,}/gi
  },
  {
    name: "bearer token",
    pattern: /\bBearer\s+[A-Za-z0-9._~+/=-]{20,}\b/g
  }
];

export function findSecrets(text) {
  const findings = [];
  for (const { name, pattern } of SECRET_PATTERNS) {
    pattern.lastIndex = 0;
    for (const match of text.matchAll(pattern)) {
      findings.push({
        name,
        index: match.index ?? 0,
        sample: mask(match[0])
      });
    }
  }
  return findings;
}

export function redactSecrets(text) {
  let output = text;
  for (const { pattern } of SECRET_PATTERNS) {
    pattern.lastIndex = 0;
    output = output.replace(pattern, "[redacted secret]");
  }
  return output;
}

function mask(value) {
  if (value.length <= 12) return "[redacted]";
  return `${value.slice(0, 4)}...${value.slice(-4)}`;
}
