use std::process::Command;

fn rivu_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rivutv"))
}

#[test]
fn test_cli_run_accepts_no_args() {
    let output = rivu_bin()
        .arg("--help")
        .output()
        .expect("failed to execute binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RivuTV"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("search"));
    assert!(stdout.contains("play"));
    assert!(stdout.contains("sources"));
}

#[test]
fn test_cli_version_flag() {
    let output = rivu_bin()
        .arg("--version")
        .output()
        .expect("failed to execute binary");
    assert!(output.status.success());
}

#[test]
fn test_cli_invalid_subcommand_returns_error() {
    let output = rivu_bin()
        .arg("invalid-command")
        .output()
        .expect("failed to execute binary");
    assert!(!output.status.success());
}

#[test]
fn test_cli_search_requires_keyword() {
    let output = rivu_bin()
        .arg("search")
        .output()
        .expect("failed to execute binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("KEYWORD"));
}

#[test]
fn test_cli_config_requires_url() {
    let output = rivu_bin()
        .arg("config")
        .output()
        .expect("failed to execute binary");
    assert!(!output.status.success());
}

#[test]
fn test_cli_play_requires_url() {
    let output = rivu_bin()
        .arg("play")
        .output()
        .expect("failed to execute binary");
    assert!(!output.status.success());
}
