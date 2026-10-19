# brief: issue #14m 受け入れ: 設定と外部境界、利用者による中断、終了コード

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14h が入ってから始める。** 14h の世界(`QaWorld`、`Scene`、`PageSpec`、`Reply`)、共通の手順(`steps/common.rs`)、設定の補助(`support/setting.rs` の `BASE`、`configure`、`configured`)、中断と保存の偽物(`Scene.fire_on`、`user_interrupts`、`stall`、`store_limit`、`store_hold`)、`Scene.marks` を使う。始める前に main の `tests/support/` と `tests/steps/common.rs` を読み、補助の名前と意味がこの brief と違えば NEEDS_CONTEXT で返す。

並行して 14i、14j、14k、14l が回る。触るファイルは重ならない。

## タスク

feature の3つの Rule のシナリオ24本を、受け入れのテストとして通す。

- 設定と外部境界(13本。T-050 は 14h で入った): T-043、T-044、T-045、T-046、T-047、T-065、T-066、T-067、T-068、T-048、T-049、T-112、T-051
- 利用者による中断(6本): T-097、T-098、T-099、T-100、T-101、T-102
- 終了コード(5本): T-103、T-104、T-105、T-106、T-107

`14-plan.md` の表の「25本(T-050 を除く)」は、T-050 を含めた数の書き違い。T-050 を除くと24本になる。

下の層のテストは、それぞれの issue で入っている。ここは受け入れの配線だけ。

## 触ってよいファイル

- `crates/webqa/tests/steps/config.rs`、`steps/interrupt.rs`、`steps/exit.rs`(14h が空の `READY` で作ったもの)
- 分けるときの新規: `steps/config_limits.rs`、`steps/exit_scenes.rs`(`acceptance.rs` に `#[path]` の行と `READY` の連結を足してよい。この brief で足したファイルの分だけ)

触らないもの: `steps/common.rs`、`steps/boundaries.rs`(T-050)、ほかの Rule のファイル、`tests/support/`(補助が足りなければ NEEDS_CONTEXT で返す)、`src/` の下、feature の写し。

## 決めてある設計

- 共通の手順(14h の表)の文言は書かない。下の「書く手順」の文言だけを書く。パラメータは `{string}`、`{int}`、`{float}` と、14h が定めた `{reason}` だけ。
- 手順の関数は、場面を組み立てるか、世界の補助を呼んで `assert_eq!` で比べる1〜3行にする。手順の中に `if` を書かない。Examples の値で振る舞いが変わる手順は、値から場面への写し方を表(配列)にして引く。
- 設定から始める場面は、`world.configure(from, to)` で `BASE` を書き換える(`scene.yaml` が `Some` のとき、規則と伏せ字は `setup::load` から作られる。14h)。
- 終了コードは `world.ran().code`(14f の `exit::code`。設定エラーは `exit::CONFIG_ERROR`)。

### 書く手順(`steps/config.rs`。設定と外部境界)

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `YAML の設定に対象 URL、入力値、2つの段階とそれぞれの done_when が書かれている` | Given | `configure` で `BASE` の `scenario:` の節を、段階 "ログイン"(`url: /home`)と "投稿"(`text: 投稿しました`)の2つにする。開始の画面に `button("ログイン")`、`Reply::op("click", 0.9).on("ログイン")` を積み、`after` に `/home`(`button("投稿")`)を積む |
| `その設定で決め打ちシナリオを実行する` | When | `mode = Scenario` で `world.run()` |
| `1つ目の段階の done_when を満たしたあと、2つ目の段階へ進んでいる` | Then | `stage_advanced` の行があり `name` が `"投稿"`、`state().stage()` が 1。その行は "ログイン" の `executed` の行より後 |
| `同じ設定で決め打ちシナリオ実行と、{int} 秒の探索モードをそれぞれ起動する` | When | `configured()` で `BASE` から始める。決め打ちシナリオで `world.run()` し、`config_error` が `None` であることを確かめて `marks["scenario"] = 1`。`world.ran = None` にし、`mode = Explore`、`configure("  error_confidence: 0.8\n", "  error_confidence: 0.8\n  max_duration_secs: n\n")` で制限時間を足し、開始の画面に `button("保存")`、`Reply::op("click", 0.9).on("保存").at(n 秒)` を積んで `world.run()` |
| `どちらも同じツールで実行でき、探索モードは遅くとも {int} 秒の時点で終わる` | Then | `marks["scenario"]` が 1、探索モードの `config_error` が `None`、`reason()` が `TimeExceeded`、`ran().clock.elapsed()` が n 秒以下 |
| `設定にログイン済みのセッション情報が指定されている` | Given | `configured()`(`BASE` の `auth.storage_state` は `./auth.json`。14h の読み込みの偽物が `{"cookies":[],"origins":[]}` を返す)。開始の画面に `button("ログアウト")` を置く(ログイン済みの画面) |
| `探索モードを起動する` | When | `mode = Explore` にして `world.run()` |
| `最初の画面状態はログイン済みの画面である` | Then | `config_error` が `None`、`judge.texts[0]` に `"ログアウト"` の行がある |
| `設定で {string} を伏せる対象に指定している` | Given | `scene.redact` に `Rule::pattern(name, BASE と同じメールアドレスの正規表現)` を足す |
| `対象ページに {string} が表示されている` | Given | 開始の画面に `link(値)` を足す |
| `判断モデルに送った画面状態に {string} は含まれない` | Then | `judge.texts` のどれにも値が無い |
| `判断モデルの API キーが環境変数で与えられ、設定ファイルには参照だけが書かれている` | Given | `configured()`(`BASE` の `api_key` は `{ env: JEV_API_KEY }`、環境に値がある)。`Reply::op("wait", 0.9)` を積む |
| `判断モデルの呼び出しに成功している` | Then | `config_error` が `None`、`answered()` が 1 |
| `1ステップを実行し、実行結果を出力する` | When | `Reply::op("click", 0.9).on_number(1)` を積んで `world.run()`(画面の文字列では値が伏せてあるので、名前では番号を引けない) |
| `実行結果の出力とログに {string} は含まれない` | Then | `report()` を文字列にしたものと、`lines()` の各行を文字列にしたもののどれにも値が無い。`report()` は `Some` |
| `エラーの表示に API キーの値は含まれない` | Then | `config_error` に `"sk-live-123"` が無い |
| `起動は設定エラーで失敗し、判断モデルを呼び出していない` | Then | `config_error` が `Some`、`code` が 2、`asked()` が 0 |
| `設定のセッション情報が読み込めない形式である` | Given | `configure("storage_state: ./auth.json", "storage_state: ./broken.json")`(14h の読み込みの偽物は `./auth.json` しか読めない) |
| `設定の {string} が {string} である` | Given | 下の表で項目を YAML の行に写し、値に置き換える。`使う操作種別` は、YAML の末尾に `ops: <値>` を足す |
| `同じ設定で決め打ちシナリオを起動しようとしても、設定エラーにならない` | Then | `world.ran = None`、`mode = Scenario` にして回し直し、`config_error` が `None` |
| `{int} ステップを実行して終了する` | When | `scene.think` を 200ミリ秒、`max_decisions` を n にし、`button("保存")`、`button("ヘルプ")`、`button("設定")` と、それぞれの `click` を積んで `world.run()`(n 回の問い合わせのあと判断予算切れで終わる) |
| `実行記録にステップごとの判断・操作・画面変化待ちの時間が含まれる` | Then | `report()` の `calls` が n 件でどれも `think_ms` が 200、`steps` が n 件でどれも `act_ms` と `settle_ms` を持つ |
| `実行記録に判断モデルの呼び出し回数、トークン数、算出したコストが含まれる` | Then | `report()` の `usage` が `{"calls": n, "input_tokens": 0, "output_tokens": 0, "cost": 0.0}`(偽の判断モデルの使用量は0。単価からの計算は単体の `cost_from_usage_and_pricing` が受け持つ) |

T-049 と T-112 の項目の写し方(`設定の {string} が {string} である` の1本で受ける):

| 項目 | 置き換える行 |
|---|---|
| 固まり判定の閾値 | `freeze_threshold: 3` |
| 行き詰まり判定の閾値 | `stuck_threshold: 3` |
| 判断モデルの呼び出し上限 | `max_decisions: 20` |
| 1時間窓あたりの操作回数の上限 | `requests_per_hour: 10` |
| ページの応答の待ち時間(秒) | `page_timeout_secs: 10` |
| 判断モデルの応答の待ち時間(秒) | `timeout_secs: 30` |
| 使う操作種別 | (末尾に `ops: <値>` を足す) |

T-048(TLS)、T-066、T-067 の Given と When と最初の Then は共通の手順で、ここで書くのは上の表の Then だけ。

### 書く手順(`steps/interrupt.rs`。利用者による中断)

中断の偽物(14h): `wait` は呼ばれた番号が `fire_on` にあればすぐ終わり、無ければ門(gate)を待つ。判断モデルの答えが尽きる、ブラウザが応答しない(`Stall::Touch`)、窓の待機が終わらない(`Stall::Sleep`)、記憶の書き出しが終わらない(`Hold::Forever`)のどれかで門が開く。実行ループは効果1つごとに `wait` を1回呼ぶ。

`marks["fire"]` に、中断の番号を入れる。`利用者が実行を中断する` は `user_interrupts = true` にし、`fire_on` に `marks["fire"]`(無ければ1)を足して `world.run()` する。門で中断する場面は、`marks["fire"] = 0`(0番の `wait` は無いので、`fire_on` では中断しない)を入れる。

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `3 ステップを実行し、探索済みの要素は {string} である` | Given | 開始の画面に `button(name)`、`button("ヘルプ")`、`button("設定")`、`button("検索")` を置き、`name`、`"ヘルプ"`、`"設定"` の `click` を積む(14f の台本と同じ。探索は操作済みの要素を選択肢から外すので、同じ要素を3回押さない) |
| `次のステップの開始を待っている` | Given | `marks["fire"] = 10`(3ステップの効果が9つで、10番目の `wait` が次のステップの開始。14f の `interrupt_cancels_wait_and_writes_records` と同じ数え方。数が違えば、その台本の数え方に合わせる) |
| `判断モデルの応答を待っている` | Given | `marks["fire"] = 0`(4回目の問い合わせで答えが尽き、門が開く) |
| `時間窓が明けることを待っている` | Given | `requests_per_hour` を 3、`stall = Stall::Sleep`、`marks["fire"] = 0` |
| `利用者が実行を中断する` | When | 上のとおり |
| `保存した探索の記憶に {string} が含まれる` | Then | `explored()` が name を含む |
| `イベントログに終了のイベントが書かれている` | Then | `kinds()` の最後が `"run_finished"` |
| `実行結果が出力されている` | Then | `report()` が `Some` |
| `判断モデルが操作 {string} と対象要素 {string} を返した` | Given | 開始の画面に `button(name)`、`Reply::op(op, 0.9).on(name)` を積む |
| `{string} をクリックしたあと、ページの応答を待っている` | Given | `stall = Stall::Touch`、`marks["fire"] = 0` |
| `ページの応答の待ち時間が過ぎるのを待たずに、実行は利用者による中断として終了している` | Then | `reason()` が `UserAborted`、`ran().clock.elapsed()` が `page_timeout`(10秒)より短い |
| `探索の記憶、終了のイベント、実行結果が書かれている` | Then | 記憶のファイルがあり、`kinds()` の最後が `"run_finished"`、`report()` が `Some` |
| `実行が利用者による中断として終了している` | Given | `user_interrupts = true`(答えを積まないので、最初の問い合わせで門が開く) |
| `さらに1ステップを実行しようとし、時刻も進む` | When | `world.run()` のあと `world.poke(3600秒)` |
| `終了理由は利用者による中断のままである` | Then | `poked` の `reason()` が `UserAborted` |
| `探索の記憶は書き出したときのままである` | Then | `poked` の状態の記憶を `to_json(origin)` にした値が、保存した記憶の JSON の値と等しい |
| `実行中に記録した問題は 無い である` | Given | `Reply::op("wait", 0.9)` を積む |
| `実行中に記録した問題は エラー表示の問題1件 である` | Given | `Reply::op("wait", 0.9).error(0.9)` を積む |
| `実行中に記録した問題は 参考情報1件だけ である` | Given | `scene.probes = true`、`Reply::op("wait", 0.9).confusing(0.9)` を積む |
| `利用者が実行を中断し、記録を書き終えて終わる` | When | `user_interrupts = true` にして `world.run()`(次の問い合わせで答えが尽き、門で中断する) |
| `利用者が実行を中断し、記録を書いている` | Given | `user_interrupts = true`、`store_hold = Hold::Forever`(1回目の中断は答えが尽きたときの門。記憶の書き出しが終わらず、もう一度門が開く) |
| `書き終わる前に、利用者がもう一度中断する` | When | `world.run()` |
| `記録を書き終わるのを待たずに終わっている` | Then | `ending` が `Ending::Abandoned`、`report()` が `None` |

T-100 は、Given(`実行がゴール達成として終了している`)と Then(`終了理由はゴール達成のままである`)が共通の手順。開始の画面でゴールに達するので実行ループは `wait` を呼ばず、`marks["fire"]` の既定の1番が記録を書く間の中断になる。ゴールで終わった実行は1回目の中断を受け流す(14f の決定)。

### 書く手順(`steps/exit.rs`。終了コード)

終了理由の場面を表 `ENDINGS: [(ExitReason, fn(&mut Scene))]` にする。どの場面も、開始の画面に `button("保存")` を置き、最初の答えを "保存" の `click` にし、`after` に開始の画面の写しを1つ積む(問題の場面が、この最初の答えと画面を飾る)。

| 終了理由 | 場面 |
|---|---|
| ゴール達成 | 段階 `[("ログイン", "ログインする", text "ようこそ")]`、`after` の画面のテキストを `"ようこそ"` |
| 未検証の終了 | 段階は上と同じ(満たさない)、2つ目の答えに `Reply::op("done", 0.9).goal(0.9)` |
| 行き詰まり | 2つ目の答えに `Reply::op("stuck", 0.9)` |
| 判断予算切れ | `max_decisions` を 1 |
| 時間切れ | `max_duration` を60秒、最初の答えに `.at(60秒)` |

問題の場面を表 `ISSUES: [(&str, fn(&mut Scene))]` にする。

| 問題 | 場面 |
|---|---|
| エラー表示の問題1件 | 最初の答えに `.error(0.9)` |
| 読み上げ名の問題1件 | 開始の画面と `after` の画面に `button("icon_save.png")` を足す |
| コンソールエラーの問題1件 | `after` の画面の `console` に `"TypeError: x is undefined"` |
| 許可ドメインの外への遷移の記録1件 | 開始の画面に `link("外部の記事")`、最初の答えの対象を "外部の記事" にし、`after` の先頭に外の画面(`origin: Some("https://other.example.net")`、`path: ""`、`button("記事")`)を入れる(中核が戻って開始の画面の写しに来る。14k の T-027 と同じ作り方) |
| 参考情報1件だけ | `scene.probes = true`、最初の答えに `.confusing(0.9)` |

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `実行が{reason}として終了し、確定の問題は記録されていない` | Given | `ENDINGS` から場面を引く |
| `ツールが終わる` | When | `world.run()` のあと、`reason()` が Given の理由であることを確かめる |
| `実行が{reason}として終了し、エラー表示の問題1件が記録されている` | Given | 終了理由の場面のあと、問題の場面 |
| `実行が{reason}として終了し、読み上げ名の問題1件が記録されている` | Given | 同上 |
| `実行が{reason}として終了し、コンソールエラーの問題1件が記録されている` | Given | 同上 |
| `実行が{reason}として終了し、許可ドメインの外への遷移の記録1件が記録されている` | Given | 同上 |
| `実行がゴール達成として終了し、記録した検出は参考情報1件だけである` | Given | ゴール達成の場面のあと、参考情報の場面 |
| `設定の固まり判定の閾値が 0 である` | Given | `configure("freeze_threshold: 3", "freeze_threshold: 0")` |
| `判断モデルを呼び出しておらず、対象ページも開いていない` | Then | `asked()` が 0、`ran().browser.observed` が 0、`touched()` が空 |
| `実行の記録の出力先が 空きの無いディスク である` | Given | `store_limit` を `Some(1)`(最初の `run_started` の行だけ書ける) |
| `実行の記録の出力先が 書き込み権限の無いディレクトリ である` | Given | 同上 |
| `1ステップを実行し、その記録を書こうとする` | When | `Reply::op("wait", 0.9)` を積んで `world.run()` |
| `実行はそれ以上ステップを進めずに止まっている` | Then | `ending` が `Ending::Failed(_)`、`asked()` が1以下、`touched()` が空、`report()` が `None` |

`ツールが終わる` で理由を確かめるのは、問題の場面が終わり方を変えていないことを保つため(問題を足したせいで別の理由で終わると、終了コードの確かめが空振りする)。理由が合わなければ、その組み合わせの場面を NEEDS_CONTEXT で返す。

## 失敗させるテスト

受け入れの24本(Examples の各行を含む)を、この brief の3つのファイルの `READY` に名前で足し、`cargo test -p webqa --test acceptance` で通す。名前は feature の Scenario の題名そのまま。

| Rule | ファイル | 題名(台帳の番号) |
|---|---|---|
| 設定と外部境界 | `config.rs` | T-043、T-044、T-045、T-046、T-047、T-065、T-066、T-067、T-068、T-048、T-049、T-112、T-051 |
| 利用者による中断 | `interrupt.rs` | T-097、T-098、T-099、T-100、T-101、T-102 |
| 終了コード | `exit.rs` | T-103、T-104、T-105、T-106、T-107 |

先に名前だけを足して `fail_on_skipped` で失敗すること(Red)を確かめてから手順を書く。

## 受け入れ条件

- 触ったモジュールのテスト(`cargo test -p webqa --test acceptance`)、`just fmt-check`、`just clippy` とコミットのフックが通る。全体のテストとカバレッジ100%は、PR の CI で確かめる。
- 24本が `READY` にあり、走って通る。
- 共通の手順の文言をこの brief のファイルに書いていない。統合のとき、ほかの Rule のファイルとあいまいな手順にならない(親が全シナリオを回して確かめる)。
- 触ってよいファイルの外に差分が無い。

## 親に決めてほしいこと

1. **T-045 の認証状態の注入を、設定が通ることと最初の画面で確かめること**。偽のブラウザは保存領域を受け取らない。ログイン済みのページが開くことは、ブラウザの契約のテスト(`storage_state_opens_logged_in_page`)と 14g の実行ファイルの層が受け持つ。
2. **T-047 で、API キーが判断モデルに届くことを見ないこと**。偽の判断モデルはキーを受け取らない。キーが橋渡しに届くことは `webqa-judge` の `api_key_env_reaches_bridge` が受け持つ。
3. **T-068 の「読み込めない形式」を、読み込みの偽物が読めない保存領域のパスで作ること**。中身が壊れた保存領域を拒むことは、#3 の設定の単体のテストが受け持つ。案B は、14h の読み込みの偽物に壊れたファイルを足すこと(`tests/support/` を変えるので推さない)。
4. **T-044 を、1つの世界で2回回して確かめること**。1回目の結果は `marks` に残し、`world.ran` を捨てて回し直す。60秒の時間切れは、答えの `at` で時計を進めて作る。
5. **中断の番号の既定を1番にすること**(`marks["fire"]` が無いとき)。T-100 のように実行ループが `wait` を呼ばない場面で、記録を書く間に中断が来る。
6. **T-107 の2行を、どちらも保存の書き込みの上限で作ること**。空きの無いディスクと権限の無いディレクトリの違いは、14c の保存のテスト(`full_disk_keeps_previous_memory` ほか)が受け持つ。
7. **T-104 の4行目(時間切れと外への遷移)の場面**。外の画面から中核が戻るあいだに時刻が60秒に達し、遷移の記録のあとに時間切れで終わる組み立てにした。理由が時間切れにならなければ、外の画面への `click` を2つ目の答えにずらす。
