# brief: issue #33a インタビューのモードの受け入れの土台

共通の決まりは同じディレクトリの `common.md`。分け方は `33-plan.md`。並列に回す手順は skill `parallel-work`。

**#30(試行の外のループ)と #31(`webqa interview`、`webqa check-docs`)が入ってから始める。**
始める前に main の実物を読み、この brief の型やシグネチャと違えば NEEDS_CONTEXT で返す。

## 目的

インタビューのモードの受け入れシナリオ131本を、2段目の5本(33b〜33f)が並列に配線できる土台を作る。
この単位では、シナリオそのものは配線しない(Background と共通の手順だけ書く)。

## 触ってよいファイル

- `crates/webqa/tests/interview_acceptance.rs`(新規。実行ファイルの本体)
- `crates/webqa/tests/features/interview_mode.feature`(新規。`tasks/loopeng/InterviewMode.feature` の写し)
- `crates/webqa/tests/support/interview/`(新規。世界、場面、偽物の判断モデルとブラウザの組み立て、台帳の読み取り)
- `crates/webqa/tests/steps/interview/common.rs`(新規。Background と共通の手順)
- `crates/webqa/tests/steps/interview/` の Rule ごとの空のファイル(新規。`READY` だけを置く。下の表)
- `crates/webqa/tests/fixtures/bridge/`(新規。場面の橋渡しの台本)と `crates/webqa/tests/fixtures/reset/runs.sh`(新規。回ごとに場面の時計を進め、決めた回から終わらない初期化の台本)
- `crates/webqa/Cargo.toml`(`[[test]]` に `interview_acceptance` を `harness = false` で足す。cucumber の `main` を走らせるため)

触らないもの: `crates/webqa/tests/acceptance.rs`、`crates/webqa/tests/support/` と `steps/` の既存のファイル(読み込むのはよい)、`crates/` の下の製品のコード。製品のコードに足りない口があれば NEEDS_CONTEXT で返す。

## 決めてある設計

### 実行ファイル

- `tests/interview_acceptance.rs` は、`acceptance.rs` と同じ形にする。`#[path]` で support と steps を読み込み、`main` で土台の確かめを回してから cucumber を回す。
- 偽物の部品は、既存の `support/fakes.rs`、`support/page.rs`、`support/stuck.rs`、`support/memory_store.rs` を `#[path]` で使い回す。使わない関数が `dead_code` になる場合は、`acceptance.rs` の `check_fakes` と同じく、土台の確かめで一度ずつ使う。
- 走らせるシナリオは、Rule ごとのファイルの `READY` に名前のあるものだけにする。名前の数え方は、`Scenario:` と `Scenario Outline:` の両方を数える(`acceptance.rs` の `scenarios_named` と同じ)。
- `fail_on_skipped` を付ける。

### 世界(`InterviewWorld`)

1回のシナリオで、次を持つ。

- 場面: 許可ドメイン、ペルソナの文書、目的の設定(書いた目的か、推測させるか)、インタビューの設定(試行の回数、迷いの閾値ほか)、伏せる対象、判断モデルの答えの列、試行ごとのページの列、初期化のコマンドの台本。
- 実行の結果: `webqa::interview::trials::interview` の `Ending`、保存先に書かれた台帳(`write_ledger` の本文)、イベントの行、実行結果、判断モデルへの問いの記録、終了コード。

実行は、手順が最初に結果を求めたときに1回だけ回す(`acceptance.rs` の `world.run()` と同じ形)。

### 偽物

- 判断モデル: 問いの種類(真偽、選択、自由な文)と画像の有無を記録し、場面に積んだ答えを順に返す。答えを使い切ったら、`tests/support/stuck.rs` の形で、止まらずに失敗する待ちにする。
- 起動(`Launch`): 試行ごとに、場面のページの列から偽物のブラウザを作って返す。開始の URL を開いた状態で返す(#30 の `Launch` の契約)。
- 初期化のコマンド: `crates/webqa/tests/fixtures/reset/` の台本を使う。
- 保存先: `support/memory_store.rs` の保存先に台帳を書かせ、本文を世界から読めるようにする。

### 台帳の読み取り

- 台帳は `webqa_core::interview::ledger::read` で読み戻し、手順はその値で確かめる。文字列の部分一致で済ませない。
- 課題の番号は `I-` と16進8桁で、画面・要素・種類から計算される。feature の例の番号(`I-0001` など)は、手順の中で実際の番号に読み替える。番号の読み替えの補助を support に置く。

### Background と共通の手順

`steps/interview/common.rs` に、次を書く。

- Background の3つ: 「対象は許可ドメイン {string} のページである」「ペルソナの文書に、名前 {string}、ITの慣れ {string}、しないこと {string} などが書かれている」「インタビューのモードで、試行の回数は {int} 回、迷いの度合いの閾値は {float}、迷いとみなす確信度の閾値は {float} に設定されている」。
- `33-plan.md` の共通の手順の表の16個。

文言は feature の文と一字一句同じにする。`{string}`、`{int}`、`{float}` 以外の広いパラメータと正規表現は使わない。

### Rule ごとのファイル

次の空のファイルを置き、`READY` を空の配列で置く。`interview_acceptance.rs` の `ready()` は、全部の `READY` を連結する。

| ファイル | Rule | 受け持つ brief |
|---|---|---|
| `steps/interview/input.rs` | ペルソナと目的の入力 | 33b |
| `steps/interview/modes.rs` | 既存のモードとの関係 | 33b |
| `steps/interview/trials.rs` | 試行 | 33c |
| `steps/interview/questions.rs` | 判断モデルへの問い | 33d |
| `steps/interview/endings.rs` | 試行の終わり方 | 33d |
| `steps/interview/persona.rs` | ペルソナから外れないこと | 33d |
| `steps/interview/issues.rs` | 課題の見つけ方 | 33e |
| `steps/interview/ledger.rs` | 台帳の書式 | 33e |
| `steps/interview/select.rs` | 課題を選ぶことと TUI | 33f |
| `steps/interview/rerun.rs` | 再実行と照らし合わせ | 33f |
| `steps/interview/guide.rs` | 手引きと形式の検査 | 33f |

1ファイルがファイルごとの Halstead の量の上限(8000)に近づいたら、`steps/interview/<名前>_<区切り>.rs` に分ける。分けたファイルは、元のファイルの中で `#[path]` で読み込み、`interview_acceptance.rs` は触らない。

## テスト

| テスト | 期待値 |
|---|---|
| 土台の確かめ(`main` の中) | feature の写しが `tasks/loopeng/InterviewMode.feature` と1バイトも違わない |
| 土台の確かめ | `READY` の名前がどれも feature の見出しに1つだけあり、2つのファイルが同じ名前を持たない |
| 土台の確かめ | 偽物の判断モデルが、積んだ答えを順に返し、問いの種類と画像の有無を記録する |
| 土台の確かめ | 偽物の判断モデルが答えを使い切ると、止まらずに失敗する(`stuck.rs` の待ち) |
| 土台の確かめ | 書いた台帳を読み戻すと、課題の番号が `I-` と16進8桁である |
| `cargo test -p webqa --test interview_acceptance` | 0本のシナリオで成功し、あいまいな手順の出力が無い |
| `cargo test -p webqa --test acceptance` | 探索モードの190本がそのまま通る |

止まり続けるテストを書かない。`tokio::time` はこのクレートで禁止なので、待つなら std の `recv_timeout` か `stuck.rs` を使う。

## 親に決めてほしいこと

1. **共通の文言の引数の型**。推す案: 数字は Examples の値に合わせ、試行の回数と上限は `{int}`、閾値は `{float}` にする。
2. **台帳の読み取りの補助の置き場**。推す案: `support/interview/ledger.rs` に置き、番号の読み替え(例の番号と実際の番号の対応)もここに持たせる。2段目の担当が同じ読み替えを書かずに済む。
3. **実行を回す時点**。推す案: 探索モードと同じく、手順が最初に結果を求めたときに1回だけ回す。「1ステップを実行する」系の手順は、判断モデルの答えを使い切ったら止まる形(#14 の 14h と同じ)にする。
