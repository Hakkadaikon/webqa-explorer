//! Tests of the values of inputs and checks.

use super::tests::entries;

/// (snapshot, the value of each entry).
const VALUES: [(&str, &[Option<&str>]); 16] = [
    (r#"- textbox "空" [ref=e1]"#, &[None]),
    (r#"- textbox "a" [ref=e2]: abc"#, &[Some("abc")]),
    (r#"- textbox "b" [ref=e3]: "  前後  ""#, &[Some("  前後  ")]),
    (
        r#"- textbox "c" [ref=e4]: "a\nb \"c\" \\ \x01""#,
        &[Some("a\nb \"c\" \\ \u{1}")],
    ),
    (r#"- searchbox "d" [ref=e5]: "123""#, &[Some("123")]),
    (
        "- textbox \"名前\" [ref=e6]:\n  - /placeholder: 山田\n  - text: 太郎",
        &[Some("太郎")],
    ),
    (r#"- 'checkbox "e" [checked] [ref=e7]'"#, &[Some("checked")]),
    (
        r#"- checkbox "f" [checked=mixed] [ref=e8]"#,
        &[Some("mixed")],
    ),
    (r#"- radio "g" [ref=e9]"#, &[None]),
    (r#"- switch "h" [checked] [ref=e10]"#, &[Some("checked")]),
    (r#"- button "閉じる" [ref=e11]: ×"#, &[None]),
    (
        "- combobox \"国\" [ref=e12]:\n  - option \"日本\" [box=0,0,0,0]\n  - option \"米国\" [selected] [box=0,0,0,0]",
        &[None],
    ),
    ("- button \"保存\" [ref=e13]\n- text: 地の文", &[None]),
    (
        "- textbox \"後\" [ref=e14]\n  - button \"間\" [ref=e15]\n  - text: 届かない",
        &[None, None],
    ),
    ("- textbox \"同\" [ref=e16]\n- text: 同じ段", &[None]),
    ("- textbox \"深\" [ref=e17]:\n    - text: 2段下", &[None]),
];

#[test]
fn values_of_inputs_and_checks() {
    for (text, values) in VALUES {
        let found: Vec<Option<String>> =
            entries(text).into_iter().map(|x| x.element.value).collect();
        let expected: Vec<Option<String>> = values.iter().map(|v| v.map(str::to_owned)).collect();
        assert_eq!(found, expected, "{text}");
    }
}
