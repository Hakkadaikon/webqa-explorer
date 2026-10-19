# issue #61 replay が出力先の記憶を読まず、元の実行と同じ操作列を返す

共通の決まりは `common.md`。進め方は skill `parallel-work`(全体のテストとカバレッジは PR の CI で確かめる)。

## 背景

`webqa replay` を元の実行と同じ `--out`(既定は `.webqa`)で回すと、元の実行が書いた探索の記憶が残っている。
候補の絞り込みが変わり、操作の列が元とずれる(14g のレビューで実測)。
feature の「その出力から同じ操作列を再実行できる」(`tasks/loopeng/WebExploratoryQA.feature:749`、T-042)に反する。

## 決めてある設計(親の決定、2026-10-20)

- 元の実行は、**開始時の記憶**を実行結果 `report.json` の項目 `start_memory` に残す。
  - 中身は `memory::load` が返した記憶を `Memory::to_json(origin)` にし、`redactor.json` を通したもの(`memory::save` と同じ形)。
  - 記憶の読み込みで壊れたファイルを脇へ退けた場合は、空の記憶を書く。
- `webqa replay` は `start_memory` から始め、出力先の記憶(`memory/<origin>.json`)を**読まず、書きもしない**。
- `start_memory` の無い実行結果(この issue より前のもの)は、空の記憶から始める。形式の版(`schema`)は1のまま(項目を足すだけ)。
- `start_memory` が記憶として読めなければ、設定エラー(終了コード2)。文は `ReplayError` と同じ決まった文にし、記憶の中身を出さない。
- 再実行の実行結果の `start_memory` は、渡された記憶をそのまま書く。

## 形

- `run::session::Plan` に、記憶の出どころを持たせる。

```rust
/// Where the memory of a run comes from.
pub enum Start {
    /// The stored memory of the origin, written back at the end.
    Stored,
    /// The memory a replay starts from; the stored one is left alone.
    Given(Memory),
}
```

- `Plan { origin, meta, start: Start }`。`begin` は `Stored` なら `memory::load`、`Given` なら渡された記憶を使う。`finish` は `Given` なら `memory::save` を呼ばない。
- 開始時の記憶は `begin` で JSON にして、`finish` の実行結果の組み立てに渡す。`store::report::build` に `start_memory: &Value`(伏せ字を通したもの)を足す。
- `replay::Replay::from_report` は、`start_memory` を `Option<serde_json::Value>` で読み、`Replay` の外に返す(例: `(Mode, Self, Option<Value>)`)。記憶への変換は、設定から origin が分かったあと `cli/app.rs` の `configure` で `Memory::from_json` を呼んで行う。

## 触ってよいファイル

- `crates/webqa/src/run/session.rs`、`run/session/tests*.rs`
- `crates/webqa/src/store/report.rs`、`store/report/sections.rs`、`store/report/tests*.rs`、スナップショット `store/report/snapshots/`(`start_memory` の行が増える)
- `crates/webqa/src/replay.rs`、`replay/tests*.rs`
- `crates/webqa/src/cli/app.rs`、`cli/start.rs`
- `crates/webqa/tests/cli/modes.rs`(実行ファイルのテスト)
- `docs/architecture.md` の「出力」(実行結果に開始時の記憶を残すこと、replay は出力先の記憶を読まないこと)
- `tasks/test-design/WebExploratoryQA.md` の T-042 の行(テスト名を足す)

触らないもの: `crates/webqa/src/cli.rs`、`cli/tests.rs`、`crates/webqa/tests/cli.rs`、`crates/webqa-tui/`(#57 が触る)。

## テスト(名前と期待値)

1. `replay_starts_from_given_memory_and_keeps_stored`(`run/session/tests`):
   保存先の偽物に、"保存" を操作済みにした記憶を置く。
   `Start::Given` に空の記憶を渡して `conduct` すると、最初の問いの対象の選択肢に "保存" がある。
   保存先の記憶は書かれない(`memories` が空、記憶の読み込みも0回)。
2. `stored_start_is_written_back`(同): `Start::Stored` では今までどおり読み、終わりに書く(書き込み1回)。
3. `report_keeps_start_memory`(`store/report/tests`): `build` の結果の `start_memory` が、渡した値と等しい。スナップショットにも行が入る。
4. `start_memory_is_redacted`(同): 記憶の要素名に伏せる値を入れると、`start_memory` には伏せる対象の名前だけが入り、値は出ない。
5. `from_report_reads_start_memory`(`replay/tests`): `start_memory` のある実行結果は `Some(値)`、無い実行結果は `None`。
6. `broken_start_memory_is_config_error`(`cli/start.rs` か `cli/app.rs` の単体): 記憶として読めない `start_memory` は `Stop` の終了コード2、文は決まった文で記憶の中身を含まない。
7. `replay_with_same_out_repeats_operations`(`tests/cli/modes.rs`):
   同じ `--out` で `run` のあとに `replay` する。
   両方のイベントログの `decided` の (op, target) の列が等しく、出力先の記憶ファイルは `replay` の前後で同じバイト列。
   今の「別の `--out` で replay する」回避とコメントは消す。

## 仕様

feature は変えない。T-042 の「同じ操作列を再実行できる」をこの直しで満たす。TLA+ は記憶の出どころを持たないので変えない。

## 親に決めてほしいこと

1. **開始時の記憶の置き場**。案A(推す): `report.json` の項目にする。replay が読むファイルは1つのままで、`Store` に新しいメソッドが要らない。案B: 実行のディレクトリに `memory.json` を別に置く。`Store` に書き込みのメソッドが1つ増える。
2. **`start_memory` の無い古い実行結果**。案A(推す): 空の記憶から始める(版は1のまま)。案B: 形式の版を2に上げ、古い実行結果は再実行できない設定エラーにする。
