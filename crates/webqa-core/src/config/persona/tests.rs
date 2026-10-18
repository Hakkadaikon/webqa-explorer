//! Tests of reading the persona document.

use super::*;

/// A valid persona document.
pub const PERSONA: &str = "name: 佐藤さん
profile: 60代。孫に頼まれて、週末にスマートフォンで買い物をする
it_literacy: スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている
wants: 孫の誕生日プレゼントを注文する
avoids:
  - URL を手で打たない
  - 開発者ツールを開かない
";

/// The value of [`PERSONA`].
pub fn persona() -> Persona {
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

/// The error message of reading `text`.
fn rejected(text: &str) -> String {
    parse(text).unwrap_err().to_string()
}

/// [`PERSONA`] with the lines starting with any of `prefixes` removed.
pub fn without(prefixes: &[&str]) -> String {
    PERSONA
        .lines()
        .filter(|line| !starts_with_any(line, prefixes))
        .flat_map(|line| [line, "\n"])
        .collect()
}

fn starts_with_any(line: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|prefix| line.starts_with(prefix))
}

#[test]
fn reads_all_required_fields() {
    assert_eq!(parse(PERSONA).unwrap(), persona(), "every field as written");
    let block = PERSONA.replacen(
        "profile: 60代。孫に頼まれて、週末にスマートフォンで買い物をする\n",
        "profile: |\n  60代。\n  週末に買い物をする\n",
        1,
    );
    let profile = parse(&block).unwrap().profile;
    assert_eq!(profile, "60代。\n週末に買い物をする\n", "two lines kept");
}

/// Each field removed, with the message expected.
const REMOVED: [(&[&str], &str); 6] = [
    (&["name:"], "persona.name を書く"),
    (&["profile:"], "persona.profile を書く"),
    (&["it_literacy:"], "persona.it_literacy を書く"),
    (&["wants:"], "persona.wants を書く"),
    (&["avoids:", "  - "], "persona.avoids を書く"),
    (&["name:", "wants:"], "persona.name を書く"),
];

/// Documents with a blank value, with the message expected.
fn blank_values() -> [(String, &'static str); 5] {
    [
        (
            PERSONA.replacen(
                "it_literacy: スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている",
                "it_literacy: \"   \"",
                1,
            ),
            "persona.it_literacy を書く",
        ),
        (
            PERSONA.replacen("name: 佐藤さん", "name:", 1),
            "persona.name を書く",
        ),
        (
            format!("{}avoids: []\n", without(&["avoids:", "  - "])),
            "persona.avoids を書く",
        ),
        (
            format!(
                "{}avoids: [\"URL を手で打たない\", \" \"]\n",
                without(&["avoids:", "  - "])
            ),
            "persona.avoids を書く",
        ),
        (String::new(), "persona.name を書く"),
    ]
}

#[test]
fn missing_field_is_config_error() {
    for (prefixes, want) in REMOVED {
        assert_eq!(rejected(&without(prefixes)), want, "{prefixes:?}");
    }
    for (text, want) in blank_values() {
        assert_eq!(rejected(&text), want, "{text:?}");
    }
    empty_documents_miss_the_name();
}

fn empty_documents_miss_the_name() {
    for text in ["", "\n  \n", "# メモだけ\n"] {
        assert_eq!(rejected(text), "persona.name を書く", "{text:?}");
    }
}

/// Documents that are not a persona.
fn malformed() -> [String; 4] {
    [
        "name: [佐藤さん\n".to_owned(),
        "- 佐藤さん\n- 60代\n".to_owned(),
        format!("{PERSONA}age: 60\n"),
        format!("{PERSONA}name: 鈴木さん\n"),
    ]
}

#[test]
fn malformed_persona_rejected() {
    for text in malformed() {
        let message = rejected(&text);
        assert!(
            message.starts_with("persona を決まった形式で読めない: "),
            "{text:?}: {message}"
        );
    }
}
