//! Тесты парсера индекса apt (этап 2), без сети.

use depgraph::apt::{parse_deps, parse_packages};

const FIXTURE: &str = "\
Package: curl
Version: 8.5.0
Depends: libc6 (>= 2.34), libcurl4t64 | libcurl3-gnutls, zlib1g
Priority: optional

Package: git
Depends: libc6, libgit2 (>= 1.6)
Pre-Depends: libselinux1

Package: libc6
Depends:
";

fn names(items: &[&str]) -> Vec<String> {
    items.iter().copied().map(str::to_string).collect()
}

#[test]
fn parses_records_separated_by_blank_lines() {
    let map = parse_packages(FIXTURE);
    assert_eq!(
        map.get("curl").unwrap(),
        &names(&["libc6", "libcurl4t64", "libcurl3-gnutls", "zlib1g"])
    );
    assert_eq!(map.get("git").unwrap(), &names(&["libc6", "libgit2", "libselinux1"]));
    assert!(map.get("libc6").unwrap().is_empty());
}

#[test]
fn keeps_alternatives_and_drops_version_parentheses() {
    assert_eq!(
        parse_deps("libc6 (>= 2.34), libcurl4t64 | libcurl3-gnutls"),
        names(&["libc6", "libcurl4t64", "libcurl3-gnutls"])
    );
    assert!(parse_deps(" , ").is_empty());
}

#[test]
fn merges_continuation_lines_of_a_field() {
    let map = parse_packages("Package: vlc\nDepends: libvlc5,\n libavcodec60\n");
    assert_eq!(map.get("vlc").unwrap(), &names(&["libvlc5", "libavcodec60"]));
}

#[test]
fn ignores_records_without_package_name() {
    assert!(parse_packages("Depends: libc6\n").is_empty());
}
