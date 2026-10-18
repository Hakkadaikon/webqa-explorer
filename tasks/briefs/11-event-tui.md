# brief: issue #11 イベントログと TUI(部品だけ)

共通の決まりは同じディレクトリの `common.md`。

## タスク

イベントログの型と、ログを追って表示する TUI の部品を書く。
設計は `docs/architecture.md` の「イベントログとTUI」。受け入れシナリオは無い。
`webqa watch` のサブコマンドとタイマーの購読(tears)の配線は、この issue では作らない(実行ループの issue で配線する)。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod event;` を足すだけ)、`crates/webqa-core/src/event.rs`(新規)
- `crates/webqa-tui/src/lib.rs`、`crates/webqa-tui/src/tail.rs`、`crates/webqa-tui/src/view.rs`、`crates/webqa-tui/tests/render.rs`、`crates/webqa-tui/tests/snapshots/`(新規)
- `crates/webqa-core/Cargo.toml`、`crates/webqa-tui/Cargo.toml`(依存は `.workspace = true` で)、`Cargo.lock`

## 決めてある設計

- `webqa-core::event`:
  - `Line { seq: u64, elapsed_ms: u64, step: Option<u64>, event: Event }`。JSON では `event` の中身を同じ階層に平たく出す(`#[serde(flatten)]`)。`step` は無ければ出さない。
  - `Event` は `#[serde(tag = "kind", rename_all = "snake_case")]` の enum。今は次の4つだけ:
    - `RunStarted { schema: u32, mode: Mode, started_at: Option<String> }`(`Mode` は `explore` / `scenario`)
    - `Decided { op: String, target: Option<String>, p: f64 }`
    - `Executed { changed: bool, accesses: u32 }`
    - `RunFinished { reason: String }`
  - 1行の JSON への変換 `to_json_line(&Line) -> String`(改行なし)と、読み取り `parse_line(&str) -> Result<Line, serde_json::Error>`。
- `webqa-tui::tail::Tail`: `feed(&mut self, chunk: &str) -> Vec<Line>`。前回の残りに `chunk` をつなぎ、改行で終わった行だけを読む。改行で終わっていない最後の部分は次回に回す。読めない行は飛ばし、`skipped()` で数を返す。
- `webqa-tui::view::State`: `fold(self, &Line) -> State` の純粋な関数。持つのは、モード、今のステップ番号、最後の判断(op と target)、画面が変わったステップの数、終了理由。`render(&State, &mut ratatui::Frame)` で描く。

## 失敗させるテスト

1. `webqa-core::event` の `event_lines_match_documented_jsonl`: architecture.md の JSONL の例の4行を `parse_line` で読み、`to_json_line` で書き戻すと、キーの順も含めて元の行と同じ文字列になる。`step` の無い行に `step` が出ない。
2. `webqa-tui::tail` の `partial_last_line_waits_for_newline`: `"{...}\n{..."` を渡すと1行だけ返り、続きの `"...}\n"` を渡すと2行目が返る。空の文字列を渡すと何も返らない。
3. `webqa-tui::tail` の `malformed_line_is_skipped_and_counted`: 読めない行を挟むと、その行は返らず `skipped()` が1になり、前後の行は返る。空行は数えずに飛ばす。
4. `webqa-tui::view` の `fold_tracks_step_and_finish`: 例の4行を順に畳むと、モードは explore、ステップは1、最後の判断は click と保存、変わったステップは1、終了理由は goal_achieved。
5. `webqa-tui/tests/render.rs` の `render_matches_snapshot`: 例の4行を畳んだ状態を、ratatui のテスト用の端末(60x10)に描き、insta のスナップショットと一致する。

## 受け入れ条件

`just check` が終了コード0。変えたファイルの mutation の生き残り0。
