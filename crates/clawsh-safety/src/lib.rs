use regex::Regex;
use std::sync::OnceLock;

static DANGEROUS_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn patterns() -> &'static Vec<Regex> {
    DANGEROUS_PATTERNS.get_or_init(|| {
        let raw = [
            r"rm\s+-[a-zA-Z]*r[a-zA-Z]*f",
            r"rm\s+-[a-zA-Z]*f[a-zA-Z]*r",
            r"\bdd\b.+of=",
            r"\bmkfs\b",
            r"chmod\s+-R\s+[0-7]*7[0-7]*\s+/",
            r":\(\)\s*\{.*\}.*:",
            r">\s*/dev/sd[a-z]",
        ];
        raw.iter().map(|p| Regex::new(p).unwrap()).collect()
    })
}

pub fn is_dangerous(cmd: &str) -> bool {
    patterns().iter().any(|re| re.is_match(cmd))
}
