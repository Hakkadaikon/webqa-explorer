//! Tests of reading the elements hard to see.

use super::*;

/// Answers to `hard_to_see` with the elements read from them.
const SIGHTS: [(&str, &[(usize, &str)]); 18] = [
    ("", &[]),
    ("\n  \n", &[]),
    ("1: 小さい\n2: 薄い", &[(1, "小さい"), (2, "薄い")]),
    (
        " 2 :  文字が薄い  \r\n\n1:小さい",
        &[(2, "文字が薄い"), (1, "小さい")],
    ),
    ("2: 色: 薄い", &[(2, "色: 薄い")]),
    ("1：小さい", &[(1, "小さい")]),
    ("- 1: 小さい\n* 2: 薄い", &[(1, "小さい"), (2, "薄い")]),
    ("・1: 小さい\n• 2: 薄い", &[(1, "小さい"), (2, "薄い")]),
    ("１: 小さい\n２：薄い", &[(1, "小さい"), (2, "薄い")]),
    ("次のとおり。\n1: 小さい", &[(1, "小さい")]),
    ("見えにくい要素:\n1: 小さい", &[(1, "小さい")]),
    ("```\n1: 小さい\n```", &[(1, "小さい")]),
    ("なし", &[]),
    ("1: \n2:", &[]),
    (": 小さい\n1.小さい", &[]),
    ("1: 小さい\n1: 薄い", &[(1, "小さい")]),
    (
        "2: 薄い\n\n1: 小さい\n2: 薄い",
        &[(2, "薄い"), (1, "小さい")],
    ),
    ("**1**: 小さい", &[]),
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

#[test]
fn hard_to_see_cut_to_the_first_three() {
    let shot = shot();
    let step = Step {
        elements: 9,
        ..step(Some(&shot))
    };
    let at = |text| {
        let answers = changed(full(true), &[(HARD_TO_SEE, Some(&[(text, NAN)]))]);
        read_step(&answers, &step).map(|r| r.hard_to_see)
    };
    let four = "5: a\n4: b\n4: c\n3: d\n2: e";
    assert_eq!(
        at(four),
        Ok(Some(sights(&[(5, "a"), (4, "b"), (3, "d")]))),
        "three distinct numbers, the repeat not counted"
    );
    assert_eq!(
        at("1: a\n2: b\n3: c\n10: d"),
        Err(Unusable::NotAnOption(QuestionId::new(HARD_TO_SEE))),
        "a number not asked is an anomaly even past the cut"
    );
}
