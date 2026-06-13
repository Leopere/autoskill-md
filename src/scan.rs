use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::constants::DEFAULT_IGNORES;
use crate::secrets::redact_secrets;

const CODE_EXTENSIONS: &[&str] = &["go", "rs", "js", "jsx", "mjs", "cjs", "ts", "tsx", "py"];
const MANIFESTS: &[&str] = &["package.json", "go.mod", "Cargo.toml", "pyproject.toml"];
const MAX_FILE_BYTES: u64 = 512 * 1024;
const MAX_FILES: usize = 1200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub file: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanResult {
    pub root: PathBuf,
    pub name: String,
    pub purpose: String,
    pub api_base: String,
    pub auth: String,
    pub docs: Vec<String>,
    pub support: String,
    pub limits: String,
    pub safe_actions: Vec<String>,
    pub risky_actions: Vec<String>,
    pub notes: Vec<String>,
    pub routes: Vec<Route>,
    pub languages: Vec<String>,
    pub files_scanned: usize,
    pub warnings: Vec<String>,
}

#[derive(Default)]
struct ScanState {
    root: PathBuf,
    name: String,
    purpose: String,
    api_base: String,
    auth: String,
    docs: BTreeSet<String>,
    support: String,
    limits: String,
    safe_actions: BTreeSet<String>,
    risky_actions: BTreeSet<String>,
    notes: BTreeSet<String>,
    routes: Vec<Route>,
    languages: BTreeSet<String>,
    files_scanned: usize,
    warnings: Vec<String>,
}

pub fn scan_project(root: &Path, config: &Config) -> ScanResult {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut state = ScanState {
        root: root.clone(),
        ..ScanState::default()
    };
    let ignore = DEFAULT_IGNORES
        .iter()
        .map(|value| value.to_string())
        .chain(config.ignore.iter().cloned())
        .collect::<HashSet<_>>();
    let mut files = Vec::new();
    collect_files(&root, &root, &ignore, &mut files, &mut state);
    sort_files(&mut files);

    for file in files {
        if state.files_scanned >= MAX_FILES {
            state
                .warnings
                .push(format!("Stopped after {MAX_FILES} files."));
            break;
        }
        scan_file(&file, &mut state);
    }

    apply_config(&mut state, config);
    to_result(state)
}

fn collect_files(
    root: &Path,
    dir: &Path,
    ignore: &HashSet<String>,
    files: &mut Vec<PathBuf>,
    state: &mut ScanState,
) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            state
                .warnings
                .push(format!("Could not read {}: {error}", relative(root, dir)));
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if ignore.contains(&name) {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            collect_files(root, &path, ignore, files, state);
        } else if kind.is_file() && should_scan(&path) {
            files.push(path);
        }
    }
}

fn should_scan(path: &Path) -> bool {
    let name = file_name(path);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    CODE_EXTENSIONS.contains(&extension)
        || MANIFESTS.contains(&name.as_str())
        || name.to_ascii_lowercase().starts_with("readme")
}

fn sort_files(files: &mut [PathBuf]) {
    files.sort_by(|a, b| {
        file_priority(a)
            .cmp(&file_priority(b))
            .then_with(|| a.cmp(b))
    });
}

fn file_priority(path: &Path) -> usize {
    match file_name(path).as_str() {
        "package.json" => 0,
        "go.mod" => 1,
        "Cargo.toml" => 2,
        "pyproject.toml" => 3,
        name if name.to_ascii_lowercase().starts_with("readme") => 4,
        _ => 10,
    }
}

fn scan_file(file: &Path, state: &mut ScanState) {
    let rel = relative(&state.root, file);
    let metadata = match fs::metadata(file) {
        Ok(metadata) => metadata,
        Err(error) => {
            state
                .warnings
                .push(format!("Could not stat {rel}: {error}"));
            return;
        }
    };
    if metadata.len() > MAX_FILE_BYTES {
        state.warnings.push(format!("Skipped large file {rel}."));
        return;
    }

    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) => {
            state
                .warnings
                .push(format!("Could not read {rel}: {error}"));
            return;
        }
    };
    let clean = redact_secrets(&text);
    let name = file_name(file);
    let extension = file
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    state.files_scanned += 1;

    match name.as_str() {
        "package.json" => scan_package_json(&clean, state),
        "go.mod" => scan_go_mod(&clean, state),
        "Cargo.toml" => scan_cargo_toml(&clean, state),
        "pyproject.toml" => scan_pyproject(&clean, state),
        name if name.to_ascii_lowercase().starts_with("readme") => scan_readme(&clean, state),
        _ => {}
    }

    if CODE_EXTENSIONS.contains(&extension) {
        state
            .languages
            .insert(language_for_extension(extension).to_string());
        scan_hints(&clean, state);
        scan_routes(&clean, &rel, extension, state);
    }
}

fn scan_package_json(text: &str, state: &mut ScanState) {
    let Ok(value) = serde_json::from_str::<JsonValue>(text) else {
        state
            .warnings
            .push("Could not parse package.json.".to_string());
        return;
    };
    set_if_empty(&mut state.name, json_string(&value, "name"));
    set_if_empty(&mut state.purpose, json_string(&value, "description"));
    add_url(&mut state.docs, json_string(&value, "homepage"));
    if let Some(repository) = json_string(&value, "repository") {
        add_url(&mut state.docs, Some(repository));
    } else if let Some(url) = value
        .get("repository")
        .and_then(|repo| repo.get("url"))
        .and_then(|url| url.as_str())
    {
        add_url(&mut state.docs, Some(url.to_string()));
    }
}

fn scan_go_mod(text: &str, state: &mut ScanState) {
    for line in text.lines() {
        if let Some(module) = line.strip_prefix("module ") {
            set_if_empty(
                &mut state.name,
                module.trim().split('/').last().map(ToString::to_string),
            );
            break;
        }
    }
}

fn scan_cargo_toml(text: &str, state: &mut ScanState) {
    let Ok(value) = text.parse::<toml::Value>() else {
        state
            .warnings
            .push("Could not parse Cargo.toml.".to_string());
        return;
    };
    let package = value.get("package").unwrap_or(&value);
    set_if_empty(&mut state.name, toml_string(package, "name"));
    set_if_empty(&mut state.purpose, toml_string(package, "description"));
    add_url(&mut state.docs, toml_string(package, "homepage"));
    add_url(&mut state.docs, toml_string(package, "repository"));
}

fn scan_pyproject(text: &str, state: &mut ScanState) {
    let Ok(value) = text.parse::<toml::Value>() else {
        state
            .warnings
            .push("Could not parse pyproject.toml.".to_string());
        return;
    };
    let project = value.get("project").unwrap_or(&value);
    set_if_empty(&mut state.name, toml_string(project, "name"));
    set_if_empty(&mut state.purpose, toml_string(project, "description"));
}

fn scan_readme(text: &str, state: &mut ScanState) {
    for line in text.lines() {
        if state.name.is_empty() {
            if let Some(title) = line.trim().strip_prefix("# ") {
                state.name = title.trim().to_string();
            }
        }
        let plain = line.trim();
        if state.purpose.is_empty()
            && !plain.is_empty()
            && !plain.starts_with('#')
            && !plain.starts_with("[!")
            && !plain.starts_with('<')
            && plain.len() < 180
        {
            state.purpose = trim_sentence(plain);
        }
        if !state.name.is_empty() && !state.purpose.is_empty() {
            break;
        }
    }
}

fn scan_hints(text: &str, state: &mut ScanState) {
    for line in text.lines() {
        let clean = clean_comment_line(line);
        if let Some(hint) = clean
            .strip_prefix("autoskill:")
            .or_else(|| clean.strip_prefix("skill:"))
        {
            apply_hint(hint.trim(), state);
        }
    }

    let block = Regex::new(r"/\*\*?([\s\S]*?)\*/").expect("valid block comment regex");
    for capture in block.captures_iter(text) {
        let Some(body) = capture.get(1) else {
            continue;
        };
        for line in body.as_str().lines() {
            apply_hint(clean_comment_line(line).as_str(), state);
        }
    }
}

fn apply_hint(line: &str, state: &mut ScanState) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    let body = line
        .strip_prefix("autoskill:")
        .or_else(|| line.strip_prefix("skill:"))
        .unwrap_or(line)
        .trim();

    let Some((key, value)) = body.split_once(':') else {
        if starts_with_action_word(body) {
            state.notes.insert(trim_sentence(body));
        }
        return;
    };

    let key = key.to_ascii_lowercase().replace([' ', '-'], "");
    let value = trim_sentence(value);
    if value.is_empty() {
        return;
    }

    match key.as_str() {
        "purpose" => set_if_empty(&mut state.purpose, Some(value)),
        "api" | "base" | "apibase" => set_if_empty(&mut state.api_base, Some(value)),
        "auth" => set_if_empty(&mut state.auth, Some(value)),
        "safe" | "safeaction" => {
            state.safe_actions.insert(value);
        }
        "risky" | "write" | "riskyaction" => {
            state.risky_actions.insert(value);
        }
        "docs" | "doc" => add_url(&mut state.docs, Some(value)),
        "support" => set_if_empty(&mut state.support, Some(value)),
        "limits" | "rate" => set_if_empty(&mut state.limits, Some(value)),
        _ => {
            state.notes.insert(format!("{key}: {value}"));
        }
    }
}

fn scan_routes(text: &str, file: &str, extension: &str, state: &mut ScanState) {
    match extension {
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" => scan_node_routes(text, file, state),
        "go" => scan_go_routes(text, file, state),
        "rs" => scan_rust_routes(text, file, state),
        "py" => scan_python_routes(text, file, state),
        _ => {}
    }
}

fn scan_node_routes(text: &str, file: &str, state: &mut ScanState) {
    let route = Regex::new(
        r#"\b(?:app|router|server|fastify)\s*\.\s*(get|post|put|patch|delete|head|options)\s*\(\s*["'`]([^"'`]+)["'`]"#,
    )
    .expect("valid node route regex");
    for capture in route.captures_iter(text) {
        add_route(state, &capture[1].to_ascii_uppercase(), &capture[2], file);
    }
    scan_next_route(file, state);
}

fn scan_go_routes(text: &str, file: &str, state: &mut ScanState) {
    let handle =
        Regex::new(r#"\b(?:Handle|HandleFunc)\s*\(\s*["`]([^"`]+)["`]"#).expect("valid go regex");
    for capture in handle.captures_iter(text) {
        add_route(state, "GET", &capture[1], file);
    }

    let method =
        Regex::new(r#"\b(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s*\(\s*["`]([^"`]+)["`]"#)
            .expect("valid go method regex");
    for capture in method.captures_iter(text) {
        add_route(state, &capture[1], &capture[2], file);
    }
}

fn scan_rust_routes(text: &str, file: &str, state: &mut ScanState) {
    let attr = Regex::new(r#"#\[\s*(get|post|put|patch|delete|head|options)\s*\(\s*"([^"]+)""#)
        .expect("valid rust attr regex");
    for capture in attr.captures_iter(text) {
        add_route(state, &capture[1].to_ascii_uppercase(), &capture[2], file);
    }

    let route = Regex::new(
        r#"\.route\s*\(\s*"([^"]+)"\s*,\s*(get|post|put|patch|delete|head|options)\s*\("#,
    )
    .expect("valid rust route regex");
    for capture in route.captures_iter(text) {
        add_route(state, &capture[2].to_ascii_uppercase(), &capture[1], file);
    }
}

fn scan_python_routes(text: &str, file: &str, state: &mut ScanState) {
    let route = Regex::new(
        r#"@\w+(?:\.\w+)?\s*\.\s*(get|post|put|patch|delete|head|options|route)\s*\(\s*["']([^"']+)["']"#,
    )
    .expect("valid python route regex");
    for capture in route.captures_iter(text) {
        let method = if capture[1].eq_ignore_ascii_case("route") {
            "GET".to_string()
        } else {
            capture[1].to_ascii_uppercase()
        };
        add_route(state, &method, &capture[2], file);
    }
}

fn scan_next_route(file: &str, state: &mut ScanState) {
    let normalized = file.replace('\\', "/");
    if !normalized.contains("/api/") && !normalized.starts_with("api/") {
        return;
    }
    if ![".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"]
        .iter()
        .any(|suffix| normalized.ends_with(suffix))
    {
        return;
    }

    let mut api = normalized
        .trim_start_matches("src/")
        .trim_start_matches("pages/api/")
        .trim_start_matches("app/api/")
        .to_string();
    if !api.starts_with("/api/") {
        api = format!("/api/{api}");
    }
    for suffix in [".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"] {
        api = api.trim_end_matches(suffix).to_string();
    }
    api = api.trim_end_matches("/route").to_string();
    api = Regex::new(r"\[([^\]]+)\]")
        .expect("valid next param regex")
        .replace_all(&api, ":$1")
        .to_string();
    add_route(state, "GET", &api, file);
}

fn add_route(state: &mut ScanState, method: &str, route_path: &str, file: &str) {
    if !route_path.starts_with('/') {
        return;
    }
    let route = Route {
        method: method.to_ascii_uppercase(),
        path: normalize_route_path(route_path),
        file: file.to_string(),
    };
    if state
        .routes
        .iter()
        .any(|item| item.method == route.method && item.path == route.path)
    {
        return;
    }
    let action = format!("{} {}", route.method, route.path);
    if is_safe_method(&route.method) {
        state.safe_actions.insert(action);
    } else {
        state.risky_actions.insert(action);
    }
    state.routes.push(route);
}

fn apply_config(state: &mut ScanState, config: &Config) {
    if !config.name.is_empty() {
        state.name = config.name.clone();
    }
    if !config.purpose.is_empty() {
        state.purpose = config.purpose.clone();
    }
    if !config.api_base.is_empty() {
        state.api_base = config.api_base.clone();
    }
    if !config.auth.is_empty() {
        state.auth = config.auth.clone();
    }
    if !config.support.is_empty() {
        state.support = config.support.clone();
    }
    if !config.limits.is_empty() {
        state.limits = config.limits.clone();
    }
    for action in &config.safe_actions {
        state.safe_actions.insert(action.clone());
    }
    for action in &config.risky_actions {
        state.risky_actions.insert(action.clone());
    }
    for doc in &config.docs {
        add_url(&mut state.docs, Some(doc.clone()));
    }

    if state.name.is_empty() {
        state.name = state
            .root
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "project".to_string());
    }
    if state.purpose.is_empty() {
        state.purpose = "This project exposes code and docs for agents.".to_string();
    }
    if state.api_base.is_empty() {
        state.api_base = guess_api_base(&state.routes);
    }
    if state.auth.is_empty() {
        state.auth = "Auth rules are not set. Check the API docs before you call it.".to_string();
    }
    if state.limits.is_empty() {
        state.limits = "No rate limit was found. Use a slow pace.".to_string();
    }
}

fn to_result(mut state: ScanState) -> ScanResult {
    state.routes.sort_by(|a, b| {
        format!("{} {}", a.path, a.method).cmp(&format!("{} {}", b.path, b.method))
    });
    ScanResult {
        root: state.root,
        name: state.name,
        purpose: state.purpose,
        api_base: state.api_base,
        auth: state.auth,
        docs: state.docs.into_iter().collect(),
        support: state.support,
        limits: state.limits,
        safe_actions: state.safe_actions.into_iter().collect(),
        risky_actions: state.risky_actions.into_iter().collect(),
        notes: state.notes.into_iter().collect(),
        routes: state.routes,
        languages: state.languages.into_iter().collect(),
        files_scanned: state.files_scanned,
        warnings: state.warnings,
    }
}

fn guess_api_base(routes: &[Route]) -> String {
    if routes.iter().any(|route| route.path.starts_with("/api/v1")) {
        "/api/v1".to_string()
    } else if routes.iter().any(|route| route.path.starts_with("/api")) {
        "/api".to_string()
    } else {
        "/".to_string()
    }
}

fn clean_comment_line(line: &str) -> String {
    line.trim()
        .trim_start_matches('/')
        .trim_start_matches('#')
        .trim_start_matches('*')
        .trim()
        .to_string()
}

fn starts_with_action_word(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    ["use", "read", "ask", "do", "do not", "prefer", "stop"]
        .iter()
        .any(|word| lower.starts_with(word))
}

fn language_for_extension(extension: &str) -> &str {
    match extension {
        "go" => "go",
        "rs" => "rust",
        "py" => "python",
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" => "node",
        _ => "",
    }
}

fn is_safe_method(method: &str) -> bool {
    matches!(method, "GET" | "HEAD" | "OPTIONS")
}

fn normalize_route_path(route_path: &str) -> String {
    let mut output = route_path.replace("//", "/");
    while output.ends_with('/') && output.len() > 1 {
        output.pop();
    }
    if output.is_empty() {
        "/".to_string()
    } else {
        output
    }
}

fn json_string(value: &JsonValue, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn toml_string(value: &toml::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn set_if_empty(target: &mut String, value: Option<String>) {
    if target.is_empty() {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            *target = value;
        }
    }
}

fn add_url(set: &mut BTreeSet<String>, value: Option<String>) {
    let Some(value) = value else {
        return;
    };
    let clean = value
        .trim()
        .trim_start_matches("git+")
        .trim_end_matches(".git");
    if clean.starts_with("http://") || clean.starts_with("https://") || clean.starts_with('/') {
        set.insert(clean.to_string());
    }
}

fn trim_sentence(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches(['.', ';'])
        .to_string()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .to_string()
}
