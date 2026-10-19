# issue #31 インタビューと形式の検査のサブコマンド、終了コード

インタビューのモードを実行するサブコマンド `webqa interview` と、ペルソナと目的の文書の形式を検査するサブコマンド `webqa check-docs` を足す。
設定エラー(ペルソナの欠け、読めない前回の台帳、前回の台帳と食い違う頭)は、ブラウザを開く前に終了コード2で終える。
インタビューの課題だけを見つけて終わったら、終了コードは0にする。

共通の決まりは `common.md`、手順は skill `parallel-work` と `implementing`。
試行のループ(#30)は別の担当が並列に進める。
この issue は #30 の公開の形(`tasks/briefs/30-interview-loop.md` の「公開の形」)を、そのシグネチャのまま使う。
#30 の最初のコミット(公開の型)が入ったら、`git merge --no-edit issue-30` で取り込む。
それまでは、#30 に依らない部分(`check-docs`、引数、前回の台帳の読み込みと頭の確かめ)を先に作る。
仮の型や stub は置かない。

## 前提(integ2 で確かめた)

- CLI は `cli.rs`(clap の derive、`Command` は `Run`、`Explore`、`Replay`)、`cli/app.rs`(サブコマンドの振り分けと起動の順)、`cli/start.rs`(起動の段)にある(14g)。
- 終了コードは `exit.rs` の `code(&Ending)`、`CONFIG_ERROR`(2)、`RUNTIME_ERROR`(3)(14f)。
- 設定は `setup::load(yaml, mode, env, read)` が読む。インタビューの設定は `config::interview::load(config, read)` が `Interview` にする(#18)。
- ペルソナの文書は `config::persona::parse(text)`、目的の文書は `config::purpose::parse(text)` で検証できる。
  誤りの文は `InterviewError` の Display で、項目の名前を含む(`MissingField`、`NoDeclarations` など)。
- 台帳は `interview::ledger::read(text)` で読み戻せ、壊れていれば `LedgerError`(#24)。照らし合わせの頭の食い違いは `compare::Mismatch`(#25)。

## 引数

```text
webqa interview --config <設定> [--out <出力先>] [--previous <前回の台帳>]
webqa check-docs persona <ファイル>
webqa check-docs purpose <ファイル>
```

- `interview` の `--config` と `--out` は `run` と同じ(`--out` の既定は `.webqa`)。
- `--previous` を省けば、照らし合わせない(すべての課題を新しく出たとして示す)。
- 制限時間は設定だけで決め、引数では上書きしない(14g と同じ利用者の決定)。
- `check-docs` は文書を1つ読み、正しければ何も書かずに終了コード0で終わる。
  誤りがあれば、誤りの文(項目の名前を含む)を標準エラーに1行書いて、終了コード2で終わる。
  ブラウザも判断モデルも起動しない。

## 触ってよいファイル

- `crates/webqa/src/cli.rs`(`Command` に `Interview`、`CheckDocs` を足す)、`cli/app.rs`、`cli/start.rs`、`cli/tests.rs`
- `crates/webqa/src/cli/` の下の新規ファイル(`interview.rs`、`check.rs` など。起動の段と `Launch` の本物)
- `crates/webqa/src/exit.rs`、`exit/tests.rs`
- `crates/webqa/src/setup.rs`、`setup/` の下(インタビューのモードの材料の読み込み)
- `crates/webqa-core/src/interview/compare.rs`(試行の前に頭を確かめる関数を足すだけ)
- `crates/webqa/tests/cli.rs`、`tests/cli/` の下、`tests/fixtures/cli/` の下(実行ファイルの層)
- `docs/architecture.md` の CLI の節(サブコマンドの一覧)

触らないもの: `interview/trials*`、`run.rs`、`run/`、`store/`、`ports.rs`、`event.rs`、`machine.rs`、`tests/support/fakes.rs`、`tests/trials_props.rs`(#30 の範囲)。

## 決めた設計

### 起動の順(`webqa interview`)

1. 設定を読む。`setup::load` に加え、`config::interview::load` でペルソナと目的を読む。
   ペルソナの欠け、読めない書式は設定エラー(終了コード2)。判断モデルも対象ページも開かない(T-002)。
2. `--previous` があれば、ファイルを読み、`ledger::read` で読み戻す。読めなければ設定エラー(T-111)。
3. 前回の台帳の頭を、今回のペルソナ、目的、開始の URL と確かめる。
   `webqa-core` の `compare.rs` に次を足し、`compare` の中の確かめもこれで行う(同じ処理を2つ持たない)。

   ```rust
   /// Whether `previous` can be compared with a run of `persona`, `goal` and `start`.
   pub fn check_previous(previous: &Head, persona: &str, goal: &Goal, start: &str) -> Result<(), Mismatch>;
   ```

   目的を推測させた実行どうしは、推測の文の違いを食い違いにしない(feature の「推測した文の違いは…設定エラーにもなっていない」)。
   食い違いは設定エラー(終了コード2)。
4. 判断モデルを起動する(14g と同じ。種類が無い・コマンドが無いは2、起動できない・申告を返さないは3)。
5. 本物の `Launch` を作る。試行ごとに新しいブラウザを、表示領域 1280×720、認証状態つきで開く(14g の起動の材料を使う)。
6. 時計を起動のあとから数える(14g の決定と同じ)。
7. 保存先(`FileStore`)を作る。
8. `interview::trials::interview(..)` を呼ぶ。`Plan` は次で埋める。
   - `env`: `std::env::vars_os()` を集めたもの
   - `withheld`: API キーの値(`judge.api_key` が無ければ空)
   - `start`: `redactor.url` を通した開始の URL
   - `previous`: 2 で読んだ台帳
9. 終わったら、終了の1行を標準エラーに書き、終了コードを返す。

### 終了コード

`exit.rs` に、インタビューの終わり方から終了コードを決める関数を足す。

```rust
pub fn interview_code(ending: &trials::Ending) -> u8 {
    match ending {
        Ending::Wrote { confirmed, .. } => u8::from(*confirmed),
        Ending::Mismatch(_) => CONFIG_ERROR,
        Ending::ResetFailed(_) | Ending::Failed(_) | Ending::Abandoned => RUNTIME_ERROR,
    }
}
```

インタビューの課題は終了コードに入れない(T-095)。
確定の問題(#8)が記録されたときだけ1にする。

### 終了の1行

`webqa interview` は、終わったときに次の1行を標準エラーに書く。

- 台帳を書いた: `終了理由 <completed|time_up|interrupted>、台帳 <実行のディレクトリ>/ledger.md`
- 初期化のコマンドの失敗: `初期化のコマンドが失敗した、台帳 <…>/ledger.md`(`ResetError` の文は値を含まないので、そのまま足してよい)
- そのほかは 14g の `describe` に合わせる。

## テスト

単体(`cli/tests.rs`、`exit/tests.rs`、`setup/` の下)と、実行ファイルの層(`tests/cli.rs` と `tests/cli/` の下)。
値は `assert_eq!` で比べる。

| テスト | 層 | 確かめること |
|---|---|---|
| `interview_arguments_are_read` | 単体 | `interview --config a --out b --previous c` と、`--previous` を省いた形が読める |
| `check_docs_arguments_are_read` | 単体 | `check-docs persona a` と `check-docs purpose a` が読め、種類の無い形は `Err` |
| `interview_issues_exit_zero` | 単体(`webqa::exit`) | `Wrote { confirmed: false }` は0、`true` は1、`Mismatch` は2、`ResetFailed`・`Failed`・`Abandoned` は3(T-095) |
| `previous_head_checked_before_trials` | 単体(`webqa-core::interview::compare`) | ペルソナの名前が違えば `Mismatch::Persona`、書いた目的が違えば `Goal`、推測どうしで開始の URL が違えば `Start`、推測どうしで推測の文だけ違えば `Ok` |
| `interview_config_error_exits_two_before_opening` | 実行ファイル | ペルソナの文書から5項目のどれかを消すと終了コード2、橋渡しの記録に起動が無く、試験用のサイトにアクセスが無い(T-002) |
| `broken_previous_ledger_exits_two` | 実行ファイル | 課題の区切りを崩した台帳を `--previous` に渡すと終了コード2、サイトにアクセスが無い(T-111) |
| `mismatched_previous_ledger_exits_two` | 実行ファイル | ペルソナの名前が違う台帳を渡すと終了コード2、サイトにアクセスが無い |
| `check_accepts_valid_documents` | 実行ファイル | 正しいペルソナの文書と目的の文書で、どちらも終了コード0、標準エラーが空(T-114) |
| `check_reports_field_and_exits_two` | 実行ファイル | ITの慣れの無いペルソナの文書と、done_when に宣言の無い目的の文書で、終了コード2、標準エラーに項目の名前(`ITの慣れ`、`done_when`)がある(T-115) |
| `interview_runs_and_writes_ledger` | 実行ファイル | 試験用のサイトと橋渡しで試行2回を回すと、終了コード0で、実行のディレクトリに `ledger.md` があり、頭の試行の回数が2 |

T-115 の `webqa-core::config::purpose::done_when_without_declarations_rejected` は #18 で入っている。

## 親に決めてほしいこと

1. **インタビューのサブコマンドの名前**。案A(推す): `webqa interview`。案B: `webqa run --interview`。A のほうが、`run` と `explore` と同じ並びで読みやすい。
2. **`check-docs` の形**。案A(推す): `check-docs persona <ファイル>` と `check-docs purpose <ファイル>` の2つの種類。案B: `check-docs --config <設定>` で設定が指す2つの文書をまとめて検査する。A は手引き(#32)から1つずつ呼べる。
3. **前回の台帳の頭の食い違い**。案A(推す): 試行の前に確かめて設定エラー(終了コード2)。#30 の brief の「親に決めてほしいこと」2 と合わせて決める。

## 仕様

TLA+ と feature を変える必要は無い見込み。
前回の台帳の頭の食い違いを設定エラーにすることは、feature にシナリオが無い。
推す案で決めたら、feature の「再実行と照らし合わせ」の Rule に1本足すかを親が決める(仕様)。

## 台帳の issue の列(親が直す)

T-042 と T-089 は #30 に移す(#30 の brief の末尾を参照)。
