//! The answer key and the labels the repository ships read, and what they
//! point at exists in the planted site.

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use webqa_eval::truth::{Element, read_labels, read_truth};

type Outcome = Result<(), Box<dyn Error>>;

fn read(relative: &str) -> Result<String, Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()).into())
}

#[derive(Deserialize)]
struct Purpose {
    screens: Vec<Screen>,
}

#[derive(Deserialize)]
struct Screen {
    url: String,
}

/// The page of a path in the site, if it is there.
fn page(path: &str) -> Option<String> {
    let file = if path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };
    read(&format!("eval/site/{file}")).ok()
}

/// The problems of a pointer: a path that is neither a screen of the
/// purpose nor a file of the site, and an element whose name the page lacks.
fn problems(
    screens: &[Screen],
    who: &str,
    path: Option<&str>,
    element: Option<&Element>,
) -> Vec<String> {
    let Some(path) = path else {
        return vec![];
    };
    let page = page(path);
    let screen = screens.iter().any(|s| s.url == path);
    let lost = (!screen && page.is_none())
        .then(|| format!("{who}: {path} is not a screen or a file of eval/site"));
    lost.into_iter()
        .chain(unnamed(who, path, element, page.as_deref()))
        .collect()
}

fn unnamed(who: &str, path: &str, element: Option<&Element>, page: Option<&str>) -> Option<String> {
    let (element, page) = element.zip(page)?;
    (!page.contains(&element.name)).then(|| format!("{who}: {} is not in {path}", element.name))
}

fn screens() -> Result<Vec<Screen>, Box<dyn Error>> {
    let purpose: Purpose = serde_saphyr::from_str(&read("eval/purpose.yaml")?)?;
    Ok(purpose.screens)
}

#[test]
fn the_answer_key_points_at_the_site() -> Outcome {
    let screens = screens()?;
    let plants = read_truth(&read("eval/truth.yaml")?)?;
    let found: Vec<String> = plants
        .iter()
        .flat_map(|p| problems(&screens, &p.id, p.path.as_deref(), p.element.as_ref()))
        .collect();
    assert_eq!(found, Vec::<String>::new());
    Ok(())
}

#[test]
fn the_labels_point_at_the_site() -> Outcome {
    let screens = screens()?;
    let labels = read_labels(&read("eval/labels.yaml")?)?;
    let found: Vec<String> = labels
        .iter()
        .flat_map(|l| {
            let who = format!("{:?}", l.kind);
            problems(&screens, &who, l.path.as_deref(), l.element.as_ref())
        })
        .collect();
    assert_eq!(found, Vec::<String>::new());
    Ok(())
}
