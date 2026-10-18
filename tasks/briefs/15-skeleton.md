# brief: issue #15 受け入れシナリオ1本を偽物で端から端まで通す

共通の決まりは同じディレクトリの `common.md`。並行処理と async の設計が要るタスクなので、細部の判断は任せる。ただし、下の「決めてある設計」から外れるときと、ほかの issue の受け持ちに踏み込むときは、止めて報告する。

## タスク

受け入れシナリオ「判断モデルを差し替えても同じ探索ができる」(`tasks/loopeng/WebExploratoryQA.feature`、台帳の T-050)を、cucumber で偽物の部品を使って端から端まで通す。
目的は、ポートの形、実行ループ、受け入れのテストの土台を先に決めて、部品どうしの噛み合わせを早く確かめることにある。
設計は `docs/architecture.md` の「1ステップの流れ」「効果の振り分け」「判断モデルの抽象」「テストの層」「ディレクトリ構成」。

## 触ってよいファイル

- `crates/webqa-core/src/ports.rs`、`crates/webqa-core/src/judge.rs`(問いと答えの型だけ)、`crates/webqa-core/src/effect.rs`、`crates/webqa-core/src/machine.rs`(仮の1ステップだけ)、`crates/webqa-core/src/lib.rs`
- `crates/webqa-judge/src/lib.rs`、`crates/webqa-judge/src/scripted.rs`
- `crates/webqa/src/lib.rs`、`crates/webqa/src/run.rs`
- `crates/webqa/tests/acceptance.rs`(`harness = false`)、`crates/webqa/tests/steps/`、`crates/webqa/tests/support/`、`crates/webqa/tests/features/web_exploratory_qa.feature`(`tasks/loopeng/` の版の写し)
- 各 crate の `Cargo.toml`(依存は `.workspace = true`)、`Cargo.lock`

## 決めてある設計

- `webqa-core::judge`: architecture.md の `Question`(`Noul`、`Choice`、`Score`)、`QuestionId`、`Answers`(問いごとの確率の表と使用量)、`JudgeError` を、その形で置く。問いの組み立てと応答の検証は issue #7 の受け持ちなので、ここでは型と、操作種別と対象要素の2問を作る最小の関数だけにする。
- `webqa-core::ports`: `Judge` は architecture.md の trait のとおり(`Pin<Box<dyn Future + Send>>` を返す、`&Redacted<ScreenText>` を受ける)。`Browser`(観測 `observe` と操作 `touch`)、`Clock`(実行の開始からの経過 `elapsed` と待機 `sleep_until`)、`Store`(イベントの行を書く `write_event`)も同じ形の trait にする。どれも `Box<dyn …>` で使えること。
- `webqa-core::effect`: architecture.md の入れ子の `Effect`(`Outside { Ask, Sleep }`、`PageEffect { Look, Touch }`)。今使うのは `Ask` と `Touch` だけでよいが、型は4つとも置く。
- `webqa-core::machine`: 仮の1ステップ。観測 → 問い合わせの効果 → 答えの最大確率の操作と対象で `Touch` の効果 → 1ステップで終了、だけ。本物の状態機械は issue #12 で入れ替える。仮であることを doc コメントに書く。
- 画面の型は `webqa-core::screen`(issue #6 で main に入る)を使い、判断モデルへ送る文字列は `screen_text` を `Redactor` に通して `Redacted<ScreenText>` にする。
- `webqa-judge::scripted`: 台本(問いの種類ごとの答えの列)どおりに答える `Judge` の実装。
- `webqa::run`: architecture.md の `run` と `advance` の形で、効果を実行して結果を中核へ返す。
- 受け入れのテスト: cucumber の `World` に、偽のブラウザ(画面の列を返し、受けた操作を記録する)、`scripted` の判断モデル、偽の時計、メモリに書く保存先を持たせる。feature の写しのうち、このシナリオだけを名前で絞って走らせる(ほかのシナリオは後の issue で足す)。

## 失敗させるテスト

1. 受け入れ: 「判断モデルを差し替えても同じ探索ができる」。Given 判断モデルとして、操作 "click" と対象要素 "保存" を返すだけのスタブを設定している / When 1ステップを実行する / Then ブラウザで "保存" をクリックしている。
2. `webqa-judge::scripted` の単体: 台本の答えを順に返し、台本が尽きたら `JudgeError` を返す。
3. `webqa::run` の単体: 偽の部品で1ステップを回すと、判断モデルが1回呼ばれ、ブラウザに `Touch` が1回届き、保存先に判断と実行のイベントが1行ずつ書かれる。判断モデルに渡った文字列は伏せ字を通っている(伏せる対象の値が含まれない)。

## 受け入れ条件

`just check` が終了コード0(受け入れのテストも `cargo test` で走る)。変えたファイルの mutation の生き残り0。
