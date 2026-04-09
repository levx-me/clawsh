use clawsh_safety::is_dangerous;

#[test]
fn test_rm_rf_is_dangerous() {
    assert!(is_dangerous("rm -rf /home/user"));
    assert!(is_dangerous("rm -rf *"));
}

#[test]
fn test_dd_is_dangerous() {
    assert!(is_dangerous("dd if=/dev/zero of=/dev/sda"));
}

#[test]
fn test_mkfs_is_dangerous() {
    assert!(is_dangerous("mkfs.ext4 /dev/sdb1"));
}

#[test]
fn test_chmod_recursive_is_dangerous() {
    assert!(is_dangerous("chmod -R 777 /"));
}

#[test]
fn test_safe_commands_are_not_dangerous() {
    assert!(!is_dangerous("ls -la"));
    assert!(!is_dangerous("git status"));
    assert!(!is_dangerous("cargo build"));
    assert!(!is_dangerous("rm -f /tmp/myfile.txt"));
}
