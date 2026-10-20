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

fn label(
    kind: KindId,
    path: Option<&str>,
    element: Option<(&str, &str)>,
    verdict: Verdict,
) -> Label {
    let Plant { path, element, .. } = plant("", kind, path, element);
    Label {
        kind,
        path,
        element,
        verdict,
    }
}

#[test]
fn a_label_rules_on_kind_path_and_element() {
    let l = label(
        KindId::Visibility,
        Some("/p"),
        Some(("link", "料金")),
        Verdict::Real,
    );
    let issues = [
        issue(KindId::Visibility, "/p", Some(("link", "料金"))),
        issue(KindId::Hesitant, "/p", Some(("link", "料金"))),
        issue(KindId::Visibility, "/q", Some(("link", "料金"))),
        issue(KindId::Visibility, "/p", Some(("link", "別"))),
        issue(KindId::Visibility, "/p", None),
    ];
    assert_eq!(
        issues.map(|f| rules(&l, &f)),
        [true, false, false, false, false]
    );
}

#[test]
fn the_first_label_that_fits_wins() {
    let issues = [issue(KindId::Hesitant, "/", None)];
    let real = label(KindId::Hesitant, None, None, Verdict::Real);
    let wrong = label(KindId::Hesitant, Some("/"), None, Verdict::False);
    let first_real = score(&[], &[real.clone(), wrong.clone()], &issues, None);
    let first_wrong = score(&[], &[wrong, real], &issues, None);
    assert_eq!(
        (
            first_real.real,
            first_real.wrong,
            first_wrong.real,
            first_wrong.wrong
        ),
        (1, 0, 0, 1)
    );
}

#[test]
fn rulings_do_not_count_an_issue_that_hit_a_plant() {
    let plants = [plant("a", KindId::Hesitant, Some("/"), None)];
    let labels = [
        label(KindId::Hesitant, None, None, Verdict::Real),
        label(KindId::Hesitant, None, None, Verdict::False),
    ];
    let s = score(
        &plants,
        &labels,
        &[issue(KindId::Hesitant, "/", None)],
        None,
    );
    assert_eq!(
        (s.hit_issues, s.real, s.wrong, s.labeled_precision()),
        (1, 0, 0, Some(1.0))
    );
}
