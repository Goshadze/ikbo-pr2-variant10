//! Точка входа CLI-приложения (этап 1).

use std::process::ExitCode;

/// Разбирает параметры командной строки и выводит их в формате ключ-значение.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if wants_help(&args) {
        println!("{}", depgraph::usage());
        return ExitCode::SUCCESS;
    }
    match depgraph::parse_args(&args) {
        Ok(config) => {
            print!("{config}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("ошибка: {error}");
            eprintln!("используйте --help для списка параметров");
            ExitCode::FAILURE
        }
    }
}

/// Проверяет, запрошена ли справка (`-h` или `--help`).
fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-h" || arg == "--help")
}
