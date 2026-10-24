# 探索モードの使い方

この文書は探索モードを初めて使う人に向けて、1回回して結果を読むまでを案内する。
手本の対象はPlaywrightのTodoMVCのデモ(<https://demo.playwright.dev/todomvc/>)である。
数字と出力は、2026-10-25(日本時間)にこのデモを1回回したときのものである。
版はmainのc9d2d94である。
判断モデルの選び方で操作の順番が変わるので、回すたびに結果も変わる。
探索モードの仕組みは[探索モード](arch/explore-mode.md)に、設定の項目の全体は[設定](arch/config.md)にある。

## 用意

### 環境とビルド

環境は[CONTRIBUTING.md](../CONTRIBUTING.md)のとおりに用意し、`webqa`をビルドする。

```sh
sh scripts/setup.sh
cargo build -p webqa
```

`scripts/setup.sh`は`just install-browsers`でChromiumも入れる。
実行ファイルは`target/debug/webqa`にできる。
Chromiumを既定の場所以外に入れたときだけ、その場所を環境変数`PLAYWRIGHT_BROWSERS_PATH`で渡す。

### 判断モデルのキー

次の操作は判断モデルが選ぶ。
手本ではリポジトリの`bridge/jev.py`を橋渡しにして、JEVの判断モデルを使う。
JEVは、選択肢から1つを選ぶ問いに確率つきで答える判断モデルである。
橋渡しはVercel AI Gatewayへ送るので、JEVを使えるGatewayのキーが要る。

キーはリポジトリの直下の`.env`に`WEBQA_JUDGE_API_KEY`として書く。

```sh
WEBQA_JUDGE_API_KEY=<キー>
```

`webqa`は起動のときに作業ディレクトリの`.env`を読み、まだ設定されていない変数だけを足す。
`.env`は`.gitignore`で除外してある。
キーを設定ファイルに直接書くと、設定エラーで起動しない。
キーが橋渡しに渡る流れは、[判断モデル](arch/judge.md)の「外部コマンドとの橋渡し」にある。

## 設定ファイル

リポジトリの直下に、次の設定を`todomvc.yaml`として置く。
2026-10-25の手本の実行で、このまま動いた設定である。

```yaml
target:
  url: https://demo.playwright.dev/todomvc/
  allowed_origins: []
judge:
  kind: command
  command: ["./bridge/jev.py", "--model", "typesafe-ai/jev-1.14.0", "--timeout", "60"]
  api_key: { env: WEBQA_JUDGE_API_KEY }
  timeout_secs: 70
  pricing: { input_per_1k: 0.0, output_per_1k: 0.0 }
limits:
  freeze_threshold: 3
  stuck_threshold: 3
  max_decisions: 15
  requests_per_hour: 120
  page_timeout_secs: 10
  max_duration_secs: 300
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: { keywords: [], selectors: [] }
  allowed: { selectors: [] }
inputs:
  What needs to be done?: { env: WEBQA_TODO, secret: false }
```

| 項目 | 何のためか |
|---|---|
| `target.url` | 探索を始めるURL |
| `target.allowed_origins` | 探索してよいオリジン。空なら`target.url`のオリジンだけを許す |
| `judge.kind`、`judge.command` | 判断モデルへの橋渡しのコマンド。`./bridge/jev.py`は作業ディレクトリからの相対パスである |
| `judge.api_key` | キーを持つ環境変数の名前 |
| `judge.timeout_secs` | 判断モデルの答えを待つ秒数。橋渡しの`--timeout`より長くする |
| `judge.pricing` | 実行結果のコストの計算に使う単価。手本では0にしたが、Gatewayの利用料はかかる |
| `limits.max_decisions` | 判断モデルに問う回数の上限。手本では15回で止める |
| `limits.max_duration_secs` | 実行の時間の上限(秒) |
| `limits.requests_per_hour` | 1時間に対象へアクセスする回数の上限 |
| `limits`のほかの項目 | 固まりと行き詰まりの閾値、ページを待つ秒数、確信度の閾値 |
| `guard` | 触ってはいけない要素。手本では何も足さず、既定の禁止の語だけを使う |
| `inputs` | 入力欄に打つ値 |

探索モードは選択肢の問いだけを送るので、橋渡しに`--text-model`は要らない。

### 入力欄の値

`inputs`のキーは、入力欄の読み上げ名である。
TodoMVCの入力欄の読み上げ名は`What needs to be done?`なので、これをキーにする。
値は設定に書かず、環境変数から読む。
手本では`.env`に次の行を足した。

```sh
WEBQA_TODO=牛乳を買う
```

`inputs`の値は既定では記録で伏せ字になる。
TODOの文字は秘密でないので、`secret: false`で伏せないようにした。

## 回す

リポジトリの直下で、次のコマンドを実行する。

```sh
target/debug/webqa explore --config todomvc.yaml --out .webqa
```

実行中の様子は、別の端末から`webqa watch`で見られる。
渡すのは出力先の`runs/`の下にできた実行のディレクトリである。
実行IDは開始の時刻(UTC)で、`20261024T174730.617Z`のような形になる。
実行を始めたら、`ls .webqa/runs/`でいちばん新しいディレクトリを選ぶ。

```sh
target/debug/webqa watch .webqa/runs/<実行ID>
```

`watch`の画面はCtrl-Cで閉じる。
画面の中身は[問題の記録と実行結果](arch/records.md#watchの画面)にある。

## 結果を読む

### 標準エラーの1行

実行が終わると、終わり方を標準エラーに1行で書く。
手本の実行は約24秒、判断15回で終わり、次の1行を書いた。

```text
終了理由 budget_exceeded、記録 .webqa/runs/20261024T174730.617Z
```

`budget_exceeded`は判断モデルに問う回数が`limits.max_decisions`に達したことを表す。
ほかの終了理由は[探索モード](arch/explore-mode.md#終わり方)にある。

### 終了コード

確定の問題を見つけなければ0、見つければ1で終わる。
設定か引数の誤りは2で、実行時エラーと、判断モデルへの問いが3回続けて失敗したときは3である。
手本の終了コードは1だった。
確定の問題を見つけたためで、中身は[TodoMVCで見つかった問題](#todomvcで見つかった問題)に書いた。
細かな決まりと例外は[コマンド](arch/cli.md#終了コード)にある。

### 出力先の構成

手本の実行は出力先に次のものを書いた。

```text
.webqa/
  memory/
    https%3A%2F%2Fdemo.playwright.dev.json   探索の記憶(オリジンごと)
  runs/
    20261024T174730.617Z/
      report.json     実行結果
      events.jsonl    イベントログ
      screenshots/    問題を見つけた画面
```

### report.json

`report.json`は実行結果で、実行の終わりに1回だけ書く。
まず読むのは次の項目である。

| 項目 | 中身 | 手本の値 |
|---|---|---|
| `reason` | 終了理由 | `budget_exceeded` |
| `issues` | 見つけた問題 | `outside_domain`が2件 |
| `usage` | 判断モデルを呼んだ回数、トークン数、コスト | 15回、入力7,803トークン、出力2,636トークン、コスト0.0 |
| `steps` | ブラウザへの操作の記録 | |
| `calls` | 判断モデルの答えの記録 | |
| `start_memory` | 実行を始めたときの探索の記憶 | |

手本のコストが0.0なのは、`judge.pricing`の単価を0にしたためである。
項目の全体は[問題の記録と実行結果](arch/records.md#実行結果)にある。

### events.jsonl

`events.jsonl`は1行に1つのイベントをJSONで書いたログである。
手本の実行には`run_started`、`asked`、`decided`、`executed`、`issue`、`recovered`、`run_finished`の行が出た。
`webqa watch`の画面は、このログを読んで表示している。
イベントの種類は[問題の記録と実行結果](arch/records.md#イベントログ)にある。

### スクリーンショット

問題を見つけると、その画面のスクリーンショットを`screenshots/`に残す。
ファイル名は問題の種類とステップ番号で、手本では`ISSUE_outside_domain_step8.png`などができた。
スクリーンショットは画面をそのまま残し、伏せ字にしない。

### TodoMVCで見つかった問題

手本の実行は入力欄に打ってEnterを押す操作を2回繰り返した。
そのあと「Mark all as complete」を押すなどして、ページの要素を順に操作した。
途中で「real TodoMVC app.」のリンクを押し、todomvc.comへ出た。

外へ出たリンクは<https://todomvc.com/>と<https://github.com/remojansen/>の2つである。
どちらも確定の問題`outside_domain`として記録された。
確定の問題があるので、この実行の終了コードは1だった。

外へ出るたびに、中核は判断モデルに問わずに開始のURLを開き直し、探索を続けた。
イベントログには、このときの操作が`recovered`の行として、`cause`を`outside`にして残る。

### 外へのリンクを問題にしないには

設定で外へのリンクを扱う方法は2つある。
どちらも手本では試していない。

- `target.allowed_origins`にリンク先のオリジンを足す。
  一覧が空でなくなると開始のURLのオリジンは自動では許されないので、`https://demo.playwright.dev`も書く。
  足したオリジンは探索する範囲にも入り、そのサイトにもアクセスする。
  許しを得ていない相手のサイトは足さない。
- `guard.forbidden.selectors`に外へのリンクのCSSセレクタを書く。
  当たった要素は禁止要素になり、探索の候補から外れる。
  ただし、禁止要素のある画面では、操作の種類から`enter`が外れる。
  同じ画面の入力欄では、打った値をEnterで送れなくなる。

禁止要素と許可ドメインの決まりは、[ガードレール、負荷制御、機密情報](arch/safety.md)にある。

## 続けて回す

同じ`--out`で回すと、探索の記憶を引き継ぐ。
前回までに操作した要素は後回しになる。
手本の2回目は外へのリンクから選び、未操作の要素が無くなって`goal_achieved`で終わった。

最初からやり直すときは、新しい`--out`を渡すか、出力先の`memory/`にある対象のオリジンのファイル(手本では`https%3A%2F%2Fdemo.playwright.dev.json`)を消す。

すべての要素を操作済みの記憶で回すと、既知の不具合に当たる。
`http 400`の応答の異常が1回出て、そのまま`goal_achieved`で終わる([#103](https://github.com/Hakkadaikon/webqa-explorer/issues/103))。

## 再実行

`webqa replay`は`report.json`に記録した判断モデルの答えで、同じ実行をやり直す。
入力欄の値は、渡した設定の`inputs`から補う。

再実行も対象のサイトにアクセスし、結果は出力先の`runs/`に新しい実行IDで書く。

```sh
target/debug/webqa replay .webqa/runs/<実行ID>/report.json --config todomvc.yaml --out .webqa
```

再実行の決まりは[コマンド](arch/cli.md#再実行)にある。
