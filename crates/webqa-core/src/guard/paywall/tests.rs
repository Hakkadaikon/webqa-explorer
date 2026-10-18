//! Tests of the paywall guard.

use url::Url;

use super::*;

fn owned(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|t| (*t).to_owned()).collect()
}

/// A screen with only headings and hits.
fn facts(path: &str, headings: &[&str], hits: &[bool]) -> PaywallFacts {
    PaywallFacts {
        url: Url::parse(&format!("https://app.example.test{path}")).unwrap(),
        headings: owned(headings),
        dialogs: Vec::new(),
        buttons: Vec::new(),
        hits: hits.to_vec(),
    }
}

/// A screen at `/home` with a modal and buttons outside it.
fn with_dialog(dialogs: &[&str], buttons: &[&str], hits: &[bool]) -> PaywallFacts {
    PaywallFacts {
        dialogs: owned(dialogs),
        buttons: owned(buttons),
        ..facts("/home", &[], hits)
    }
}

#[test]
fn paywall_by_each_axis() {
    let guard = Paywall::default();
    let cases = [
        with_dialog(
            &["プレミアムにアップグレード", "閉じる"],
            &["保存"],
            &[true, true, true],
        ),
        PaywallFacts {
            buttons: owned(&["アップグレード"]),
            ..facts("/article/1", &["Subscribe to continue"], &[])
        },
        facts("/pricing", &["プランを選ぶ"], &[true]),
        facts("/upgrade", &["プランを選ぶ"], &[true]),
        facts("/checkout", &["プランを選ぶ"], &[true]),
        with_dialog(&["お知らせ", "OK"], &[], &[false, false, false]),
    ];
    for f in &cases {
        assert!(guard.detects(f), "{}", f.url);
    }
}

#[test]
fn pricing_word_alone_is_not_paywall() {
    let guard = Paywall::default();
    let history = PaywallFacts {
        buttons: owned(&["ダウンロード"]),
        ..facts("/billing/history", &["料金の履歴"], &[true, true])
    };
    assert!(!guard.detects(&history), "a pricing word alone");
    assert!(!guard.detects(&facts("/", &[], &[])), "an empty page");
}

#[test]
fn upgrade_button_alone_is_not_paywall() {
    let header = PaywallFacts {
        buttons: owned(&["Upgrade", "プレミアム"]),
        ..facts("/home", &["ダッシュボード"], &[true, true])
    };
    assert!(
        !Paywall::default().detects(&header),
        "paying words on buttons outside a modal"
    );
}

#[test]
fn blocked_needs_more_than_half() {
    let cases: [(&[bool], bool); 7] = [
        (&[], false),
        (&[false], true),
        (&[true], false),
        (&[true, false], false),
        (&[false, false, true], true),
        (&[true, true, false, false], false),
        (&[false, false, false, true], true),
    ];
    for (hits, want) in cases {
        assert_eq!(blocked(hits), want, "{hits:?}");
    }
}

#[test]
fn extra_words_and_segments() {
    let extended = Paywall::new(&["お試し期間終了".into()], &["plans".into()]);
    let ended = facts("/home", &["お試し期間終了"], &[]);
    let in_dialog = with_dialog(&["お試し期間終了"], &[], &[]);
    let plans = facts("/plans", &[], &[]);
    for f in [&ended, &in_dialog, &plans] {
        assert!(extended.detects(f), "{}", f.url);
        assert!(!Paywall::default().detects(f), "{}", f.url);
    }
    let empty = Paywall::new(&[String::new()], &[String::new()]);
    assert!(
        !empty.detects(&facts("/home", &["お知らせ"], &[])),
        "an empty word or segment matches nothing"
    );
}
