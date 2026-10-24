# テストの層

webqaのテストを層に分け、それぞれの置き場所、対象、実行環境をまとめる。
カバレッジ100%と品質ゲートの基準は[品質ゲート](../quality-gates.md)にある。

## 層の一覧

| 層 | 置き場所 | 対象 | 実行環境 |
|---|---|---|---|
| 単体 | 各crateの`src`の中 | 判定関数、表の行、設定の検証、伏せ字、done_when、名前の正規化 | なし |
| 全状態の探索 | `webqa-core/tests/exhaustive.rs` | 状態機械の不変条件と、遷移ごとの性質 | なし |
| ゴールデン | `webqa-core/tests/`、`webqa-tui/tests/render.rs`、`webqa`の`store::report`の単体 | ガードレールの判定、インタビューの問いの文面と台帳の形、TUIの描画、実行結果の形 | なし |
| 契約 | `webqa-browser/tests/contract.rs`、`webqa-judge/tests/command.rs`、`webqa/tests/reset.rs`、`webqa/tests/reset_stop.rs` | アダプタが材料を正しく集めて操作すること。橋渡しと初期化のコマンドの起動 | Chromium、fixtureのページ、試験用のコマンド |
| 保存 | `webqa/tests/store.rs`、`webqa-tui/tests/ledger_file.rs` | 記憶と実行結果のファイルの読み書き。台帳の書き戻しと、外での書き換えの検出 | `CARGO_TARGET_TMPDIR`の下の一時ディレクトリ |
| 性質 | `webqa/tests/trials_props.rs` | 試行の外のループの安全性(実行全体の窓、期限、試行の数) | 偽の時計と偽の部品。乱数のseedと回数を固定する |
| 文書 | `webqa/tests/guide.rs`、`webqa-browser/tests/container.rs` | 配布する手引きの形。コンテナのイメージのPlaywrightの版 | なし |
| 受け入れ | `webqa/tests/acceptance.rs`、`webqa/tests/interview_acceptance.rs` | 探索モードと決め打ちシナリオモードの受け入れシナリオ、インタビューのモードの受け入れシナリオ | 偽の部品 |
| 実行ファイル | `webqa/tests/cli.rs`、`webqa/tests/watch.rs`、`webqa/tests/dotenv.rs` | 起動、設定エラー、終了コード、`.env`の読み込み、`watch`、端末の無い`ledger` | Chromium |

統合テストは、1つの話題ごとに1つのファイルを`tests/`の直下に置く。
1つのファイルの中身が大きければ、同じ名前のディレクトリにモジュールを分ける。
例えば`contract.rs`の部品は`contract/`に置く。

## 全状態の探索

中核の状態機械は純粋なので、小さな定数を与えれば全状態を探索できる。
探索は4通りで、段階の無いシナリオ、段階のあるシナリオ、探索モード、インタビューである。
到達した状態ごとに、仕様の不変条件をRustの述語として確かめる。
遷移ごとに、優先順位や終了の吸収などの性質も確かめる。
乱数を使わないので、毎回同じ結果になる。
候補の選び方と仕様との対応は[中核の状態機械](core.md)にある。

## ゴールデン

ガードレールのゴールデンは、店やSaaSの画面の断片をJSONの表に書く。
置き場所は`webqa-core/tests/fixtures/guard/`である。
表の1行が、1つの断片と期待する判定の組になる。

インタビューの問いの文面と台帳の形は、`webqa-core/tests/fixtures/interview/`と比べる。

TUIの描画と実行結果の形は、instaのスナップショットと比べる。
スナップショットは、それぞれのテストの隣の`snapshots/`に置く。

## 受け入れテスト

受け入れシナリオは、cucumber crateでRustのテストとして走らせる。
`webqa/tests/acceptance.rs`は`harness = false`のテストである。
featureファイルは`webqa/tests/features/web_exploratory_qa.feature`に置く。

手順の定義は`webqa/tests/steps/`に、話題ごとに分けて置く。
各ファイルは、受け持つシナリオの名前を`READY`の一覧に持つ。
`acceptance.rs`は、一覧の名前がfeatureのシナリオに1つずつ当たることを確かめる。
2つのファイルが同じシナリオを受け持っても失敗にする。
一覧に載ったシナリオだけを走らせ、手順の飛ばしは失敗にする。

インタビューのモードの受け入れシナリオは、別の実行ファイル`webqa/tests/interview_acceptance.rs`で走らせる。
cucumberは1つの実行ファイルに世界の型を1つしか持てないからである。
featureファイルは`webqa/tests/features/interview_mode.feature`に置く。
手順の定義は`webqa/tests/steps/interview/`に、偽物と場面の組み立ては`webqa/tests/support/interview/`に置く。
`READY`の一覧の確かめ方は探索モードと同じである。
これに加えて、仕組みが2つある。

1つ目に、featureファイルは`tasks/loopeng/InterviewMode.feature`の写しである。
`interview_acceptance.rs`は、走らせる前に2つのファイルの中身が同じであることを確かめる。
仕様のfeatureを変えたら、写しも同じに変える。

2つ目に、別のテストに任せるシナリオは`DELEGATED`の一覧に載せる。
「探索モードと決め打ちシナリオの受け入れシナリオは、そのまま通る」は、CIで`acceptance.rs`が確かめるので、ここでは走らせない。
`DELEGATED`の名前も、featureのシナリオに1つずつ当たることを確かめる。
`READY`と`DELEGATED`を合わせた数は、featureのシナリオの数と同じでなければならない。

## 偽物の部品

受け入れのテストは、`webqa/tests/support/`の偽物で実行ループを回す。
偽物は、ブラウザ、判断モデル、時計、保存先、中断の5つである。

| 偽物 | 振る舞い |
|---|---|
| ブラウザ | 決めた順に画面を見せ、受けた操作を記録する。応答を止めることもできる |
| 判断モデル | 決めた答えを順に返し、尽きたら待ち続ける |
| 時計 | 判断モデルが考えるときと、実行ループが待つときだけ時刻が進む |
| 保存先 | 記録をメモリに書き、テストから中身を読める |
| 中断 | 決めた呼び出しで、または何かが待ち続けたときに中断を起こす |

偽物を使うので、シナリオは決定的に速く回り、mutationテストにも耐える。
`support/fakes.rs`の偽物は、`run`の単体テスト、`interview/trials/tests.rs`、性質のテストでも使う。

## 層の受け持ち

受け入れの層だけでは確かめられない振る舞いは、別の層が受け持つ。

| 振る舞い | 受け持つ層 |
|---|---|
| 認証状態の注入 | 契約(保存した認証状態で開くと、ログイン済みのページになる)。読めない認証状態は契約で、形式の誤りは単体 |
| APIキーを環境変数から読む | 契約(試験用のコマンドが、渡された環境変数を答えに含める) |
| ページが応答しない | 契約(待ち時間切れを「変化なし」の観測に変える)。そのあとの回数は受け入れ |
| リダイレクト、コンソールのエラー | 契約 |
| 試行ごとに開いたブラウザに前の保存領域が残らない | 契約 |
| 初期化のコマンドにシェルを通さず、APIキーを渡さない | 契約(試験用のコマンドが、受け取った引数と環境変数を書き出す) |
| 記憶と実行結果の保存先 | 保存。受け入れは保存先の偽物を使う |
| 試行の外のループの窓、期限、試行の数 | 性質(乱数で作った試行の長さと時刻を、偽の時計で回す) |
| 配布する手引きの前付け、手順、例、置き場所 | 文書 |
| 台帳の画面のキーの割り当て、描画、書き戻し、終了コード | 単体(注入したキーの列で画面を回す) |
| 本物の端末で、本物のキーで台帳の画面が動くこと | 手で確かめる |

性質のテストは、性質ごとに1つの関数にする。
反例が出たら、最小の反例を単体のテストのケースに足して固定する。

## 台帳の画面の端末

テストからは本物の端末に届かない。
そこで台帳の画面は、本物の端末に触る処理を`webqa-tui/src/keys.rs`の薄い層に分ける。
薄い層は、標準入力が端末かの確認、rawモードの出入り、crosstermからのイベントの読みだけで、分岐を持たない。
端末のイベントをキーに変える関数は、テストで作ったイベントを渡して単体で確かめる。

`webqa ledger`の本体は`webqa/src/ledger.rs`の`show`で、rawモードに入る関数とキーの出どころを引数で受ける。
単体のテストは、決めたキーの列と、rawモードに入れたか入れなかったかを返す関数を注入する。
その口で、キーの割り当て、描画、台帳の書き戻し、外での書き換えの扱い、終了コードを確かめる。
実行ファイルのテストは、`webqa ledger`を標準入力に端末の無い状態で起動する。
そして、台帳を開けないときの2と、端末が無いときの3を確かめる。

## 契約テスト

契約テストは、本物のChromiumと本物の子プロセスで回す。
カバレッジ100%にはアダプタも含まれ、除外の注釈は足さない。
そのため、CIでもChromiumを入れて契約テストを回す。

ブラウザの契約テストのページは、Playwrightの`route`で応答を返す。
ページの元は`webqa-browser/tests/fixtures/pages/`のHTMLである。
応答の遅れと別のオリジンは、`route`で作れる。
HTTPの3xxの転送先への要求は`route`に来ない。
そこで3xxの転送先だけは、テストの中で127.0.0.1のソケットを開いて返す。
オリジンが別になるので、許可ドメインの試験にもそのまま使える。

1つのテストのプロセスで同時に開くブラウザは、4つまでにしている。
これより多いと、Chromiumの負荷でスナップショットがページの待ちの上限の1秒を超える。
枠は`webqa-browser/tests/contract/slot.rs`にあり、スレッドの数を絞らずに回せる。

橋渡しの契約テストは、`webqa-judge/tests/fixtures/bridge.sh`を相手に回す。
初期化のコマンドの契約テストは、`webqa/tests/fixtures/reset/`のスクリプトを起動する。
どちらも、書き出すファイルを`CARGO_TARGET_TMPDIR`の下にテストごとの名前で置く。

## Playwrightのdriverとブラウザ

playwright-rsは、ビルドのときにNodeのdriverを取得する。
CIでは`PLAYWRIGHT_DRIVER_CACHE_DIR`で置き場所を決め、CIのキャッシュに載せる。

Chromiumは、`webqa-browser/examples/install-browsers.rs`の小さなプログラムで入れる。
入れ方は2通りある。

| レシピ | 入れるもの | 使う場所 |
|---|---|---|
| `just install-browsers` | Chromiumだけ | 手元。`just setup`が呼ぶ |
| `just install-browsers-with-deps` | ChromiumとOSのライブラリ | CI |

OSのライブラリを入れるときは、パッケージ管理のためにsudoを求めることがある。
そのため`just setup`は、Chromiumだけを入れる。
CIのrunnerはパスワード無しでsudoできるので、OSのライブラリも入れる。
手元でライブラリが足りなければ、開発者が`just install-browsers-with-deps`を手で実行する。
OSにライブラリを入れたくなければ、[コンテナ](../container.md)の中で回す。

`examples/`のプログラムは、`clippy --all-targets`と複雑度の検査の対象になる。
カバレッジの計測とmutationの対象には入らない。

使う版は[使っている依存](dependencies.md)にある。
