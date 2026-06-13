pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SPEC_VERSION: &str = "2026-06-13";
pub const SPEC_URL: &str = "https://colinknapp.com/specs/skill-discovery.html";
pub const CREDIT_URL: &str = "https://colinknapp.com";
pub const DEFAULT_OUT: &str = ".well-known/skills.md";
pub const DEFAULT_CONFIG: &str = "autoskill.config.json";

pub const DEFAULT_IGNORES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".turbo",
    ".venv",
    "__pycache__",
    "__tests__",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "target",
    "test",
    "tests",
    "vendor",
];
