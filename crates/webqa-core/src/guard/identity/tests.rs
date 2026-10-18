//! Tests of the element existence and identity guard.

use super::*;

fn node(role: &str, name: &str, node: &str) -> Node {
    Node {
        role: role.to_owned(),
        name: name.to_owned(),
        node: node.to_owned(),
    }
}

fn b(name: &str, n: &str) -> Node {
    node("button", name, n)
}

fn l(name: &str, n: &str) -> Node {
    node("link", name, n)
}

fn target(nodes: &[Node], index: usize) -> Target {
    Target::at(nodes, index).unwrap()
}

#[test]
fn target_at_counts_same_role_and_name() {
    let nodes = [b("保存", "e1"), l("保存", "e2"), b("保存", "e3")];
    let want = |role: &str, nth, n: &str| Target {
        role: role.to_owned(),
        name: "保存".to_owned(),
        nth,
        node: n.to_owned(),
    };
    assert_eq!(Target::at(&nodes, 2), Some(want("button", 1, "e3")), "e3");
    assert_eq!(Target::at(&nodes, 1), Some(want("link", 0, "e2")), "e2");
    assert_eq!(Target::at(&nodes, 3), None, "out of range");
    assert_eq!(Target::at(&[], 0), None, "empty");
}

#[test]
fn missing_element_aborts() {
    let saved = target(&[b("保存", "e1")], 0);
    assert_eq!(recheck(&saved, &[]), Recheck::Missing, "empty");
    assert_eq!(
        recheck(&saved, &[l("利用規約", "e2")]),
        Recheck::Missing,
        "another element only"
    );
}

#[test]
fn role_or_position_change_aborts() {
    let saved = target(&[b("保存", "e1")], 0);
    let cases = [
        (vec![l("保存", "e3")], Recheck::Missing),
        (vec![b("保存する", "e1")], Recheck::Missing),
        (vec![b("保存", "e9")], Recheck::Replaced),
    ];
    for (nodes, want) in &cases {
        assert_eq!(recheck(&saved, nodes), *want, "{nodes:?}");
    }
    let lower = target(&[b("保存", "e1"), b("保存", "e2")], 1);
    assert_eq!(
        recheck(&lower, &[b("保存", "e2")]),
        Recheck::Missing,
        "the upper one went away and the lower moved up"
    );
}

#[test]
fn swapped_same_name_buttons_abort() {
    let asked = [b("保存", "e1"), b("保存", "e2")];
    let swapped = [b("保存", "e2"), b("保存", "e1")];
    for index in [0, 1] {
        assert_eq!(
            recheck(&target(&asked, index), &swapped),
            Recheck::Replaced,
            "{index}"
        );
    }
}

#[test]
fn same_name_unique_by_position_passes() {
    let asked = [b("保存", "e1"), b("保存", "e2")];
    let added = [l("ヘルプ", "e0"), b("保存", "e1"), b("保存", "e2")];
    for index in [0, 1] {
        let t = target(&asked, index);
        assert_eq!(recheck(&t, &asked), Recheck::Same, "unchanged {index}");
        assert_eq!(recheck(&t, &added), Recheck::Same, "added {index}");
    }
}
