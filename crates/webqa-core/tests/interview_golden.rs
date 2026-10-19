//! Golden tests of the wording of the interview questions.

use std::error::Error;

use serde_json::Value;
use url::Url;
use webqa_core::config::persona::Persona;
use webqa_core::config::purpose::{ScreenPurpose, UrlPattern};
use webqa_core::config::raw::Op;
use webqa_core::interview::ask::{Step, Trial, Visit, mismatch_questions, questions};
use webqa_core::interview::outcome::{Ending, Outcome};
use webqa_core::judge::{Image, ImageFormat, Question};

const STEP_WITH_IMAGE: &str = include_str!("fixtures/interview/step_with_image.json");
const STEP_TEXT_ONLY: &str = include_str!("fixtures/interview/step_text_only.json");
const MISMATCH: &str = include_str!("fixtures/interview/mismatch.json");

fn persona() -> Persona {
    Persona {
        name: "佐藤さん".to_owned(),
        profile: "60代。孫に頼まれて、週末にスマートフォンで買い物をする".to_owned(),
        it_literacy: "スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている"
            .to_owned(),
        wants: "孫の誕生日プレゼントを注文する".to_owned(),
        avoids: vec![
            "URL を手で打たない".to_owned(),
            "開発者ツールを開かない".to_owned(),
        ],
    }
}

fn step_questions(persona: &Persona, image: Option<&Image>) -> Vec<Question> {
    questions(&Step {
        persona,
        goal: "注文を確定できる画面まで進む",
        ops: &[Op::Click, Op::Back],
        elements: 2,
        image,
    })
}

fn mismatch() -> Result<Vec<Question>, Box<dyn Error>> {
    let products = ScreenPurpose {
        pattern: UrlPattern::parse("/products/*").ok_or("a malformed pattern")?,
        purpose: "商品をカートに入れる".to_owned(),
    };
    let first = Url::parse("https://shop.example.test/products/42")?;
    let second = Url::parse("https://shop.example.test/products/43")?;
    let visits = [
        Visit {
            url: &first,
            guess: Some("会員登録をする"),
        },
        Visit {
            url: &second,
            guess: Some("会員登録をする"),
        },
        Visit {
            url: &first,
            guess: Some("商品をカートに入れる"),
        },
    ];
    let trials = [Trial {
        outcome: Outcome::Finished(Ending::Goal),
        visits: &visits,
    }];
    let screens = [products];
    let asked = mismatch_questions(&screens, &trials);
    Ok(asked.into_iter().map(|q| q.question).collect())
}

/// `questions` as JSON, and the fixture read.
fn compared(questions: &[Question], fixture: &str) -> serde_json::Result<(Value, Value)> {
    Ok((
        serde_json::to_value(questions)?,
        serde_json::from_str(fixture)?,
    ))
}

#[test]
fn interview_prompt_snapshot() -> Result<(), Box<dyn Error>> {
    let persona = persona();
    let image = Image::new(ImageFormat::Png, vec![0x89, b'P', b'N', b'G']);
    let (got, want) = compared(&step_questions(&persona, Some(&image)), STEP_WITH_IMAGE)?;
    assert_eq!(got, want, "the eight questions with an image");
    let (got, want) = compared(&step_questions(&persona, None), STEP_TEXT_ONLY)?;
    assert_eq!(got, want, "the seven questions without an image");
    let (got, want) = compared(&mismatch()?, MISMATCH)?;
    assert_eq!(got, want, "the mismatch question");
    Ok(())
}
