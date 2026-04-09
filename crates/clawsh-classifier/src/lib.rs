use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, PartialEq, Eq)]
pub enum InputKind {
    Posix,
    NaturalLanguage,
}

static POSIX_PROGRAMS: OnceLock<Regex> = OnceLock::new();
static NL_INDICATORS: OnceLock<Regex> = OnceLock::new();

fn posix_re() -> &'static Regex {
    POSIX_PROGRAMS.get_or_init(|| {
        Regex::new(
            r"(?x)^(
                ls|cd|pwd|echo|cat|grep|find|cp|mv|rm|mkdir|rmdir|
                touch|chmod|chown|ps|kill|top|df|du|tar|zip|unzip|
                curl|wget|git|cargo|npm|pip|python|python3|node|
                docker|kubectl|ssh|scp|rsync|sed|awk|cut|sort|uniq|
                head|tail|wc|tr|tee|xargs|which|whereis|man|
                export|source|alias|unalias|history|jobs|fg|bg|
                make|gcc|clang|rustc|go|java|
                systemctl|service|journalctl|apt|brew|yum|dnf|
                vim|nano|emacs|less|more|
                /[a-zA-Z]|\./
            )\b"
        ).unwrap()
    })
}

fn nl_re() -> &'static Regex {
    NL_INDICATORS.get_or_init(|| {
        Regex::new(
            r"(?i)\b(show|find|kill|delete|create|list|how|what|why|
                     get|give|open|close|stop|start|restart|check|
                     using|files|process|folder|directory|disk|memory|
                     modified|created|running|installed)\b"
        ).unwrap()
    })
}

pub fn classify(input: &str) -> InputKind {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed.starts_with('#') {
        return InputKind::Posix;
    }

    if trimmed.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return InputKind::NaturalLanguage;
    }

    // NL indicators with no flags or paths => natural language
    // e.g. "kill the process using port 8080" vs "kill -9 1234"
    let has_flag = trimmed.contains(" -");
    if nl_re().is_match(trimmed) && !trimmed.contains('/') && !has_flag {
        return InputKind::NaturalLanguage;
    }

    if posix_re().is_match(trimmed) {
        return InputKind::Posix;
    }

    InputKind::Posix
}
