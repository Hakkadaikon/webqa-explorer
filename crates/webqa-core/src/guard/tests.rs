//! Tests of the folding, words and path segments shared by the guards.

use url::Url;

use super::*;

fn owned(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_owned()).collect()
}

#[test]
fn words_match_at_ascii_boundaries() {
    let words = Words::new(&[], &owned(&["pay", "購入"]));
    let cases = [
        ("Pay now", true),
        ("PAY", true),
        ("Display", false),
        ("Prepay", false),
        ("payment", false),
        ("今すぐ購入", true),
        ("ｐａｙ", true),
        ("", false),
    ];
    for (text, want) in cases {
        assert_eq!(words.any_in(text), want, "{text}");
    }
}

#[test]
fn empty_keyword_is_ignored() {
    let words = Words::new(&[], &owned(&["", "  ", "　"]));
    assert!(!words.any_in("保存"), "an empty word matches nothing");
    assert!(
        !words.any_in(""),
        "an empty word does not match an empty text"
    );
    let segments = Segments::new(&[], &owned(&["", "-"]));
    let root = Url::parse("https://app.example.test/").unwrap();
    assert!(!segments.any_in(&root), "an empty segment matches nothing");
}

#[test]
fn segments_ignore_case_hyphen_and_query() {
    let segments = Segments::new(&[], &owned(&["signin", "pricing"]));
    let cases: [(&str, bool); 7] = [
        ("https://app.example.test/sign-in", true),
        ("https://app.example.test/Sign_In/", true),
        ("https://app.example.test/en/pricing", true),
        ("https://app.example.test/pricing-history", false),
        ("https://app.example.test/?next=/pricing", false),
        ("https://app.example.test/#/pricing", false),
        ("data:text/html,pricing", false),
    ];
    assert_segments(&segments, &cases);
}

fn assert_segments(segments: &Segments, cases: &[(&str, bool)]) {
    for (url, want) in cases {
        assert_eq!(segments.any_in(&Url::parse(url).unwrap()), *want, "{url}");
    }
}

#[test]
fn words_check_boundaries_only_at_ascii_ends() {
    let words = Words::new(&[], &owned(&["pay", "購入", "退会", "1-click"]));
    let cases = [
        ("購入3点", true),
        ("1-Click購入", true),
        ("退会API", true),
        ("PayPay", false),
        ("Prepay", false),
        ("pay2", false),
        ("x1-click", false),
        ("1-click!", true),
    ];
    for (text, want) in cases {
        assert_eq!(words.any_in(text), want, "{text}");
    }
}

#[test]
fn fold_turns_halfwidth_kana_and_hyphens_to_one_form() {
    let cases = [
        ("ｻｲﾝｲﾝ", "サインイン"),
        ("ｶｰﾄﾞ番号", "カード番号"),
        ("ｱｯﾌﾟｸﾞﾚｰﾄﾞ", "アップグレード"),
        ("ｳﾞｧ｡｢ｦﾝ｣", "ヴァ。「ヲン」"),
        ("ﾞｱﾟ", "゛ア゜"),
        ("Check-out", "check out"),
        ("－Ｃｈｅｃｋ－ｏｕｔ－", "check out"),
    ];
    for (text, want) in cases {
        assert_eq!(fold(text), want, "{text}");
    }
}

#[test]
fn segments_decode_and_drop_parameters_and_extensions() {
    let segments = Segments::new(&[], &owned(&["checkout", "プラン"]));
    let cases = [
        ("https://app.example.test/checkout.php", true),
        ("https://app.example.test/checkout;jsessionid=1", true),
        ("https://app.example.test/%63heckout", true),
        ("https://app.example.test/%43HECKOUT", true),
        ("https://app.example.test/check%2Dout.html", true),
        ("https://app.example.test/プラン", true),
        ("https://app.example.test/checkout.v2.php", false),
        ("https://app.example.test/checkouts", false),
        ("https://app.example.test/%6", false),
        ("https://app.example.test/%zzcheckout", false),
        ("https://app.example.test/%+6checkout", false),
    ];
    assert_segments(&segments, &cases);
}
