//! Разбор файла тестового репозитория (формат этапа 3).

use std::fmt;

use crate::apt::DepMap;

/// Допустимые символы имени пакета тестового репозитория.
fn is_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_uppercase())
}

/// Ошибки разбора тестового репозитория.
#[derive(Debug)]
pub enum TestRepoError {
    /// Строка не соответствует формату или имя вне `[A-Z]+`.
    InvalidLine { line: usize, text: String },
    /// Файл не читается.
    Read { path: String, source: String },
}

impl fmt::Display for TestRepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestRepoError::InvalidLine { line, text } => {
                write!(f, "строка {line}: некорректный формат «{text}», ожидается «A: B C»")
            }
            TestRepoError::Read { path, source } => {
                write!(f, "не удалось прочитать {path}: {source}")
            }
        }
    }
}

impl std::error::Error for TestRepoError {}

/// Читает файл тестового репозитория и возвращает карту зависимостей.
pub fn parse_file(path: &str) -> Result<DepMap, TestRepoError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(source) => {
            return Err(TestRepoError::Read { path: path.to_string(), source: source.to_string() });
        }
    };
    parse_text(&text)
}

/// Разбирает текст файла: `A: B C` — пакет A зависит от B и C; цикл допустим.
/// Пустые строки и строки с `#` игнорируются; ошибка — с номером строки.
pub fn parse_text(text: &str) -> Result<DepMap, TestRepoError> {
    let mut map = DepMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, rest) = line.split_once(':').ok_or_else(|| error(index + 1, raw))?;
        if !is_name(name.trim()) {
            return Err(error(index + 1, raw));
        }
        let mut deps = Vec::new();
        for dep in rest.split_whitespace() {
            if !is_name(dep) {
                return Err(error(index + 1, raw));
            }
            deps.push(dep.to_string());
        }
        map.insert(name.trim().to_string(), deps);
    }
    Ok(map)
}

/// Собирает ошибку строки с её номером.
fn error(line: usize, text: &str) -> TestRepoError {
    TestRepoError::InvalidLine { line, text: text.to_string() }
}
