//! Tests of the forbidden element guard.

use url::Url;

use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

/// A candidate with no destination and on neither list.
fn named(role: &str, name: &str) -> Candidate {
    Candidate {
        role: role.to_owned(),
        name: name.to_owned(),
        href: None,
        form_action: None,
        forbidden_listed: false,
        allowed_listed: false,
    }
}

fn with_action(role: &str, name: &str, action: &str) -> Candidate {
    Candidate {
        form_action: Some(url(action)),
        ..named(role, name)
    }
}

fn with_href(name: &str, href: &str) -> Candidate {
    Candidate {
        href: Some(url(href)),
        ..named("link", name)
    }
}

fn listed(name: &str, forbidden_listed: bool, allowed_listed: bool) -> Candidate {
    Candidate {
        forbidden_listed,
        allowed_listed,
        ..named("button", name)
    }
}

fn assert_cases(guard: &Forbidden, cases: &[(Candidate, bool)]) {
    for (c, want) in cases {
        assert_eq!(guard.forbids(c), *want, "{} {:?}", c.name, c);
    }
}

#[test]
fn forbidden_by_keyword() {
    let cases = [
        (named("button", "購入する"), true),
        (named("button", "アカウントを削除"), true),
        (named("button", "退会"), true),
        (named("button", "サインイン"), true),
        (named("button", "Checkout"), true),
        (named("button", "購入はこちら"), true),
        (named("button", "今すぐ買う"), true),
        (named("textbox", "カード番号"), true),
        (named("button", "ＣＨＥＣＫＯＵＴ"), true),
        (named("button", "Sign  In"), true),
        (listed("購入テスト", false, true), false),
        (named("button", "保存"), false),
        (named("link", "Display settings"), false),
        (named("link", "Payments"), false),
        (named("button", ""), false),
    ];
    assert_cases(&Forbidden::default(), &cases);
}

#[test]
fn forbidden_by_destination() {
    let app = "https://app.example.test";
    let cases = [
        (
            with_action("button", "OK", &format!("{app}/account/delete")),
            true,
        ),
        (with_href("次へ", &format!("{app}/checkout/confirm")), true),
        (with_href("次へ", &format!("{app}/sign-in")), true),
        (with_action("button", "", &format!("{app}/checkout")), true),
        (with_href("次へ", &format!("{app}/settings")), false),
        (
            with_action("button", "検索", &format!("{app}/search")),
            false,
        ),
        (
            with_href("次へ", &format!("{app}/login?next=/checkout")),
            false,
        ),
    ];
    assert_cases(&Forbidden::default(), &cases);
}

#[test]
fn spelling_variants_are_forbidden() {
    let cases = [
        (named("button", "購入3点"), true),
        (named("button", "1-Click購入"), true),
        (named("button", "退会API"), true),
        (named("button", "ｻｲﾝｲﾝ"), true),
        (named("textbox", "ｶｰﾄﾞ番号"), true),
        (named("button", "Check-out"), true),
        (named("button", "注文確定"), true),
        (named("button", "ご注文手続きへ"), true),
        (named("button", "レジに進む"), true),
        (named("button", "Order now"), true),
        (named("button", "Confirm order"), true),
        (named("button", "Place order"), true),
        (named("link", "PayPay"), false),
        (named("link", "Display"), false),
        (named("link", "Prepay"), false),
    ];
    assert_cases(&Forbidden::default(), &cases);
}

#[test]
fn encoded_or_suffixed_destinations_are_forbidden() {
    let app = "https://app.example.test";
    let cases = [
        (with_href("次へ", &format!("{app}/checkout.php")), true),
        (
            with_href("次へ", &format!("{app}/checkout;jsessionid=1")),
            true,
        ),
        (
            with_action("button", "OK", &format!("{app}/%63heckout")),
            true,
        ),
        (with_href("次へ", &format!("{app}/checkouts")), false),
    ];
    assert_cases(&Forbidden::default(), &cases);
}

#[test]
fn forbidden_by_list() {
    let cases = [
        (listed("実行", true, false), true),
        (listed("保存", false, true), false),
        (listed("実行", true, true), false),
    ];
    assert_cases(&Forbidden::default(), &cases);
}

#[test]
fn allowed_list_wins_over_every_axis() {
    let shop = "https://shop.example.test";
    let everything = Candidate {
        href: Some(url(&format!("{shop}/checkout/confirm"))),
        form_action: Some(url(&format!("{shop}/account/delete"))),
        ..listed("購入して退会", true, true)
    };
    let cases = [
        (everything.clone(), false),
        (
            Candidate {
                allowed_listed: false,
                ..everything
            },
            true,
        ),
        (
            Candidate {
                allowed_listed: true,
                ..with_href("次へ", &format!("{shop}/checkout"))
            },
            false,
        ),
        (
            Candidate {
                allowed_listed: true,
                ..with_action("button", "OK", &format!("{shop}/checkout"))
            },
            false,
        ),
    ];
    assert_cases(&Forbidden::default(), &cases);
}

#[test]
fn extra_keywords_extend_defaults() {
    let danger = named("button", "危険な操作");
    let extended = Forbidden::new(&["危険".into()]);
    assert!(extended.forbids(&danger), "an added keyword forbids");
    assert!(
        !Forbidden::default().forbids(&danger),
        "the defaults do not forbid it"
    );
    assert!(
        !Forbidden::new(&[String::new()]).forbids(&named("button", "保存")),
        "an empty keyword forbids nothing"
    );
    assert!(
        extended.forbids(&named("button", "退会")),
        "the defaults stay after adding"
    );
}
