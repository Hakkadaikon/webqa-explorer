//! Tests of the shared looks.

use super::*;

fn text(line: &Line) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[test]
fn hints_put_each_key_before_what_it_does() {
    let line = hints(&[("q", "終わる"), ("j", "下へ")]);
    assert_eq!(text(&line), " q  終わる   j  下へ  ");
}

#[test]
fn labelled_splits_at_the_first_space() {
    let line = labelled("指示 a b");
    assert_eq!(
        line.spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect::<Vec<_>>(),
        ["指示 ", "a b"]
    );
    assert_eq!(text(&labelled("-")), "- ");
}
