//! Разбор и валидация параметров командной строки (этап 1).

use std::fmt;
use std::iter::Peekable;
use std::path::Path;
use std::slice::Iter;

/// Имя файла с изображением графа по умолчанию.
const DEFAULT_OUTPUT_FILE: &str = "graph.png";

/// Допустимые значения параметра `--test-mode` (для сообщений об ошибках).
const TEST_MODE_EXPECTED: &str = "url | file";

/// Допустимые значения параметра `--ascii-tree` (для сообщений об ошибках).
const ASCII_TREE_EXPECTED: &str = "off | on";

/// Префиксы URL, допустимые в режиме реального репозитория.
const URL_PREFIXES: [&str; 2] = ["http://", "https://"];

/// Режим работы с тестовым репозиторием.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    /// Параметр `--repo` содержит URL реального репозитория.
    Url,
    /// Параметр `--repo` содержит путь к локальному тестовому файлу.
    File,
}

impl TestMode {
    /// Преобразует текстовое значение (`url` или `file`) в режим.
    pub fn parse(value: &str) -> Option<TestMode> {
        match value {
            "url" => Some(TestMode::Url),
            "file" => Some(TestMode::File),
            _ => None,
        }
    }

    /// Текстовое представление режима для вывода в формате ключ-значение.
    pub fn as_str(self) -> &'static str {
        match self {
            TestMode::Url => "url",
            TestMode::File => "file",
        }
    }
}

/// Ошибки разбора и валидации параметров командной строки.
#[derive(Debug)]
pub enum ConfigError {
    /// Обязательный параметр не задан.
    MissingRequired(String),
    /// У параметра отсутствует значение.
    MissingValue(String),
    /// Передан неизвестный параметр.
    UnknownOption(String),
    /// Один и тот же параметр передан несколько раз.
    DuplicateOption(String),
    /// Значение строкового параметра пустое или состоит из пробелов.
    EmptyValue(String),
    /// Значение не входит в допустимый набор.
    InvalidValue { option: String, value: String, expected: &'static str },
    /// URL репозитория имеет неподдерживаемую схему.
    InvalidUrl(String),
    /// Файл тестового репозитория не найден или недоступен.
    FileNotFound(String),
    /// Передан аргумент, который не является параметром.
    UnexpectedArgument(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingRequired(name) => {
                write!(f, "не задан обязательный параметр --{name}")
            }
            ConfigError::MissingValue(name) => {
                write!(f, "у параметра {name} отсутствует значение")
            }
            ConfigError::UnknownOption(name) => {
                write!(f, "неизвестный параметр {name}, см. --help")
            }
            ConfigError::DuplicateOption(name) => {
                write!(f, "параметр {name} задан более одного раза")
            }
            ConfigError::EmptyValue(name) => {
                write!(f, "значение параметра {name} не может быть пустым")
            }
            ConfigError::InvalidValue { option, value, expected } => {
                write!(f, "недопустимое значение «{value}» для --{option}, ожидается {expected}")
            }
            ConfigError::InvalidUrl(url) => {
                write!(f, "некорректный URL репозитория (ожидается http:// или https:// и имя хоста): {url}")
            }
            ConfigError::FileNotFound(path) => {
                write!(f, "файл тестового репозитория не найден: {path}")
            }
            ConfigError::UnexpectedArgument(arg) => {
                write!(f, "лишний аргумент командной строки: {arg}, см. --help")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Пользовательские параметры приложения (этап 1).
#[derive(Debug, Clone)]
pub struct Config {
    /// Имя анализируемого пакета.
    pub package: String,
    /// URL репозитория или путь к файлу тестового репозитория.
    pub repo: String,
    /// Режим работы с тестовым репозиторием.
    pub test_mode: TestMode,
    /// Имя файла с изображением графа.
    pub output_file: String,
    /// Выводить ли зависимости в формате ASCII-дерева (этап 5).
    pub ascii_tree: bool,
    /// Подстрока для фильтрации пакетов; пустая строка — без фильтра.
    pub filter: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "package={}", self.package)?;
        writeln!(f, "repo={}", self.repo)?;
        writeln!(f, "test_mode={}", self.test_mode.as_str())?;
        writeln!(f, "output_file={}", self.output_file)?;
        let tree = if self.ascii_tree { "on" } else { "off" };
        writeln!(f, "ascii_tree={tree}")?;
        writeln!(f, "filter={}", display_filter(&self.filter))
    }
}

/// Представляет фильтр для вывода: пустой фильтр показываем пометкой.
fn display_filter(filter: &str) -> String {
    if filter.is_empty() {
        "(пусто)".to_string()
    } else {
        filter.to_string()
    }
}

/// Промежуточное состояние разбора: заполненные слоты параметров.
#[derive(Default)]
struct ParsedState {
    package: Option<String>,
    repo: Option<String>,
    test_mode: Option<TestMode>,
    output_file: Option<String>,
    ascii_tree: Option<bool>,
    filter: Option<String>,
}

/// Разбирает аргументы командной строки и возвращает валидный [`Config`].
pub fn parse_args(args: &[String]) -> Result<Config, ConfigError> {
    let mut state = ParsedState::default();
    let mut iter = args.iter().peekable();
    while let Some(token) = iter.next() {
        let (name, inline_value) = split_option(token)?;
        let value = read_value(&mut iter, &name, inline_value)?;
        apply_option(&mut state, &name, &value)?;
    }
    build_config(state)
}

/// Разбивает токен на имя параметра и возможное значение после `=`.
fn split_option(token: &str) -> Result<(String, Option<String>), ConfigError> {
    if let Some(rest) = token.strip_prefix("--") {
        let (name, value) = rest
            .split_once('=')
            .map(|(n, v)| (n, Some(v)))
            .unwrap_or((rest, None));
        return Ok((format!("--{name}"), value.map(str::to_string)));
    }
    if let Some(rest) = token.strip_prefix('-') {
        if !rest.is_empty() {
            return Ok((format!("-{rest}"), None));
        }
    }
    Err(ConfigError::UnexpectedArgument(token.to_string()))
}

/// Возвращает значение параметра: после `=` или из следующего токена.
fn read_value(
    iter: &mut Peekable<Iter<'_, String>>,
    name: &str,
    inline: Option<String>,
) -> Result<String, ConfigError> {
    if let Some(value) = inline {
        return Ok(value);
    }
    if matches!(iter.peek(), Some(next) if next.starts_with('-')) {
        return Err(ConfigError::MissingValue(name.to_string()));
    }
    iter.next()
        .map(|value| value.to_string())
        .ok_or_else(|| ConfigError::MissingValue(name.to_string()))
}

/// Записывает значение параметра в соответствующий слот состояния.
fn apply_option(state: &mut ParsedState, name: &str, value: &str) -> Result<(), ConfigError> {
    match name {
        "--package" | "-p" => set_string(&mut state.package, "package", value)?,
        "--repo" | "-r" => set_string(&mut state.repo, "repo", value)?,
        "--test-mode" | "-t" => set_choice(
            &mut state.test_mode,
            "test-mode",
            value,
            TEST_MODE_EXPECTED,
            TestMode::parse,
        )?,
        "--output" | "-o" => set_string(&mut state.output_file, "output", value)?,
        "--ascii-tree" | "-a" => set_choice(
            &mut state.ascii_tree,
            "ascii-tree",
            value,
            ASCII_TREE_EXPECTED,
            parse_flag,
        )?,
        "--filter" | "-f" => set_string(&mut state.filter, "filter", value)?,
        other => return Err(ConfigError::UnknownOption(other.to_string())),
    }
    Ok(())
}

/// Сохраняет строковое значение после проверки на дубли и пустоту.
fn set_string(slot: &mut Option<String>, option: &str, value: &str) -> Result<(), ConfigError> {
    if slot.is_some() {
        return Err(ConfigError::DuplicateOption(option.to_string()));
    }
    if value.trim().is_empty() {
        return Err(ConfigError::EmptyValue(option.to_string()));
    }
    *slot = Some(value.to_string());
    Ok(())
}

/// Сохраняет значение из допустимого набора после проверки на дубли.
fn set_choice<T>(
    slot: &mut Option<T>,
    option: &str,
    value: &str,
    expected: &'static str,
    parse: impl Fn(&str) -> Option<T>,
) -> Result<(), ConfigError> {
    if slot.is_some() {
        return Err(ConfigError::DuplicateOption(option.to_string()));
    }
    let parsed = parse(value).ok_or_else(|| ConfigError::InvalidValue {
        option: option.to_string(),
        value: value.to_string(),
        expected,
    })?;
    *slot = Some(parsed);
    Ok(())
}

/// Преобразует текстовое значение флага (`on` или `off`) в булево.
fn parse_flag(value: &str) -> Option<bool> {
    match value {
        "on" => Some(true),
        "off" => Some(false),
        _ => None,
    }
}

/// Собирает итоговый [`Config`], применяя значения по умолчанию.
fn build_config(state: ParsedState) -> Result<Config, ConfigError> {
    let package = required(&state.package, "package")?;
    let repo = required(&state.repo, "repo")?;
    let test_mode = state.test_mode.unwrap_or(TestMode::Url);
    validate_repo(&repo, test_mode)?;
    Ok(Config {
        package,
        repo,
        test_mode,
        output_file: state.output_file.unwrap_or_else(|| DEFAULT_OUTPUT_FILE.to_string()),
        ascii_tree: state.ascii_tree.unwrap_or(false),
        filter: state.filter.unwrap_or_default(),
    })
}

/// Возвращает значение обязательного параметра или ошибку.
fn required(slot: &Option<String>, option: &str) -> Result<String, ConfigError> {
    slot.clone().ok_or_else(|| ConfigError::MissingRequired(option.to_string()))
}

/// Проверяет аргумент `--repo` с учётом выбранного режима.
fn validate_repo(repo: &str, mode: TestMode) -> Result<(), ConfigError> {
    match mode {
        TestMode::Url => ensure_url(repo),
        TestMode::File => ensure_file(repo),
    }
}

/// Проверяет, что адрес репозитория имеет схему http или https.
fn ensure_url(repo: &str) -> Result<(), ConfigError> {
    let valid = URL_PREFIXES.iter().any(|prefix| {
        repo.strip_prefix(prefix).is_some_and(|rest| {
            let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
            !host.is_empty() && !repo.chars().any(char::is_whitespace)
        })
    });
    if valid {
        Ok(())
    } else {
        Err(ConfigError::InvalidUrl(repo.to_string()))
    }
}

/// Проверяет, что путь указывает на существующий обычный файл.
fn ensure_file(path: &str) -> Result<(), ConfigError> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err(ConfigError::FileNotFound(path.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().copied().map(str::to_string).collect()
    }

    #[test]
    fn parses_all_options_with_long_names() {
        let config = parse_args(&args(&[
            "--package", "curl",
            "--repo", "https://archive.ubuntu.com/ubuntu/",
            "--test-mode", "url",
            "--output", "graph.png",
            "--ascii-tree", "on",
            "--filter", "lib",
        ]))
        .expect("все параметры заданы корректно");
        assert_eq!(config.package, "curl");
        assert_eq!(config.repo, "https://archive.ubuntu.com/ubuntu/");
        assert_eq!(config.test_mode, TestMode::Url);
        assert_eq!(config.output_file, "graph.png");
        assert!(config.ascii_tree);
        assert_eq!(config.filter, "lib");
    }

    #[test]
    fn applies_defaults_for_missing_optional_options() {
        let config = parse_args(&args(&[
            "--package", "curl",
            "--repo", "https://archive.ubuntu.com/ubuntu/",
        ]))
        .expect("обязательные параметры заданы");
        assert_eq!(config.test_mode, TestMode::Url);
        assert_eq!(config.output_file, DEFAULT_OUTPUT_FILE);
        assert!(!config.ascii_tree);
        assert!(config.filter.is_empty());
    }

    #[test]
    fn accepts_inline_values_after_equals() {
        let config = parse_args(&args(&[
            "--package=curl",
            "--repo=https://archive.ubuntu.com/ubuntu/",
            "--ascii-tree=on",
        ]))
        .expect("значения после = допустимы");
        assert_eq!(config.package, "curl");
        assert!(config.ascii_tree);
    }

    #[test]
    fn accepts_short_option_names() {
        let config = parse_args(&args(&[
            "-p", "curl",
            "-r", "Cargo.toml",
            "-t", "file",
            "-o", "out.png",
            "-a", "on",
            "-f", "lib",
        ]))
        .expect("короткие имена допустимы");
        assert_eq!(config.output_file, "out.png");
        assert_eq!(config.test_mode, TestMode::File);
    }

    #[test]
    fn rejects_missing_required_package() {
        let error = parse_args(&args(&["--repo", "https://example.com/"]))
            .expect_err("пакет не задан");
        assert!(matches!(error, ConfigError::MissingRequired(name) if name == "package"));
    }

    #[test]
    fn rejects_missing_required_repo() {
        let error = parse_args(&args(&["--package", "curl"]))
            .expect_err("репозиторий не задан");
        assert!(matches!(error, ConfigError::MissingRequired(name) if name == "repo"));
    }

    #[test]
    fn rejects_unknown_option() {
        let error = parse_args(&args(&[
            "--package", "curl",
            "--repo", "https://example.com/",
            "--bogus", "x",
        ]))
        .expect_err("неизвестный параметр");
        assert!(matches!(error, ConfigError::UnknownOption(name) if name == "--bogus"));
    }

    #[test]
    fn rejects_duplicate_option() {
        let error = parse_args(&args(&["--package", "curl", "--package", "wget"]))
            .expect_err("параметр задан дважды");
        assert!(matches!(error, ConfigError::DuplicateOption(name) if name == "package"));
    }

    #[test]
    fn rejects_empty_string_value() {
        let error = parse_args(&args(&[
            "--package", "",
            "--repo", "https://example.com/",
        ]))
        .expect_err("пустое значение");
        assert!(matches!(error, ConfigError::EmptyValue(name) if name == "package"));
    }

    #[test]
    fn rejects_invalid_test_mode_value() {
        let error = parse_args(&args(&[
            "--package", "curl",
            "--repo", "https://example.com/",
            "--test-mode", "auto",
        ]))
        .expect_err("недопустимый режим");
        assert!(matches!(error, ConfigError::InvalidValue { .. }));
    }

    #[test]
    fn rejects_invalid_ascii_tree_value() {
        let error = parse_args(&args(&[
            "--package", "curl",
            "--repo", "https://example.com/",
            "--ascii-tree", "yes",
        ]))
        .expect_err("недопустимое значение флага");
        assert!(matches!(error, ConfigError::InvalidValue { .. }));
    }

    #[test]
    fn rejects_url_without_supported_scheme() {
        let error = parse_args(&args(&["--package", "curl", "--repo", "ftp://example.com/"]))
            .expect_err("схема ftp не поддерживается");
        assert!(matches!(error, ConfigError::InvalidUrl(_)));
    }

    #[test]
    fn rejects_url_without_host_or_with_whitespace() {
        for url in ["https://", "https:///path", "https://example.com/a b"] {
            assert!(parse_args(&args(&["-p", "curl", "-r", url])).is_err());
        }
    }

    #[test]
    fn rejects_missing_test_repository_file() {
        let error = parse_args(&args(&[
            "--package", "A",
            "--repo", "/nonexistent/repo.txt",
            "--test-mode", "file",
        ]))
        .expect_err("файл не существует");
        assert!(matches!(error, ConfigError::FileNotFound(_)));
    }

    #[test]
    fn accepts_existing_test_repository_file() {
        let config = parse_args(&args(&[
            "--package", "A",
            "--repo", "Cargo.toml",
            "--test-mode", "file",
        ]))
        .expect("файл существует");
        assert_eq!(config.test_mode, TestMode::File);
    }

    #[test]
    fn rejects_option_without_value() {
        let error = parse_args(&args(&["--package"])).expect_err("значение не задано");
        assert!(matches!(error, ConfigError::MissingValue(name) if name == "--package"));
    }

    #[test]
    fn rejects_unexpected_positional_argument() {
        let error = parse_args(&args(&[
            "curl",
            "--repo", "https://example.com/",
        ]))
        .expect_err("лишний аргумент");
        assert!(matches!(error, ConfigError::UnexpectedArgument(_)));
    }

    #[test]
    fn prints_all_parameters_as_key_value_pairs() {
        let config = parse_args(&args(&[
            "--package", "curl",
            "--repo", "https://archive.ubuntu.com/ubuntu/",
        ]))
        .expect("параметры корректны");
        let text = format!("{config}");
        assert!(text.contains("package=curl"));
        assert!(text.contains("repo=https://archive.ubuntu.com/ubuntu/"));
        assert!(text.contains("test_mode=url"));
        assert!(text.contains("output_file=graph.png"));
        assert!(text.contains("ascii_tree=off"));
        assert!(text.contains("filter="));
    }
}
