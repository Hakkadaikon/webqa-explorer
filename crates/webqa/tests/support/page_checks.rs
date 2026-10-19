//! Checks of the pages built: the layout and every builder.

use url::Url;
use webqa_core::ports::Observation;
use webqa_core::screen::visible;

use super::PageSpec;

/// Checks the pages built: the layout and every builder.
pub fn check() {
    let origin = "https://app.example.test";
    let page = PageSpec::at("/")
        .with(PageSpec::button("保存"))
        .with(PageSpec::link("利用規約").below())
        .build(origin);
    let shown = visible(&page.screen, page.viewport);
    let names: Vec<&str> = shown.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(page.screen.elements.len(), 2, "both elements");
    assert_eq!(names, ["保存"], "only the first is in the viewport");
    let refs: Vec<&str> = page.nodes.iter().map(|n| n.node.as_str()).collect();
    assert_eq!(refs, ["e1", "e2"], "the references");
    assert_eq!(page.text, "保存\n利用規約", "the text of the page");
    check_listed(origin);
    check_origin();
}

/// A page with everything set.
fn check_listed(origin: &str) {
    let field = PageSpec::textbox("名前")
        .href("/next")
        .form_action("/post")
        .forbidden_listed()
        .allowed_listed()
        .node("n1");
    let spec = PageSpec {
        headings: vec![("料金".to_owned(), false), ("下".to_owned(), true)],
        dialog: vec!["同意".to_owned()],
        blocked: true,
        console: vec!["boom".to_owned()],
        text: Some("全体".to_owned()),
        ..PageSpec::at("/a")
            .with(field)
            .with(PageSpec::button("送信"))
    };
    let page = spec.build(origin);
    check_candidate(&page);
    check_facts(&page);
}

fn check_candidate(page: &Observation) {
    let first = page.candidates.first();
    let links = first.map(|c| (c.href.clone(), c.form_action.clone()));
    let next = Url::parse("https://app.example.test/next").ok();
    let post = Url::parse("https://app.example.test/post").ok();
    assert_eq!(links, Some((next, post)), "the links against the page");
    let flags = first.map(|c| (c.forbidden_listed, c.allowed_listed));
    assert_eq!(flags, Some((true, true)), "the lists");
    let node = page.nodes.first().map(|n| n.node.as_str());
    assert_eq!(node, Some("n1"), "the reference");
}

fn check_facts(page: &Observation) {
    let paywall = &page.paywall;
    assert_eq!(paywall.headings, ["料金"], "the heading in the viewport");
    assert_eq!(paywall.buttons, ["送信"], "the button in the viewport");
    assert_eq!(paywall.hits, [false, false], "every operation is covered");
    assert_eq!(paywall.dialogs, ["同意"], "the names in the modal");
    assert_eq!(
        (page.console.as_slice(), page.text.as_str()),
        (["boom".to_owned()].as_slice(), "全体"),
        "the rest"
    );
    let ys: Vec<i64> = page
        .outline
        .iter()
        .filter_map(|e| e.rect.map(|r| r.y))
        .collect();
    assert_eq!(ys, [0, 40, 0, 1040], "the elements, then the headings");
}

/// A URL that is all in the origin.
fn check_origin() {
    let spec = PageSpec {
        origin: Some("javascript:alert(1)".to_owned()),
        path: String::new(),
        ..PageSpec::default()
    };
    let page = spec.build("https://app.example.test");
    assert_eq!(page.screen.url.as_str(), "javascript:alert(1)", "the URL");
}
