use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{load_config, write_default_config};
use crate::constants::{CREDIT_URL, DEFAULT_CONFIG, DEFAULT_OUT, SPEC_URL, VERSION};
use crate::generate::generate_skills_markdown;
use crate::readability::{check_readability, Readability};
use crate::scan::scan_project;
use crate::secrets::find_secrets;

#[derive(Default)]
struct Options {
    root: Option<String>,
    out: Option<String>,
    config: Option<String>,
    strict: bool,
    quiet: bool,
    json: bool,
    help: bool,
    version: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonStatus {
    ok: bool,
    strict: bool,
    output: String,
    readability: JsonReadability,
    problems: Vec<String>,
    warnings: Vec<String>,
    credit_url: &'static str,
    spec_url: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonReadability {
    grade: f64,
    max_grade: f64,
    ok: bool,
}

pub fn run(argv: Vec<String>) -> i32 {
    match run_inner(argv) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("autoskill-md: {error}");
            1
        }
    }
}

fn run_inner(argv: Vec<String>) -> Result<i32, String> {
    let (command, options) = parse_args(argv)?;

    if options.version {
        println!("autoskill-md {VERSION}");
        println!("Credit: {CREDIT_URL}");
        return Ok(0);
    }

    if options.help || command.is_none() {
        print_help();
        return Ok(0);
    }

    match command.as_deref() {
        Some("init") => init(options),
        Some("generate") => generate(options),
        Some("check") => check(options),
        Some(command) => Err(format!("Unknown command: {command}")),
        None => Ok(0),
    }
}

fn init(options: Options) -> Result<i32, String> {
    let root = root_path(&options)?;
    let file = write_default_config(&root, options.config.as_deref())?;
    if !options.quiet {
        println!("Wrote {}", relative(&root, &file));
        println!("Credit: {CREDIT_URL}");
    }
    Ok(0)
}

fn generate(options: Options) -> Result<i32, String> {
    let root = root_path(&options)?;
    let out = root.join(options.out.as_deref().unwrap_or(DEFAULT_OUT));
    let config = load_config(&root, options.config.as_deref().or(Some(DEFAULT_CONFIG)))?;
    let scan = scan_project(&root, &config);
    let markdown = generate_skills_markdown(&scan);
    let validation = validate(&markdown);

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    fs::write(&out, markdown)
        .map_err(|error| format!("Could not write {}: {error}", out.display()))?;

    if !options.quiet {
        println!("Wrote {}", relative(&root, &out));
        println!("Credit: {CREDIT_URL}");
        println!("Spec: {SPEC_URL}");
        println!("Reading grade: {}", validation.readability.grade);
        print_warnings(&scan.warnings);
        print_problems(&validation.problems);
    }

    Ok(if options.strict && !validation.problems.is_empty() {
        1
    } else {
        0
    })
}

fn check(options: Options) -> Result<i32, String> {
    let root = root_path(&options)?;
    let out = root.join(options.out.as_deref().unwrap_or(DEFAULT_OUT));
    let config = load_config(&root, options.config.as_deref().or(Some(DEFAULT_CONFIG)))?;
    let scan = scan_project(&root, &config);
    let expected = generate_skills_markdown(&scan);
    let validation = validate(&expected);
    let mut problems = validation.problems;

    match fs::read_to_string(&out) {
        Ok(current) => {
            if normalize_newlines(&current) != normalize_newlines(&expected) {
                problems.push(format!(
                    "{} is stale. Run autoskill-md generate.",
                    relative(&root, &out)
                ));
            }
        }
        Err(_) => {
            problems.push(format!("Missing {}.", relative(&root, &out)));
        }
    }

    if options.json {
        let status = JsonStatus {
            ok: problems.is_empty(),
            strict: options.strict,
            output: relative(&root, &out),
            readability: validation.readability.into(),
            problems: problems.clone(),
            warnings: scan.warnings.clone(),
            credit_url: CREDIT_URL,
            spec_url: SPEC_URL,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&status)
                .map_err(|error| format!("Could not write JSON status: {error}"))?
        );
    } else if !options.quiet {
        if problems.is_empty() {
            println!("skills.md is up to date.");
        }
        println!("Credit: {CREDIT_URL}");
        println!("Spec: {SPEC_URL}");
        println!("Reading grade: {}", validation.readability.grade);
        print_warnings(&scan.warnings);
        print_problems(&problems);
    }

    Ok(if options.strict && !problems.is_empty() {
        1
    } else {
        0
    })
}

struct Validation {
    readability: Readability,
    problems: Vec<String>,
}

fn validate(markdown: &str) -> Validation {
    let readability = check_readability(markdown);
    let mut problems = Vec::new();
    if !readability.ok {
        problems.push(format!(
            "Reading grade {} is above {}.",
            readability.grade, readability.max_grade
        ));
    }
    for secret in find_secrets(markdown) {
        problems.push(format!("Found {}: {}", secret.name, secret.sample));
    }
    Validation {
        readability,
        problems,
    }
}

impl From<Readability> for JsonReadability {
    fn from(value: Readability) -> Self {
        JsonReadability {
            grade: value.grade,
            max_grade: value.max_grade,
            ok: value.ok,
        }
    }
}

fn parse_args(argv: Vec<String>) -> Result<(Option<String>, Options), String> {
    let mut args = argv.into_iter();
    let mut command = None;
    let mut options = Options::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => options.help = true,
            "--version" | "-V" => options.version = true,
            "--root" => options.root = Some(required_value("--root", args.next())?),
            "--out" => options.out = Some(required_value("--out", args.next())?),
            "--config" => options.config = Some(required_value("--config", args.next())?),
            "--strict" => options.strict = true,
            "--quiet" => options.quiet = true,
            "--json" => options.json = true,
            value if value.starts_with('-') => return Err(format!("Unknown flag: {value}")),
            value if command.is_none() => command = Some(value.to_string()),
            value => return Err(format!("Unexpected argument: {value}")),
        }
    }

    Ok((command, options))
}

fn required_value(flag: &str, value: Option<String>) -> Result<String, String> {
    value.ok_or_else(|| format!("{flag} needs a value"))
}

fn root_path(options: &Options) -> Result<PathBuf, String> {
    match &options.root {
        Some(root) => Ok(PathBuf::from(root)),
        None => std::env::current_dir().map_err(|error| format!("Could not read cwd: {error}")),
    }
}

fn print_help() {
    println!(
        "autoskill-md {VERSION}

Usage:
  autoskill-md init [--root path]
  autoskill-md generate [--root path] [--out .well-known/skills.md] [--strict]
  autoskill-md check [--root path] [--out .well-known/skills.md] [--strict] [--json]
  autoskill-md --version

Defaults:
  --root current directory
  --out .well-known/skills.md

Credit: {CREDIT_URL}
Spec: {SPEC_URL}"
    );
}

fn print_warnings(warnings: &[String]) {
    for warning in warnings {
        println!("Warning: {warning}");
    }
}

fn print_problems(problems: &[String]) {
    for problem in problems {
        println!("Problem: {problem}");
    }
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n")
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}
