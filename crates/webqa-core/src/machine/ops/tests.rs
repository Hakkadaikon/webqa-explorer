//! The table of operation kinds.

use super::*;
use crate::judge::op_name;
use crate::machine::testing::rules;

#[test]
fn offered_follows_table_order() {
    let rules = Rules {
        ops: vec![Op::Stuck, Op::Click, Op::Back],
        ..rules()
    };
    let names: Vec<&str> = offered(&rules).iter().map(|row| row.name).collect();
    assert_eq!(
        names,
        ["click", "back", "stuck"],
        "in the order of the table"
    );
    for row in &OPS {
        assert_eq!(
            row.name,
            op_name(row.op),
            "{:?} as the judge names it",
            row.op
        );
    }
}
