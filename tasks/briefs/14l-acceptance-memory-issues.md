# brief: issue #14l 受け入れ: 探索の記憶と優先順位、問題の記録

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14h が入ってから始める。** 14h の世界(`QaWorld`、`Scene`、`PageSpec`、`ElementSpec`、`Reply`)、共通の手順(`steps/common.rs`)、`remember`、`Scene.marks` を使う。始める前に main の `tests/support/` と `tests/steps/common.rs` を読み、補助の名前と意味がこの brief と違えば NEEDS_CONTEXT で返す。

並行して 14i、14j、14k、14m が回る。触るファイルは重ならない。

## タスク

feature の2つの Rule のシナリオ19本を、受け入れのテストとして通す。

- 探索の記憶と優先順位(12本): T-029、T-061、T-062、T-063、T-030、T-031、T-032、T-033、T-034、T-070、T-095、T-096
- 問題の記録(7本): T-037、T-064、T-038、T-039、T-040、T-041、T-042

下の層のテストは、それぞれの issue で入っている。ここは受け入れの配線だけ。

## 触ってよいファイル

- `crates/webqa/tests/steps/memory.rs`、`steps/issues.rs`(14h が空の `READY` で作ったもの)
- 分けるときの新規: `steps/memory_select.rs`、`steps/issues_report.rs`(`acceptance.rs` に `#[path]` の行と `READY` の連結を足してよい。この brief で足したファイルの分だけ)

触らないもの: `steps/common.rs`、ほかの Rule のファイル、`tests/support/`(補助が足りなければ NEEDS_CONTEXT で返す)、`src/` の下、feature の写し。

## 決めてある設計

- 共通の手順(14h の表)の文言は書かない。下の「書く手順」の文言だけを書く。パラメータは `{string}`、`{int}`、`{float}` だけ。
- 手順の関数は、場面を組み立てるか、世界の補助を呼んで `assert_eq!` で比べる1〜3行にする。手順の中に `if` を書かない。値で振る舞いが変わるところは表(配列)にして引く。
- 判断モデルの答えを積まない場面では、最初の問い合わせで判断モデルが待ち続け、土台が止める(14h)。止めた実行も、記憶、終了の行、実行結果を書く。

### 記憶の読み方(`memory.rs` に置く補助)

保存した記憶は `world.ran().store.memories[scene.origin()]` の JSON(`explore/graph/file.rs` の形)。

- `screens`: `{"s1": {"path", "elements": [{role, name, nth, operated, forbidden}]}, …}`
- `edges`: `[{"from": "s1", "element": {role, name, nth}, "op", "to": "s2"}, …]`

補助 `saved(world) -> Value` と、辺をパスの組にした `routes(world) -> Vec<(String, String)>`(`from` と `to` の画面の `path`)を置く。

### 画面の名前

feature の画面の名前は、パスに写す。表は `memory.rs` の定数にする。

| 名前 | パス | 要素 |
|---|---|---|
| "トップ" | `/` | 手順が置く要素 |
| "設定" | `/settings` | `button("通知")` |

### 「選ぶ」の確かめ方

探索モードの候補の絞り込み(`machine/offer.rs`)は、判断モデルへの問いに出る。

- 要素の段(未操作、未探索の分岐が残る操作済み): `target` の `Choice` の選択肢が、候補の要素の番号だけになる。
- 移動の段: 操作種別の選択肢が `back`、`scroll_down`、`scroll_up` だけになり、`target` は選択肢を持たない。

補助 `offered_names(world, k) -> Vec<String>`: `judge.questions[k]` の `target` の選択肢の番号を、`judge.texts[k]` の `n: role "name"` の行で名前に直す。`offered_ops(world, k) -> Vec<String>`: `operation` の選択肢。待ち続けた問いも `questions` に入る(14h の `SceneJudge`)。

### 書く手順(`steps/memory.rs`)

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `前回の実行で画面 {string} から {string} を押して画面 {string} へ遷移した記録がある` | Given | 開始の画面を `/`、`button(押した要素)` にし、`remember` で `[("/", [button 押した要素]), ("/settings", [button "通知"])]` と辺 `(0, 押した要素, 1)` を入れる |
| `同じ対象で新しい実行を始める` | When | `world.run()` |
| `探索の記憶に {string} から {string} への遷移が含まれている` | Then | `routes()` に、名前をパスにした組がある |
| `この対象の探索の記憶はまだ無い` | Given | 何もしない(`scene.memories` は空) |
| `新しい実行を始める` | When | `world.run()` |
| `探索の記憶は空で、探索済みの要素も無い` | Then | `memory_reset` の行が無く、保存した記憶の `screens` が1つ(開始の画面)、`edges` が空、`explored()` が空 |
| `この対象の探索の記憶のファイルが読めない形に壊れている` | Given | `scene.memories` に、最初の許可ドメインのキーで `"{ not json"` を入れる |
| `探索の記憶は空で始まっている` | Then | 保存した記憶の `screens` が1つ、`edges` が空 |
| `記憶を読めなかったことが実行記録に残っている` | Then | `memory_reset` の行があり、`cause` が `"unreadable"`。`store.aside` に最初の許可ドメインがある |
| `対象 {string} の探索の記憶がある` | Given | その origin の画面(`/`、`button("別の対象の画面")`)を1つ持つ `Memory` を作り、`to_json(その origin)` を `scene.memories` にその origin のキーで入れる(`remember` は最初の許可ドメインにしか書かないので、`Memory::visit` と `to_json` を直接呼ぶ) |
| `対象 {string} で新しい実行を始める` | When | `scene.origin()` が引数と等しいことを確かめて `world.run()` |
| `探索の記憶に {string} の画面は含まれない` | Then | 保存した記憶(最初の許可ドメインのもの)の `origin` が引数でなく、どの要素の名前にも `"別の対象の画面"` が無い |
| `対象ページに未操作の要素 {string} がある` | Given | 開始の画面に `link(name)` を足す |
| `操作済みの要素 {string} の遷移先に未探索の分岐が残っている` | Given | 開始の画面に `button(name)` を置き(`show_button`)、`remember` で `[("/", 開始の画面の要素すべて), ("/settings", [button "通知"])]` と辺 `(0, name, 1)` を入れる("通知" は未操作のまま) |
| `探索モードで次の操作を選ぶ` | When | `mode = Explore` にして `world.run()` |
| `{string} を選んでいる` | Then | `offered_names(0)` が `[name]` |
| `対象ページに未操作の要素は無い` | Given | 何もしない |
| `対象ページの要素はすべて探索し終えている` | Given | 開始の画面を `/settings`、`button("通知")` にする |
| `戻った先の画面 {string} に未操作の要素 {string} が残っている` | Given | `remember` で `[("/", [button "保存", link 未操作の要素]), ("/settings", 開始の画面の要素すべて)]` と辺 `(0, "保存", 1)`、`(1, "通知", 1)` を入れる(辺の先が自分なので "通知" は探索し終えている。戻る操作は辺にならないので、"トップ" の未操作の要素は今の画面から辿れない) |
| `選んだ操作は {string}、{string}、{string} のいずれかである` | Then | `offered_ops(0)` が3つの部分集合で空でなく、`target` の選択肢が空 |
| `{string} を押すと画面 {string} へ遷移し、そこに未操作の要素 {string} がある` | Given | `mode = Explore`、開始の画面を `/`、`button(押す要素)` だけにする。`after` に `/settings`(`button(未操作の要素)`)、開始の画面の写しをこの順に積む |
| `{string} を押して画面 {string} へ戻る` | When | `Reply::op("click", 0.9).on(押す要素)` と `Reply::op("back", 0.9)` を積んで `world.run()` |
| `{string} は操作済みだが、まだ探索し終えた要素として扱われていない` | Then | `explored()` が name を含み、最後の問い(`judge.questions` の最後。戻った先で待ち続けた問い)の `offered_names` が `[name]`(未探索の分岐が残る段に入っている) |
| `対象ページの未操作の要素は {string} だけである` | Given | 開始の画面を `button(name)` だけにする |
| `{string} を選んでいない` | Then | `offered_names(0)` に name が無い |
| `対象ページの要素は {string} を除いてすべて探索し終えている` | Given | 開始の画面を `/settings`、`button("通知")`、`button(除く要素)` にする |
| `{string} を選んでおらず、選んだ操作は {string}、{string}、{string} のいずれかである` | Then | `offered_names(0)` に1つ目が無く、`offered_ops(0)` が3つの部分集合 |
| `画面 {string} を表示していて、表示領域より下にリンク {string} がある` | Given | `mode = Explore`、開始の画面を名前のパスにし、`button("保存")` と `link(name).below()` を置く |
| `操作 {string} を実行し、表示領域に {string} が現れる` | When | `Reply::op(op, 0.9)` を積み、`after` に、そのリンクを表示領域に出した開始の画面の写しを積んで `world.run()` |
| `今の画面は探索の記憶の {string} のままである` | Then | 保存した記憶の `screens` が1つで、その `path` が名前のパス |
| `探索の記憶に新しい画面は加わっていない` | Then | 保存した記憶の `edges` が空 |
| `画面 {string} を表示している` | Given | `mode = Explore`、開始の画面を名前のパスにし、`button("保存")` を置く |
| `ボタン {string} をクリックし、ページ全体の操作可能要素が変わる` | When | 開始の画面に `button(name)` を足し、`Reply::op("click", 0.9).on(name)` を積み、`after` に同じパスで `button("通知")` だけの画面を積んで `world.run()` |
| `探索の記憶に新しい画面が加わり、{string} からその画面への遷移が記録されている` | Then | `screens` が2つ、`routes()` が `[(名前のパス, 名前のパス)]`(同じパスでも要素が違うので別の画面) |

T-034 の "購入する" は、既定の禁止の語に当たる(`Forbidden::new` の既定の語)。禁止要素は候補から外れ、移動の段になる。

T-070 の "退会" は記憶の中で未操作のまま残るが、禁止要素なので候補に入らない。"通知" は探索し終えているので、移動の段になる。

### 書く手順(`steps/issues.rs`)

問題は、イベントの `issue` の行(`issue`、`class`、`label`、`screenshot`)と、実行結果の `issues` で見る。種類の名前は serde の名前(`error_display`、`accessible_name`、`console_error`、`semantic`)。

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `{int} ステップ目で判断モデルがエラー表示ありを確信度 {float} で返す` | Given | `Reply::op("wait", 0.9)` を n − 1 回、`Reply::op("wait", 0.9).error(p)` を1回積む |
| `エラー表示の判定閾値は {float} に設定されている` | Given | `error_confidence` |
| `{int} ステップ目を実行する` | When | `world.run()` |
| `エラー表示の問題として記録されている` | Then | `issue` が `"error_display"` の行がある |
| `種類とステップ番号 {int} を含む名前のスクリーンショットが保存されている` | Then | `store.shots` の名前に `webqa::store::screenshots::name(IssueKind::ErrorDisplay, n)` がある |
| `判断モデルがエラー表示ありを確信度 {float} で返す` | Given | `Reply::op("wait", 0.9).error(p)` を積む |
| `エラー表示の問題は記録されていない` | Then | `error_display` の行が無い |
| `対象ページに読み上げ名が {string} のボタンがある` | Given | 開始の画面に `button(name)` を足す |
| `アクセシビリティの問題として記録されている` | Then | `issue` が `"accessible_name"` の行がある |
| `アクセシビリティの問題として記録されていない` | Then | その行が無い |
| `操作のあとにブラウザのコンソールへエラーログが出力される` | Given | 開始の画面に `button("保存")`、`Reply::op("click", 0.9).on("保存")` を積み、`after` に、`console` に `"TypeError: x is undefined"` を入れた開始の画面の写しを積む |
| `コンソールエラーの問題として記録されている` | Then | `issue` が `"console_error"` の行がある |
| `判断モデルが「わかりにくい表示」があると返す` | Given | `scene.probes = true`、`Reply::op("wait", 0.9).confusing(0.9)` を積む(閾値は `error_confidence` と共有する。14b の決定) |
| `その検出は参考情報として記録され、確定した不具合とは別に分類されている` | Then | `issue` が `"semantic"` の行があり、`class` が `"reference"`、`label` が `"わかりにくい表示"`。`class` が `"confirmed"` の `issue` の行は無い |
| `実行が問題を1件記録して終了した` | Given | 段階を `[("保存", "保存する", text "保存しました")]`、開始の画面に `button("保存")`、`Reply::op("click", 0.9).on("保存").error(0.9)` を積み、`after` に、テキストを `"保存しました"` にした開始の画面の写しを積む(1回の操作でゴールに達する) |
| `実行結果を出力する` | When | `world.run()` |
| `出力には見つけた問題と、そこに至る操作の記録が含まれる` | Then | `report()` の `issues` が1件で `kind` が `"error_display"`、`steps` に "保存" の `click` がある |
| `その出力から同じ操作列を再実行できる` | Then | `Replay::from_report(実行結果の文字列)` が `Ok((Mode::Scenario, replay))`。`replay` に `judge.texts[0]` と `judge.questions[0]` を渡した答えの `operation` と `target` が、最初の問いの答えと等しい |

T-037 のステップ番号は、14d の決定(最初の観測をステップ0にし、問い合わせと中核の操作のたびに1つ進める)で、5回目の問い合わせのステップが5になる。名前が合わなければ NEEDS_CONTEXT で返す。

T-039 の `button` は、仮の値の名前(`issue::is_placeholder_name`)に当たる。

## 失敗させるテスト

受け入れの19本(Examples の各行を含む)を、この brief のファイルの `READY` に名前で足し、`cargo test -p webqa --test acceptance` で通す。名前は feature の Scenario の題名そのまま。

| Rule | ファイル | 題名(台帳の番号) |
|---|---|---|
| 探索の記憶と優先順位 | `memory.rs` | T-029、T-061、T-062、T-063、T-030、T-031、T-032、T-033、T-034、T-070、T-095、T-096 |
| 問題の記録 | `issues.rs` | T-037、T-064、T-038、T-039、T-040、T-041、T-042 |

先に名前だけを足して `fail_on_skipped` で失敗すること(Red)を確かめてから手順を書く。

## 受け入れ条件

- 触ったモジュールのテスト(`cargo test -p webqa --test acceptance`)、`just fmt-check`、`just clippy` とコミットのフックが通る。全体のテストとカバレッジ100%は、PR の CI で確かめる。
- 19本が `READY` にあり、走って通る。
- 共通の手順の文言をこの brief のファイルに書いていない。統合のとき、ほかの Rule のファイルとあいまいな手順にならない(親が全シナリオを回して確かめる)。
- 触ってよいファイルの外に差分が無い。

## 親に決めてほしいこと

1. **「選ぶ」を、判断モデルに出した候補で確かめること**(T-030〜T-034、T-070)。探索モードでも、操作を選ぶのは判断モデルで、中核は候補を絞るだけ。答えを積まずに最初の問いで止め、問いの選択肢を見る。案B は、判断モデルの偽物に「選択肢の最初を選ぶ」答えを足すこと(`tests/support/` を変えるので推さない)。
2. **T-042 の再実行を、`Replay` に同じ問いを1回渡して確かめること**。`tests/support/` の `Fakes` は外から使えず、`Replay` を判断モデルにして場面を回し直す補助が無い。実行ファイルを通す再実行は、14g の `webqa replay` のテスト(T-042 の実行ファイルの層)が受け持つ。
3. **T-061 と T-062 の「空の記憶で始まる」を、保存した記憶で確かめること**。実行の前の記憶は外から見えない。保存した記憶が開始の画面1つと辺0なら、前の記憶を引き継いでいない。
4. **画面の名前("トップ"、"設定")をパスに写すこと**。feature の画面の名前は表示の名前で、記憶の画面はパスと要素で識別する。
