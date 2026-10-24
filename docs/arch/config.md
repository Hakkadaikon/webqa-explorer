# 設定

設定はYAMLのファイル1つに書き、コマンドの`--config`で渡す。
実行のモードは設定では決めず、CLIのサブコマンドで決める([コマンド](cli.md))。
1つの設定に、モードごとに要る項目を足して使う。

## 例

この例は、`webqa-core`の設定の読み込みのテストが文書から読んで使う。
例を書き換えると、テストの入力も変わる。

```yaml
target:
  url: https://app.example.test
  allowed_origins: ["https://app.example.test", "https://*.app.example.test"]
judge:
  kind: command
  command: ["./bridge/jev.py", "--text-model", "<言語モデルの名前>", "--text-images", "--timeout", "25"]
  api_key: { env: JEV_API_KEY }
  timeout_secs: 30
  pricing: { input_per_1k: 0.0, output_per_1k: 0.0 }
limits:
  freeze_threshold: 3
  stuck_threshold: 3
  max_decisions: 200
  requests_per_hour: 600
  page_timeout_secs: 10
  max_duration_secs: 60    # 書かなければ時間切れは起きない
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: { keywords: ["購入"], selectors: ["#danger"] }
  allowed: { selectors: [] }
  leave_paywall: back
redact:
  - { name: メールアドレス, pattern: "[\\w.+-]+@[\\w-]+\\.[\\w.]+" }
inputs:
  メールアドレス: { env: TEST_EMAIL }
auth:
  storage_state: ./auth.json
scenario:
  stages:
    - name: ログイン
      goal: ログインを済ませる
      done_when:
        url: /home
        element: { role: button, name: ログアウト }
```

## 読み込みの流れ

読み込みは2つの手順に分ける。

1. YAMLを生の型に読む。未知の項目は拒否する。
2. 生の型を検証し、参照する環境変数とファイルを読んで、検証済みの型に変える。

そのあと、モードに応じた検査を行う。
YAMLはserde-saphyrで読む。
環境変数とファイルの読み取りは関数として渡し、検証そのものは`webqa-core`の純粋な関数にする。
誤りが複数あっても、最初に見つけた1つだけを設定エラーとして示す。
設定エラーのときは、ブラウザも判断モデルも起動せず、終了コード2で終わる。

## 項目の一覧

| 項目 | 必須か | 中身 |
|---|---|---|
| `target` | 必須 | 対象のサービス |
| `judge` | 必須 | 判断モデル |
| `limits` | 必須 | 閾値と上限 |
| `guard` | 必須 | ガードレール |
| `redact` | 任意。既定は空 | 伏せる値 |
| `inputs` | 任意。既定は空 | 入力欄に打つ値 |
| `auth` | 任意 | 認証状態 |
| `ops` | 任意。既定は9種すべて | 使う操作種別 |
| `scenario` | 決め打ちシナリオで必須 | 段階 |
| `interview` | インタビューのモードで必須 | インタビューの設定 |

### target

| 項目 | 必須か | 中身 |
|---|---|---|
| `target.url` | 必須 | 開始のURL |
| `target.allowed_origins` | 必須。空でもよい | 許可するオリジンの一覧 |

`target.url`がURLとして読めなければ、設定エラーにする。
`target.allowed_origins`が空なら、`target.url`のオリジンだけを許可する。
許可するオリジンは、スキーム、ホスト、ポートで書く。
パス、クエリ、fragment、ユーザー名を含むものは、設定エラーにする。
`*`はホストの先頭に`*.`の形でだけ書ける。
オリジンの照合の規則は[安全のための仕組み](safety.md)にある。

### judge

| 項目 | 必須か | 中身 |
|---|---|---|
| `judge.kind` | 必須 | アダプタの名前。`command`だけがある |
| `judge.command` | `command`で必須 | 橋渡しのコマンドと引数の配列 |
| `judge.api_key` | 任意 | APIキーを持つ環境変数。`{ env: 変数名 }`の形で書く |
| `judge.timeout_secs` | 必須 | 答えを待つ秒数。1以上 |
| `judge.pricing.input_per_1k` | 必須 | 送ったトークン1000個の単価。0以上 |
| `judge.pricing.output_per_1k` | 必須 | 受け取ったトークン1000個の単価。0以上 |

`judge.api_key`は、`{ env: 変数名 }`の形だけを受ける。
キーを直書きした設定は、ほかの誤りより先に設定エラーにする。
`~`や`{}`のように、この形に合わない値も同じ設定エラーにする。
型の合わない値のエラー文には受け取った値が入るので、先に止めてキーを表に出さない。
`judge.api_key`を省くと、判断モデルの認証は外の仕組みに任せる。
単価は、実行結果のコストの計算に使う([問題の記録と実行結果](records.md))。
アダプタと橋渡しの中身は[判断モデル](judge.md)にある。

### limits

| 項目 | 必須か | 中身 |
|---|---|---|
| `limits.freeze_threshold` | 必須 | 画面が変わらないステップが何回続けば固まったとみなすか |
| `limits.stuck_threshold` | 必須 | 進まないステップが何回続けば行き詰まりとみなすか |
| `limits.max_decisions` | 必須 | 判断のステップ数の上限。インタビューのモードでは1ステップで最大2回呼ぶ |
| `limits.requests_per_hour` | 必須 | 1時間の時間窓で対象へアクセスする回数の上限 |
| `limits.page_timeout_secs` | 必須 | 画面の読み込みと操作を待つ秒数 |
| `limits.max_duration_secs` | 任意 | 実行の時間の上限(秒)。書かなければ時間切れにならない |
| `limits.back_confidence` | 必須 | 判断モデルが選んだ`back`の操作に要る確信度。インタビューのモードでは`interview.back_confidence`を使う |
| `limits.error_confidence` | 必須 | エラー表示と意味の判断が要る検出を、問題として記録する確信度 |

回数と秒数は1以上の整数にする。
0や負の値は、YAMLを生の型に読むときに拒否する。
確信度は、0より大きく1以下の有限の値にする。
この検査は、生の型を検証するときに行う。
閾値と上限が中核でどう効くかは、[中核の状態機械](core.md)と[安全のための仕組み](safety.md)にある。

### guard

| 項目 | 必須か | 中身 |
|---|---|---|
| `guard.forbidden.keywords` | 必須 | 触ってはいけない要素を表す語 |
| `guard.forbidden.selectors` | 必須 | 触ってはいけない要素のCSSセレクタ |
| `guard.allowed.selectors` | 必須 | `forbidden`に当たっても触ってよい要素のCSSセレクタ |
| `guard.leave_paywall` | 任意。既定は`back` | 課金画面から離れる操作 |

`guard.leave_paywall`には、ブラウザの操作のうち対象の要らない6つの名前だけを書ける。
`back`、`scroll_down`、`scroll_up`、`wait`、`reload`、`return`である。
ほかの名前は、大文字と小文字の違いも含めて設定エラーにする。
ガードレールの振る舞いは[安全のための仕組み](safety.md)にある。

### redact

伏せる値を、名前と値の出どころの組で並べる。
出どころは、正規表現か環境変数のどちらかである。

```yaml
redact:
  - { name: メールアドレス, pattern: "[\\w.+-]+@[\\w-]+\\.[\\w.]+" }
  - { name: 社員番号, env: TEST_ID }
```

伏せた値の場所には、`[[名前]]`を書く。
`inputs`の値とAPIキーは、ここに書かなくても伏せる。
ただし`inputs`は、値ごとに`secret: false`で伏せない指定にできる。
APIキーの場所には`[[APIキー]]`を書く。
使えない正規表現と、空の文字列に当たる正規表現は、設定エラーにする。
伏せ字の仕組みは[安全のための仕組み](safety.md)にある。

### inputs

入力欄に打つ値を、入力欄の読み上げ名をキーにして書く。
値は`{ env: 変数名 }`の形で、環境変数から読む。
キーには、伏せ字を通す前の名前を書き、`[[名前]]`の形では書かない。
値は既定で伏せる。
検索語のように秘密でない値は、次のように書けば伏せない。

```yaml
inputs:
  検索欄: { env: WEBQA_SEARCH, secret: false }
```

`secret`は`inputs`だけが受け取り、`judge.api_key`に書くと設定エラーになる。
入力欄の選び方と、値が無いときの振る舞いは[探索モード](explore-mode.md)にある。

### auth

`auth.storage_state`には、Playwrightの保存形式で書いた認証状態のJSONファイルを指定する。
起動のときにファイルを読み、`cookies`と`origins`の配列を持つJSONかを確かめる。
読めないときと形式が合わないときは、設定エラーにする。

### ops

判断モデルが選べる操作種別の一覧である。
書ける名前は次の9つである。
`click`、`input`、`scroll_down`、`scroll_up`、`back`、`enter`、`wait`、`done`、`stuck`。
省けば9つすべてを使い、空の一覧は設定エラーにする。
操作種別の中身は[探索モード](explore-mode.md)と[決め打ちシナリオモード](scenario-mode.md)にある。

### scenario

決め打ちシナリオの段階を、順に並べる。
各段階は、名前`name`、判断モデルに渡すゴール`goal`、段階を終える条件`done_when`を持つ。
`done_when`には、`url`、`element`、`text`を書ける。
`scenario.stages`が空なら、どのモードでも設定エラーにする。
段階の進め方は[決め打ちシナリオモード](scenario-mode.md)にある。

### interview

インタビューのモードの設定である。
ペルソナと目的の文書へのパス、試行の回数、閾値、試行の前に回す初期化のコマンドを書く。
項目と既定値は[インタビューのモード](interview-mode.md)にある。
ほかのモードでも、項目の形はYAMLを生の型に読むときに確かめる。
中身の検査と文書の読み込みは、インタビューのモードでだけ行う。

## モードごとの検査

モードによって変わる検査は、設定とモードを受け取る`webqa-core`の関数で行う。
CLIは、起動のときにこの関数を呼ぶ。

| モード | 検査 |
|---|---|
| 決め打ちシナリオ(`run`) | `scenario`が無いか、`ops`が`enter`だけなら設定エラー |
| 探索(`explore`) | `ops`に`click`も`input`も無ければ設定エラー。`scenario`は読まない |
| インタビュー(`interview`) | 探索と同じ検査に加え、`interview`の項目を検査し、ペルソナと目的の文書を読む |
| 再実行(`replay`) | 実行結果に記録したモードの検査 |

探索で`click`も`input`も無いと、要素を操作できず探索が進まない。
`done`だけの一覧もこれに当たる。
決め打ちシナリオで`ops`が`enter`だけだと、禁止要素のある画面では`enter`も外れ、選べる操作が無くなる。

## 環境変数

設定は、`judge.api_key`、`inputs`、`redact`の`env`で環境変数を参照する。
参照した環境変数が無ければ、設定エラーにする。
エラー文には変数の名前だけを書き、値は書かない。

コマンドは起動のときに、作業ディレクトリの`.env`を読む。
`.env`の変数は、まだ設定されていないものだけを足す。
すでにある環境変数の値が勝つ。
`.env`が無ければ何もしない。
`.env`があっても読めないか解釈できないときは、設定エラーにする。
このエラー文には、`.env`の行を含めない。

## 設定エラーと秘密

設定エラーの文には、秘密の値を含めない。
serde-saphyrのエラーは、既定では元のYAMLの抜粋を含む。
そこで、読み込みの`Options`で`with_snippet`を`false`にする。
APIキーの形は、YAMLを生の型に読む前に別に確かめる。
秘密の値を持つ型`Secret`は、`Debug`で中身を出さない。
インタビューの設定エラーの文には、ファイルの中身、パス、コマンドの引数を含めない。
目的の文書の`screens`に書いたURLの型(画面に当てるURLの書式)も含めない。

TLSの検証を無効にする項目は作らない。
未知の項目は設定エラーにするので、そうした項目を書くと起動しない。

主な設定エラーの文は次のとおり。

| 原因 | 文 |
|---|---|
| YAMLを読めない | 設定の YAML を読めない: (理由) |
| APIキーの直書き | API キーを設定ファイルに直接書かず、`{ env: 変数名 }` で環境変数を指定する |
| 環境変数が無い | 環境変数 (名前) が設定されていない |
| 確信度が範囲の外 | (項目) は 0 より大きく 1 以下の値にする |
| 単価が負か数でない | (項目) は 0 以上の値にする |
| 段階が空、または決め打ちシナリオで`scenario`が無い | scenario.stages に段階を1つ以上書く |
| `ops`が空 | ops に操作種別を1つ以上書く |
| URLとして読めない | (項目) を URL として読めない |
| 認証状態の形式が合わない | auth.storage_state を Playwright の保存形式として読めない |
| 探索で要素を操作できない | 探索モードでは ops に click か input を1つ以上書く |
| 決め打ちシナリオで`ops`が`enter`だけ | 決め打ちシナリオでは ops に enter のほかの操作種別を1つ以上書く |
| 課金画面の離脱操作が対象を要する | guard.leave_paywall は back、scroll_down、scroll_up、wait、reload、return のどれかにする |
| `judge.kind`がアダプタの表に無い名前 | 判断モデルの種類 `<名前>` は無い |
| 橋渡しのコマンドが無い | judge.command に橋渡しのコマンドを書く |
| `.env`を読めない | 作業ディレクトリの .env を読めない |
