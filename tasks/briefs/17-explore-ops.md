# brief: issue #17 探索モードで要素を操作できない設定を拒む

共通の決まりは同じディレクトリの `common.md`。

## タスク

探索モードで、使う操作種別(`ops`)に click も input も無い設定を、設定エラーにする中核の関数を書く。
実行のモードは設定でなく CLI のサブコマンドで決まるので、YAML の検証(`validate`)には入れず、検証済みの設定とモードを受け取る関数にする。
CLI が起動時にこの関数を呼ぶ配線と、実行ファイルのテスト `explore_without_click_or_input_exits_two` は #14 の受け持ちで、範囲外。
設計は `docs/architecture.md` の「設定」。受け入れシナリオは `tasks/loopeng/WebExploratoryQA.feature` の「探索モードで使う操作種別に click も input も無ければ設定エラーにする」(台帳 `tasks/test-design/WebExploratoryQA.md` の T-112)。

## 触ってよいファイル

- `crates/webqa-core/src/config.rs`(`ConfigError` に腕を1つ足すことと、`mod mode;` と `pub use` を足すことだけ)
- `crates/webqa-core/src/config/mode.rs`(新規)と、テストを分けるなら `crates/webqa-core/src/config/mode/tests.rs`

`config/validate/check.rs` は issue #4 が並行して触るので触らない。`config/raw.rs`、`event.rs` も触らない。

## 決めてある設計

- モードの型は既存の `crate::event::Mode`(`Explore`、`Scenario`)を使う。新しい型を作らない。
- `ConfigError` に `ExploreNeedsElementOp` を足す。文は `探索モードでは ops に click か input を1つ以上書く`。値を持たない。
- `pub fn check_mode(config: &Config, mode: Mode) -> Result<(), ConfigError>`:
  - `Mode::Scenario` なら、`ops` の中身によらず `Ok(())`。
  - `Mode::Explore` なら、`config.ops` に `Op::Click` か `Op::Input` が1つ以上あれば `Ok(())`、無ければ `Err(ConfigError::ExploreNeedsElementOp)`。
  - `ops` が空の設定は `validate` が `NoOps` で先に拒むので、ここには来ない前提。ただし空を渡されたら、探索モードでは `ExploreNeedsElementOp`、決め打ちシナリオでは `Ok(())` を返す(上の規則のまま)。
- 「click か input があるか」は名前の付いた述語(例: `fn operates_elements(op: &Op) -> bool`)にし、`iter().any` で引く。腕2本までの決まりを守る。
- `config.rs` で `pub use mode::check_mode;` として、`webqa_core::config::check_mode` で呼べるようにする。

## 失敗させるテスト(単体)

テストの設定は、既存のテスト(`config/validate.rs` の `load` など)と同じやり方で YAML から作る。`ConfigError` は `PartialEq` を持たないので、既存のテストと同じく文(`to_string()`)か `matches!` でない方法で、種類まで確かめる(`assert_eq!` で `Display` の文を比べる)。

1. `explore_mode_needs_click_or_input`: 次の表を回す。
   - `ops: [done]` と `Explore` → `ExploreNeedsElementOp` の文。同じ設定で `Scenario` → `Ok(())`。
   - `ops: [scroll_down, back]` と `Explore` → `ExploreNeedsElementOp`。`Scenario` → `Ok(())`。
   - `ops: [click]`、`ops: [input]`、`ops: [done, input]`、`ops: [scroll_up, click, wait]` は、`Explore` でも `Scenario` でも `Ok(())`。
2. `explore_check_with_empty_ops`: `ops` が空の `Config` を直接組み立てられない場合は、`Config` の `ops` を空にした値を作って渡す(作れなければこのテストは省き、報告に書く)。`Explore` で `ExploreNeedsElementOp`、`Scenario` で `Ok(())`。
3. 既存の `ConfigError` の文のテストがあれば、`ExploreNeedsElementOp` の文をその表に足す。

## 受け入れ条件

`just check` が終了コード0。`config/mode.rs` の mutation の生き残り0。`check.rs`、`raw.rs`、`event.rs` に差分が無い。
