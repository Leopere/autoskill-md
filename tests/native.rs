use autoskill_md::config::Config;
use autoskill_md::constants::{CREDIT_URL, SPEC_URL, SPEC_VERSION};
use autoskill_md::generate::generate_skills_markdown;
use autoskill_md::readability::check_readability;
use autoskill_md::scan::scan_project;
use autoskill_md::secrets::{find_secrets, redact_secrets};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn scan_project_reads_metadata_hints_and_routes() {
    let root = temp_dir("scan");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"name":"tickets-api","description":"This API lets agents read ticket status.","homepage":"https://example.com/docs"}"#,
    )
    .unwrap();
    fs::write(root.join("go.mod"), "module github.com/example/tickets\n").unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"tickets-rs\"\ndescription = \"Ticket helpers.\"\n",
    )
    .unwrap();
    fs::write(
        root.join("pyproject.toml"),
        "[project]\nname = \"tickets-py\"\ndescription = \"Ticket tools.\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/routes.js"),
        r#"
// autoskill: auth: Public reads need no auth.
// autoskill: safe: GET ticket status by id.
app.get("/api/v1/tickets/:id", handler);
app.post("/api/v1/tickets", handler);
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/main.go"),
        r#"package main
func main() { http.HandleFunc("/api/v1/health", h) }
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        r#"#[get("/api/v1/profile")]
async fn profile() {}
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/app.py"),
        r#"@app.delete("/api/v1/tickets/{ticket_id}")
def delete_ticket(): pass
"#,
    )
    .unwrap();

    let scan = scan_project(&root, &Config::default());
    assert_eq!(scan.name, "tickets-api");
    assert_eq!(scan.purpose, "This API lets agents read ticket status.");
    assert_eq!(scan.api_base, "/api/v1");
    assert_eq!(scan.auth, "Public reads need no auth");
    assert_eq!(scan.languages, vec!["go", "node", "python", "rust"]);
    assert!(scan.docs.contains(&"https://example.com/docs".to_string()));
    assert!(scan
        .safe_actions
        .contains(&"GET /api/v1/tickets/:id".to_string()));
    assert!(scan
        .safe_actions
        .contains(&"GET ticket status by id".to_string()));
    assert!(scan
        .risky_actions
        .contains(&"POST /api/v1/tickets".to_string()));
    assert!(scan
        .risky_actions
        .contains(&"DELETE /api/v1/tickets/{ticket_id}".to_string()));
}

#[test]
fn generated_markdown_matches_spec_shape() {
    let scan = autoskill_md::scan::ScanResult {
        root: PathBuf::from("."),
        name: "tickets-api".to_string(),
        purpose: "This API lets agents read ticket status.".to_string(),
        api_base: "/api/v1".to_string(),
        auth: "Public reads need no auth.".to_string(),
        limits: "Use a slow pace.".to_string(),
        languages: vec!["node".to_string()],
        routes: vec![],
        safe_actions: vec!["GET /api/v1/tickets/:id".to_string()],
        risky_actions: vec!["Ask before ticket changes".to_string()],
        docs: vec!["https://example.com/docs".to_string()],
        support: "https://example.com/support".to_string(),
        notes: vec![],
        files_scanned: 1,
        warnings: vec![],
    };

    let markdown = generate_skills_markdown(&scan);
    assert!(markdown.starts_with("# Skills"));
    assert!(markdown.contains("## Purpose"));
    assert!(markdown.contains("## Auth"));
    assert!(markdown.contains("## Safe Actions"));
    assert!(markdown.contains("## Risky Actions"));
    assert!(markdown.contains(SPEC_URL));
    assert!(markdown.contains(SPEC_VERSION));
    assert!(markdown.contains(CREDIT_URL));
    assert!(markdown.contains("CC-BY-4.0"));
    assert!(find_secrets(&markdown).is_empty());
    assert!(check_readability(&markdown).ok);
}

#[test]
fn docs_stay_below_grade_7() {
    let mut files = vec!["README.md".to_string(), "LICENSE".to_string()];
    for entry in fs::read_dir("docs").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push(path.to_string_lossy().to_string());
        }
    }
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        let result = check_readability(&text);
        assert!(result.ok, "{file} grade {} is above 7", result.grade);
    }
}

#[test]
fn secrets_are_found_and_redacted() {
    let text = "api_key = abcdefghijklmnopqrstuvwxyz123456";
    let findings = find_secrets(text);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].name, "named secret");
    assert_eq!(
        redact_secrets("Use Bearer abcdefghijklmnopqrstuvwxyz1234567890"),
        "Use [redacted secret]"
    );
}

#[test]
fn cli_generate_and_check_work() {
    let root = temp_dir("cli");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"name":"cli-api","description":"This API lets agents read status."}"#,
    )
    .unwrap();
    fs::write(
        root.join("src/app.js"),
        r#"app.get("/api/status", handler);"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_autoskill-md");
    let generate = Command::new(bin)
        .args(["generate", "--root"])
        .arg(&root)
        .arg("--quiet")
        .status()
        .unwrap();
    assert!(generate.success());

    let markdown = fs::read_to_string(root.join(".well-known/skills.md")).unwrap();
    assert!(markdown.contains("# Skills"));
    assert!(markdown.contains(SPEC_URL));

    let check = Command::new(bin)
        .args(["check", "--root"])
        .arg(&root)
        .args(["--strict", "--json"])
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let stdout = String::from_utf8_lossy(&check.stdout);
    assert!(stdout.contains(r#""ok": true"#));
    assert!(stdout.contains(r#""creditUrl": "https://colinknapp.com""#));
}

#[test]
fn cli_check_strict_fails_when_stale() {
    let root = temp_dir("stale");
    fs::write(
        root.join("package.json"),
        r#"{"name":"stale-api","description":"This API lets agents read status."}"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_autoskill-md");
    assert!(Command::new(bin)
        .args(["generate", "--root"])
        .arg(&root)
        .arg("--quiet")
        .status()
        .unwrap()
        .success());
    fs::write(
        root.join(".well-known/skills.md"),
        "# Skills\n\nOld text.\n",
    )
    .unwrap();

    let check = Command::new(bin)
        .args(["check", "--root"])
        .arg(&root)
        .args(["--strict", "--json"])
        .output()
        .unwrap();
    assert!(!check.status.success());
    let stdout = String::from_utf8_lossy(&check.stdout);
    assert!(stdout.contains("stale"));
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("autoskill-md-{label}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}
