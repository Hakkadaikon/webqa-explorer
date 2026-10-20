use super::*;
use crate::truth::{Element, Label, Plant, Verdict};

fn plant(id: &str, kind: KindId, path: Option<&str>, element: Option<(&str, &str)>) -> Plant {
    Plant {
        id: id.into(),
        kind,
        path: path.map(Into::into),
        element: element.map(|(role, name)| Element {
            role: role.into(),
            name: name.into(),
        }),
        note: String::new(),
    }
}

fn issue(kind: KindId, path: &str, element: Option<(&str, &str)>) -> Found {
    Found {
        number: "I-00000000".into(),
        kind,
        path: path.into(),
        element: element.map(|(role, name)| Element {
            role: role.into(),
            name: name.into(),
        }),
    }
}

#[test]
fn a_plant_hits_on_kind_path_and_element() {
    let p = plant("a", KindId::Visibility, Some("/p"), Some(("link", "料金")));
    let issues = [
        issue(KindId::Visibility, "/p", Some(("link", "料金"))),
        issue(KindId::Hesitant, "/p", Some(("link", "料金"))),
        issue(KindId::Visibility, "/q", Some(("link", "料金"))),
        issue(KindId::Visibility, "/p", Some(("link", "別"))),
        issue(KindId::Visibility, "/p", None),
    ];
    assert_eq!(
        issues.map(|f| hits(&p, &f)),
        [true, false, false, false, false]
    );
    let any = plant("b", KindId::Unreached, None, None);
    assert!(hits(&any, &issue(KindId::Unreached, "/anywhere", None)));
}

fn scene() -> Score {
    let plants = [
        plant("a", KindId::Visibility, Some("/p"), Some(("link", "料金"))),
        plant("b", KindId::Unreached, None, None),
    ];
    let labels = [Label {
        kind: KindId::Hesitant,
        path: Some("/".into()),
        element: None,
        verdict: Verdict::Real,
    }];
    let found = [
        issue(KindId::Visibility, "/p", Some(("link", "料金"))),
        issue(KindId::Visibility, "/p", Some(("link", "料金"))),
        issue(KindId::Hesitant, "/", None),
        issue(KindId::Visibility, "/q", Some(("link", "x"))),
    ];
    score(&plants, &labels, &found, None)
}

#[test]
fn counts_hits_misses_labels_and_unlabeled() {
    let s = scene();
    assert_eq!(
        (s.plants, s.found_plants, s.missed_plants),
        (2, vec!["a".to_owned()], vec!["b".to_owned()])
    );
    assert_eq!(
        (s.issues, s.hit_issues, s.real, s.wrong, s.unlabeled),
        (
            4,
            2,
            1,
            0,
            vec![issue(KindId::Visibility, "/q", Some(("link", "x")))]
        )
    );
}

#[test]
fn ratios_follow_the_counts() {
    let s = scene();
    assert_eq!(
        (s.recall(), s.strict_precision(), s.labeled_precision()),
        (Some(0.5), Some(0.5), Some(1.0))
    );
}

#[test]
fn a_false_label_counts_as_wrong() {
    let labels = [Label {
        kind: KindId::Hesitant,
        path: None,
        element: None,
        verdict: Verdict::False,
    }];
    let s = score(&[], &labels, &[issue(KindId::Hesitant, "/", None)], None);
    assert_eq!((s.wrong, s.real, s.labeled_precision()), (1, 0, Some(0.0)));
}

#[test]
fn a_kind_filter_counts_only_that_kind() {
    let plants = [
        plant("a", KindId::Visibility, None, None),
        plant("b", KindId::Unreached, None, None),
    ];
    let found = [issue(KindId::Visibility, "/", None)];
    let s = score(&plants, &[], &found, Some(KindId::Unreached));
    assert_eq!(
        (s.plants, s.issues, s.recall(), s.strict_precision()),
        (1, 0, Some(0.0), None)
    );
}

#[test]
fn nothing_to_count_gives_none() {
    let s = score(&[], &[], &[], None);
    assert_eq!(
        (s.recall(), s.strict_precision(), s.labeled_precision()),
        (None, None, None)
    );
}
