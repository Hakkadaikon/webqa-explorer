# brief: issue #59 JEV を AI Gateway 経由で呼ぶ橋渡しのコマンド

共通の決まりは同じディレクトリの `common.md`。

## タスク

`bridge/jev.py` に、橋渡しの形式(版 2)の要求の行を JEV の TypeSafe 互換の入口へ写し、答えの行を返すコマンドを書く。

- 起動してすぐに版の行を書く。
- 要求の行を1行ずつ読み、真偽・選択・点数の問いは JEV へ、自由な文の問いは AI Gateway の言語モデルへ送る。
- 答えを版 2 の答えの行に直して書く。失敗は `error` の行にする。

形式は `tasks/briefs/07-judge.md` と `tasks/briefs/19-judge-image-text.md` の「橋渡しの形式」、写し方は `tasks/jev-api.md` の「橋渡しの形式との対応」と、19 の brief の「JEV への写し方(版 2)」。

受け持たないもの:

- 実物の API への接続の確かめ(キーがセッションに無い。親が Network secrets のある環境で行う)。
- 画像の縮小。標準のライブラリでは画像を読めない。表示領域は 1280×720 に固定する決定(14g)で、JEV の上限(1辺 280〜1568、画素数 78,400〜2,458,624)に収まる。収まらない画像は JEV が 400 を返し、`api` の失敗になる。
- 代わりのモデル(`providerOptions`)、`TypeSafe-Danger-Zone-Long-Context`、multipart での画像の送信。

## 触ってよいファイル

- `bridge/jev.py`(新規。実行の権限を付ける)と `bridge/gateway.py`(HTTP、時間の予算、再試行、失敗。jev.py のファイルの Halstead の上限のため分けた)
- `bridge/jev_test.py`(写し方)、`bridge/jev_flow_test.py`(再試行、失敗、全体の流れ)、`bridge/jev_budget_test.py`(時間の予算)、`bridge/fake.py`(偽の HTTP、時計、要求の組み立て)。テストのファイルを分けたのは、ファイルごとの Halstead の上限のため
- `justfile`: Python の整形、静的解析、テストの対象に `bridge` を足すだけ。閾値と設定は変えない。
- `scripts/metrics.sh`: 複雑度の対象に `bridge` を足すだけ。
- `docs/quality-gates.md`、`docs/architecture.md`: 対象の範囲と、橋渡しのコマンドの引数を書くだけ。

## 決めてある設計

### 起動と引数

`bridge/jev.py [--model <名前>] [--text-model <名前>] [--timeout <秒>]`。設定には `judge.command: ["./bridge/jev.py", "--text-model", "…"]` のように書く。

| 引数 | 既定 | 意味 |
|---|---|---|
| `--model` | `typesafe-ai/jev-1.14.0` | JEV のモデル。再現のため版を固定した名前を既定にする(`tasks/jev-api.md` の対応しないところ11) |
| `--text-model` | なし | 自由な文の問いを送る言語モデル。無ければ自由な文の問いを含む要求は `api` の失敗 |
| `--timeout` | `30` | 要求の1行あたりの時間の予算(秒)。0以下と数でない値は引数の誤りとして拒む |

- 版の行は `{"version":2,"images":<真偽>}`。`images` は `--model` が `typesafe-ai/jev-1.13.0` か `typesafe-ai/jev-1.14.0` のときだけ `true`。
- API キーは環境変数 `WEBQA_JUDGE_API_KEY`。あれば `Authorization: Bearer <キー>` を付け、無ければ付けない(Network secrets の中継が足す)。
- 標準エラーには何も書かない。標準出力は1行ごとに flush する。
- 標準入力の終わりで終了コード0で終わる。

#### 時間の予算(親の決定)

- 要求の行を読んだ時刻に、`--timeout` の秒数だけ先を期限にする。時計は `handle` の引数 `now` で差し替えられる(既定は `time.monotonic`)。
- HTTP の呼び出しのたびに、`urlopen` には期限までの残りの時間を渡す。残りが0以下なら呼ばずに `timeout` の失敗にする。
- 自由な文の問いの呼び出しも、同じ期限の内に入れる。
- 429 と 529 の再試行は、待つ秒数が残りの時間より短いときだけ行う。足りなければ再試行せず、そこまでの失敗(`http 429` など)を返す。

## 要求の写し方

入口は `https://ai-gateway.vercel.sh/typesafe/v1/systemone`。本体は `{"model", "state", "questions"}`(画像があれば `assets`)。

| 版 2 の問い | JEV の問い |
|---|---|
| `{"kind":"noul","id","prompt"}` | `{"type":"noul","instructions":prompt}` |
| `{"kind":"choice","id","prompt","options"}` | `{"type":"choice","instructions":prompt,"criteria":{選択肢: null}}` |
| `{"kind":"score","id","prompt","levels"}` | `{"type":"score","instructions":prompt,"criteria":levels}` |
| `{"kind":"text","id","prompt"}` | JEV に送らない(下の自由な文) |

- 画像が無ければ `state` は `screen` の文字列。
- 画像があれば、参照 `<uuid4>-shot` を作り、`state` を `{"screen": screen, "shot": 参照}`、`assets` を `{参照: {"type":"image","format":"png" か "jpeg","data_base64": data}}` にし、ヘッダー `TypeSafe-Danger-Zone-Images: true` を付ける。`media_type` の `image/png` は `png`、`image/jpeg` は `jpeg`。
- JEV に送る問いが1つも無ければ、JEV を呼ばない。

### 自由な文の問い

入口は `https://ai-gateway.vercel.sh/v1/chat/completions`(OpenAI 互換)。問い1つにつき1回送る。
本体は `{"model": --text-model, "messages":[{"role":"user","content": prompt + "\n\n画面:\n" + screen}]}`。
答えの文は `choices[0].message.content`。文字列でなければ `api` の失敗。画像は送らない(shortcut。インタビューで画像の要る問いが出たら足す)。

### 答えの写し方

- Choice: 要求の `options` の順に `{"option": o, "p": probabilities[o]}`。`probabilities` に無い選択肢は `p` を省く。
- Noul: `[{"option":"true","p": noul}]`。
- Score: 段の番号の順に `{"option": "0", "p": probabilities["0"]}`、…。無い段は `p` を省く。
- Text: `[{"option": 答えの文}]`。
- `usage` は JEV の `input_tokens`、`output_tokens` と、言語モデルの `prompt_tokens`、`completion_tokens` を足した数。
- 答えの行は `{"version":2,"seq":seq,"answers":{…},"usage":{"input_tokens":…,"output_tokens":…}}`。

### 失敗

| 起きたこと | 答えの行 |
|---|---|
| HTTP の 429 か 529 | `retry-after`(整数の秒。無いか読めなければ1、0〜5に収める)だけ待って再試行する。待つ秒数が期限までの残りに足りなければ再試行しない。再試行は2回まで。3回目も 429 か 529 なら下の行 |
| HTTP の 200 以外 | `{"error":{"kind":"api","message":"http <状態コード>"}}` |
| つながらない、応答の途中で切れた(`URLError`、`OSError`、`http.client.HTTPException`) | `{"error":{"kind":"api","message":"network"}}` |
| 待ち時間切れ(`TimeoutError`、`URLError` の中の `TimeoutError`)、期限までの残りが無い | `{"error":{"kind":"timeout","message":"timeout"}}` |
| 応答が JSON として読めない、答えに問いが欠けている、形が違う | `{"error":{"kind":"api","message":"malformed response"}}` |
| `--text-model` が無いのに自由な文の問いがある | `{"error":{"kind":"api","message":"no text model"}}` |
| 要求の行が JSON として読めない、`seq` が無い | 何も書かない(中核は待ち時間切れで扱う) |

`message` にキー、画面の文字列、応答の本体を入れない。

## 失敗させるテスト(1〜7 は `bridge/jev_test.py`、8〜13 は `bridge/jev_flow_test.py`、14〜18 は `bridge/jev_budget_test.py`)

HTTP は `jev.urlopen` を偽物に差し替え、ネットワークに出ない。待つことは `jev.sleep` を差し替える。期待値は `assertEqual` で値ごと比べる。

1. `test_declaration_by_model`: 既定のモデルと `typesafe-ai/jev-1.13.0` は `{"version":2,"images":true}`、`typesafe-ai/jev` は `{"version":2,"images":false}`。
2. `test_questions_map_to_jev`: noul、choice、score の3問で、送る本体の `questions` が上の表のとおり。`model` は引数の名前、`state` は画面の文字列、`assets` のキーが無い。
3. `test_answers_map_back`: 応答2(`tasks/jev-api.md`)の形を返すと、choice は options の順、noul は `true` の1つ、score は段の順、`usage` は `{"input_tokens":426,"output_tokens":100}`。`probabilities` に無い選択肢は `p` を省く。
4. `test_image_goes_to_assets`: `image` のある要求で、`state` が `{"screen","shot"}`、`assets` の参照が `shot` と同じで `-shot` で終わり、`format` が `png`、`data_base64` が要求の `data`。ヘッダーに `Typesafe-danger-zone-images: true`。jpeg は `jpeg`。
5. `test_key_header_only_with_key`: `WEBQA_JUDGE_API_KEY` があれば `Authorization` が `Bearer <キー>`、無ければヘッダーが無い。
6. `test_text_question_goes_to_language_model`: text の1問で、chat completions の本体が上の形、答えが `[{"option": 文}]`、JEV を呼ばない。使用量は2つを足した数。
7. `test_text_without_model_is_api_error`。
8. `test_no_questions_makes_no_call`: `questions: []` は `answers: {}`、使用量0、HTTP を呼ばない。
9. `test_retries_busy_then_answers`: 429(`retry-after: 2`)、529(無し)のあと 200 なら答えを返し、待った秒は `[2, 1]`。`retry-after: 99` は5、`soon` は1、`-3` は0。
10. `test_gives_up_after_two_retries`: 429 が3回続くと `http 429`。
11. `test_failures_become_error_lines`: 400、`URLError`、`TimeoutError`、JSON でない応答、問いが欠けた応答が、上の表の行になる。`message` に画面の文字列もキーも含まない。
12. `test_bad_request_line_is_skipped`: JSON でない行と `seq` の無い行は何も書かず、次の行に答える。
13. `test_main_loop`: `main` が版の行を書き、要求ごとに1行を書き、入力の終わりで0を返す。スクリプトとして走らせても同じ(`runpy`)。

14. `test_each_call_gets_what_is_left`: `--timeout 30`、時計が 100、110、112、115 と進むと、429(`retry-after: 2`)のあとの2回の `urlopen` の待ち時間は `[20, 15]`、待った秒は `[2]`。
15. `test_no_retry_when_the_wait_does_not_fit`: 残りが4秒のときの 429(`retry-after: 5`)は待たずに `http 429`。要求は1回。
16. `test_text_questions_share_the_budget`: 真偽と自由な文の2問で、時計が 0、5、12 なら待ち時間は `[25, 18]`。
17. `test_no_time_left_is_a_timeout`: 残りが0なら HTTP を呼ばずに `timeout`。
18. `test_timeout_must_be_more_than_zero`: `0.5` は受け、`0`、`-1`、`nan`、`soon` は終了コード2で拒む。

ほかに、レビューで足したもの: 失敗の表に `URLError(TimeoutError())`(timeout)、`ConnectionResetError`、`RemoteDisconnected`、`IncompleteRead`(network)。`test_main_loop` は答えの行を丸ごと比べる。`test_text_request_has_no_image_header`: 画像のある要求でも、自由な文の問いの要求に `TypeSafe-Danger-Zone-Images` を付けない。

## 親に決めてほしいこと

1. 自由な文の言語モデルの既定。案(推す): 既定を置かず、設定の `judge.command` の引数で `--text-model` を必ず書かせる。AI Gateway のモデルの名前と値段は変わるため。
2. 自由な文の問いに画像を送るか。案(推す): 今は送らない。インタビューのモード(#30、#31)で要ると分かったら足す。
