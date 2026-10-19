# issue #57 webqa watch でイベントログを追って TUI を回す

文中の `ファイル:行` は、この brief を書いた時点の行を指す。

共通の決まりは `common.md`。進め方は skill `parallel-work`(全体のテストとカバレッジは PR の CI で確かめる)。

## 背景

#11 で、イベントログの行を読む `webqa_tui::tail::Tail`、状態を畳み込む `view::State::fold`、描く `view::render` を作った。
実行中のイベントログ(`<out>/runs/<実行ID>/events.jsonl`)を追いかけて描き直す本体と、`webqa watch` サブコマンドがまだ無い。

## 決めてある設計

- `webqa watch <実行のディレクトリ>` で起動する。CLI の実行とは別のプロセス(`docs/architecture.md:81`)。
- 一定の間隔(200ミリ秒)ごとに `events.jsonl` を、前回読んだ位置から先だけ読む。ファイルの変更通知は使わない(`docs/architecture.md:926-930`)。
- 待ちは注入した時計(`clock::RealClock` の `sleep_until`)で行う。`tokio::time` は使わない(clippy の禁止)。
- まだ `events.jsonl` が無ければ、空として読み、現れるのを待つ(実行より先に watch を起動できる)。ディレクトリそのものが無ければ、終了コード2。
- ファイルが前回より短くなったら、読み直さずに待つ。`shortcut: 書き直されたログは追わない、ログを書き直す機能が入ったら読み直す` のコメントを付ける。
- `run_finished` を読んだら、読むのをやめ、最後の画面を残して Ctrl-C を待つ。
- 実行中の画面からの操作は終了だけ(`docs/architecture.md:934`)。終了は Ctrl-C(`interrupt::CtrlC`)で受け、終了コード0。
- 端末は raw モードにしない。Ctrl-C がそのまま SIGINT で届き、キーを読む処理が要らない。
- 描くのは ratatui の `CrosstermBackend`(標準出力)。端末の大きさは `crossterm::terminal::size()` で毎回読み、読めなければ 80×24 で描く(標準出力がパイプのときのため)。
- 状態の更新は純粋な関数のまま(`State::fold`)。描画のゴールデンは既存の `tests/render.rs` が受け持つ。

## 形

- `webqa-tui` に `follow` モジュールを足す。

```rust
/// What the viewer has read of one run.
#[derive(Debug, Default)]
pub struct Follow {
    tail: Tail,
    state: State,
    finished: bool,
}

impl Follow {
    /// Folds the lines `chunk` completes; whether the state changed.
    pub fn read(&mut self, chunk: &[u8]) -> bool;
    /// The state to draw.
    pub fn state(&self) -> &State;
    /// Whether `run_finished` was read.
    pub fn finished(&self) -> bool;
}

/// The area to draw in: the terminal's size, or 80×24 when unknown.
pub fn area(size: Option<(u16, u16)>) -> Rect;
```

- `webqa` に `watch.rs` を足す。`events.jsonl` の新しいバイトを読む関数と、読む・描く・待つを Ctrl-C と競わせる繰り返し。

```rust
/// The bytes of `path` after `offset`, and the new offset. A missing file
/// reads as empty; a shorter file reads nothing.
pub fn appended(path: &Path, offset: u64) -> io::Result<(Vec<u8>, u64)>;

/// Follows the run in `dir` until interrupted; the exit code.
pub async fn watch(dir: &Path, clock: &mut dyn Clock, interrupt: &mut dyn Interrupt, out: impl Write) -> u8;
```

- `cli.rs` の `Command` に `Watch(WatchArgs { dir: PathBuf })` を足し、`drive` で `Watch` だけ `watch::watch` へ渡す(`cli/app.rs` は触らない)。

## 依存

- `webqa-tui/Cargo.toml` に `crossterm.workspace = true` を足す(端末の大きさを読む)。ワークスペースの `Cargo.toml` には `crossterm = "0.29.0"` がもう書いてあり、ratatui 0.30.2 の既定の機能が同じ 0.29.0 を使うので、`Cargo.lock` に新しい版は入らない見込み。
- ワークスペースに書いてある `tears = "0.11.1"` は使わない(下の「親に決めてほしいこと」1)。
- clippy の `allowed-duplicate-crates` は、`cargo tree -d` で新しい重複が出たときだけ足し、理由を1行書く(利用者が承認済み)。

## 触ってよいファイル

- `crates/webqa-tui/src/follow.rs`(新規)、`crates/webqa-tui/src/lib.rs`(`pub mod follow;` の1行)、`crates/webqa-tui/Cargo.toml`
- `crates/webqa/src/watch.rs`、`watch/tests.rs`(新規)、`crates/webqa/src/lib.rs`(`pub mod watch;` の1行)
- `crates/webqa/src/cli.rs`、`cli/tests.rs`
- `crates/webqa/tests/watch.rs`(新規の実行ファイルのテスト。`tests/cli.rs` とは別の実行ファイルにする)
- `.config/clippy/clippy.toml`(重複が出たときだけ)
- `docs/architecture.md` の「イベントログとTUI」(tears のタイマーでなく注入した時計で読むこと、raw モードにしないこと、終了は Ctrl-C)

触らないもの: `crates/webqa/src/cli/app.rs`、`cli/start.rs`、`run/`、`store/`、`replay.rs`、`tests/cli.rs`、`tests/cli/`(#61 が触る)。`crates/webqa-tui/src/view.rs`、`tests/render.rs`(#27 がスナップショットの名前を変えている)。

## テスト(名前と期待値)

1. `follow_folds_appended_lines`(`webqa-tui` の単体): `EXAMPLE` の4行を2回に分けて `read` すると、`state()` は4行を順に `fold` した `State` と等しい。2回とも `true`。
2. `follow_unchanged_without_complete_line`(同): 改行の無い断片と空のチャンクでは `false` で、`state()` は変わらない。
3. `follow_knows_the_run_finished`(同): `run_finished` の行を読むまで `finished()` は `false`、読んだあと `true`。
4. `area_falls_back_when_size_is_unknown`(同): `area(None) == Rect::new(0, 0, 80, 24)`、`area(Some((120, 40))) == Rect::new(0, 0, 120, 40)`。
5. `appended_reads_after_offset`(`webqa::watch` の単体): 無いファイルは `(vec![], 0)`。10バイト書くと `(その10バイト, 10)`。さらに5バイト足して offset 10 から読むと `(その5バイト, 15)`。offset 15 のままファイルを3バイトにすると `(vec![], 15)`。
6. `watch_draws_and_waits_after_finish`(同。偽の時計、偽の中断、`Vec<u8>` の出力): 実行のディレクトリに2行を書いておく。中断は3回目の待ちで起きる。終了コード0、出力に "保存" がある。`run_finished` を書いた場面では、それ以降はファイルを読まない(読んだ回数が `run_finished` を読んだ回で止まる)。
7. `missing_dir_exits_two`(同): 無いディレクトリでは、すぐ終了コード2。
8. `watch_follows_a_live_log`(`tests/watch.rs`、実行ファイル):
   空の実行のディレクトリで `webqa watch <dir>` を、標準出力をパイプにし `process_group(0)` で起動する。
   `EXAMPLE` の1、2行目を書き足すと、10秒以内に標準出力に "保存" が出る。
   `run_finished` を書き足したあと `kill -INT -- -<pid>` を送ると、終了コード0。
9. `arguments_are_read`、`bad_arguments_are_errors`(`cli/tests.rs` の既存): `watch <dir>` が `Command::Watch` に読めるように直し、`watch` を誤りとして確かめていた行を、引数の無い `watch` が誤りになる行に変える。

## 仕様

feature に `webqa watch` の Scenario は無いので変えない。`docs/architecture.md` の「tears のタイマーの購読で読む」を、注入した時計で読む形に直す(設計の文書で、feature と TLA+ は変えない)。

## 親に決めてほしいこと

1. **tears を使うか**。案A(推す): 使わない。読む・描く・待つは数行の繰り返しで足り、raw モードを使わないので実行ファイルのテストで端末の偽物(pty)が要らない。`Cargo.lock` に新しい依存も入らない。案B: 文書どおり tears のタイマーで回す。tears とその依存が `Cargo.lock` に入り、raw モードの端末を確かめるために pty の dev-dependency が要る。
2. **終了の操作**。案A(推す): Ctrl-C だけ。raw モードにしないのでキーは読まない(打ったキーが画面に出ることがあるが、操作は無い)。案B: `q` でも終われるようにする。raw モードとキーを読む処理が要り、案Aの利点が消える。
3. **`run_finished` の後**。案A(推す): 読むのをやめ、最後の画面のまま Ctrl-C を待つ(issue の範囲どおり)。案B: 自動で終わる。
