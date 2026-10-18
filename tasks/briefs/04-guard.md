# brief: issue #4 ガードレールの判定

共通の決まりは同じディレクトリの `common.md`。

## タスク

`webqa-core::guard` に、ガードレールの5つの判定を、ブラウザにも判断モデルの型にも依存しない純粋な関数として書く。
5つは、禁止要素(`forbidden`)、課金画面(`paywall`)、低確信度の「戻る」(`confidence`)、要素の存在と同一性(`identity`)、許可ドメイン(`domain`)である。
設計は `docs/architecture.md` の「ガードレール」「判断モデルの抽象」(確信度の定義)「設定」(許可ドメインの書き方)「複雑度の上限に耐える書き方」「テストの層」。
受け入れシナリオは `tasks/loopeng/WebExploratoryQA.feature` の Rule「ガードレール」と T-028、T-052(台帳 `tasks/test-design/WebExploratoryQA.md` の issue 列に #4 を含む行: T-005〜T-013、T-028、T-052)。
issue #4 のコメント(#3 からの引き継ぎ)にある「許可ドメインにパスやクエリを含む値を拒むか」も、ここで決めて閉じる(下の「許可ドメイン」)。

範囲の外:

- 材料を集めるブラウザ側(当たり判定、フォームの実効の送信先、要素参照、CSS セレクタの評価、リダイレクト後の URL)は #9。ここでは集まった材料を受ける型だけを決める。
- 判定を状態機械につなぐこと、既定の離脱操作(`guard.leave_paywall`)を実行すること、判断モデルの答えから確率を取り出すことは #12 と #7。
- `ports.rs`、`judge.rs`、`effect.rs`、`machine.rs` は並行する #15 が作る。触らず、参照もしない。確率は `f64` の素の値で受ける。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod guard;` を足すだけ)
- `crates/webqa-core/src/guard.rs` と `crates/webqa-core/src/guard/` の下(新規。`forbidden.rs`、`paywall.rs`、`confidence.rs`、`identity.rs`、`domain.rs`、それぞれのテストのファイル `guard/<モジュール>/tests.rs`、`Debug` の手書きを分けるなら `guard/debug.rs`)
- `crates/webqa-core/tests/guard_golden.rs` と `crates/webqa-core/tests/fixtures/guard/`(新規)
- `crates/webqa-core/src/config/validate/check.rs`(許可ドメインの検証を `guard::domain` に寄せるところだけ。下の「設定とのつなぎ」)
- 依存は足さない(`url`、`serde`、`serde_json`、`thiserror` は入っている)。

## 決めてある設計

### 共通(`guard.rs`)

- `guard.rs` は5つの `pub mod` と、`forbidden` と `paywall` が共有する2つの小さな型だけを持つ。どちらも `pub(crate)`。
- 文字の畳み込み `fold(&str) -> String`: 全角の ASCII(U+FF01〜U+FF5E)を半角に、全角の空白(U+3000)を半角の空白にし、小文字にし、空白の連なりを半角空白1つにまとめ、前後の空白を除く。`as` は使わず `char::from_u32` で変える。
- `Words`: 語の一覧。`Words::new(defaults: &[&str], extra: &[String]) -> Words` は、既定と追加の語をどちらも `fold` して持つ。`fold` して空になる語は捨てる(空の語はどの文字列にも一致してしまうため)。
  `Words::any_in(&self, text: &str) -> bool` は、`fold(text)` の中に、どれかの語が「境界で区切られて」現れれば真。境界は、一致した範囲の直前と直後の文字が ASCII の英数字でないこと(文字列の端も境界)。日本語の語は前後が英数字でなければよいので、実質は部分一致になる。英語の語は単語として一致する(`pay` は `display` に一致しない、`checkout` は `Checkout` に一致する)。
- `Segments`: URL のパスの区切りの一覧。`Segments::new(defaults: &[&str], extra: &[String]) -> Segments` は、各語を小文字にして `-` と `_` を除いて持つ。空になる語は捨てる。
  `Segments::any_in(&self, url: &url::Url) -> bool` は、URL のパスを `/` で区切った各部分を、小文字にして `-` と `_` を除き、どれかの語と完全に等しければ真。クエリ、フラグメント、ホストは見ない。`/sign-in` は `signin` に、`/Pricing/` は `pricing` に一致し、`/pricing-history` は `pricinghistory` なので `pricing` に一致しない。パスを持たない URL(`javascript:`、`data:`)は、`path_segments()` が無いので偽。
- 判定軸が複数あるもの(`forbidden`、`paywall`)は、軸ごとに1つの関数にし、`[fn(&Self, &材料) -> bool; N]` の静的な配列を `iter().any` でたどる。

### 禁止要素(`guard/forbidden.rs`)

- 材料 `Candidate { role: String, name: String, href: Option<Url>, form_action: Option<Url>, forbidden_listed: bool, allowed_listed: bool }`。
  - `href`: リンクの遷移先。`form_action`: 送信ボタンと入力欄の実効の送信先(アダプタが `formaction` 属性か `el.form?.action` から取る)。無ければ `None`。
  - `forbidden_listed` と `allowed_listed`: 設定の `guard.forbidden.selectors` と `guard.allowed.selectors` に一致したか。CSS セレクタの評価はアダプタが行う。
  - `role` は判定に使わない(材料として持ち、fixture と `Debug` で読む)。`allowed_listed` は禁止より優先する(利用者の決定)。真なら、どの軸で禁止に当たっていても禁止にしない。
  - `Debug` は `role`、`forbidden_listed`、`allowed_listed` だけを出し、`name`、`href`、`form_action` は `…` にする(`screen::Element` と同じ扱い)。
- 判定器 `Forbidden { words: Words, segments: Segments }`。
  - `Forbidden::new(extra_keywords: &[String]) -> Forbidden`: 既定の語に、設定の `guard.forbidden.keywords` を足す。`Default` は `new(&[])` と同じ。
  - `Forbidden::forbids(&self, c: &Candidate) -> bool`: `c.allowed_listed` が真なら偽。そうでなければ次の4軸の OR。
    1. 名前のキーワード: `words.any_in(&c.name)`
    2. 送信先: `c.form_action` が `Some` で `segments.any_in` が真
    3. 遷移先: `c.href` が `Some` で `segments.any_in` が真
    4. 禁止リスト: `c.forbidden_listed`
- 既定の語 `pub const DEFAULT_KEYWORDS: &[&str]`(この順で置く):
  `購入`、`買う`、`注文する`、`注文を確定`、`決済`、`支払`、`課金`、`削除`、`退会`、`解約`、`サインイン`、`カード番号`、`buy`、`purchase`、`checkout`、`check out`、`place order`、`pay`、`payment`、`delete`、`close account`、`unsubscribe`、`cancel subscription`、`sign in`、`sign-in`、`signin`。
  `ログイン` と `log in` は入れない(決め打ちシナリオの段階でログインを操作できなくなるため。親の判断待ち)。
- 既定のパスの区切り `pub const DEFAULT_SEGMENTS: &[&str]`:
  `checkout`、`purchase`、`buy`、`payment`、`pay`、`delete`、`destroy`、`unsubscribe`、`withdraw`、`deactivate`、`closeaccount`、`deleteaccount`、`signin`。
  送信先と遷移先の2軸で共有する。設定から足す口は今は無い(設定に項目が無いため)。

### 課金画面(`guard/paywall.rs`)

- 材料 `PaywallFacts { url: Url, headings: Vec<String>, dialogs: Vec<String>, buttons: Vec<String>, hits: Vec<bool> }`。一覧の並びは問わない。
  - `headings`: 表示領域の、モーダルの外の見出し。
  - `dialogs`: モーダル(`dialog`)の中の名前(見出しとボタン)。
  - `buttons`: 表示領域の、モーダルの外のボタンの名前。判定に使わない(材料として持ち、fixture と `Debug` で読む)。課金を促す語はここから探さない(利用者の決定。ヘッダーの「Upgrade」ボタンだけでは課金画面にしない)。
  - `hits`: 主要な操作ごとの当たり判定。主要な操作は「表示領域の操作可能要素のうち、モーダルの外にあるもの」で、アダプタが中心で `elementFromPoint` を呼び、自分に当たれば `true`。
  - `Debug` は `headings`、`dialogs`、`buttons`、`hits` の件数だけを出し、`url` は出さない。
- `pub fn blocked(hits: &[bool]) -> bool`: 当たらないもの(`false`)の数の2倍が全体の数より大きい(過半数が塞がれている)なら真。空なら偽。整数で比べ、小数を使わない。
- 判定器 `Paywall { words: Words, segments: Segments }`。
  - `Paywall::new(extra_words: &[String], extra_segments: &[String]) -> Paywall`。`Default` は `new(&[], &[])`。設定に足す項目は今は無いので、引数だけ用意する。
  - `Paywall::detects(&self, f: &PaywallFacts) -> bool`: 次の3軸の OR。
    1. 課金を促す語: `headings` か `dialogs` のどれかで `words.any_in` が真
    2. 課金系の URL: `segments.any_in(&f.url)`
    3. 主要な操作が塞がれている: `blocked(&f.hits)`
- 既定の語 `pub const DEFAULT_WORDS: &[&str]`: `アップグレード`、`プレミアム`、`有料プラン`、`有料会員`、`upgrade`、`premium`、`subscribe`。
  `料金` と `プラン` は入れない(「料金の履歴」のような通常のページを取り違えないため)。
- 既定のパスの区切り `pub const DEFAULT_SEGMENTS: &[&str]`: `pricing`、`upgrade`、`checkout`、`subscribe`、`premium`。

### 低確信度の「戻る」(`guard/confidence.rs`)

- `pub fn confidence(probabilities: &[f64]) -> Option<f64>`: 有限の値の最大値。空、または有限の値が1つも無ければ `None`。範囲(0〜1)の検証は #7 の応答の検証が受け持つので、ここではしない。
- `pub fn is_low(confidence: Option<f64>, threshold: f64) -> bool`: `None` なら真。`Some(c)` なら `c < threshold` のとき真(閾値ちょうどは低くない)。`threshold` は設定の `limits.back_confidence`(検証済みで `(0, 1]`)。
- この関数は「戻る」に対してだけ呼ぶ。操作種別は受けない(呼び分けは #12)。

### 要素の存在と同一性(`guard/identity.rs`)

- 材料 `Node { role: String, name: String, node: String }`。`node` はアクセシビリティ情報の要素参照(`Ai` 形式のスナップショットの ref)。一覧はページ全体の操作可能要素をページの順に並べたもの。
- `Target { role: String, name: String, nth: usize, node: String }`: 問い合わせたときの対象。`nth` は、同じ role と同じ名前(正規化しない、完全一致)の中での0から始まる順番。名前を `screen::normalize_name` で正規化しないのは、名前が変わったノードを別の要素として扱うため。
- `Target::at(nodes: &[Node], index: usize) -> Option<Target>`: 問い合わせ時の一覧の `index` 番目(0から)の要素から `Target` を作る。範囲の外なら `None`。
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Recheck { Same, Missing, Replaced }`。
- `pub fn recheck(target: &Target, nodes: &[Node]) -> Recheck`: 取り直した一覧から、同じ role と名前の `nth` 番目を引く。無ければ `Missing`。あって `node` が同じなら `Same`、違えば `Replaced`。操作してよいのは `Same` のときだけ。
- `Node` と `Target` の `Debug` は `name` を `…` にする。

### 許可ドメイン(`guard/domain.rs`)

- `AllowedOrigin { scheme: String, host: String, port: u16, subdomains: bool }`(フィールドは非公開)。`#[derive(Debug, Clone, PartialEq, Eq)]`。
- `AllowedOrigin::parse(s: &str) -> Result<AllowedOrigin, InvalidOrigin>`: 次のどれかに当たれば `Err(InvalidOrigin)`。
  - `Url::parse` が失敗する
  - ホストが無い(`javascript:`、`data:` など)
  - `port_or_known_default()` が `None`
  - パスが `""` と `"/"` 以外、クエリがある、フラグメントがある、ユーザー名かパスワードがある
  - ホストの `*` が先頭の `*.` 以外にある、または `*.` を除いたホストが空
  
  `*.` で始まるホストは `subdomains: true` にして、`*.` を除いたものを `host` に持つ。
- `#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)] #[error("許可ドメインはスキーム、ホスト、ポートだけのオリジンで書く")] pub struct InvalidOrigin;` 文に値を入れない。
- `AllowedOrigin::allows(&self, url: &Url) -> bool`: スキームが等しく、`port_or_known_default()` が `Some(self.port)` で、ホストが次を満たせば真。
  - `subdomains: false`: `host_str()` が `host` と等しい。
  - `subdomains: true`: `host_str()` が `.` + `host` で終わり、その前に1文字以上ある。深さは問わない(`a.b.app.example.test` も一致)。`host` 自身には一致しない。
  
  ホストの無い URL(`javascript:`、`data:`)は偽。
- `pub fn allowed_or_start(listed: Vec<AllowedOrigin>, start: &Url) -> Vec<AllowedOrigin>`: 判定に使う一覧を作る。`listed`(`target.allowed_origins`)が空なら、`start`(`target.url`)のオリジン1つ(`subdomains: false`)だけにする(利用者の決定)。`start` にホストか既知のポートが無ければ空(何も許可しない)。空でなければ `listed` のまま。
- `pub fn is_allowed(url: &Url, allowed: &[AllowedOrigin]) -> bool`: どれかが `allows` なら真。`allowed` が空なら偽。
- `Navigation { requested: Url, landed: Url }`: 操作で求めた URL と、リダイレクトのあとに着いた最終的な URL(アダプタが集める)。
- `pub fn left_allowed(nav: &Navigation, allowed: &[AllowedOrigin]) -> bool`: `!is_allowed(&nav.landed, allowed)`。`requested` は判定に使わない(記録のために持つ)。

### 設定とのつなぎ

- `config/validate/check.rs` の `origin` を `AllowedOrigin::parse(origin).map(drop).map_err(|_| ConfigError::InvalidUrl("target.allowed_origins"))` にし、`wildcard_at_start` を消す(規則を `guard::domain` の1か所に寄せる)。`Target.allowed_origins` の型は `Vec<String>` のまま変えない。`AllowedOrigin` への変換は使う側(#12、#14)が行う。
- 同じファイルのテストの `BAD_ORIGINS` に、`"https://app.example.test/p?q"`、`"https://app.example.test/path"`、`"https://app.example.test#f"`、`"https://user@app.example.test"` を足す。既存の4つはそのまま拒まれること。
- 設定の型(`config.rs`、`config/raw.rs`)は触らない。`guard.forbidden.keywords` は `Forbidden::new` の引数として使う。課金画面の語と URL を設定から足す項目は作らない。

## 失敗させるテスト

単体はそれぞれのモジュールの `tests.rs`(`guard/forbidden/tests.rs` など)。URL は `Url::parse` で作る。表で回し、テスト関数の複雑度と行数の上限に収める。

### `guard`

1. `words_match_at_ascii_boundaries`: 既定なし、追加 `["pay", "購入"]` の `Words` で、`"Pay now"` 真、`"PAY"` 真、`"Display"` 偽、`"Prepay"` 偽(前に英字が続く)、`"payment"` 偽、`"今すぐ購入"` 真、`"ｐａｙ"`(全角)真、`""` 偽。
2. `empty_keyword_is_ignored`: 追加 `["", "  ", "　"]` だけの `Words` は、`"保存"` にも `""` にも一致しない。`Segments` も追加 `["", "-"]` だけなら `https://app.example.test/` に一致しない。
3. `segments_ignore_case_hyphen_and_query`: 追加 `["signin", "pricing"]` の `Segments` で、`https://app.example.test/sign-in` 真、`https://app.example.test/Sign_In/` 真、`https://app.example.test/en/pricing` 真、`https://app.example.test/pricing-history` 偽、`https://app.example.test/?next=/pricing` 偽、`https://app.example.test/#/pricing` 偽、`data:text/html,pricing` 偽。

### `guard::forbidden`

4. `forbidden_by_keyword`: `Forbidden::default()` で、`href` と `form_action` が `None`、両リストが偽の候補について:
   - 真: `button "購入する"`、`button "アカウントを削除"`、`button "退会"`、`button "サインイン"`、`button "Checkout"`、`button "購入はこちら"`、`button "今すぐ買う"`(代替テキストが名前になった画像ボタン)、`textbox "カード番号"`、`button "ＣＨＥＣＫＯＵＴ"`、`button "Sign  In"`。
   - 偽: `button "保存"`、`link "Display settings"`、`link "Payments"`、`button ""`(名前の無いボタン、ほかの手がかりも無い)、`button "購入テスト"`(`allowed_listed: true`)。
5. `forbidden_by_destination`: 名前が手がかりにならない候補で:
   - 真: `button "OK"` で `form_action` が `https://app.example.test/account/delete`、`link "次へ"` で `href` が `https://app.example.test/checkout/confirm`、`link "次へ"` で `href` が `https://app.example.test/sign-in`、`button ""` で `form_action` が `https://app.example.test/checkout`(名前の無い画像ボタン)。
   - 偽: `link "次へ"` で `href` が `https://app.example.test/settings`、`button "検索"` で `form_action` が `https://app.example.test/search`、`link "次へ"` で `href` が `https://app.example.test/login?next=/checkout`(クエリは見ない)。
6. `forbidden_by_list`: `button "実行"` で `forbidden_listed: true` は真。`button "保存"` で `allowed_listed: true` だけは偽。`button "実行"` で両方真は偽(許可リストが優先する)。
6b. `allowed_list_wins_over_every_axis`: 名前のキーワード、遷移先、送信先、禁止リストのすべてに当たる候補は、`allowed_listed: true` なら偽、`false` なら真。遷移先だけ、送信先だけで当たる候補も、`allowed_listed: true` なら偽。
7. `extra_keywords_extend_defaults`: `Forbidden::new(&["危険".into()])` は `button "危険な操作"` を禁止し、`Forbidden::default()` は禁止しない。`Forbidden::new(&["".into()])` は `button "保存"` を禁止しない。追加しても既定の語は残る(`button "退会"` は真)。

### `guard::paywall`

8. `paywall_by_each_axis`: `Paywall::default()` で、feature の Examples の6行がどれも真。
   - URL `https://app.example.test/home`、`dialogs: ["プレミアムにアップグレード", "閉じる"]`、`buttons: ["保存"]`、`hits: [true, true, true]`
   - URL `https://app.example.test/article/1`、`headings: ["Subscribe to continue"]`、`buttons: ["アップグレード"]`、`hits: []`
   - URL が `https://app.example.test/pricing`、`/upgrade`、`/checkout` で、`headings: ["プランを選ぶ"]`、`hits: [true]`(3行)
   - URL `https://app.example.test/home`、`dialogs: ["お知らせ", "OK"]`、`hits: [false, false, false]`
9. `pricing_word_alone_is_not_paywall`: URL `https://app.example.test/billing/history`、`headings: ["料金の履歴"]`、`buttons: ["ダウンロード"]`、`hits: [true, true]` は偽。URL `https://app.example.test/`、一覧がすべて空も偽。
9b. `upgrade_button_alone_is_not_paywall`: URL `https://app.example.test/home`、`headings: ["ダッシュボード"]`、`dialogs: []`、`buttons: ["Upgrade", "プレミアム"]`、`hits: [true, true]` は偽。
10. `blocked_needs_more_than_half`: `blocked(&[])` 偽、`[false]` 真、`[true]` 偽、`[true, false]` 偽、`[false, false, true]` 真、`[true, true, false, false]` 偽、`[false, false, false, true]` 真。
11. `extra_words_and_segments`: `Paywall::new(&["お試し期間終了".into()], &["plans".into()])` は `headings: ["お試し期間終了"]`(URL `/home`、`hits: []`)、`dialogs: ["お試し期間終了"]`(同じ)、URL `/plans`(一覧がすべて空)をどれも真にし、`Paywall::default()` はどちらも偽。空の文字列の追加は何にも一致しない。

### `guard::confidence`

12. `back_confidence_boundary`: `(閾値, 確率の一覧, is_low)` が、`(0.7, [0.69], 真)`、`(0.7, [0.7], 偽)`、`(0.7, [0.71], 偽)`、`(0.5, [0.6], 偽)`、`(0.5, [0.49], 真)`、`(1.0, [1.0], 偽)`、`(0.7, [0.1, 0.69, 0.2], 真)`、`(0.7, [0.1, 0.7], 偽)`。`is_low(confidence(&一覧), 閾値)` で確かめる。`confidence(&[0.1, 0.7])` は `Some(0.7)`。
13. `back_without_probability_is_low`: `confidence(&[])` は `None`、`confidence(&[f64::NAN])` は `None`、`confidence(&[f64::NAN, 0.8])` は `Some(0.8)`、`confidence(&[f64::INFINITY])` は `None`。`is_low(None, 0.7)` と `is_low(None, 0.01)` は真。

### `guard::identity`

以下、`b(name, node)` は `role: "button"` の `Node`、`l(name, node)` は `role: "link"` の `Node`。

14. `target_at_counts_same_role_and_name`: `[b("保存","e1"), l("保存","e2"), b("保存","e3")]` で、`Target::at(_, 2)` は `Target { role: "button", name: "保存", nth: 1, node: "e3" }`、`at(_, 1)` は `nth: 0, node: "e2"` の link、`at(_, 3)` は `None`。`Target::at(&[], 0)` は `None`。
15. `missing_element_aborts`: 対象 `Target::at(&[b("保存","e1")], 0)` を、取り直した `[]` で `Missing`、`[l("利用規約","e2")]` で `Missing`。
16. `role_or_position_change_aborts`: 対象 `button "保存" nth 0 e1` を、`[l("保存","e3")]`(同じ位置でリンクに差し替わった)で `Missing`、`[b("保存する","e1")]`(名前が変わった)で `Missing`、`[b("保存","e9")]`(同じ role と名前で別のノード)で `Replaced`。対象 `Target::at(&[b("保存","e1"), b("保存","e2")], 1)` を、`[b("保存","e2")]`(上が消えて下が繰り上がった)で `Missing`。
17. `swapped_same_name_buttons_abort`: 問い合わせ時 `[b("保存","e1"), b("保存","e2")]` の上(`at(_, 0)`)を、取り直した `[b("保存","e2"), b("保存","e1")]` で `Replaced`。下(`at(_, 1)`)も `Replaced`。
18. `same_name_unique_by_position_passes`: 問い合わせ時 `[b("保存","e1"), b("保存","e2")]` の上と下を、変わらない一覧で、どちらも `Same`。先頭に `l("ヘルプ","e0")` が足された一覧 `[l("ヘルプ","e0"), b("保存","e1"), b("保存","e2")]` でも、どちらも `Same`。

### `guard::domain`

19. `allowed_origin_rejects_more_than_origin`: `AllowedOrigin::parse` が `Err(InvalidOrigin)` を返すもの: `""`、`"not an origin"`、`"https://a.*.test"`、`"https://*"`、`"https://*."`、`"https://app.example.test/p?q"`、`"https://app.example.test/path"`、`"https://app.example.test?q"`、`"https://app.example.test#f"`、`"https://user@app.example.test"`、`"https://user:pw@app.example.test"`、`"javascript:alert(1)"`、`"data:text/html,hello"`。`Ok` になるもの: `"https://app.example.test"`、`"https://app.example.test/"`、`"https://*.app.example.test"`、`"http://localhost:8080"`。`InvalidOrigin` の `to_string()` は決めた文と等しい。
20. `origin_must_match_exactly`: 許可が `["https://app.example.test"]` のとき、`is_allowed` は:
    - 真: `https://app.example.test/settings`、`https://app.example.test:443/`(既定のポートを明示)、`https://APP.example.test/`
    - 偽: `https://app.example.test.evil.test/`、`https://app.examp1e.test/`、`http://app.example.test/`、`http://app.example.test:443/`(スキームだけが違う)、`https://app.example.test:8443/`、`javascript:alert(1)`、`data:text/html,hello`、`https://sub.app.example.test/`
    
    許可が `["https://app.example.test", "https://*.app.example.test"]` のとき: `https://sub.app.example.test/` 真、`https://a.b.app.example.test/` 真、`https://app.example.test/` 真、`http://sub.app.example.test/` 偽、`https://sub.app.example.test:8443/` 偽、`https://subapp.example.test/` 偽。
    許可が `["https://*.app.example.test"]` だけのとき: `https://app.example.test/` 偽。
    許可が空のとき: `https://app.example.test/` 偽。
21. `judges_final_url`: 許可 `["https://app.example.test"]` で、`left_allowed` が:
    - 真: `requested` `https://app.example.test/go`、`landed` `https://other.example.net/`(リダイレクトで外へ出た)
    - 偽: `requested` `https://other.example.net/r`、`landed` `https://app.example.test/home`(外から中へ戻された)
    - 偽: `requested` と `landed` がどちらも `https://app.example.test/home`
21b. `empty_list_allows_only_start_origin`: `allowed_or_start(空, https://app.example.test/login?next=/)` で、`https://app.example.test/home` と `https://app.example.test:443/` は中、`https://other.example.net/`、`https://sub.app.example.test/`、`http://app.example.test/`、`https://app.example.test:8443/` は外。`allowed_or_start(["https://app.example.test"], https://other.example.net/)` では `https://app.example.test/` が中、`https://other.example.net/` が外(一覧があれば開始の URL は足さない)。開始の URL が `data:text/html,hello` と `foo://app.example.test/` なら空の一覧。

### `Debug`(秘密の値の扱い)

22. `debug_hides_page_strings`: 名前 `"山田太郎の注文"`、`href` `https://app.example.test/u/token123` の `Candidate`、`headings: ["山田太郎"]`、`dialogs: ["山田太郎", "token123"]` と URL `https://app.example.test/u/token123` の `PaywallFacts`、名前 `"山田太郎"` の `Node` と `Target` の `format!("{:?}")` に、`山田太郎` と `token123` が含まれない。

### ゴールデン(`crates/webqa-core/tests/guard_golden.rs`)

fixture は `crates/webqa-core/tests/fixtures/guard/` の JSON。テストの中に `#[derive(Deserialize)]` の行の型を置き、URL は文字列で読んで `Url::parse` で変える(`url` の serde の機能は足さない)。URL は絶対 URL で書く(下でパスだけ書いたものは、ECサイトなら `https://shop.example.test`、SaaS なら `https://app.example.test` を前に付ける)。各行は `case`(日本語の短い説明)と期待値 `expect: bool` を持つ。比べるときは `assert_eq!(実際, row.expect, "{}", row.case)`。fixture の行数が0なら失敗させる。

23. `forbidden_fixtures`: `forbidden.json`。行は `{ case, role, name, href, form_action, forbidden_listed, allowed_listed, expect }`(`href` と `form_action` は `null` 可)。`Forbidden::default()` で判定する。少なくとも次を入れる。
    - feature の Examples の行(期待値は `true`。ただし許可リストにも一致する `button "購入テスト"` は `false`)。禁止リストにもキーワードにも当たるが許可リストに当たる行(`false`)。お支払いフォームの `textbox "カード番号"` は `form_action` を `https://shop.example.test/checkout/payment` にする。
    - 見逃しを狙う行(`true`): 全角の `ＢＵＹ ＮＯＷ`、`今すぐ 購入`、`Place order`、`Sign in`、名前の無い画像ボタンで `form_action` が `/checkout`、`link "次へ"` で `href` が `/account/deactivate`、`button "OK"` で `form_action` が `/subscription/unsubscribe`。
    - 過検知を見る行(`false`): ECサイトの `link "商品一覧"`、`button "カートを見る"`、`link "Display settings"`、SaaS の `button "保存"`、`link "請求書の履歴"`(`href` `/billing/history`)、`link "ヘルプ"`。
24. `paywall_fixtures`: `paywall.json`。行は `{ case, url, headings, dialogs, buttons, hits, expect }`。`Paywall::default()` で判定する。少なくとも次を入れる。
    - feature の Examples の6行(`true`)と、「料金の履歴」の行(`false`)。
    - 見逃しを狙う行(`true`): `headings: ["Upgrade to Premium"]`、`dialogs: ["ＰＲＥＭＩＵＭ"]`、URL `/en/Pricing`、主要な操作4つのうち3つが当たらないモーダル。
    - 過検知を見る行(`false`): `headings: ["料金プランの比較"]`(URL `/help/plans`)、`buttons: ["Unsubscribe"]`、ヘッダーの `buttons: ["Upgrade"]` だけ、URL `/pricing-history`、主要な操作4つのうち2つが当たらないページ、要素も文言も無い `/` のページ。

## 受け入れ条件

`just check` が終了コード0(カバレッジ100%を含む)。`guard.rs`、`guard/` の下、`config/validate/check.rs` の mutation の生き残り0。`ports.rs`、`judge.rs`、`effect.rs`、`machine.rs` に差分が無いこと。
