//! Golden tests of the guards over fragments of shop and `SaaS` screens.

use serde::Deserialize;
use url::{ParseError, Url};
use webqa_core::guard::forbidden::{Candidate, Forbidden};
use webqa_core::guard::paywall::{Paywall, PaywallFacts};

const FORBIDDEN: &str = include_str!("fixtures/guard/forbidden.json");
const PAYWALL: &str = include_str!("fixtures/guard/paywall.json");

#[derive(Deserialize)]
struct ForbiddenRow {
    case: String,
    role: String,
    name: String,
    href: Option<String>,
    form_action: Option<String>,
    forbidden_listed: bool,
    allowed_listed: bool,
    expect: bool,
}

#[derive(Deserialize)]
struct PaywallRow {
    case: String,
    url: String,
    headings: Vec<String>,
    dialogs: Vec<String>,
    buttons: Vec<String>,
    hits: Vec<bool>,
    expect: bool,
}

/// Parses an optional URL of a row.
fn url(s: Option<&str>) -> Result<Option<Url>, ParseError> {
    s.map(Url::parse).transpose()
}

fn candidate(row: &ForbiddenRow) -> Result<Candidate, ParseError> {
    Ok(Candidate {
        role: row.role.clone(),
        name: row.name.clone(),
        href: url(row.href.as_deref())?,
        form_action: url(row.form_action.as_deref())?,
        forbidden_listed: row.forbidden_listed,
        allowed_listed: row.allowed_listed,
    })
}

fn facts(row: &PaywallRow) -> Result<PaywallFacts, ParseError> {
    Ok(PaywallFacts {
        url: Url::parse(&row.url)?,
        headings: row.headings.clone(),
        dialogs: row.dialogs.clone(),
        buttons: row.buttons.clone(),
        hits: row.hits.clone(),
    })
}

#[test]
fn forbidden_fixtures() {
    let rows: Vec<ForbiddenRow> = serde_json::from_str(FORBIDDEN).unwrap();
    assert_ne!(rows.len(), 0, "the fixture has rows");
    let guard = Forbidden::default();
    for row in &rows {
        assert_eq!(
            guard.forbids(&candidate(row).unwrap()),
            row.expect,
            "{}",
            row.case
        );
    }
}

#[test]
fn paywall_fixtures() {
    let rows: Vec<PaywallRow> = serde_json::from_str(PAYWALL).unwrap();
    assert_ne!(rows.len(), 0, "the fixture has rows");
    let guard = Paywall::default();
    for row in &rows {
        assert_eq!(
            guard.detects(&facts(row).unwrap()),
            row.expect,
            "{}",
            row.case
        );
    }
}
