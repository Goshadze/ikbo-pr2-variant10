//! Точка входа CLI-приложения (этап 2: сбор данных).

use std::error::Error;
use std::process::ExitCode;

use depgraph::apt::{fetch_packages, DepMap};
use depgraph::config::{parse_args, Config, TestMode};

/// Разбирает параметры, собирает данные и выводит прямые зависимости пакета.
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

/// Выполняет логику этапа 2 по разобранным параметрам.
fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let config = parse_args(args)?;
    print!("{config}");
    if config.test_mode == TestMode::File {
        return Err("режим file будет поддерживаться с этапа 3".into());
    }
    print_direct(&config, &fetch_packages(&config.repo)?)
}

/// Печатает прямые зависимости пакета, по одному имени в строке.
fn print_direct(config: &Config, data: &DepMap) -> Result<(), Box<dyn Error>> {
    let deps = data.get(&config.package)
        .ok_or_else(|| format!("пакет {} не найден в репозитории", config.package))?;
    for dep in deps {
        println!("{dep}");
    }
    Ok(())
}

/// Проверяет, запрошена ли справка (`-h` или `--help`).
fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-h" || arg == "--help")
}
