//! The screenshot: when it goes to the judge, in what form, and what is
//! hidden in the text that goes with it.

use cucumber::{given, then, when};
use serde_json::json;
use webqa_core::judge::ImageFormat;
use webqa_core::redact::Redactor;
use webqa_core::screen::screen_text;
use webqa_judge::table::{StartError, start};

use super::replies::{SAVE, button, reply, step};
use crate::bridge::RunDir;
use crate::fakes::{FakeClock, PNG};
use crate::judge::Asked;
use crate::scene::found;
use crate::start::start as started_by;
use crate::world::{InterviewWorld, Ran};

/// The screenshot as a request line of the bridge carries it: its media
/// type and the Base64 of [`PNG`].
fn wired() -> serde_json::Value {
    json!({ "media_type": "image/png", "data": "iVBORw==" })
}

/// The first request the bridge read carries the screenshot as wired.
fn image_item_is_as_wired(world: &InterviewWorld) {
    let request = found(world.ran().bridge.first(), "the bridge read a request");
    assert_eq!(request.get("image"), Some(&wired()), "the item `image`");
}

/// The first call of the judge of the scene.
fn first_call(ran: &Ran) -> &Asked {
    found(ran.judge.asked.first(), "the judge was asked")
}

/// The format and the bytes of the screenshot of `asked`.
fn shot_of(asked: &Asked) -> Option<(ImageFormat, Vec<u8>)> {
    let shot = asked.image.as_ref();
    shot.map(|image| (image.format(), image.bytes().to_vec()))
}

#[given(expr = "判断モデルのアダプタが、起動のときに画像を扱えると申告している")]
fn adapter_reads_images(world: &mut InterviewWorld) {
    world.scene.images = true;
    button(&mut world.scene, SAVE);
}

#[then(
    expr = "判断モデルに、アクセシビリティ情報のテキストと、画面のスクリーンショットを渡している"
)]
async fn text_and_screenshot(world: &mut InterviewWorld) {
    world.run().await;
    let first = first_call(world.ran());
    assert_eq!(first.screen, r#"1: button "保存""#, "the text");
    let shot = Some((ImageFormat::Png, PNG.to_vec()));
    assert_eq!(shot_of(first), shot, "the screenshot");
}

#[given(expr = "判断モデルとして外部コマンドとの橋渡しを使う設定である")]
#[given(expr = "判断モデルとして 外部コマンドとの橋渡し を使う設定である")]
fn bridge_judge(world: &mut InterviewWorld) {
    world.scene.bridge = Some(true);
    button(&mut world.scene, SAVE);
    reply(&mut world.scene, step("click", Some("1"), 0.0));
}

#[then(expr = "外部コマンドに書いた問いの JSON に、画像を渡す項目がある")]
async fn json_has_image(world: &mut InterviewWorld) {
    world.run().await;
    image_item_is_as_wired(world);
}

/// The kind of the judge is a setting, and only `command` starts: the form
/// of the request is written once, in the wire of that one judge, so a
/// setting of the kind cannot change it.
#[then(expr = "その項目の名前と形は、判断モデルの種類の設定を変えても変わらない")]
async fn image_item_is_fixed(world: &mut InterviewWorld) {
    world.run().await;
    let dir = RunDir::new();
    let started = started_by(&world.scene, &[], &dir.0);
    let mut setup = found(started.ok(), "the configuration starts").setup;
    "openai".clone_into(&mut setup.judge.kind);
    let other = start(&setup.judge, Box::new(FakeClock::default())).await;
    let error = other.err();
    let unknown = Some(StartError::UnknownKind("openai".to_owned()));
    assert_eq!(error, unknown, "no other kind starts");
    image_item_is_as_wired(world);
}

#[given(expr = "判断モデルとして 台本どおりに答えるスタブ を使う設定である")]
fn stub_judge(world: &mut InterviewWorld) {
    world.scene.images = true;
    button(&mut world.scene, SAVE);
}

/// What the adapter declared at the start.
fn declared(world: &InterviewWorld) -> bool {
    world.scene.bridge.unwrap_or(world.scene.images)
}

/// Whether the first question the adapter got carried the screenshot.
fn attached(world: &InterviewWorld) -> bool {
    let ran = world.ran();
    match world.scene.bridge {
        Some(_) => found(ran.bridge.first(), "the bridge read a request")
            .get("image")
            .is_some(),
        None => first_call(ran).image.is_some(),
    }
}

/// The questions carry a screenshot exactly when the adapter declared that
/// it reads images: checked for the declaration of the scene and for the
/// other one.
async fn follows_declaration(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(attached(world), declared(world), "the declared one");
    let flipped = !declared(world);
    match world.scene.bridge {
        Some(_) => world.scene.bridge = Some(flipped),
        None => world.scene.images = flipped,
    }
    world.again();
    world.run().await;
    assert_eq!(attached(world), flipped, "the other declaration");
}

#[when(expr = "実行を始めてアダプタを起動する")]
#[when(expr = "実行を始めて橋渡しを起動する")]
async fn start_adapter(world: &mut InterviewWorld) {
    world.run().await;
}

#[then(expr = "アダプタの画像を扱えるかの申告を読み、それに合わせて問い合わせを作っている")]
async fn adapter_declaration(world: &mut InterviewWorld) {
    follows_declaration(world).await;
}

#[then(expr = "橋渡しが返した JSON の版の項目から、画像を扱えるかの申告を読んでいる")]
async fn bridge_declaration(world: &mut InterviewWorld) {
    follows_declaration(world).await;
}

#[then(expr = "判断モデルに渡したテキストではメールアドレスが伏せられている")]
async fn text_hides_email(world: &mut InterviewWorld) {
    world.run().await;
    let scene = &world.scene;
    let page = scene.start.build(&scene.origin());
    let plain = screen_text(
        &Redactor::default().screen(page.screen.clone()),
        page.viewport,
    );
    let redactor = found(Redactor::new(&scene.redact).ok(), "the rules read");
    let hidden = screen_text(&redactor.screen(page.screen), page.viewport);
    let (plain, hidden) = (plain.as_inner().as_str(), hidden.as_inner().as_str());
    assert_ne!(plain, hidden, "the rule hides the address");
    assert_eq!(first_call(world.ran()).screen, hidden, "the text sent");
}

#[then(expr = "判断モデルに渡したスクリーンショットは、画面をそのまま写したものである")]
async fn screenshot_is_as_taken(world: &mut InterviewWorld) {
    world.run().await;
    let shot = Some((ImageFormat::Png, PNG.to_vec()));
    assert_eq!(shot_of(first_call(world.ran())), shot, "the screenshot");
}
