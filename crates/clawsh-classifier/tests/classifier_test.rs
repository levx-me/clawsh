use clawsh_classifier::{classify, InputKind};

#[test]
fn test_posix_commands_classified_correctly() {
    assert_eq!(classify("ls -la"), InputKind::Posix);
    assert_eq!(classify("git status"), InputKind::Posix);
    assert_eq!(classify("cargo build --release"), InputKind::Posix);
    assert_eq!(classify("cd /home/user"), InputKind::Posix);
    assert_eq!(classify("echo hello world"), InputKind::Posix);
}

#[test]
fn test_natural_language_classified_correctly() {
    assert_eq!(classify("kill the process using port 8080"), InputKind::NaturalLanguage);
    assert_eq!(classify("show files modified last week"), InputKind::NaturalLanguage);
    assert_eq!(classify("how much disk space do I have"), InputKind::NaturalLanguage);
}

#[test]
fn test_shell_builtins_are_posix() {
    assert_eq!(classify("export FOO=bar"), InputKind::Posix);
    assert_eq!(classify("source ~/.bashrc"), InputKind::Posix);
}
