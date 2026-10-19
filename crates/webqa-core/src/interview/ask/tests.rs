//! Tests of the questions of an interview step and of sending them.

mod prompt;
mod send;
use std::future;
use std::sync::LazyLock;
use std::task::{Context, Poll, Waker};

use url::Url;

use super::*;
use crate::config::persona::tests::persona;
use crate::judge::{ImageFormat, QuestionId};
use crate::redact::{Redactor, Rule};
use crate::screen::{Element, Rect, screen_text};

/// The goal of the test steps.
pub const GOAL: &str = "注文を確定できる画面まで進む";

/// The preamble of [`persona()`] with [`GOAL`].
pub const P: &str = "あなたは次の人として、この画面を使っている。
名前: 佐藤さん
どんな人か: 60代。孫に頼まれて、週末にスマートフォンで買い物をする
ITの慣れ: スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている
達成したいこと: 孫の誕生日プレゼントを注文する
しないこと:
- URL を手で打たない
- 開発者ツールを開かない
目指すこと: 注文を確定できる画面まで進む
この人ならどうするか、どう感じるかで答える。";

static PERSONA: LazyLock<Persona> = LazyLock::new(persona);

const VIEWPORT: Viewport = Viewport {
    width: 800,
    height: 600,
};
const IN: Rect = Rect {
    x: 0,
    y: 0,
    width: 10,
    height: 10,
};
const OUT: Rect = Rect {
    x: 0,
    y: 900,
    width: 10,
    height: 10,
};

/// The screenshot of the tests.
pub fn shot() -> Image {
    Image::new(ImageFormat::Png, b"shot of test@example.test".to_vec())
}

/// A step of the test persona with two elements.
pub fn step(image: Option<&Image>) -> Step<'_> {
    Step {
        persona: &PERSONA,
        goal: GOAL,
        ops: &[Op::Click, Op::Back],
        elements: 2,
        image,
    }
}

/// What a [`Recorder`] received in one call.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Call {
    screen: String,
    image: Option<Vec<u8>>,
    questions: Vec<Question>,
}

/// A judge that records what it is asked.
struct Recorder {
    images: bool,
    calls: Vec<Call>,
}

impl Recorder {
    const fn new(images: bool) -> Self {
        Self {
            images,
            calls: Vec::new(),
        }
    }

    fn record(
        &mut self,
        screen: &Redacted<ScreenText>,
        image: Option<&Image>,
        questions: &[Question],
    ) -> Pending<'static, Result<Answers, JudgeError>> {
        self.calls.push(Call {
            screen: screen.as_inner().as_str().to_owned(),
            image: image.map(|image| image.bytes().to_vec()),
            questions: questions.to_vec(),
        });
        Box::pin(future::ready(Ok(Answers::default())))
    }
}

impl Judge for Recorder {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.record(screen, None, questions)
    }

    fn reads_images(&self) -> bool {
        self.images
    }

    fn ask_with_image<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        image: &'a Image,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.record(screen, Some(image), questions)
    }
}

/// Sends the questions and polls the answer once, with a waker that does
/// nothing.
fn sent(
    judge: &mut Recorder,
    screen: &Redacted<ScreenText>,
    image: Option<&Image>,
    questions: &[Question],
) {
    let mut pending = send(judge, screen, image, questions);
    let answers = pending
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()));
    assert_eq!(answers, Poll::Ready(Ok(Answers::default())), "answered");
}

fn element(role: &str, name: &str, rect: Option<Rect>) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: None,
        rect,
    }
}

fn screen(elements: Vec<Element>) -> Screen {
    Screen {
        url: Url::parse("https://shop.example.test/cart").unwrap(),
        elements,
    }
}

/// The screen text of two links in the viewport.
fn cart_text() -> Redacted<ScreenText> {
    let elements = vec![
        element("link", "カート", Some(IN)),
        element("link", "ヘルプ", Some(IN)),
    ];
    screen_text(&Redactor::default().screen(screen(elements)), VIEWPORT)
}

/// The screen text of a secret, redacted.
fn secret_text() -> Redacted<ScreenText> {
    let rule = Rule::literal("メールアドレス", "test@example.test");
    let redactor = Redactor::new(&[rule]).unwrap();
    let elements = vec![element("text", "test@example.test", Some(IN))];
    screen_text(&redactor.screen(screen(elements)), VIEWPORT)
}

fn p(s: &str) -> String {
    format!("{P}\n\n{s}")
}

fn ids(questions: &[Question]) -> Vec<&str> {
    questions.iter().map(|q| q.id().as_str()).collect()
}

/// The prompt of `question`.
const fn prompt(question: &Question) -> &String {
    match question {
        Question::Noul { prompt, .. }
        | Question::Choice { prompt, .. }
        | Question::Score { prompt, .. }
        | Question::Text { prompt, .. } => prompt,
    }
}

/// The question of the operation kind offering `options`.
fn operation(options: Vec<String>) -> Question {
    Question::Choice {
        id: QuestionId::operation(),
        prompt: p("次に行う操作の種別を選ぶ"),
        options,
    }
}

/// The question of the element offering `options`.
fn target(options: Vec<String>) -> Question {
    Question::Choice {
        id: QuestionId::target(),
        prompt: p("操作する要素の番号を選ぶ"),
        options,
    }
}

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().copied().map(str::to_owned).collect()
}

fn lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}
