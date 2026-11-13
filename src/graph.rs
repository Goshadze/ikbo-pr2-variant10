//! Обход графа зависимостей (этап 3): BFS без рекурсии.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::apt::DepMap;

/// Результат обхода: порядок обнаружения (BFS) и родитель каждого узла.
pub type Traversal = (Vec<String>, BTreeMap<String, String>);

/// Итеративный BFS от пакета; `reverse = true` — обход обратных рёбер.
/// Узел, имя которого содержит подстроку фильтра, не добавляется и не расширяется.
pub fn traverse(deps: &DepMap, start: &str, filter: &str, reverse: bool) -> Traversal {
    let edges = if reverse { reversed(deps) } else { deps.clone() };
    let mut order = Vec::new();
    let mut parents = BTreeMap::new();
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    if kept(start, filter) && has_node(deps, start) {
        queue.push_back(start.to_string());
    }
    while let Some(node) = queue.pop_front() {
        if !visited.insert(node.clone()) {
            continue;
        }
        order.push(node.clone());
        for next in edges.get(&node).cloned().unwrap_or_default() {
            if kept(&next, filter) && !visited.contains(&next) {
                parents.entry(next.clone()).or_insert(node.clone());
                queue.push_back(next);
            }
        }
    }
    (order, parents)
}

/// Инвертирует карту: зависимость → пакеты, которые от неё зависят.
pub fn reversed(deps: &DepMap) -> DepMap {
    let mut reverse_map = DepMap::new();
    for (package, list) in deps {
        for dep in list {
            reverse_map.entry(dep.clone()).or_default().push(package.clone());
        }
    }
    reverse_map
}

/// Узел проходит фильтр: регистрозависимая проверка подстроки (решение A7).
fn kept(name: &str, filter: &str) -> bool {
    filter.is_empty() || !name.contains(filter)
}

/// Узел существует, если он объявлен или упомянут как чужая зависимость.
pub fn has_node(deps: &DepMap, name: &str) -> bool {
    deps.contains_key(name) || deps.values().any(|list| list.iter().any(|dep| dep == name))
}
