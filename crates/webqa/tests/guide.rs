//! Tests of the guide that has an AI write the persona and purpose documents.

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use webqa_core::config::persona;
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

#[test]
fn guide_has_frontmatter_steps_and_examples() -> Fallible<()> {
    let text = guide()?;
    let front = text.strip_prefix("---\n").ok_or("no front matter")?;
    let front = front
        .split("\n---\n")
        .next()
        .ok_or("unclosed front matter")?;
    assert_eq!(front.lines().filter(|l| l.starts_with("name: ")).count(), 1);
    assert_eq!(
        front
            .lines()
            .filter(|l| l.starts_with("description: "))
            .count(),
        1
    );
    assert_eq!(text.matches("webqa check-docs persona ").count(), 1);
    assert_eq!(text.matches("webqa check-docs purpose ").count(), 1);
    let personas = yaml_blocks(&text, "it_literacy:");
    assert_eq!(personas.len(), 1);
    assert_eq!(persona::parse(personas[0])?.avoids.len(), 2);
    let purposes = yaml_blocks(&text, "goal:");
    assert_eq!(purposes.len(), 1);
    purpose::parse(purposes[0])?;
    Ok(())
}

#[test]
fn guide_shows_done_when_at_confirm_screen() -> Fallible<()> {
    let text = guide()?;
    assert!(text.contains("確定の操作ができる画面まで来たこと"));
    assert!(text.contains("確定の操作そのものを実行したこと"));
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
