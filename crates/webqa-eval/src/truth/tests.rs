use super::*;

const TRUTH: &str = "
- id: faint-price-link
  kind: visibility
  path: /product-a.html
  element: { role: link, name: 料金を見る }
  note: 薄い色の小さなリンク
- id: no-compare
  kind: unreached
  note: 料金を比べる手段が無い
";

#[test]
fn reads_plants_with_and_without_path_and_element() {
    let plants = read_truth(TRUTH).unwrap();
    assert_eq!(
        plants,
        vec![
            Plant {
                id: "faint-price-link".into(),
                kind: KindId::Visibility,
                path: Some("/product-a.html".into()),
                element: Some(Element {
                    role: "link".into(),
                    name: "料金を見る".into()
                }),
                note: "薄い色の小さなリンク".into(),
            },
            Plant {
                id: "no-compare".into(),
                kind: KindId::Unreached,
                path: None,
                element: None,
                note: "料金を比べる手段が無い".into(),
            },
        ]
    );
}

#[test]
fn rejects_an_unknown_field_and_kind() {
    let unknown_kind = read_truth("- id: x\n  kind: other\n  note: n\n");
    let unknown_field = read_truth("- id: x\n  kind: visibility\n  note: n\n  extra: 1\n");
    assert_eq!(
        (
            matches!(unknown_kind, Err(EvalError::Yaml(_))),
            matches!(unknown_field, Err(EvalError::Yaml(_)))
        ),
        (true, true),
        "{unknown_kind:?} {unknown_field:?}"
    );
}

#[test]
fn reads_labels_and_an_empty_document() {
    let labels = read_labels("- kind: hesitant\n  path: /\n  verdict: real\n").unwrap();
    assert_eq!(
        labels,
        vec![Label {
            kind: KindId::Hesitant,
            path: Some("/".into()),
            element: None,
            verdict: Verdict::Real
        }]
    );
    assert_eq!(read_labels("").unwrap(), vec![]);
    assert_eq!(read_labels("[]").unwrap(), vec![]);
}

#[test]
fn kind_ids_follow_the_core_kinds() {
    assert_eq!(
        [
            Kind::Hesitant,
            Kind::Visibility,
            Kind::Unreached,
            Kind::Mismatch
        ]
        .map(KindId::from),
        [
            KindId::Hesitant,
            KindId::Visibility,
            KindId::Unreached,
            KindId::Mismatch
        ]
    );
}

fn is_yaml_error<T>(result: &Result<T, EvalError>) -> bool {
    matches!(result, Err(EvalError::Yaml(_)))
}

#[test]
fn rejects_an_unknown_field_in_a_label_or_an_element() {
    let label = "- kind: hesitant\n  verdict: real\n";
    let element = "  element: { role: link, name: n, extra: 1 }\n";
    assert_eq!(
        (
            is_yaml_error(&read_labels(&format!("{label}  extra: 1\n"))),
            is_yaml_error(&read_labels(&format!("{label}{element}"))),
            is_yaml_error(&read_truth(&format!(
                "- id: x\n  kind: visibility\n  note: n\n{element}"
            ))),
        ),
        (true, true, true)
    );
}

#[test]
fn core_kinds_follow_the_kind_ids() {
    let ids = [
        KindId::Hesitant,
        KindId::Visibility,
        KindId::Unreached,
        KindId::Mismatch,
    ];
    assert_eq!(ids.map(|id| KindId::from(Kind::from(id))), ids);
    assert_eq!(
        ids.map(|id| Kind::from(id).label()),
        ["判断に迷う", "視認性が悪い", "目的に届かない", "目的のずれ"]
    );
}
