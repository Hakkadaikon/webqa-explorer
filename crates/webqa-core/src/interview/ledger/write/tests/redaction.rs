//! Tests of the redaction of every string the ledger writes.

use super::hesitant;
use crate::config::raw::Op;
use crate::interview::issue::tests::sample::sample;
use crate::interview::issue::{
    Act, Detail, Halt, Issue, Place, ScreenMark, Target, UncheckedPurpose,
};
use crate::interview::ledger::tests::{expecting, golden, hard_to_see, lines, persona_trait};
use crate::interview::ledger::{Goal, Head, Ledger, read, write};
use crate::interview::natural::{PersonaTrait, Trait};
use crate::redact::{Redactor, Rule};

/// The secrets, each with the name of its rule, one for each string that
/// is redacted.
const SECRETS: [(&str, &str); 19] = [
    ("名前", "秘01"),
    ("書いた目的", "秘02"),
    ("推測した目的", "秘03"),
    ("開始", "秘04"),
    ("画面", "秘05"),
    ("型", "秘06"),
    ("性質", "秘07"),
    ("操作の要素", "秘08"),
    ("目的の型", "秘09"),
    ("目的の文", "秘10"),
    ("推測", "秘11"),
    ("ずれの目的", "秘12"),
    ("役割", "秘13"),
    ("要素の名前", "秘14"),
    ("届かない期待", "秘15"),
    ("迷いの期待", "秘16"),
    ("見えにくい理由", "秘17"),
    ("見えにくい期待", "秘18"),
    ("操作の役割", "秘19"),
];

/// The secret named `name`.
fn secret(name: &str) -> String {
    let found = SECRETS.iter().find(|(it, _)| *it == name);
    found.map(|(_, s)| (*s).to_owned()).unwrap()
}

/// The golden ledger with a secret in each string that is redacted, the
/// purpose written.
fn planted() -> Ledger {
    let mut ledger = golden();
    ledger.head = Head {
        persona: secret("名前"),
        goal: Goal::Written(secret("書いた目的")),
        start: secret("開始"),
        ..golden().head
    };
    ledger.entries[0].issue.place = Place::Pattern(secret("型"));
    ledger.entries[0].issue.detail = Detail::Mismatch {
        purpose: secret("ずれの目的"),
        guesses: vec![secret("推測")],
    };
    ledger.entries[1].issue = planted_hesitant();
    ledger.entries[1].persona_trait = Some(PersonaTrait {
        content: secret("性質"),
        ..persona_trait(Trait::ItLiteracy)
    });
    ledger.entries[2].issue.detail = Detail::Visibility {
        reason: secret("見えにくい理由"),
        expectation: secret("見えにくい期待"),
    };
    ledger.entries[3].issue.detail = Detail::Unreached {
        halt: Halt::GaveUp,
        expectation: Some(secret("届かない期待")),
    };
    ledger.unchecked_purposes[0] = UncheckedPurpose {
        pattern: secret("目的の型"),
        purpose: secret("目的の文"),
    };
    ledger
}

/// The golden hesitation issue with a secret in its URL, its element's
/// role and name, its path's element and its expectation.
fn planted_hesitant() -> Issue {
    let secret_target = |role, name| Target {
        role: secret(role),
        name: secret(name),
        ..sample("I-5334A6B3", 2, 3).target.unwrap()
    };
    let mark = ScreenMark::parse("S-B9520D77").unwrap();
    let path = vec![Act {
        op: Op::Click,
        target: Some(secret_target("操作の役割", "操作の要素")),
    }];
    Issue {
        place: Place::Screen {
            url: secret("画面"),
            mark,
        },
        target: Some(secret_target("役割", "要素の名前")),
        detail: expecting(&secret("迷いの期待")),
        path,
        ..sample("I-5334A6B3", 2, 3)
    }
}

/// `ledger` written with a literal rule for each secret.
fn masked(ledger: &Ledger) -> String {
    let rules: Vec<Rule> = SECRETS.map(|(name, s)| Rule::literal(name, s)).to_vec();
    write(ledger, &Redactor::new(&rules).unwrap())
}

/// `ledger` written with every secret masked: the secrets left, and the
/// names of the rules not seen as masked among `names`.
fn leaks(ledger: &Ledger, names: &[&str]) -> (Vec<&'static str>, Vec<String>) {
    let text = masked(ledger);
    let left = SECRETS
        .map(|(_, s)| s)
        .into_iter()
        .filter(|s| text.contains(s));
    let unmasked = names.iter().map(|name| format!("[[{name}]]"));
    let unmasked = unmasked.filter(|masked| !text.contains(masked.as_str()));
    (left.collect(), unmasked.collect())
}

/// The names of the secrets, but `without`.
fn names_but(without: &str) -> Vec<&'static str> {
    SECRETS
        .map(|(name, _)| name)
        .into_iter()
        .filter(|it| *it != without)
        .collect()
}

#[test]
fn ledger_text_passes_redaction() {
    let none = (vec![], vec![]);
    let written = planted();
    assert_eq!(
        leaks(&written, &names_but("推測した目的")),
        none,
        "written purpose"
    );
    let mut inferred = planted();
    inferred.head.goal = Goal::Inferred(vec![secret("推測した目的")]);
    assert_eq!(
        leaks(&inferred, &names_but("書いた目的")),
        none,
        "inferred purpose"
    );
    mail_redaction();
    quote_redaction();
}

const MAIL: &str = "test@example.test";

/// The golden ledger with the mail address in the persona's name, the
/// visibility reason and the hesitation element, written with the
/// addresses masked by a pattern.
fn mailed() -> String {
    let mut ledger = golden();
    ledger.head.persona = MAIL.to_owned();
    ledger.entries[1].issue.target = Some(Target {
        name: format!("{MAIL} に送る"),
        ..sample("I-5334A6B3", 2, 3).target.unwrap()
    });
    ledger.entries[2].issue = hard_to_see(&format!("{MAIL} の表示が小さい"));
    let rule = Rule::pattern("メールアドレス", r"[a-z]+@example\.test");
    write(&ledger, &Redactor::new(&[rule]).unwrap())
}

/// The mail addresses masked by a pattern rule.
fn mail_redaction() {
    let text = mailed();
    assert_eq!(text.find(MAIL), None, "no mail address");
    let reason = "  - 理由: \"[[メールアドレス]] の表示が小さい\"";
    assert_eq!(lines(&text)[54], reason, "the reason");
    let ends = lines(&text)[45].ends_with("(理由: [[メールアドレス]] の表示が小さい)");
    assert!(ends, "the instruction");
    let persona = "- ペルソナ: \"[[メールアドレス]]\"";
    assert_eq!(lines(&text)[3], persona, "the persona");
    let back = read(&text).map(|it| it.entries[2].issue.detail.clone());
    let expected = hard_to_see("[[メールアドレス]] の表示が小さい").detail;
    assert_eq!(back, Ok(expected), "read back");
}

/// A secret with a quote, masked before it is written as JSON.
fn quote_redaction() {
    let redactor = Redactor::new(&[Rule::literal("合言葉", "a\"b")]).unwrap();
    let text = write(&hesitant("x a\"b y", "次へ"), &redactor);
    let leaked = (text.contains("a\"b"), text.contains("a\\\"b"));
    assert_eq!(leaked, (false, false), "no secret");
    let masked = text.contains("  - 期待: \"x [[合言葉]] y\"");
    assert!(masked, "masked before JSON");
}
