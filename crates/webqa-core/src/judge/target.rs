//! Choosing the element to operate on.

/// The first of `ranked` that is not in `avoid`, or the first of `ranked`
/// when every one is. `None` when `ranked` is empty.
///
/// `ranked` is [`super::Verdict::targets`]. The caller passes as `avoid` the
/// elements operated since the screen last changed, so the same number
/// points at the same element.
#[must_use]
pub fn pick_target(ranked: &[usize], avoid: &[usize]) -> Option<usize> {
    ranked
        .iter()
        .find(|n| !avoid.contains(n))
        .or_else(|| ranked.first())
        .copied()
}

/// The tests sit here, not in `target/tests.rs`: `.gitignore` ignores every
/// `target/` directory.
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::judge::ask::tests::ASK8;
    use crate::judge::{Answers, QuestionId, Usage, read_answers};

    #[test]
    fn same_unchanged_target_switches_to_next() {
        let cases: [(&[usize], &[usize]); 9] = [
            (&[1, 2], &[1]),
            (&[1, 2], &[]),
            (&[1, 2], &[3]),
            (&[2, 1, 3], &[2]),
            (&[2, 1, 3], &[2, 1]),
            (&[2, 1, 3], &[3, 2, 1]),
            (&[1], &[1]),
            (&[], &[]),
            (&[], &[1]),
        ];
        let got = cases.map(|(ranked, avoid)| pick_target(ranked, avoid));
        let want = [
            Some(2),
            Some(1),
            Some(1),
            Some(1),
            Some(3),
            Some(2),
            Some(1),
            None,
            None,
        ];
        assert_eq!(got, want, "{cases:?}");
        switches_from_the_judges_pick();
    }

    /// Element 1 is "保存" and 2 is "ヘルプ". The screen did not change after
    /// clicking 1, so the judge's top pick gives way to 2.
    fn switches_from_the_judges_pick() {
        let rows = [
            ("operation", "click", 0.9),
            ("error_shown", "true", 0.1),
            ("goal_reached", "true", 0.0),
        ];
        let mut table: BTreeMap<_, _> = rows
            .map(|(id, o, p)| (QuestionId::new(id), vec![(o.to_owned(), p)]))
            .into();
        table.insert(
            QuestionId::target(),
            vec![("1".to_owned(), 0.8), ("2".to_owned(), 0.6)],
        );
        let answers = Answers {
            table,
            usage: Usage::default(),
        };
        let targets = read_answers(&answers, &ASK8).unwrap().targets;
        assert_eq!(pick_target(&targets, &[1]), Some(2), "the next element");
    }
}
