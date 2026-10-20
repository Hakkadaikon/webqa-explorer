use super::*;
use webqa_core::interview::issue::ScreenMark;

const LEDGER: &str = include_str!("../../tests/fixtures/ledger.txt");

#[test]
fn takes_body_and_unchecked_issues_with_their_paths() {
    let found = found(LEDGER).unwrap();
    let keys: Vec<_> = found
        .iter()
        .map(|f| {
            (
                f.number.as_str(),
                f.kind,
                f.path.as_str(),
                f.element.clone(),
            )
        })
        .collect();
    assert_eq!(
        keys,
        vec![
            (
                "I-00000001",
                KindId::Visibility,
                "/product-a.html",
                Some(Element {
                    role: "link".into(),
                    name: "料金を見る".into()
                })
            ),
            ("I-00000002", KindId::Mismatch, "/attendance.html", None),
            ("I-00000003", KindId::Hesitant, "/", None),
        ]
    );
}

#[test]
fn a_screen_url_that_is_not_a_url_stays_as_written() {
    assert_eq!(
        path(&Place::Screen {
            url: "not a url".into(),
            mark: ScreenMark::parse("S-00000001").unwrap()
        }),
        "not a url"
    );
}

#[test]
fn a_broken_ledger_is_an_error() {
    assert_eq!(
        found("not a ledger"),
        Err(EvalError::Ledger(
            "台帳の 1 行目が決まった書式でない".into()
        ))
    );
}
