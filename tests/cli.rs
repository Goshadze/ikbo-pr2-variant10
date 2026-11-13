//! Интеграционные тесты CLI-интерфейса (этап 1).

use std::process::{Command, Output};

fn run_depgraph(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_depgraph"))
        .args(args)
        .output()
        .expect("бинарный файл depgraph должен быть собран")
}

#[test]
fn prints_key_value_pairs_on_success() {
    let output = run_depgraph(&[
        "--package", "curl",
        "--repo", "Cargo.toml",
        "--test-mode", "file",
        "--ascii-tree", "on",
    ]);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("package=curl"));
    assert!(stdout.contains("repo=Cargo.toml"));
    assert!(stdout.contains("test_mode=file"));
    assert!(stdout.contains("output_file=graph.png"));
    assert!(stdout.contains("ascii_tree=on"));
}

#[test]
fn exits_with_error_for_missing_required_option() {
    let output = run_depgraph(&["--repo", "https://archive.ubuntu.com/ubuntu/"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ошибка"));
    assert!(stderr.contains("--package"));
}

#[test]
fn exits_with_error_for_invalid_url_scheme() {
    let output = run_depgraph(&["--package", "curl", "--repo", "ftp://example.com/"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("http"));
}

#[test]
fn exits_with_error_for_unknown_option() {
    let output = run_depgraph(&[
        "--package", "curl",
        "--repo", "https://example.com/",
        "--nope", "x",
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--nope"));
}

#[test]
fn prints_help_and_exits_successfully() {
    let output = run_depgraph(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Использование"));
}

#[test]
fn file_mode_reports_support_from_stage_3() {
    let output = run_depgraph(&[
        "--package", "A",
        "--repo", "Cargo.toml",
        "--test-mode", "file",
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("с этапа 3"));
}
