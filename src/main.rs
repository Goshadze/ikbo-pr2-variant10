//! Точка входа CLI-приложения (этап 3: обход графа).

use std::error::Error;
use std::process::ExitCode;

use depgraph::apt::{fetch_packages, DepMap};
use depgraph::config::{parse_args, Config, TestMode};
use depgraph::graph::traverse;
use depgraph::testrepo::parse_file;

/// Разбирает параметры, загружает данные, обходит граф и печатает результат.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if wants_help(&args) {
        println!("{}", depgraph::usage());
        return ExitCode::SUCCESS;
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ошибка: {error}");
            eprintln!("используйте --help для списка параметров");
            ExitCode::FAILURE
        }
    }
}

/// Выполняет логику этапа 3: данные → обход → порядок BFS по строке на пакет.
fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let config = parse_args(args)?;
    print!("{config}");
    let data = load_data(&config)?;
    let order = traverse(&data, &config.package, &config.filter, false).0;
    if order.is_empty() {
        return Err(format!("пакет {} не найден или исключён фильтром", config.package).into());
    }
    for name in &order {
        println!("{name}");
    }
    Ok(())
}

/// Загружает данные по режиму: URL — индекс apt, file — тестовый файл.
fn load_data(config: &Config) -> Result<DepMap, Box<dyn Error>> {
    match config.test_mode {
        TestMode::Url => Ok(fetch_packages(&config.repo)?),
        TestMode::File => Ok(parse_file(&config.repo)?),
    }
}

/// Проверяет, запрошена ли справка (`-h` или `--help`).
fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-h" || arg == "--help")
}
