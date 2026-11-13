# Depgraph

Граф зависимостей пакетов Ubuntu (Rust, apt).
Практическая работа № 2 по конфигурационному управлению, вариант 10.

## Описание

CLI-прототип. При запуске проверяет параметры и выводит их значения.

Выполнен этап 1.

## Функции

- Разбор коротких и длинных параметров; для длинных поддерживается `--name=value`.
- Сообщения об ошибках: неизвестный или повторный параметр, отсутствие значения,
  пустая строка, недопустимый режим, неверный URL или отсутствующий файл.
- Вывод всех параметров в формате `ключ=значение`.

## Настройки

Параметры командной строки:

- `-p`, `--package NAME` — имя пакета (обязательно)
- `-r`, `--repo URL_OR_PATH` — адрес репозитория или тестовый файл (обязательно)
- `-t`, `--test-mode MODE` — `url` или `file`, по умолчанию `url`
- `-o`, `--output PATH` — файл изображения, по умолчанию `graph.png`
- `-a`, `--ascii-tree MODE` — `on` или `off`, по умолчанию `off`
- `-f`, `--filter TEXT` — исключаемая подстрока; без параметра фильтра нет
- `-h`, `--help` — справка

Имя изображения и режим дерева пока только проверяются и выводятся.
Визуализация относится к этапу 5.

Разбор тестового графа появится на этапе 3. Сейчас путь в режиме `file`
проверяется на существование.

## Запуск

Требуется Rust с Cargo.

```sh
cargo build --release
cargo test
./run.sh
```

`run.sh` собирает проект и запускает пример. Справка:

```sh
./target/release/depgraph --help
```

## Примеры

```sh
./target/release/depgraph -p curl -r https://archive.ubuntu.com/ubuntu/
```

```text
package=curl
repo=https://archive.ubuntu.com/ubuntu/
test_mode=url
output_file=graph.png
ascii_tree=off
filter=(пусто)
```

Ошибки (сообщения выводятся в stderr, код выхода — 1):

```sh
# Не указан пакет
./target/release/depgraph --repo https://example.com/
# Неверный адрес репозитория
./target/release/depgraph -p curl -r ftp://example.com/
# Неверный тестовый режим
./target/release/depgraph -p curl -r https://example.com/ -t auto
# Пустое имя изображения
./target/release/depgraph -p curl -r https://example.com/ -o ''
# Неверный режим дерева
./target/release/depgraph -p curl -r https://example.com/ -a yes
# Пустой фильтр
./target/release/depgraph -p curl -r https://example.com/ -f ''
```
