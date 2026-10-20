//! Steps of this Rule of the feature, and the scenarios that have them.
//!
//! A trial is planned by `plan::course`: the judge's replies, in the order
//! the trials run, and the pages the browser of the trial shows. The steps
//! are in the files below, by what they are about.

use crate::scene::found;

#[path = "trials_basic.rs"]
mod basic;
#[path = "trials_caps.rs"]
mod caps;
#[path = "trials_command.rs"]
mod command;
#[path = "trials_cut.rs"]
mod cut;
#[path = "trials_plan.rs"]
mod plan;
#[path = "trials_reset.rs"]
mod reset;
#[path = "trials_log.rs"]
mod seen;
#[path = "trials_time.rs"]
mod time;
#[path = "trials_window.rs"]
mod window;

use plan::{
    End, HOUR, WINDOW_MS, course, fill, first, minutes, said, second_and_rest, step_name, think,
};
use seen::{
    before_wait, elapsed_of, ending_of, has, issues_of, kind_of, launches, openings, started_at,
    within,
};

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "試行ごとにブラウザを新しく開き、認証状態を入れ直す",
    "初期化のコマンドを設定に書けば、各試行の前に実行する",
    "初期化のコマンドを書かなければ、試行の前に何も実行しない",
    "初期化のコマンドが失敗したら、その試行をせずに実行を止めて終了コード3で終わる",
    "初期化のコマンドはシェルを通さず、設定に書いた引数のまま実行する",
    "初期化のコマンドに、判断モデルの API キーを渡さない",
    "初期化のコマンドの出力をログに残すときも、設定で指定した機密情報を伏せる",
    "試行ごとに探索の記憶を空で始め、保存しない",
    "1つの試行が判断予算切れで終わっても、次の試行は判断予算を使っていない状態で始める",
    "試行の最初のページの表示も、実行全体の時間窓のアクセスに1回数える",
    "初期化のコマンドが対象サーバーへ送るアクセスは、実行全体の時間窓に数えない",
    "残りから最初の表示の1回を引いた数が上限の半分以上なら、その数を中核の時間窓の上限にして次の試行を始める",
    "残りから最初の表示の1回を引いた数が上限の半分を下回るなら、窓が明けるまで待ち、上限から1を引いた数で次の試行を始める",
    "次の試行を始めるかは、初期化のコマンドの前に判定する",
    "試行の中核の時間窓は、実行の開始からの経過時間で、実行全体の時間窓と同じ時刻に区切る",
    "試行に渡した中核の時間窓の上限は、その試行の間は後の窓でも変えない",
    "上限を小さくして始めた試行は、その回数に達したら窓が明けるまで問い合わせも操作もしない",
    "窓を待ってから始めた試行も、中核の時間窓の操作回数を0から数える",
    "試行の間で窓を待っている間に中断したら、窓を待たずに、終えた試行までの結果で台帳を書いて終わる",
    "試行の間で窓を待っている間に制限時間に達したら、窓を待たずに時間切れで終わる",
    "制限時間は実行の開始から測り、試行をまたいで引き継ぐ",
    "初期化のコマンドの最中に期限が来たら、終わるのを待ってから、試行を始めずに打ち切る",
    "初期化のコマンドの最中に期限が来て、そのあと待ち時間も超えたら、コマンドを止めて打ち切りとして時間切れで終える",
    "打ち切りの試行で見た手がかりは、出現回数にも課題にも入れない",
    "ブラウザの起動や最初の表示の最中に期限が来たら、待たずに打ち切る",
    "期限や中断で打ち切った試行は、試行の回数にも目的に届かない試行にも数えない",
    "終えた試行が0回なら、課題を出さず、そのことを台帳に書く",
    "実行中に中断したら、残りの試行をせず、終えた試行までの結果で台帳を書いて終わる",
];
