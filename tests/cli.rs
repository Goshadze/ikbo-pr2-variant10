//! Интеграционные тесты CLI-интерфейса, без сети.

use std::path::Path;
use std::process::{Command, Output};

fn run_depgraph(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_depgraph"))
        .args(args)
        .output()
        .expect("бинарный файл depgraph должен быть собран")
}

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .to_string_lossy()
        .to_string()
}

#[test]
fn prints_key_value_pairs_on_success() {
    let output = run_depgraph(&[
        "--package", "A",
        "--repo", &fixture("cycle.txt"),
        "--test-mode", "file",
        "--ascii-tree", "on",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("package=A"));
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
fn file_mode_prints_graph_in_bfs_order() {
    let output = run_depgraph(&["--package", "A", "--repo", &fixture("cycle.txt"), "-t", "file"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("A\nB\n"));
}

#[test]
fn file_mode_reports_invalid_line_with_number() {
    let output = run_depgraph(&["--package", "C", "--repo", &fixture("invalid.txt"), "-t", "file"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("строка 2"));
}

#[test]
fn file_mode_applies_substring_filter() {
    let output = run_depgraph(&[
        "--package", "A",
        "--repo", &fixture("filter.txt"),
        "-t", "file",
        "-f", "AB",
    ]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("A\n"));
}

#[test]
fn reports_package_not_found_in_test_file() {
    let output = run_depgraph(&["--package", "Z", "--repo", &fixture("leaf.txt"), "-t", "file"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("не найден"));
}
