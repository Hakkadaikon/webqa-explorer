//! Steps of this Rule of the feature, and the scenarios that have them.
//! The run before is a run of the scene, whose ledger the next run gets.

#[path = "rerun_course.rs"]
pub mod course;
#[path = "rerun_given.rs"]
mod given;
#[path = "rerun_then.rs"]
mod then;
#[path = "rerun_when.rs"]
mod when;

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "前回も今回も出た課題は、残っているとして示す",
    "今回だけ出た課題は、新しく出たとして示す",
    "その画面に届いた試行の数が前回以上で、1回も起きなければ、解消したとして示す",
    "その画面に届いた試行の数が前回より1少なければ、1回も起きなくても解消したとせず、確かめられなかったとして示す",
    "前回課題が出た画面に届かなければ、確かめられなかったとして示す",
    "前回選んだ課題が残っていれば、選択を引き継ぐ",
    "新しく出た課題は、選ばれていない状態で台帳に出す",
    "前回の台帳を渡さなければ、すべての課題を新しく出たとして示す",
    "前回の台帳が決まった書式で読めなければ、照らし合わせずに設定エラーにする",
    "前回の台帳のペルソナか目的が今回と違えば、照らし合わせずに設定エラーにする",
    "目的を推測させた実行どうしは、推測した文が違っても、開始の URL が同じなら照らし合わせる",
];
