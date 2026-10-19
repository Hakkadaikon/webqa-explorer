//! Tests of reading the elements hard to see.

use super::*;

/// Answers to `hard_to_see` with the elements read from them.
const SIGHTS: [(&str, &[(usize, &str)]); 5] = [
    ("", &[]),
    ("\n  \n", &[]),
    ("1: 小さい\n2: 薄い", &[(1, "小さい"), (2, "薄い")]),
    (
        " 2 :  文字が薄い  \r\n\n1:小さい",
        &[(2, "文字が薄い"), (1, "小さい")],
    ),
    ("2: 色: 薄い", &[(2, "色: 薄い")]),
];

fn sights(listed: &[(usize, &str)]) -> Vec<HardToSee> {
    let listed = listed.iter().map(|&(element, reason)| HardToSee {
        element,
        reason: reason.to_owned(),
    });
    listed.collect()
}

#[test]
fn hard_to_see_required_with_image_support() {
    let shot = shot();
    let step = step(Some(&shot));
    let missing = read_step(&full(false), &step);
    assert_eq!(
        missing,
        Err(Unusable::Missing(QuestionId::new(HARD_TO_SEE))),
        "asked, not answered"
    );
    let reply = Reply {
        verdict: verdict(),
        hesitation: 0.7,
        expectation: "押すと注文が確定すると思った".to_owned(),
        hard_to_see: Some(sights(&[(2, "文字が薄い")])),
        purpose_guess: "商品をカートに入れる".to_owned(),
    };
    assert_eq!(read_step(&full(true), &step), Ok(reply), "the one listed");
    for (text, listed) in SIGHTS {
        let answers = changed(full(true), &[(HARD_TO_SEE, Some(&[(text, NAN)]))]);
        let got = read_step(&answers, &step).map(|r| r.hard_to_see);
        assert_eq!(got, Ok(Some(sights(listed))), "{text:?}");
    }
}

#[test]
fn hard_to_see_limited_to_max_options() {
    let shot = shot();
    let step = Step {
        elements: 300,
        ..step(Some(&shot))
    };
    let at = |text| {
        let answers = changed(full(true), &[(HARD_TO_SEE, Some(&[(text, NAN)]))]);
        read_step(&answers, &step).map(|r| r.hard_to_see)
    };
    assert_eq!(at("255: x"), Ok(Some(sights(&[(255, "x")]))), "last asked");
    assert_eq!(
        at("256: x"),
        Err(Unusable::NotAnOption(QuestionId::new(HARD_TO_SEE))),
        "past the options"
    );
}
