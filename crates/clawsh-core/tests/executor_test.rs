use clawsh_core::executor::execute;

#[tokio::test]
async fn test_simple_command_succeeds() {
    let result = execute("echo hello").await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
}

#[tokio::test]
async fn test_failing_command_returns_nonzero() {
    let result = execute("ls /nonexistent_path_xyz_123").await.unwrap();
    assert_ne!(result.exit_code, 0);
    assert!(!result.stderr.is_empty());
}

#[tokio::test]
async fn test_pipe_command() {
    let result = execute("echo hello | grep hello").await.unwrap();
    assert_eq!(result.exit_code, 0);
}
