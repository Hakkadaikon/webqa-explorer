//! Tests of when the mismatch questions are not asked.

use super::*;

/// Screens, trials and the guesses each question carries.
type Case<'a> = (
    &'a [fn() -> ScreenPurpose],
    &'a [(Outcome, Visits<'a>)],
    &'a [&'a [&'a str]],
);

fn check(cases: &[Case<'_>]) {
    for (i, (screens, trials, want)) in cases.iter().enumerate() {
        let screens: Vec<_> = screens.iter().map(|screen| screen()).collect();
        let got: Vec<_> = asked(&screens, trials)
            .into_iter()
            .map(|(_, g)| g)
            .collect();
        let want: Vec<_> = want.iter().map(|guesses| owned(guesses)).collect();
        assert_eq!(got, want, "case {i}");
    }
}

const UNDECLARED: [Case<'_>; 4] = [
    (
        &[products],
        &[(FINISHED, &[("/help", Some("問い合わせをする"))])],
        &[],
    ),
    (&[], &THREE, &[]),
    (
        &[products],
        &[(
            FINISHED,
            &[
                ("/products/42", Some("商品を比べる")),
                ("/help", Some("問い合わせをする")),
            ],
        )],
        &[&["商品を比べる"]],
    ),
    (
        &[products],
        &[(
            FINISHED,
            &[
                ("/products/42/reviews", Some("口コミを読む")),
                ("/products/", Some("一覧を見る")),
            ],
        )],
        &[],
    ),
];

#[test]
fn no_mismatch_question_for_undeclared_screen() {
    check(&UNDECLARED);
}

const UNREACHED: [Case<'_>; 3] = [
    (
        &[products],
        &[
            (FINISHED, &[("/cart", Some("注文を確かめる"))]),
            (FINISHED, &[("/help", Some("問い合わせをする"))]),
            (
                FINISHED,
                &[("/cart", Some("買う")), ("/help", Some("聞く"))],
            ),
        ],
        &[],
    ),
    (&[products], &[], &[]),
    (&[products], &[(FINISHED, &[])], &[]),
];

#[test]
fn no_mismatch_question_without_reaching_trial() {
    check(&UNREACHED);
}

const CUT: [Case<'_>; 3] = [
    (
        &[products],
        &[(INTERRUPTED, &[("/products/42", Some("会員登録をする"))])],
        &[],
    ),
    (
        &[products],
        &[(TIME_UP, &[("/products/42", Some("会員登録をする"))])],
        &[],
    ),
    (
        &[products],
        &[
            (INTERRUPTED, &[("/products/42", Some("会員登録をする"))]),
            (FINISHED, &[("/products/42", Some("商品を比べる"))]),
        ],
        &[&["商品を比べる"]],
    ),
];

#[test]
fn cut_trial_reach_does_not_trigger_mismatch_question() {
    check(&CUT);
}

const UNUSABLE: [Case<'_>; 3] = [
    (
        &[products],
        &[(FINISHED, &[("/products/42", None), ("/products/43", None)])],
        &[],
    ),
    (
        &[products],
        &[(
            FINISHED,
            &[
                ("/products/42", Some("")),
                ("/products/43", Some("   ")),
                ("/products/44", Some("\n")),
            ],
        )],
        &[],
    ),
    (
        &[products],
        &[(
            FINISHED,
            &[
                ("/products/42", None),
                ("/products/43", Some(" 商品を比べる ")),
            ],
        )],
        &[&["商品を比べる"]],
    ),
];

#[test]
fn no_mismatch_question_without_usable_guess() {
    check(&UNUSABLE);
}

const MERGED: [Case<'_>; 2] = [
    (
        &[products],
        &[
            (FINISHED, &[("/products/42", Some("会員登録をする"))]),
            (
                FINISHED,
                &[
                    ("/products/43", Some(" 会員登録をする ")),
                    ("/products/43", Some("商品を比べる")),
                ],
            ),
        ],
        &[&["会員登録をする", "商品を比べる"]],
    ),
    (
        &[products],
        &[(
            FINISHED,
            &[
                ("/products/42", Some("会員登録をする")),
                ("/products/43", Some("会員 登録をする")),
                ("/products/44", Some(" 一行目\n二行目 ")),
            ],
        )],
        &[&["会員登録をする", "会員 登録をする", "一行目\n二行目"]],
    ),
];

#[test]
fn guesses_across_matching_screens_merged_trimmed() {
    check(&MERGED);
    let screens = [products()];
    let prompts: Vec<_> = with_trials(MERGED[0].1, |trials| {
        let asked = mismatch_questions(&screens, trials).into_iter();
        asked.map(|q| prompt_of(&q.question)).collect()
    });
    let lines = "\n- 会員登録をする\n- 商品を比べる\n推測は";
    let found: Vec<_> = prompts.iter().map(|p| p.contains(lines)).collect();
    assert_eq!(found, [true], "one question with the guesses: {prompts:?}");
}

/// The prompt of a choice.
fn prompt_of(question: &Question) -> String {
    match question {
        Question::Noul { prompt, .. }
        | Question::Choice { prompt, .. }
        | Question::Score { prompt, .. }
        | Question::Text { prompt, .. } => prompt.clone(),
    }
}
