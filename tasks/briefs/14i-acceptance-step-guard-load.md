# brief: issue #14i 受け入れ: 1ステップの問い合わせと実行、ガードレール、対象サーバへの負荷

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14h が入ってから始める。** 14h の世界(`QaWorld`、`Scene`、`PageSpec`、`ElementSpec`、`Reply`、`Aim`)と共通の手順(`steps/common.rs`)を使う。始める前に main の `tests/support/` と `tests/steps/common.rs` を読み、補助の名前と意味がこの brief と違えば NEEDS_CONTEXT で返す。

並行して 14j、14k、14l、14m が回る。触るファイルは重ならない。

## タスク

feature の3つの Rule のシナリオ23本を、受け入れのテストとして通す。

- 1ステップの問い合わせと実行(7本): T-001、T-002、T-003、T-004、T-055、T-056、T-072
- ガードレール(11本): T-005、T-006、T-007、T-113、T-008、T-009、T-010、T-011、T-012、T-013、T-014
- 対象サーバへの負荷(5本): T-035、T-036、T-053、T-071、T-054

下の層のテスト(台帳のテスト名)は、それぞれの issue で入っている。ここは受け入れの配線だけ。

## 触ってよいファイル

- `crates/webqa/tests/steps/step.rs`、`steps/guard.rs`、`steps/load.rs`(14h が空の `READY` で作ったもの)
- 1つのファイルが Halstead のファイルの上限(体積8000)に近づいたら、`steps/guard_pages.rs` のように新しいファイルに分け、`acceptance.rs` に `#[path]` の1行を足してよい(この brief で足したファイルの分だけ)

触らないもの: `steps/common.rs`、ほかの Rule のファイル、`tests/support/`(補助が足りなければ NEEDS_CONTEXT で返す)、`src/` の下、feature の写し。

## 決めてある設計

- 共通の手順(14h の表)の文言は書かない。下の「書く手順」の文言だけを書く。パラメータは `{string}`、`{int}`、`{float}` だけ。
- 手順の関数は、場面(`world.scene`)を組み立てるか、世界の補助を呼んで `assert_eq!` で比べる1〜3行にする。手順の中に `if` を書かない。Examples の値で振る舞いが変わる手順は、値から場面への写し方を表(配列)にして引く。
- 開始の画面は、最初の許可ドメインの `/`(`scene.start`)。Given が足す要素は `scene.start` に足す。
- 「1ステップを実行する」の結果は 14h のとおり: Given で積んだ答えのあと、次の問い合わせで判断モデルが待ち続け、土台が止める。止めた実行は「終了していない」。When の1ステップは `world.when_step()`。

### 書く手順(`steps/step.rs`。1ステップの問い合わせと実行)

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `対象ページに操作可能なボタン {string} とリンク {string} がある` | Given | `button(a)` と `link(b)` を足す |
| `判断モデルに送った画面状態はスクリーンショットではなくアクセシビリティ情報のテキストである` | Then | `judge.texts[0]` が、表示領域の要素を `n: role "name"` の行にした文字列(`"1: button \"保存\"\n2: link \"ヘルプ\""`)と等しい。画像つきの問い(`ask_with_image`)は呼ばれていない(`SceneJudge` は `ask` だけを数える) |
| `その画面状態には {string} と {string} が操作可能要素として含まれる` | Then | `judge.texts[0]` の行の名前の列に2つがある |
| `決め打ちシナリオを実行している` | Given | `mode` は `Scenario`、段階 `[("ログイン", "ログインする", url "/home")]`(満たさない) |
| `判断モデルに次の4項目を問い合わせている` | Then | `judge.questions[0]` の名前の列が `["operation", "target", "error_shown", "goal_reached"]`(表の4行を名前に写して比べる)。`operation` の `Choice` の選択肢は `["click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"]`、`target` は `Choice`(番号)、`error_shown` と `goal_reached` は `Noul`(真偽) |
| `{string} は禁止要素に分類されておらず、実行の直前にも存在する` | Given | 名前の要素を足す。直前に積んだ答えの操作が `input` なら `textbox(name)`、ほかは `button(name)` |
| `ブラウザで操作 {string} を実行している` | Then | `touched()` の when_step の操作が `op` |
| `探索済みの要素は {string} だけ である` | Then | `explored()` が `{name}`(Examples の `"保存" だけ` を `<探索済み>` に入れた文。`だけ` と `である` の間に半角空白が入る) |
| `探索済みの要素は 空 である` | Then | `explored()` が空(`空` の前後に半角空白が入る) |
| `直前のステップで {string} をクリックしたが画面が変化しなかった` | Given | `button(name)` と `link("ヘルプ")` を足し、`Reply::op("click", 0.9).on(name)` を積む(画面は変わらない) |
| `判断モデルが対象要素の候補として {string} を {float}、{string} を {float} の確信度で返す` | Given | `Reply::op("click", 0.9).on_ranked(a, pa).on_ranked(b, pb)` を積む |
| `{string} はクリックしていない` | Then | when_step の操作の対象が `name` でない |
| `対象ページに操作可能な要素が1つも無い` | Given | 開始の画面の要素を空にする |
| `判断モデルに送った画面状態の操作可能要素の一覧は空である` | Then | `judge.texts[0]` が `""` |
| `ブラウザで {string} を実行している` | Then | `touched()` に `(op, None)` がある |
| `判断モデルの応答が {word…}` | — | 下の T-056 の表。Examples の6行を、文言ごとの Given 6本にする(`判断モデルの応答が 存在しない要素番号 99 を対象にした click である` など、行の文をそのまま書いた手順) |
| `ブラウザ操作をしていない` | Then | `touched()` が空 |
| `応答の異常として記録されている` | Then | `kinds()` に `"anomaly"` がある |
| `判断モデルの呼び出し回数は 1 増えている` | Then | `answered()` が 1 |
| `対象ページの表示領域にボタン {string} があり、表示領域より下にリンク {string} がある` | Given | `button(a)`、`link(b).below()` |
| `判断モデルに送った画面状態の操作可能要素の一覧に {string} が含まれる` | Then | `judge.texts[0]` の名前の列に `name` がある |
| `その一覧に {string} は含まれない` | Then | 名前の列に `name` が無い |

T-003 の場面: Examples の各行で、`Reply::op(<操作>, 0.9).on("保存")`(共通の手順)。`scroll_down` などの対象の要らない操作では、`target` の答えは読まれない(中核は要素の要らない操作の対象を読まない)。`back` の 0.9 は閾値 0.7 以上なので通る。

T-056 の6行の場面:

| 行 | 場面 |
|---|---|
| 存在しない要素番号 99 を対象にした click | `button("保存")`、`Reply::op("click", 0.9).on_number(99)` |
| 操作可能な要素が1つも無い画面で返った click | 要素なし、`Reply::op("click", 0.9).on_number(1)` |
| 8種のどれでもない操作種別 "drag" | `button("保存")`、`Reply::op("drag", 0.9)` |
| 必要な項目が欠けた形式の崩れた応答 | `button("保存")`、`Reply::malformed()` |
| API のエラー応答 | `button("保存")`、`Reply::Fail(JudgeError::Api)` |
| 設定の待ち時間内に返らないタイムアウト | `button("保存")`、`Reply::Fail(JudgeError::Timeout)`(判断モデルの橋渡しの時計は使わない。待ち時間切れは偽物が `Timeout` で返す。申し送りの「時計を揃える」は、この形で要らなくなる) |

### 書く手順(`steps/guard.rs`。ガードレール)

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `「戻る」の確信度の閾値は {float} に設定されている` | Given | `back_confidence` |
| `判断モデルが操作 {string} を確信度 {float} で返す` | Given | `Reply::op(op, p)` |
| `ブラウザで「戻る」を実行している` | Then | `touched()` に `("back", None)` がある |
| `ブラウザで「戻る」を実行していない` | Then | `touched()` に `"back"` が無い |
| `判断モデルが操作 {string} を確信度なしで返す` | Given | `Reply::op_without_p(op)` |
| `対象ページに {要素} がある` | Given | 下の T-007 の表の11行を、文言ごとの Given にする(`対象ページに ボタン "購入する" がある` は `対象ページに ボタン {string} がある` の1本で受けてよい。ほかの行は行の文をそのまま書く) |
| `ブラウザで {string} を操作していない` | Then | `touched()` の対象に `name` が無い |
| `{string} は探索済みの要素に含まれない` | Then | `explored()` に `name` が無い |
| `対象ページに 許可リストにも一致するが名前が "購入" を含むボタン {string} がある` | Given | `button(name).allowed_listed()` |
| `判断モデルが操作 {string} と対象要素 {string} を返す` | Given | `button(name)` と `link("ヘルプ")` を足し、`Reply::op(op, 0.9).on(name)` |
| `問い合わせから実行までの間に {string} がページから消える` | Given | `looks` に、その要素を除いた開始の画面を積む |
| `ブラウザで何もクリックしていない` | Then | `touched()` に `"click"` が無い |
| `判断モデルが操作 {string} とボタン {string} を返す` | Given | `button(name)` と `link("ヘルプ")`、`Reply::op(op, 0.9).on(name)` |
| `問い合わせから実行までの間に、同じ位置の {string} がボタンからリンクに差し替わる` | Given | `looks` に、その要素を `link(name).node("e9")` にした画面を積む |
| `対象ページの上下に {string} という名前のボタンが2つある` | Given | `button(name)` を2つ(参照 `e1`、`e2`) |
| `判断モデルが操作 {string} と上の {string} を返す` | Given | `Reply::op(op, 0.9).on_nth(name, 0)` |
| `問い合わせから実行までの間に2つの {string} の位置が入れ替わる` | When | `looks` に、2つの参照を入れ替えた画面(上が `e2`、下が `e1`)を積んで `world.run()` |
| `ブラウザで上の {string} をクリックしている` | Then | `touched()` の最初の `click` の対象が `ElementId { role: "button", name, nth: 0 }` |
| `対象ページに {課金画面の手がかり}` | Given | 下の T-012 の表の6行を、文言ごとの Given にする |
| `既定の離脱操作を実行している` | Then | `touched()` に `("back", None)` があり、イベントに `recovered` で `cause` が `"paywall"` の行がある |
| `時間窓の操作回数は 1 増えている` | Then | `window_used()` が 1 |
| `対象ページに {string} という見出しがあるが、主要な操作は塞がれていない` | Given | 見出し `name` と `button("保存")` |
| `対象ページの課金画面は、既定の離脱操作をしても表示されたままである` | Given | 開始の画面を URL `/pricing` の課金画面(`button("戻る")`)にし、`after` は積まない(離脱しても同じ画面) |
| `既定の離脱操作をするステップが {int} 回続く` | When | `world.run()`(行き詰まりの閾値 3 で、3回目の離脱のあとに終わる)。そのあと `touched()` の `"back"` が n 回であることも確かめる |

T-007 の11行の場面(どれも `button("ヘルプ")` を一緒に置き、判断モデルの答えは共通の手順の `Reply::op(<操作>, 0.9).on(<名前>)`):

| 要素 | 置く要素 |
|---|---|
| ボタン "購入する" / "アカウントを削除" / "退会" / "サインイン" / "Checkout" | `button(name)` |
| 入力欄 "カード番号"(お支払いフォーム内) | `textbox("カード番号").form_action("/checkout/payment")` |
| ボタン "購入はこちら" | `button("購入はこちら")` |
| 代替テキスト "今すぐ買う" の画像ボタン | `button("今すぐ買う")`(画像ボタンの読み上げ名は代替テキスト) |
| action が "/account/delete" のフォームの送信ボタン "OK" | `button("OK").form_action("/account/delete")` |
| 遷移先が "/checkout/confirm" のリンク "次へ" | `link("次へ").href("/checkout/confirm")` |
| 設定の禁止リストに一致するボタン "実行" | `button("実行").forbidden_listed()` |

T-012 の6行の場面(どれも `after` に、課金画面でない `/` の画面(`button("保存")`)を1つ積む。離脱のあとは普通の画面に戻り、次のステップで問い合わせて土台が止める):

| 手がかり | 開始の画面 |
|---|---|
| "プレミアムにアップグレード" のモーダルが表示されているが、主要な操作は塞がれていない | `/`、`button("保存")`、`dialog: ["プレミアムにアップグレード"]` |
| モーダルの無い通常の URL のページに、見出し "Subscribe to continue" とボタン "アップグレード" だけがある | `/`、見出し "Subscribe to continue"、`button("アップグレード")` |
| 遷移した URL が "/pricing" で、課金を促す語は表示されていない | `/pricing`、`button("保存")` |
| 遷移した URL が "/upgrade" で、課金を促す語は表示されていない | `/upgrade`、`button("保存")` |
| 遷移した URL が "/checkout" で、課金を促す語は表示されていない | `/checkout`、`button("保存")` |
| 課金を促す語の無いモーダルが表示され、主要な操作がすべて塞がれている | `/`、`button("保存")`、`dialog: ["お知らせ"]`、`blocked: true` |

### 書く手順(`steps/load.rs`。対象サーバへの負荷)

時間は、判断モデルの答えの `at`(14h)と、窓の待機(`SceneClock` が `sleep_until` の時刻へ進む)だけで進む。窓は3600秒ごと。

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `1時間窓あたりの操作回数の上限を {int} 回に設定している` | Given | `requests_per_hour` |
| `現在の時間窓で {int} 回操作した` | Given | `Reply::op("wait", 0.9)` を n 回積む |
| `時間窓が明ける前に1ステップを実行しようとする` | When | `world.run()` |
| `判断モデルに問い合わせておらず、ブラウザ操作もしていない` | Then | イベントで、最後の `executed` の行と最初の `waited` の行の間に `asked`、`decided`、`recovered` が無い。`sleeps()` が `[3600秒]` |
| `時間窓が明けたあとのステップでは問い合わせる` | Then | `asked()` が `answered() + 1`(窓のあとの問い合わせで土台が止めた)、その問い合わせの時の時計が 3600秒以上(`judge` が問われたときの時刻を `texts` と並べて残していなければ、`waited` の行より後に判断モデルが呼ばれたことを `asked_in(when_step())` で確かめる) |
| `時間窓の終わりの時刻を過ぎる` | When | `Reply::op("wait", 0.9).at(3601秒)` と `Reply::op("wait", 0.9)` を積んで `world.run()` |
| `新しい時間窓では {int} 回まで操作できる` | Then | `touched()` が 1 + n 件、`sleeps()` が `[7200秒]`(2つ目の窓で n 回操作して埋まり、3つ目の窓を待つ) |
| `固まりから復旧するために対象ページを開き直す` | When | 固まりの閾値を 1 にして `world.run()`(1回の操作で画面が変わらず、次のステップで開き直す) |
| `許可ドメインの外から許可ドメインのページへ戻る` | When | 直前に積んだ答えを `Reply::op("click", 0.9).on("外部の記事")` に置き換え、`link("外部の記事")` を足し、`after` に外の画面(`origin: "https://other.example.net"`)を積んで `world.run()` |
| `課金画面から既定の離脱操作をする` | When | `after` に `/pricing` の画面、その次に `/` の画面を積んで `world.run()` |
| `時間窓の操作回数は {int} 回になっている` | Then | 最初の `waited` の行より前の `executed` の行が n 個 |
| `時間窓が明ける前のステップでは判断モデルに問い合わせない` | Then | 最初の `waited` の行より前の `asked` の行が、Given で積んだ答えの数と等しい |
| `固まりを検出している` | Given | 固まりの閾値を 2 にする(Given の2回の操作で画面が変わらず、固まりになる) |
| `許可ドメインの外にいる` | Given | 2つ目の答えを `click` "外部の記事" に置き換え、`after` に `/` の画面、外の画面をこの順に積む |
| `課金画面が表示されている` | Given | `after` に `/` の画面、`/pricing` の画面をこの順に積み、その次に `/` の画面を積む |
| `対象サーバへアクセスしていない` | Then | 最後の `executed` の行と最初の `waited` の行の間に `recovered` が無い |
| `時間窓が明けたあとのステップで ページの開き直し をしている` ほか2本(`許可ドメインへの戻り`、`既定の離脱操作`。Examples の値は引用符なしで前後に半角空白が入る) | Then | 最初の `waited` の行より後の最初の `recovered` の行の `op` と `cause` が、開き直しは (`reload`, `frozen`)、戻りは (`return`, `outside`)、離脱は (`back`, `paywall`) |
| `判断モデルが操作 {string} と禁止要素 {string} を返す` | Given | `button(name)` と `button("保存")`、`Reply::op(op, 0.9).on(name)` |

T-053 の「固まりから復旧する」で行き詰まりの閾値(3)は1回の操作では届かない。T-071 の「固まりを検出している」では、2回の操作で固まり(閾値2)と行き詰まりの回数2(閾値3)になり、窓の待機が先に採られる(待機の4段目の最初の行)。窓が明けたあと、開き直しが採られる。

## 失敗させるテスト

受け入れの23本(Examples の各行を含む)を、この brief の3つのファイルの `READY` に名前で足し、`cargo test -p webqa --test acceptance` で通す。名前は feature の Scenario の題名そのまま(台帳の各行の「受け入れ(同題の Scenario)」)。

| Rule | ファイル | 題名(台帳の番号) |
|---|---|---|
| 1ステップの問い合わせと実行 | `step.rs` | T-001、T-002、T-003、T-004、T-055、T-056、T-072 |
| ガードレール | `guard.rs` | T-005、T-006、T-007、T-113、T-008、T-009、T-010、T-011、T-012、T-013、T-014 |
| 対象サーバへの負荷 | `load.rs` | T-035、T-036、T-053、T-071、T-054 |

先に手順を書かずに `READY` へ名前を足し、`fail_on_skipped` で失敗すること(Red)を確かめてから手順を書く。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 23本が `READY` にあり、走って通る。
- 共通の手順の文言をこの brief のファイルに書いていない。統合のとき、ほかの Rule のファイルとあいまいな手順にならない(親が全シナリオを回して確かめる)。
- 触ってよいファイルの外に差分が無い。

## 親に決めてほしいこと

1. **T-003 の input の "保存" を入力欄として置くこと**。14b の「親に決めてほしいこと」2 で、ブラウザの `input` を入力欄の系統に絞ると決めた場合でも、受け入れの偽のブラウザは操作を記録するだけなので結果は変わらない。それでも本物と食い違わないよう、input の行では "保存" を `textbox` にした。
2. **T-056 の待ち時間切れを偽物の `Timeout` で作ること**。橋渡しの待ち時間の処理そのものは、`webqa-judge` の契約のテストが受け持つ。
3. **T-071 の「固まりを検出している」で、固まりの閾値を2に下げること**。Background の閾値は3で、Given の「現在の時間窓で 2 回操作した」の2回では固まりに届かない。feature の冒頭の「上限が効くことを確かめるシナリオだけ、前提で上限を小さく上書きする」に当たると読んだ。T-053 の「固まりから復旧する」の閾値1も同じ読み方。
