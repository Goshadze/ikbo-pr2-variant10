//! Тесты парсера тестового репозитория (этап 3), форматы и ошибки строк.

use depgraph::testrepo::{parse_text, TestRepoError};

#[test]
fn parses_dependencies_and_empty_lists() {
    let map = parse_text("A: B C\nX:\n").expect("корректный файл");
    assert_eq!(map.get("A").unwrap(), &names(&["B", "C"]));
    assert!(map.get("X").unwrap().is_empty());
}

#[test]
fn ignores_blank_lines_and_comments() {
    let map = parse_text("# комментарий\n\nA: B\n").expect("корректный файл");
    assert_eq!(map.get("A").unwrap(), &names(&["B"]));
}

#[test]
fn allows_cycle_in_file() {
    let map = parse_text("A: B\nB: A").expect("цикл допустим");
    assert_eq!(map.get("B").unwrap(), &names(&["A"]));
}

#[test]
fn reports_invalid_line_with_number() {
    let error = parse_text("A: B\nC: d\n").expect_err("имя вне [A-Z]+");
    assert!(matches!(error, TestRepoError::InvalidLine { line: 2, .. }));
}

#[test]
fn reports_line_without_colon_with_number() {
    let error = parse_text("A B C").expect_err("нет разделителя :");
    assert!(matches!(error, TestRepoError::InvalidLine { line: 1, .. }));
}

#[test]
fn reports_empty_name_with_number() {
    let error = parse_text(": B").expect_err("пустое имя пакета");
    assert!(matches!(error, TestRepoError::InvalidLine { line: 1, .. }));
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().copied().map(str::to_string).collect()
}
