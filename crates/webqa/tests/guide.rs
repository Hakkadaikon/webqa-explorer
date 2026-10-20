//! Tests of the guide that has an AI write the persona and purpose documents.

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use webqa_core::config::persona::{self, Persona};
use webqa_core::config::purpose::{self, Aim, Written};
use webqa_core::config::raw::DoneWhen;

type Fallible<T> = Result<T, Box<dyn Error>>;

const GUIDE: &str = "share/skills/writing-persona-purpose/SKILL.md";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn guide() -> Fallible<String> {
    Ok(fs::read_to_string(root().join(GUIDE))?)
}

/// The bodies of the fenced yaml blocks that contain `key`.
fn yaml_blocks<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    text.split("```yaml\n")
        .skip(1)
        .filter_map(|rest| rest.split("```").next())
        .filter(|block| block.contains(key))
        .collect()
}

fn front_matter_lines(text: &str) -> Fallible<Vec<&str>> {
    let front = text.strip_prefix("---\n").ok_or("no front matter")?;
    let front = front
        .split("\n---\n")
        .next()
        .ok_or("unclosed front matter")?;
    Ok(front.lines().collect())
}

fn sato() -> Persona {
    Persona {
        name: "佐藤さん".to_owned(),
        profile: "60代。孫に頼まれて、週末にスマートフォンで買い物をする".to_owned(),
        it_literacy: "スマートフォンの基本操作だけ。「カート」の意味は知っている".to_owned(),
        wants: "孫の誕生日プレゼントを注文する".to_owned(),
        avoids: vec![
            "URLを手で打たない".to_owned(),
            "「SKU」のような略語を知らない".to_owned(),
        ],
    }
}

fn assert_front_matter(text: &str) -> Fallible<()> {
    let front = front_matter_lines(text)?;
    let named = front
        .iter()
        .filter(|l| **l == "name: writing-persona-purpose");
    assert_eq!(named.count(), 1, "name");
    let described = front.iter().filter(|l| l.starts_with("description: "));
    assert_eq!(described.count(), 1, "description");
    assert_eq!(
        text.matches("webqa check-docs persona ").count(),
        1,
        "persona"
    );
    assert_eq!(
        text.matches("webqa check-docs purpose ").count(),
        1,
        "purpose"
    );
    Ok(())
}

#[test]
fn guide_has_frontmatter_steps_and_examples() -> Fallible<()> {
    let text = guide()?;
    assert_front_matter(&text)?;
    let personas = yaml_blocks(&text, "it_literacy:");
    assert_eq!(personas.len(), 1);
    assert_eq!(persona::parse(personas[0])?, sato());
    let purposes = yaml_blocks(&text, "goal:");
    assert_eq!(purposes.len(), 1);
    purpose::parse(purposes[0])?;
    assert_steps_fill_fields(&text)
}

/// The needles that `text` lacks.
fn lacking(text: &str, needles: &[String]) -> Vec<String> {
    let lacks = |n: &&String| !text.contains(n.as_str());
    needles.iter().filter(lacks).cloned().collect()
}

fn section(text: &str, from: &str, to: &str) -> Fallible<String> {
    let rest = text.split(from).nth(1).ok_or("no section")?;
    Ok(rest.split(to).next().ok_or("no end")?.to_owned())
}

fn assert_steps_fill_fields(text: &str) -> Fallible<()> {
    let persona = section(text, "## ペルソナを書く", "## 目的を書く")?;
    let keys = ["name", "profile", "it_literacy", "wants", "avoids"];
    let mut needles = keys.map(|k| format!("- `{k}`:")).to_vec();
    needles.push("インタビューのメモや分析のデータを読み".to_owned());
    assert_eq!(lacking(&persona, &needles), Vec::<String>::new(), "lacking");
    let purpose = section(text, "## 目的を書く", "## 書いたあとに確かめる")?;
    let keys = ["goal", "done_when", "screens"].map(|k| format!("`{k}`"));
    assert_eq!(lacking(&purpose, &keys), Vec::<String>::new(), "lacking");
    Ok(())
}

#[test]
fn guide_shows_done_when_at_confirm_screen() -> Fallible<()> {
    let text = guide()?;
    let needles = [
        "確定の操作ができる画面まで来たこと".to_owned(),
        "確定の操作そのものを実行したこと".to_owned(),
    ];
    assert_eq!(lacking(&text, &needles), Vec::<String>::new(), "lacking");
    let parsed = purpose::parse(yaml_blocks(&text, "goal:")[0])?;
    assert_eq!(
        parsed.aim,
        Aim::Written(Written {
            goal: "注文を確定できる画面まで進む".to_owned(),
            done_when: DoneWhen {
                url: Some("/order/confirm".to_owned()),
                element: None,
                text: None,
            },
        })
    );
    Ok(())
}

#[test]
fn guide_lives_outside_dev_skills() {
    assert!(root().join(GUIDE).is_file());
    assert!(!root().join("skills/writing-persona-purpose").exists());
}
