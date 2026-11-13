//! Загрузка и разбор индекса пакетов apt (этап 2).

use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;

use flate2::read::GzDecoder;

/// Дистрибутив Ubuntu, формат пакетов которого используется.
pub const DISTRO: &str = "noble";

/// Архитектура пакетов, индексы которой загружаются.
pub const ARCH: &str = "amd64";

/// Компоненты репозитория, из которых собираются данные.
pub const COMPONENTS: [&str; 2] = ["main", "universe"];

/// Число попыток запроса индекса компонента (сеть к archive.ubuntu.com нестабильна).
const ATTEMPTS: usize = 3;

/// Поля записи пакета, задающие прямые зависимости.
const DEP_FIELDS: [&str; 2] = ["Depends", "Pre-Depends"];

/// Карта «пакет → прямые зависимости».
pub type DepMap = BTreeMap<String, Vec<String>>;

/// Ошибки сбора данных.
#[derive(Debug)]
pub enum DataError {
    /// Не удалось запросить или распаковать индекс компонента.
    Fetch { url: String, source: String },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Fetch { url, source } => {
                write!(f, "не удалось получить {url} за {ATTEMPTS} попыток: {source}")
            }
        }
    }
}

impl std::error::Error for DataError {}

/// Загружает индексы всех компонентов и строит карту зависимостей.
pub fn fetch_packages(repo_url: &str) -> Result<DepMap, DataError> {
    let mut map = DepMap::new();
    for component in COMPONENTS {
        let url = index_url(repo_url, component);
        for attempt in 1..=ATTEMPTS {
            match fetch_component(&url) {
                Ok(text) => {
                    for (name, deps) in parse_packages(&text) {
                        map.insert(name, deps);
                    }
                    break;
                }
                Err(error) => {
                    if attempt == ATTEMPTS {
                        return Err(error);
                    }
                }
            }
        }
    }
    Ok(map)
}

/// Загружает один индекс компонента и распаковывает его из gzip.
fn fetch_component(url: &str) -> Result<String, DataError> {
    let response = ureq::get(url).call().map_err(|error| fetch_error(url, error.to_string()))?;
    let mut body = response.into_body();
    let mut decoder = GzDecoder::new(body.as_reader());
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)
        .map_err(|error| fetch_error(url, error.to_string()))?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Собирает ошибку загрузки для конкретного URL.
fn fetch_error(url: &str, source: String) -> DataError {
    DataError::Fetch { url: url.to_string(), source }
}

/// Разбирает текст индекса (записи разделены пустыми строками) в карту зависимостей.
pub fn parse_packages(text: &str) -> DepMap {
    let mut map = DepMap::new();
    for record in text.split("\n\n") {
        let (name, fields) = parse_record(record);
        if name.is_empty() {
            continue;
        }
        let mut deps = Vec::new();
        for field in DEP_FIELDS {
            if let Some(value) = fields.get(field) {
                deps.extend(parse_deps(value));
            }
        }
        map.insert(name, deps);
    }
    map
}

/// Читает поля одной записи, объединяя строки продолжения (начинаются с пробела).
fn parse_record(record: &str) -> (String, BTreeMap<&str, String>) {
    let mut fields: BTreeMap<&str, String> = BTreeMap::new();
    let mut last = "";
    for line in record.lines() {
        if line.starts_with(' ') {
            fields.entry(last).or_default().push_str(line);
        } else if let Some((key, value)) = line.split_once(':') {
            last = key;
            fields.insert(key, value.to_string());
        }
    }
    let name = fields.get("Package").cloned().unwrap_or_default().trim().to_string();
    (name, fields)
}

/// Разбивает значение поля зависимостей: по `,`, затем по `|`, скобки версии отбрасываются.
pub fn parse_deps(value: &str) -> Vec<String> {
    let mut deps = Vec::new();
    for part in value.split(',') {
        for alternative in part.split('|') {
            let name = alternative.split('(').next().unwrap_or_default().trim().to_string();
            if !name.is_empty() {
                deps.push(name);
            }
        }
    }
    deps
}

fn index_url(repo_url: &str, component: &str) -> String {
    let base = repo_url.trim_end_matches('/');
    format!("{base}/dists/{DISTRO}/{component}/binary-{ARCH}/Packages.gz")
}

#[cfg(test)]
mod tests {
    use super::index_url;

    #[test]
    fn accepts_repository_url_with_or_without_trailing_slash() {
        let expected = "https://example.com/ubuntu/dists/noble/main/binary-amd64/Packages.gz";
        assert_eq!(index_url("https://example.com/ubuntu", "main"), expected);
        assert_eq!(index_url("https://example.com/ubuntu/", "main"), expected);
    }
}
