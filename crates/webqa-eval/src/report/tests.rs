use super::*;
use crate::truth::{Element, Verdict};

fn plants() -> Vec<Plant> {
    vec![
        Plant {
            id: "a".into(),
            kind: KindId::Visibility,
            path: Some("/p".into()),
            element: Some(Element {
                role: "link".into(),
                name: "料金".into(),
            }),
            note: String::new(),
        },
        Plant {
            id: "b".into(),
            kind: KindId::Unreached,
            path: None,
            element: None,
            note: String::new(),
        },
    ]
}

fn visibility() -> Found {
    Found {
        number: "I-00000001".into(),
        kind: KindId::Visibility,
        path: "/p".into(),
        element: Some(Element {
            role: "link".into(),
            name: "料金".into(),
        }),
    }
}

fn hesitant() -> Found {
    Found {
        number: "I-00000002".into(),
        kind: KindId::Hesitant,
        path: "/".into(),
        element: None,
    }
}

const HEAD: &str = "採点した回: 2\n\n| 種類 | 再現率 | 厳しい適合率 | 人の判断を引いた適合率 |\n|---|---|---|---|\n";

#[test]
fn reports_each_kind_and_the_total_with_what_was_missed() {
    let runs = [
        vec![visibility(), hesitant()],
        vec![visibility(), hesitant()],
    ];
    assert_eq!(
        report(&runs, &plants(), &[]),
        format!(
            "{HEAD}\
| 視認性が悪い | 1.00 ± 0.00 | 1.00 ± 0.00 | 1.00 ± 0.00 |
| 判断に迷う | - | 0.00 ± 0.00 | - |
| 目的に届かない | 0.00 ± 0.00 | - | - |
| 目的のずれ | - | - | - |
| 全体 | 0.50 ± 0.00 | 0.50 ± 0.00 | 1.00 ± 0.00 |

## 見つからなかった仕込み

- b

## 仕込みに当たらなかった課題

- 判断に迷う / 2件 I-00000002 未判定
"
        )
    );
}

#[test]
fn a_single_run_has_no_spread() {
    let text = report(&[vec![visibility()]], &plants(), &[]);
    assert_eq!(
        text.lines().find(|line| line.starts_with("| 全体")),
        Some("| 全体 | 0.50 | 1.00 | 1.00 |")
    );
}

#[test]
fn the_spread_is_the_sample_deviation() {
    let text = report(&[vec![visibility()], vec![]], &plants(), &[]);
    assert_eq!(
        text.lines().find(|line| line.starts_with("| 全体")),
        Some("| 全体 | 0.25 ± 0.35 | 1.00 | 1.00 |")
    );
}

#[test]
fn nothing_missed_and_nothing_unlabeled_say_so() {
    let found = [vec![
        visibility(),
        Found {
            kind: KindId::Unreached,
            path: "/x".into(),
            element: None,
            ..hesitant()
        },
    ]];
    let text = report(&found, &plants(), &[]);
    assert_eq!(
        text.split_once("## 見つからなかった仕込み")
            .map(|(_, rest)| rest),
        Some("\n\n- なし\n\n## 仕込みに当たらなかった課題\n\n- なし\n")
    );
}

#[test]
fn unlabeled_issues_count_over_the_runs_with_their_elements() {
    let runs = [vec![visibility()], vec![visibility()], vec![hesitant()]];
    let text = report(&runs, &[], &[]);
    assert_eq!(
        text.split_once("## 仕込みに当たらなかった課題")
            .map(|(_, rest)| rest),
        Some(
            "\n\n- 視認性が悪い /p link 料金 2件 I-00000001 未判定\n- 判断に迷う / 1件 I-00000002 未判定\n"
        )
    );
}

#[test]
fn the_number_of_runs_comes_first() {
    let text = report(&[vec![], vec![], vec![]], &[], &[]);
    assert_eq!(text.lines().next(), Some("採点した回: 3"));
}

#[test]
fn a_plant_found_in_one_run_is_not_missed() {
    let text = report(&[vec![visibility()], vec![]], &plants(), &[]);
    assert_eq!(
        text.split_once("## 見つからなかった仕込み")
            .and_then(|(_, rest)| rest.split_once("## 仕込みに当たらなかった課題"))
            .map(|(missed, _)| missed),
        Some("\n\n- b\n\n")
    );
}

#[test]
fn unlabeled_issues_differing_in_path_or_element_count_apart() {
    let other_path = Found {
        number: "I-00000003".into(),
        path: "/q".into(),
        ..visibility()
    };
    let other_element = Found {
        number: "I-00000004".into(),
        element: None,
        ..visibility()
    };
    let runs = [vec![visibility(), other_path, other_element]];
    assert_eq!(
        report(&runs, &[], &[])
            .split_once("## 仕込みに当たらなかった課題")
            .map(|(_, rest)| rest),
        Some(
            "\n\n- 視認性が悪い /p link 料金 1件 I-00000001 未判定\n\
- 視認性が悪い /q link 料金 1件 I-00000003 未判定\n\
- 視認性が悪い /p 1件 I-00000004 未判定\n"
        )
    );
}

#[test]
fn a_cell_with_one_value_has_no_spread_even_over_many_runs() {
    let plants = [plants().remove(0)];
    let one_value = report(&[vec![visibility()], vec![hesitant()]], &plants, &[]);
    assert_eq!(
        one_value
            .lines()
            .find(|line| line.starts_with("| 判断に迷う")),
        Some("| 判断に迷う | - | 0.00 | - |")
    );
}

#[test]
fn issues_with_a_ruling_are_listed_with_it() {
    let rule = |kind, verdict| Label {
        kind,
        path: None,
        element: None,
        verdict,
    };
    let labels = [
        rule(KindId::Visibility, Verdict::False),
        rule(KindId::Hesitant, Verdict::Real),
    ];
    let runs = [vec![visibility(), hesitant()]];
    assert_eq!(
        report(&runs, &[], &labels)
            .split_once("## 仕込みに当たらなかった課題")
            .map(|(_, rest)| rest),
        Some(
            "\n\n- 視認性が悪い /p link 料金 1件 I-00000001 誤り\n\
- 判断に迷う / 1件 I-00000002 本当に使いにくい\n"
        )
    );
}
