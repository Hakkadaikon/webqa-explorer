use super::*;
use crate::truth::Element;

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

const HEAD: &str = "| 種類 | 再現率 | 厳しい適合率 | 人の判断を引いた適合率 |\n|---|---|---|---|\n";

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

## 未判定の課題

- 判断に迷う / 2回
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
        Some("| 全体 | 0.25 ± 0.35 | 1.00 ± 0.00 | 1.00 ± 0.00 |")
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
        Some("\n\n- なし\n\n## 未判定の課題\n\n- なし\n")
    );
}

#[test]
fn unlabeled_issues_count_over_the_runs_with_their_elements() {
    let runs = [vec![visibility()], vec![visibility()], vec![hesitant()]];
    let text = report(&runs, &[], &[]);
    assert_eq!(
        text.split_once("## 未判定の課題").map(|(_, rest)| rest),
        Some("\n\n- 視認性が悪い /p link 料金 2回\n- 判断に迷う / 1回\n")
    );
}
