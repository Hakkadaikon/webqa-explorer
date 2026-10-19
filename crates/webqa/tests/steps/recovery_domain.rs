//! The steps of the Rule that are about leaving the allowed origins.

use cucumber::{given, then, when};

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::recovery::{field, lines_of};
use crate::stages_given::after_at;
use crate::world::QaWorld;

/// The page of an article at `url`, a whole URL.
fn landing(url: &str) -> PageSpec {
    let page = PageSpec {
        origin: Some(url.to_owned()),
        ..PageSpec::at("")
    };
    page.with(PageSpec::button("記事"))
}

#[given(expr = "判断モデルが操作 {string} とリンク {string} を返す")]
fn link_reply(world: &mut QaWorld, op: String, name: String) {
    let scene = &mut world.scene;
    scene.start.elements.push(PageSpec::link(&name));
    scene.start.elements.push(PageSpec::button("保存"));
    scene.replies.push_back(Reply::op(op, 0.9).on(&name));
    drop(name);
}

#[given(expr = "{string} は {string} へ遷移する")]
fn leads_to(world: &mut QaWorld, name: String, to: String) {
    let scene = &mut world.scene;
    for element in scene.start.elements.iter_mut().filter(|e| e.name == name) {
        element.href = Some(to.clone());
    }
    *after_at(scene, 0) = landing(&to);
    drop((name, to));
}

/// Whether the log has an issue line of `kind`.
fn has_issue(world: &QaWorld, kind: &str) -> bool {
    let issues = lines_of(world, "issue");
    issues.iter().any(|line| field(line, "issue") == Some(kind))
}

/// Whether the log has a return to the allowed origins.
fn has_return(world: &QaWorld) -> bool {
    let lines = lines_of(world, "recovered");
    let back = |line: &serde_json::Value| field(line, "op") == Some("return");
    lines.iter().any(back)
}

#[then(expr = "許可ドメインの外へ遷移したことが記録されている")]
#[then(expr = "許可ドメインの外への遷移として扱う")]
async fn outside_recorded(world: &mut QaWorld) {
    world.run().await;
    assert!(has_issue(world, "outside_domain"), "the issue is recorded");
}

#[then(expr = "ドメイン外のページについて判断モデルに問い合わせていない")]
async fn not_asked_outside(world: &mut QaWorld) {
    world.run().await;
    let texts = &world.ran().judge.texts;
    assert!(
        texts.iter().all(|text| !text.contains("\"記事\"")),
        "the article page was not sent: {texts:?}"
    );
}

#[then(expr = "次のステップの前に許可ドメインのページへ戻っている")]
async fn returned_before_next(world: &mut QaWorld) {
    world.run().await;
    let returns = lines_of(world, "recovered");
    let causes: Vec<_> = returns.iter().filter_map(|l| field(l, "cause")).collect();
    assert!(has_return(world), "the return is logged");
    assert_eq!(causes, ["outside"], "the cause of the return");
    let texts = &world.ran().judge.texts;
    assert_eq!(
        texts.last(),
        texts.first(),
        "the last question is the start page"
    );
    assert!(
        world.ran().judge.stalled,
        "the next step waited for the judge"
    );
}

#[then(expr = "探索済みの要素は失われていない")]
async fn explored_kept(world: &mut QaWorld) {
    world.run().await;
    assert!(
        world.explored().contains("外部の記事"),
        "the click is remembered"
    );
}

#[given(expr = "許可ドメインのサブドメインの扱いは {string} に設定されている")]
fn subdomains(world: &mut QaWorld, setting: String) {
    if setting != "なし" {
        world
            .scene
            .origins
            .push("https://*.app.example.test".to_owned());
    }
    drop(setting);
}

#[when(expr = "操作の結果 {string} へ遷移する")]
async fn leads_to_result(world: &mut QaWorld, to: String) {
    let scene = &mut world.scene;
    scene.start.elements.push(PageSpec::link("次へ"));
    scene.replies.push_back(Reply::op("click", 0.9).on("次へ"));
    if scene.after.is_empty() {
        *after_at(scene, 0) = landing(&to);
    }
    world.run().await;
}

#[then(expr = "許可ドメインの外への遷移として扱わない")]
async fn not_outside(world: &mut QaWorld) {
    world.run().await;
    assert!(!has_issue(world, "outside_domain"), "no issue is recorded");
    assert!(!has_return(world), "no return happened");
}

#[given(expr = "許可ドメインの中の {string} は {string} へリダイレクトする")]
fn redirects(world: &mut QaWorld, from: String, to: String) {
    *after_at(&mut world.scene, 0) = landing(&to);
    drop((from, to));
}
