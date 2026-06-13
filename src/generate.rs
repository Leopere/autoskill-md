use crate::constants::{CREDIT_URL, SPEC_URL, SPEC_VERSION};
use crate::scan::ScanResult;
use crate::secrets::redact_secrets;

const MAX_ACTIONS: usize = 8;

pub fn generate_skills_markdown(scan: &ScanResult) -> String {
    let mut lines = vec![
        "# Skills".to_string(),
        String::new(),
        format!("This file tells agents how to use {}.", scan.name),
        String::new(),
        "## Purpose".to_string(),
        String::new(),
        sentence(&scan.purpose),
        String::new(),
        "## API".to_string(),
        String::new(),
        format!("- Base path: `{}`", empty_default(&scan.api_base, "/")),
        format!("- Source: {}", source_text(scan)),
        String::new(),
        "## Auth".to_string(),
        String::new(),
        format!("- {}", sentence(&scan.auth)),
        "- Do not put secrets, tokens, or private keys in this file.".to_string(),
        String::new(),
        "## Safe Actions".to_string(),
        String::new(),
    ];
    lines.extend(list_or_default(
        &scan.safe_actions,
        "Use read-only calls when auth rules allow them.",
    ));
    lines.extend([String::new(), "## Risky Actions".to_string(), String::new()]);
    lines.extend(list_or_default(
        &scan.risky_actions,
        "Ask before write, delete, email, payment, or admin work.",
    ));
    lines.extend([
        String::new(),
        "## Limits".to_string(),
        String::new(),
        format!("- {}", sentence(&scan.limits)),
        "- Stop after repeated errors.".to_string(),
        String::new(),
        "## More Info".to_string(),
        String::new(),
    ]);
    lines.extend(more_info(scan));
    lines.extend([
        String::new(),
        "## Credits".to_string(),
        String::new(),
        format!("- Spec: {SPEC_URL}"),
        format!("- Spec version: {SPEC_VERSION}"),
        format!("- Credit: {CREDIT_URL}"),
        "- License: CC-BY-4.0".to_string(),
    ]);

    format!(
        "{}\n",
        redact_secrets(&compact_blank_lines(&lines.join("\n"))).trim()
    )
}

fn source_text(scan: &ScanResult) -> String {
    let count = scan.routes.len();
    if count == 0 {
        return "No public HTTP routes found".to_string();
    }
    let languages = if scan.languages.is_empty() {
        "project files".to_string()
    } else {
        scan.languages.join(", ")
    };
    if count == 1 {
        format!("1 route from {languages}")
    } else {
        format!("{count} routes from {languages}")
    }
}

fn list_or_default(items: &[String], fallback: &str) -> Vec<String> {
    if items.is_empty() {
        return vec![format!("- {fallback}")];
    }
    items
        .iter()
        .filter(|item| !item.trim().is_empty())
        .take(MAX_ACTIONS)
        .map(|item| format!("- {}", sentence(item)))
        .collect()
}

fn more_info(scan: &ScanResult) -> Vec<String> {
    let mut lines = Vec::new();
    for doc in scan.docs.iter().take(6) {
        lines.push(format!("- Docs: {doc}"));
    }
    if !scan.support.is_empty() {
        lines.push(format!("- Support: {}", scan.support));
    }
    for note in scan.notes.iter().take(4) {
        lines.push(format!("- Note: {}", sentence(note)));
    }
    if lines.is_empty() {
        lines.push("- Add docs in `autoskill.config.json`.".to_string());
    }
    lines
}

fn sentence(value: &str) -> String {
    let clean = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() {
        return clean;
    }
    if clean.ends_with(['.', '!', '?', '`']) {
        clean
    } else {
        format!("{clean}.")
    }
}

fn empty_default<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

fn compact_blank_lines(text: &str) -> String {
    let mut output = Vec::new();
    let mut last_blank = false;
    for line in text.lines() {
        let blank = line.trim().is_empty();
        if blank && last_blank {
            continue;
        }
        output.push(line);
        last_blank = blank;
    }
    output.join("\n")
}
