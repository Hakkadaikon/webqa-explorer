//! Tests of the mismatch questions.

mod cases;

use url::Url;

use super::*;
use crate::config::purpose::UrlPattern;
use crate::interview::outcome::{Cut, Ending};

const FINISHED: Outcome = Outcome::Finished(Ending::Goal);
const INTERRUPTED: Outcome = Outcome::Cut(Cut::Interrupted);
const TIME_UP: Outcome = Outcome::Cut(Cut::TimeUp);

fn products() -> ScreenPurpose {
    ScreenPurpose {
        pattern: UrlPattern::parse("/products/*").unwrap(),
        purpose: "商品をカートに入れる".into(),
    }
}

fn cart() -> ScreenPurpose {
    ScreenPurpose {
        pattern: UrlPattern::parse("/cart").unwrap(),
        purpose: "注文内容を確かめる".into(),
    }
}

fn url(path: &str) -> Url {
    Url::parse(&format!("https://shop.example.test{path}")).unwrap()
}

/// The visits of a trial, written as `(path, guess)`.
type Visits<'a> = &'a [(&'a str, Option<&'a str>)];

fn urls(visits: Visits<'_>) -> Vec<Url> {
    visits.iter().map(|(path, _)| url(path)).collect()
}

fn visits<'a>(written: Visits<'a>, urls: &'a [Url]) -> Vec<Visit<'a>> {
    let guesses = written.iter().map(|(_, guess)| *guess);
    urls.iter()
        .zip(guesses)
        .map(|(url, guess)| Visit { url, guess })
        .collect()
}

/// Calls `f` with the trials written as `(outcome, visits)`.
fn with_trials<R>(written: &[(Outcome, Visits<'_>)], f: impl FnOnce(&[Trial<'_>]) -> R) -> R {
    let urls: Vec<_> = written.iter().map(|(_, v)| urls(v)).collect();
    let held: Vec<_> = written
        .iter()
        .zip(&urls)
        .map(|((_, v), urls)| visits(v, urls))
        .collect();
    let trials: Vec<_> = written
        .iter()
        .zip(&held)
        .map(|((outcome, _), visits)| Trial {
            outcome: *outcome,
            visits,
        })
        .collect();
    f(&trials)
}

/// The written purpose and the guesses of each question asked about
/// `screens` after `trials`.
fn asked(
    screens: &[ScreenPurpose],
    trials: &[(Outcome, Visits<'_>)],
) -> Vec<(String, Vec<String>)> {
    with_trials(trials, |trials| {
        let questions = mismatch_questions(screens, trials).into_iter();
        questions
            .map(|q| (q.screen.purpose.clone(), q.guesses))
            .collect()
    })
}

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().copied().map(str::to_owned).collect()
}

/// Three finished trials guessing at two product screens.
const THREE: [(Outcome, Visits<'_>); 3] = [
    (
        FINISHED,
        &[
            ("/products/42", Some("会員登録をする")),
            ("/products/43", Some("会員登録をする")),
        ],
    ),
    (
        FINISHED,
        &[
            ("/products/42", Some("会員登録をする")),
            ("/products/43", Some("会員登録をする")),
            ("/cart", Some("注文を確かめる")),
        ],
    ),
    (
        FINISHED,
        &[
            ("/products/42", Some("商品をカートに入れる")),
            ("/products/43", Some("商品をカートに入れる")),
        ],
    ),
];

#[test]
fn asks_mismatch_once_per_url_pattern() {
    let screens = [products()];
    let guesses = owned(&["会員登録をする", "商品をカートに入れる"]);
    with_trials(&THREE, |trials| {
        let questions = mismatch_questions(&screens, trials);
        let got: Vec<_> = questions.iter().map(|q| (q.screen, &q.guesses)).collect();
        assert_eq!(got, vec![(&products(), &guesses)], "one question");
    });
    let both = asked(&[products(), cart()], &THREE);
    let cart_guesses = owned(&["注文を確かめる"]);
    let products_first = vec![
        ("商品をカートに入れる".to_owned(), guesses.clone()),
        ("注文内容を確かめる".to_owned(), cart_guesses.clone()),
    ];
    assert_eq!(both, products_first, "in the order written");
    let cart_first = vec![
        ("注文内容を確かめる".to_owned(), cart_guesses),
        ("商品をカートに入れる".to_owned(), guesses),
    ];
    assert_eq!(asked(&[cart(), products()], &THREE), cart_first, "reversed");
}

#[test]
fn mismatch_question_passes_all_guesses_once() {
    let screens = [products()];
    let questions: Vec<_> = with_trials(&THREE, |trials| {
        let asked = mismatch_questions(&screens, trials).into_iter();
        asked.map(|q| q.question).collect()
    });
    let prompt = "URL の型 /products/* の画面には、次の目的を書いた。\n書いた目的: 商品をカートに入れる\n判断モデルは、この型に合う画面の目的を次のように推測した。\n- 会員登録をする\n- 商品をカートに入れる\n推測は、書いた目的と同じことを指すか。言い回しの違いは問わない。どれか1つでも違うことを指すなら「同じでない」を選ぶ。";
    let want = Question::Choice {
        id: QuestionId::new(PURPOSE_MATCH),
        prompt: prompt.to_owned(),
        options: owned(&["同じ", "同じでない"]),
    };
    assert_eq!(questions, vec![want], "the question with each guess once");
    assert_eq!(prompt.matches("会員登録をする").count(), 1, "once");
}

#[test]
fn different_wording_is_left_to_judge() {
    let reworded = asked(
        &[products()],
        &[(
            FINISHED,
            &[("/products/42", Some("この商品を買い物かごに追加する"))],
        )],
    );
    let want = vec![(
        "商品をカートに入れる".to_owned(),
        owned(&["この商品を買い物かごに追加する"]),
    )];
    assert_eq!(reworded, want, "asked about another wording");
    let same = asked(
        &[products()],
        &[(FINISHED, &[("/products/42", Some("商品をカートに入れる"))])],
    );
    let want = vec![(
        "商品をカートに入れる".to_owned(),
        owned(&["商品をカートに入れる"]),
    )];
    assert_eq!(same, want, "asked even about the same words");
}
