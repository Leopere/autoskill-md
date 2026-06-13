use regex::Regex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretFinding {
    pub name: String,
    pub index: usize,
    pub sample: String,
}

pub fn find_secrets(text: &str) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    for (name, pattern) in secret_patterns() {
        for found in pattern.find_iter(text) {
            findings.push(SecretFinding {
                name: name.to_string(),
                index: found.start(),
                sample: mask(found.as_str()),
            });
        }
    }
    findings
}

pub fn redact_secrets(text: &str) -> String {
    secret_patterns()
        .into_iter()
        .fold(text.to_string(), |output, (_, pattern)| {
            pattern
                .replace_all(&output, "[redacted secret]")
                .to_string()
        })
}

fn secret_patterns() -> Vec<(&'static str, Regex)> {
    vec![
        (
            "private key",
            Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----")
                .expect("valid private key regex"),
        ),
        (
            "aws access key",
            Regex::new(r"\bAKIA[0-9A-Z]{16}\b").expect("valid aws regex"),
        ),
        (
            "github token",
            Regex::new(r"\bgh[pousr]_[A-Za-z0-9_]{36,}\b").expect("valid github regex"),
        ),
        (
            "slack token",
            Regex::new(r"\bxox[baprs]-[A-Za-z0-9-]{20,}\b").expect("valid slack regex"),
        ),
        (
            "named secret",
            Regex::new(
                r#"(?i)\b(?:api[_-]?key|secret|token|password|passwd|pwd|session[_-]?id)\b\s*[:=]\s*["']?[A-Za-z0-9_./+=-]{16,}"#,
            )
            .expect("valid named secret regex"),
        ),
        (
            "bearer token",
            Regex::new(r"\bBearer\s+[A-Za-z0-9._~+/=-]{20,}\b").expect("valid bearer regex"),
        ),
    ]
}

fn mask(value: &str) -> String {
    if value.len() <= 12 {
        return "[redacted]".to_string();
    }
    format!("{}...{}", &value[..4], &value[value.len() - 4..])
}
