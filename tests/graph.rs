//! Тесты обхода графа (этап 3): порядок BFS, циклы, фильтр.

use std::path::Path;

use depgraph::apt::DepMap;
use depgraph::graph::traverse;
use depgraph::testrepo::parse_file;

fn fixture(name: &str) -> DepMap {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    parse_file(path.to_str().unwrap()).expect("фикстура должна разбираться")
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().copied().map(str::to_string).collect()
}

#[test]
fn diamond_dependency_is_visited_once_in_bfs_order() {
    let (order, parents) = traverse(&fixture("diamond.txt"), "A", "", false);
    assert_eq!(order, names(&["A", "B", "C", "D"]));
    assert_eq!(parents.get("D").unwrap(), "B");
}

#[test]
fn cycle_terminates_and_visits_each_node_once() {
    let (order, _) = traverse(&fixture("cycle.txt"), "A", "", false);
    assert_eq!(order, names(&["A", "B"]));
}

#[test]
fn filter_excludes_nodes_with_substring() {
    let (order, _) = traverse(&fixture("filter.txt"), "A", "AB", false);
    assert_eq!(order, names(&["A"]));
}

#[test]
fn filter_excludes_root() {
    let (order, _) = traverse(&fixture("filter.txt"), "ABC", "AB", false);
    assert!(order.is_empty());
}

#[test]
fn unknown_package_gives_empty_result() {
    let (order, _) = traverse(&fixture("leaf.txt"), "Z", "", false);
    assert!(order.is_empty());
}

#[test]
fn dependency_not_declared_as_package_is_a_leaf_node() {
    let (order, parents) = traverse(&fixture("leaf.txt"), "A", "", false);
    assert_eq!(order, names(&["A", "B"]));
    assert_eq!(parents.get("B").unwrap(), "A");
}
