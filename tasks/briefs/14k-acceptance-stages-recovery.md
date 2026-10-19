# brief: issue #14k 受け入れ: 決め打ちシナリオの段階、固まりとドメイン外からの復帰

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14h が入ってから始める。** 14h の世界、偽物、共通の手順、`remember`、`Scene.marks`、拍(`when_beat`、`when_asked`)を使う。始める前に main の `tests/support/` と `tests/steps/common.rs` を読み、補助の名前と意味がこの brief と違えば NEEDS_CONTEXT で返す。

並行して 14i、14j、14l、14m が回る。触るファイルは重ならない。

## タスク

feature の2つの Rule のシナリオ18本を、受け入れのテストとして通す。

- 決め打ちシナリオの段階(9本): T-086、T-087、T-088、T-110、T-111、T-089、T-090、T-091、T-092
- 固まりとドメイン外からの復帰(9本): T-025、T-059、T-060、T-026、T-093、T-094、T-027、T-028、T-052

下の層のテストは、それぞれの issue で入っている。ここは受け入れの配線だけ。

## 触ってよいファイル

- `crates/webqa/tests/steps/stages.rs`、`steps/recovery.rs`(14h が空の `READY` で作ったもの)
- 分けるときの新規: `steps/stages_given.rs`、`steps/recovery_domain.rs`(`acceptance.rs` に `#[path]` の行と `READY` の連結を足してよい。この brief で足したファイルの分だけ)

触らないもの: `steps/common.rs`、ほかの Rule のファイル、`tests/support/`、`src/` の下、feature の写し。

## 決めてある設計

### 段階の既定

「決め打ちシナリオに段階 … が定義されている」の3つの文は、`mode = Scenario` にして次の段階を入れる。

| 段階 | goal | done_when(既定) |
|---|---|---|
| "ログイン" | ログインする | `text: "ログインしました"` |
| "投稿" | 投稿する | `text: "投稿しました"` |

- T-091 と T-092 は、"ログイン" の done_when を手順で差し替える。
- テキストで書くのは、画面の識別と画面の変化(表示領域の要素の一覧と URL で決まる)を動かさずに done_when だけを満たせるため。開始の画面は `/` に `button("ログイン")` と `link("ヘルプ")`。
- 「段階を満たす」手順は、答えが積まれていれば最後の操作のあとの画面(`after` の最後。操作の数より短ければ開始の画面の写しで埋める)のテキストに、無ければ開始の画面のテキストに、その文を足す(改行でつなぐ)。

### 書く手順(`steps/stages.rs`)

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `決め打ちシナリオに段階 {string} と {string} がこの順に定義されている` | Given | 2段階 |
| `決め打ちシナリオに段階 {string} だけ が定義されている` | Given | "ログイン" だけ(T-110 の Examples の1行目を入れた文) |
| `決め打ちシナリオに段階 {string} と {string} がこの順に が定義されている` | Given | 2段階(T-110 の2行目を入れた文) |
| `段階 {string} にいて、その done_when 条件が満たされている` | Given | "ログイン" なら開始の画面のテキストに "ログインしました"。"投稿" なら開始の画面のテキストに "ログインしました" と "投稿しました"(開始の時点で "ログイン" を満たして進み、続けて "投稿" も満たす。T-087 は終わり方だけを確かめる) |
| `段階 {string} の done_when 条件は満たされていない` | Given | 何もしない |
| `段階 {string} に進んでいる` | Then | `state().stage()` が段階の番号(`"投稿"` は 1)。`stage_advanced` の行があり、`name` が等しい |
| `段階 {string} にいて、その done_when 条件は満たされていない` | Given | 何もしない |
| `段階 {string} の done_when 条件は満たされている` | Given | 開始の画面のテキストに、その段階の文("投稿しました")を足す |
| `段階 {string} のままで、判断モデルに問い合わせている` | Then | `stage()` が番号のままで、`when_asked()` |
| `対象ページを開いた時点で、段階 {string} の done_when 条件が満たされている` | Given | 開始の画面のテキストに "ログインしました" |
| `段階 {string} があれば、その done_when 条件は満たされていない` | Given | 何もしない |
| `実行を開始する` | When | `world.run()` |
| `段階 {string} に進み、実行は終了していない` | Then | `stage()` が番号で、`reason()` が `None` |
| `ブラウザ操作も判断モデルへの問い合わせも、まだ起きていない` | Then | 拍の最初が `Advance` か `End`、`touched()` が空 |
| `段階 {string} の done_when 条件も満たされている` | Given | 開始の画面のテキストにその段階の文を足す |
| `段階 {string} から段階 {string} に進む` | When | `world.run()` |
| `段階 {string} で操作せずに、実行はゴール達成として終了している` | Then | `reason()` が `GoalAchieved`、`stage()` が番号、`touched()` が空 |
| `段階 {string} の done_when を満たしてから、ブラウザ操作も判断モデルへの問い合わせも起きていない` | Then | `asked()` が 0、`touched()` が空 |
| `段階 {string} で、画面が変化せず探索済みの要素も増えないステップが {int} 回続いている` | Given | `Reply::op("wait", 0.9)` を n 回積む(画面は変わらない。固まりと行き詰まりの回数が n になる) |
| `段階 {string} の done_when 条件が満たされている` | Given | `marks["called"]` があれば、答えの列を最後の n 個に切り詰める(T-090。共通の手順が答えを足すため)。そのあと上の「段階を満たす」で、最後の操作のあとの画面に "ログインしました" を足す |
| `1ステップを実行して段階 {string} に進む` | When | `world.run()` |
| `その後、画面が変化せず探索済みの要素も増えないステップが {int} 回続いても、ページを開き直さない` | Then | 場面に `Reply::op("wait", 0.9)` を n 回足し、`world.ran = None` にして回し直す。`touched()` に `"reload"` が無い |
| `行き詰まりとしても終了しない` | Then | 回し直した実行の `reason()` が行き詰まりでない |
| `判断モデルの呼び出し上限を {int} 回に設定し、段階 {string} で {int} 回呼び出し済みである` | Given | `max_decisions` を a、`Reply::op("wait", 0.9)` を b 回積み、`marks["called"] = b` |
| `判断モデルの呼び出し回数は {int} 回のままである` | Then | 最初の `stage_advanced` の行より前の `asked` の行が n 個 |
| `探索済みの要素は {string} のままである` | Then | `explored()` が `{name}` |
| `段階 {string} で判断モデルに1回問い合わせたあと、次のステップで判断予算切れとして終了する` | Then | 場面に `Reply::op("back", 0.5)`(止める答え。操作しない)を1つ足して回し直す。`reason()` が `BudgetExceeded`、`asked()` が 4、最後の `asked` の行が `stage_advanced` の行より後 |
| `段階 {string} の done_when に、URL のパス {string} と見出し {string} の2つの宣言がある` | Given | "ログイン" の done_when を `url` と `element { role: "heading", name }` の2つにする |
| `段階 {string} にいて、今の URL のパスは {string} で、見出し {string} はページにある` | Given | 開始の画面のパスと見出し(表示領域の中) |
| `段階 {string} にいて、今の URL のパスは {string} で、見出し {string} はページに無い` | Given | 開始の画面のパスだけ |
| `段階 {string} の done_when に、見出し {string} の宣言だけがある` | Given | "ログイン" の done_when を `element { role: "heading", name }` だけにする |
| `段階 {string} にいて、見出し {string} はページにあるが表示領域より下にある` | Given | 開始の画面に見出しを `below` で足す |

T-090 の答えの列: 「段階 "ログイン" で 3 回呼び出し済み」の `wait` 3つ、共通の手順「1時間窓あたりの操作回数の上限を 10 回に設定し、現在の時間窓で 3 回操作した」の `wait` 3つ、共通の手順「探索済みの要素は "ログイン" である」の `click` "ログイン" 1つ、の7つが積まれる。「段階 "ログイン" の done_when 条件が満たされている」が最後の3つ(`wait`、`wait`、`click` "ログイン")に切り詰める。3回の問い合わせ、3回の操作、探索済みは "ログイン" になり、3回目の操作のあとの画面で "ログイン" の done_when を満たして進む。進んだあとの4回目の問い合わせで土台が止める(最初の実行)。最後の Then で止める答えを足して回し直すと、4回目の答えのあと判断予算切れで終わる。

T-089: 2回の `wait` で固まりと行き詰まりの回数が2になり、2回目のあとの画面で done_when を満たして進む(回数は0に戻る)。回し直しで `wait` を2回足すと、回数は2まで進むだけで、開き直しも行き詰まりも起きない(回数が戻らなければ、固まりは4、行き詰まりは4で、開き直すか行き詰まりで終わる)。

### 書く手順(`steps/recovery.rs`)

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `探索済みの要素は {string} と {string} である` | Given | 開始の画面に `button(a)`、`link(b)`、`button("操作1")`〜`button("操作3")` を置き、`remember` で a と b を操作済みにした記憶を入れる(問い合わせを積まない。共通の手順の「操作しても画面が変化しない」が足す要素を先に置き、記憶の画面と実行の画面の要素の集合を揃える) |
| `行き詰まりは検出されていない` | Given | 何もしない(共通の手順の新しい要素のクリックは、行き詰まりの回数を進めない) |
| `対象ページを初期状態から開き直している` | Then | `touched()` に `("reload", None)` があり、`recovered` の行の `cause` が `"frozen"` |
| `その間は判断モデルに問い合わせていない` | Then | `when_beat()` が `Beat::Core` |
| `探索済みの要素は {string} と {string} のままである` | Then | `explored()` が2つを含む |
| `判断モデルは毎回、操作 {string} と対象要素 {string} を返す` | Given | `link(target)` を足し、`Reply::op(op, 0.9).on(target)` を3回積む。直前の Given が積んだ答え("保存" のクリック)のあとの画面を、`link(target)` が現れた画面にする(`after` の先頭。画面が変化するので、固まりの回数は0から数え始める) |
| `{string} をクリックしても、ページが設定の待ち時間内に応答しない` | Given | 何もしない(偽のブラウザは画面を変えない。待ち時間切れは、アダプタが「画面が変化しなかった」の観測にする。契約のテストが受け持つ) |
| `そのステップが {int} 回続く` | When | `world.run()` |
| `1 回目と 2 回目のあとは対象ページを開き直していない` | Then | `touched()` の列が `[click 保存, click ヘルプ, click ヘルプ, click ヘルプ, reload]` で、`reload` の前に `ヘルプ` のクリックが3つ |
| `3 回目のあとに対象ページを初期状態から開き直している` | Then | 上の列の5つ目が `reload` |
| `探索済みの要素に {string} が残っている` | Then | `explored()` が `name` を含む |
| `画面が変化する操作を1回実行する` | When | `Reply::op("scroll_down", 0.9)` を積み、`after` に、開始の画面の `操作1` を `below` にした画面を積んで `world.run()` |
| `その後、画面が変化しないステップが {int} 回続いてもページを開き直さない` | Then | 場面に `Reply::op("wait", 0.9)` を n 回足して回し直し、`touched()` に `"reload"` が無い |
| `スクロールすると、表示領域の中の操作可能要素が変わる` | Given | 直前の `scroll_down` のあとの画面を、`操作1` を `below` にした画面にする |
| `表示領域はページの一番下にある` | Given | 何もしない |
| `判断モデルが操作 {string} を返し、スクロールしても表示領域の中の操作可能要素と URL は変わらない` | Given | `Reply::op(op, 0.9)`(画面は変わらない) |
| `判断モデルが操作 {string} とリンク {string} を返す` | Given | `link(name)` と `button("保存")` を足し、`Reply::op(op, 0.9).on(name)` |
| `{string} は {string} へ遷移する` | Given | そのリンクの `href` を遷移先にし、`after` に遷移先の画面(`origin: Some(遷移先)`、`path: ""`、`button("記事")`)を積む |
| `許可ドメインの外へ遷移したことが記録されている` | Then | `issue` の行の `issue` が `"outside_domain"` |
| `ドメイン外のページについて判断モデルに問い合わせていない` | Then | `judge.texts` のどれにも `"記事"` が無い |
| `次のステップの前に許可ドメインのページへ戻っている` | Then | `recovered` の行(`op` `"return"`、`cause` `"outside"`)が、待ち続けた問い合わせの前にある。`judge.texts` の最後は開始の画面の文字列 |
| `探索済みの要素は失われていない` | Then | `explored()` が `外部の記事` を含む |
| `許可ドメインのサブドメインの扱いは {string} に設定されている` | Given | `"なし"` は何もしない。`"*.app.example.test を許可"` は `origins` に `"https://*.app.example.test"` を足す |
| `操作の結果 {string} へ遷移する` | When | `link("次へ")` と `Reply::op("click", 0.9).on("次へ")` を足す。`after` が空なら、遷移先の画面(`origin: Some(遷移先)`、`path: ""`、`button("記事")`)を積む。`world.run()` |
| `許可ドメインの外への遷移として扱わない` | Then | `outside_domain` の `issue` の行が無く、`recovered` の `return` が無い |
| `許可ドメインの外への遷移として扱う` | Then | `outside_domain` の `issue` の行がある |
| `許可ドメインの中の {string} は {string} へリダイレクトする` | Given | `after` に、リダイレクトの先の画面(`origin: Some(先)`)を積む(アダプタは最終的な URL を観測に入れる。リダイレクトそのものは契約のテストが受け持つ) |

T-028 の `javascript:alert(1)` と `data:text/html,hello` は、`Url::parse` で読める URL なので、そのまま画面の URL にする。`https://app.example.test/settings` の行は許可ドメインの中なので、問題は記録されない。

T-060 の場面: 共通の手順「探索済みの要素は "保存" である」が `button("保存")` と、その `click` を積む。この brief の「判断モデルは毎回、…」が、`link("ヘルプ")` が現れた画面を "保存" のクリックのあとの画面にする。"保存" のクリックで画面が変わり(固まり0)、"ヘルプ" のクリック3回で画面が変わらず(固まり1、2、3)、3回目のあとに開き直す。行き詰まりは、"ヘルプ" の1回目で0に戻り、2回目と3回目で2までしか進まない。

## 失敗させるテスト

受け入れの18本(Examples の各行を含む)を、この brief のファイルの `READY` に名前で足し、`cargo test -p webqa --test acceptance` で通す。先に名前だけを足して `fail_on_skipped` で失敗すること(Red)を確かめてから手順を書く。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 18本が `READY` にあり、走って通る。
- 共通の手順の文言をこの brief のファイルに書いていない。
- 触ってよいファイルの外に差分が無い。

## 親に決めてほしいこと

1. **段階の done_when をテキストで書くこと**。feature は段階の done_when の中身を決めていない(T-091、T-092 を除く)。URL で書くと、done_when を満たす画面へ移ったときに画面の識別が変わり、未操作の要素が新しく見えて行き詰まりの回数が戻るなど、確かめたいこと以外が動く。
2. **T-090 で答えの列を切り詰めること**。14j の T-023 と同じ理由(共通の手順が答えを足す)。
3. **「その後、…」の Then を、場面を足して回し直して確かめること**(T-089、T-026、T-093)。14j と同じ。
4. **T-060 の「ページが応答しない」を、画面が変わらない観測で作ること**。待ち時間切れを観測に変えるのはアダプタで、契約のテスト(`page_timeout_becomes_unchanged_observation`)が受け持つ。受け入れは、そのあとの回数の数え方だけを確かめる(`docs/architecture.md` の「テストの層」の表のとおり)。
