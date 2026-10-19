//! Tests of the edges of the report: the input key, the usage and the cost.

use serde_json::json;
use webqa_core::machine::Cause;

use super::*;

#[test]
fn input_key_only_for_inputs_named_after_a_key() {
    let keys = vec!["メールアドレス".to_owned()];
    let named = Some(textbox("メールアドレス"));
    let cases = [
        (
            act(1, "input", named.clone(), Cause::Judge, [0, 0]),
            &keys,
            Some("メールアドレス"),
        ),
        (
            act(1, "click", named.clone(), Cause::Judge, [0, 0]),
            &keys,
            None,
        ),
        (act(1, "input", None, Cause::Judge, [0, 0]), &keys, None),
        (
            act(1, "input", Some(textbox("氏名")), Cause::Judge, [0, 0]),
            &keys,
            None,
        ),
        (
            act(1, "input", named, Cause::Judge, [0, 0]),
            &Vec::new(),
            None,
        ),
    ];
    for (act, inputs, expected) in cases {
        assert_eq!(sections::input_key(&act, inputs), expected, "{act:?}");
    }
}

#[test]
fn usage_counts_every_call_and_sums_the_answered_ones() {
    let price = Some(Price {
        input_per_1k: 1.0,
        output_per_1k: 1.0,
    });
    let empty = Record {
        calls: Vec::new(),
        acts: Vec::new(),
        found: Vec::new(),
    };
    let none = json!({"calls": 0, "input_tokens": 0, "output_tokens": 0, "cost": 0.0});
    assert_eq!(sections::usage_section(&empty, price), none, "no calls");
    let unpriced = sections::usage_section(&record(), None);
    let tokens = json!({"calls": 2, "input_tokens": 120, "output_tokens": 3, "cost": null});
    assert_eq!(unpriced, tokens, "a failed call counts, its tokens do not");
}

#[test]
fn cost_of_more_tokens_than_u32_is_not_capped() {
    let price = Some(Price {
        input_per_1k: 1.0,
        output_per_1k: 0.0,
    });
    let at = |tokens| {
        cost(
            Usage {
                input_tokens: tokens,
                output_tokens: 0,
            },
            price,
        )
    };
    let max = u64::from(u32::MAX);
    let got = [at(max), at(max + 1), at(max * 2)];
    let expected = [4_294_967.295, 4_294_967.296, 8_589_934.590];
    for (got, expected) in got.into_iter().zip(expected) {
        let got = got.unwrap();
        assert!((got - expected).abs() < 1e-6, "{got} for {expected}");
    }
}

#[test]
fn issue_without_screenshot_leaves_the_field_out() {
    let mut record = record();
    let mut unshot = record.found[0].clone();
    unshot.step = 2;
    unshot.shot = None;
    record.found.push(unshot);
    let report = build(
        &record,
        ExitReason::BudgetExceeded,
        &meta(),
        &start_memory(),
        &Redactor::default(),
    )
    .into_inner();
    let fields = |i: usize| {
        report["issues"][i]
            .as_object()
            .unwrap()
            .contains_key("screenshot")
    };
    assert_eq!(
        (fields(0), fields(1), &report["issues"][0]["screenshot"]),
        (
            true,
            false,
            &json!("screenshots/ISSUE_error_display_step1.png")
        ),
        "only the issue with a screenshot names it"
    );
}
