# brief: issue #10 探索の記憶

共通の決まりは同じディレクトリの `common.md`。

## タスク

`webqa-core::explore` に、探索の記憶のグラフ(`graph`)と、探索モードの候補の絞り込み(`select`)を書く。
設計は `docs/architecture.md` の「探索の記憶と候補の選び方」(要素の範囲、画面と要素の識別、候補の絞り込み)と「複雑度の上限に耐える書き方」。
受け入れシナリオは `tasks/loopeng/WebExploratoryQA.feature` の T-030〜T-034、T-070(台帳 `tasks/test-design/WebExploratoryQA.md` の issue #10 の行)。受け入れの配線は #14 の受け持ちで、ここでは単体テストだけを書く。

並行して #15 が `ports.rs`、`judge.rs`、`effect.rs`、`machine.rs` を、#4 が `guard.rs` を作る。この issue はそれらを使わず、触らない。
禁止要素かどうかは #4 の判定に頼らず、呼び出し側から要素の番号の集合で受け取る。

### 範囲の外

- ファイルへの保存(JSON の読み書き、一時ファイルと名前の変更、壊れたファイルの退避、`schema` の確認)と、JSON との変換(serde)。保存と変換は、読み込んだ値の検証(存在しない画面を指す辺、`schema` の不一致)と組で書くので、`webqa` の `store::memory` を作る issue でまとめて受け持つ。この issue では serde の derive も足さない。ただし型は、下の「決めてある設計」のとおり architecture.md の JSON の形(画面の一覧、要素ごとの `nth` と `operated`、辺の `from`・`element`・`op`・`to`)と1対1に対応させておく。
- 探索モードのゴールの判定(T-081〜T-083、T-108、T-109)。#13 の受け持ち。この issue の `Memory::untouched_reachable` を #13 がゴールの判定に使う。
- 判断モデルへの問いの組み立て(選択肢を候補に絞ること)。#7 と #12 の受け持ち。この issue は候補を返すところまで。
- 「未操作の要素が新しく見えた」の判定。`State` の受け持ち(#12)。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod explore;` を足すだけ。#15 と #4 も同じファイルに1行足すので、取り込みで衝突したら両方の行を残す)
- `crates/webqa-core/src/explore.rs` と `crates/webqa-core/src/explore/` の下(新規。`graph.rs`、`select.rs`、テストのファイル)

依存は足さない。`screen.rs` と `redact.rs` は使うだけで変えない。

## 決めてある設計

### モジュールの形

- `explore.rs` は `pub mod graph; pub mod select;` と、`pub use graph::{ElementOp, Memory, ScreenKey}; pub use select::{Candidate, Candidates, candidates};` だけを持つ。
- テストは `explore/graph/tests.rs` と `explore/select/tests.rs` に置く(`#[cfg(test)] mod tests;`)。テストの関数のパスは `webqa_core::explore::graph::tests::…` と `webqa_core::explore::select::tests::…` になる(graph のテストは Halstead の上限のため `graph/tests/` の下の4ファイルに分けたので、`…::graph::tests::finished::…` のように1段深い)。
- `mod.rs` は使わない。

### `graph`: 探索の記憶

```rust
/// 記憶の中の画面を指す。作れるのは Memory だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenKey(usize); // 画面の一覧での位置。JSON では "s{位置+1}"

/// 辺に記録する、要素への操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementOp { Click, Input }

/// 対象のオリジン1つ分の探索の記憶。
#[derive(Debug, Clone, Default)]
pub struct Memory { /* 非公開 */ }
```

非公開の中身は次の形にする(名前は任せる)。

- 画面の一覧 `Vec<Node>`。`Node` は `ScreenId` と、要素の一覧 `Vec<Entry>`。`Entry` は `ElementId`、`operated: bool`、`forbidden: bool`。
- 辺の一覧 `Vec<Edge>`。`Edge` は `from: ScreenKey`、`element: ElementId`、`op: ElementOp`、`to: ScreenKey`。
- `ScreenId` と `ElementId` は `Hash` も `Ord` も持たないので、引くときは一覧を `==` で線形にたどる。`screen.rs` に derive を足さない。
- `Debug` は derive でよい。`ScreenId` と `ElementId` の `Debug` が名前を伏せるので、名前は出ない。`String` で名前を持つフィールドを足さない。

公開する関数は次のとおり。

```rust
impl Memory {
    pub fn visit(&mut self, screen: &Redacted<Screen>, forbidden: &BTreeSet<usize>) -> ScreenKey;
    pub fn record(&mut self, from: ScreenKey, element: &ElementId, op: ElementOp, to: ScreenKey) -> bool;
    pub fn is_operated(&self, at: ScreenKey, element: &ElementId) -> bool;
    pub fn finished(&self, at: ScreenKey, element: &ElementId) -> bool;
    pub fn untouched_reachable(&self, from: ScreenKey) -> bool;
    pub fn screen_count(&self) -> usize;
    pub fn edge_count(&self) -> usize;
}
```

- `visit`: `screen_id(screen)` が等しい画面が記憶にあればその鍵を返し、無ければ末尾に足して新しい鍵を返す。
  - `forbidden` は `screen.elements` の位置(0から、ページの順。`element_ids(screen)` の並びと同じ)の集合。範囲の外の番号は無視する。
  - 新しい画面では、`element_ids(screen)` の各要素を `operated: false` で、`forbidden` は集合に入っているかで登録する。
  - 既にある画面では、記憶に無い `ElementId` を末尾に `operated: false` で足す(`ScreenId` は要素の集合なので、同じ名前の要素が増えても同じ画面になり、`nth` の大きい要素が新しく現れうる)。今の画面にある要素の `forbidden` は今回の集合で上書きする。今の画面に無い要素は消さず、`operated` と `forbidden` もそのまま残す。`operated` は `visit` では変えない。
  - 要素0件の画面も1つの画面として登録する。
- `record`: `from` の画面の `element` を操作済みにし、辺 `(from, element, op, to)` を足す。
  - 戻り値は、この呼び出しで `element` が未操作から操作済みに変わったときだけ真(#12 が「探索済みの要素の集合が増えたか」に使う)。
  - 同じ `(from, element, op, to)` の辺がもうあれば足さない。`op` か `to` が違えば別の辺として足す。
  - `from` か `to` がこの記憶に無い鍵のとき、または `element` が `from` の画面に無いときは、何も変えずに偽を返す。
  - `to == from`(操作しても画面の識別が変わらなかった)の辺も記録する。
- `is_operated`: その画面のその要素が操作済みなら真。画面か要素が記憶に無ければ偽。
- `untouched_reachable`: `from` の画面と、辺をたどって `from` から到達できるすべての画面のどこかに、`operated: false` かつ `forbidden: false` の要素があれば真。辺は要素と操作を問わずたどる。たどった画面を集合に覚え、循環があっても止まる。`from` が記憶に無ければ偽。
- `finished`(探索し終えたか): 次のすべてを満たすとき真。
  1. `is_operated(at, element)` が真。
  2. `from == at` かつ `element` が等しい辺のうち、`to != at` のすべての辺について、`untouched_reachable(to)` が偽。
  - 辺が無い、または `to == at` の辺だけの要素は、操作した時点で探索し終えたとみなす(台帳の用語定義「遷移の無い要素は操作した時点で探索し終えたとみなす」)。`to == at` の辺を数えないのは、自分の画面に残る未操作の要素で、入力欄のような遷移の無い要素がいつまでも探索し終えないのを避けるため。自分の画面に残る未操作の要素は、表示領域の中なら候補の1段目が、外なら3段目のスクロールが拾う。
  - (レビューで足した規則)`to` から到達できる画面をたどるとき、たどった画面の集合に最初から `at` を入れておく。`to` から `at` へ戻る循環を通って `at` 自身の未操作の要素を数えると、表示領域の外の未操作の要素がある画面で2段目が出続け、スクロールに落ちないため。
  - 未操作の要素、記憶に無い画面や要素は偽。
- `#13` の探索モードのゴールは `!memory.untouched_reachable(開始の画面の鍵)` で判定できる。この issue ではゴールの判定は書かない。

### `select`: 候補の絞り込み

```rust
/// 候補の1件。number は screen_text の行の番号(表示領域の中の要素に1から振った番号)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate { pub number: usize, pub id: ElementId }

/// 候補のある最上位の段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Candidates {
    /// 1段目: 今の画面の未操作の要素(禁止要素を除く)。
    Untouched(Vec<Candidate>),
    /// 2段目: 未探索の分岐が残る操作済みの要素(禁止要素を除く)。
    Unfinished(Vec<Candidate>),
    /// 3段目: 戻る、下スクロール、上スクロール。
    Moves,
}

pub fn candidates(memory: &Memory, at: ScreenKey, screen: &Redacted<Screen>, viewport: Viewport) -> Candidates;
```

- 呼び出し側は、先に `memory.visit(screen, forbidden)` で `at` を得てから呼ぶ。禁止かどうかは `visit` で記憶に入れた `forbidden` を使う。
- 段を分ける対象は、表示領域の中の要素(`in_viewport` が真の要素)だけ。判断モデルが選べるのは `screen_text` に番号の出た要素だけなので、表示領域の外の未操作の要素は1段目にも2段目にも入らない。外にしか未操作の要素が無ければ、下の段へ落ち、3段目のスクロールで表示領域に出す。
- `number` は、`screen.elements` のうち表示領域の中の要素に、ページの順で1から振った番号(`screen_text` の行の番号と同じ)。
- 要素の `ElementId` は `element_ids(screen)` の同じ位置のものを使い、記憶の `at` の画面の要素と `==` で引く。
- 1段目: 表示領域の中で、記憶の上で `operated: false` かつ `forbidden: false` の要素。ページの順。
- 2段目: 1段目が空のとき、表示領域の中で、`operated: true` かつ `forbidden: false` かつ `memory.finished(at, id)` が偽の要素。ページの順。
- 3段目: 1段目も2段目も空のとき `Candidates::Moves`。戻る・スクロールの操作種別の型は #15 と #12 の受け持ちなので、ここでは名前を持たない値にする。
- `at` の画面の要素に無い `ElementId` の要素(`visit` に渡した画面と違う画面を渡したとき)は、どの段にも入れない。
- `at` が記憶に無い鍵なら `Candidates::Moves`(禁止の印を引けないので、要素を候補にしない側に倒す)。
- 要素0件の画面、表示領域の中に要素が無い画面は `Candidates::Moves`。

### 複雑度

- 段の判定は `fn untouched(entry) -> bool`、`fn unfinished(memory, at, entry) -> bool` のような名前の付いた述語に分け、`&&` と `||` を1つの式に並べない。
- 到達できる画面のたどりは、作業用の列とたどった鍵の集合を持つ小さな関数に分ける(`while let` の中で分岐を重ねない)。
- 一覧は `get` で引き、添字と `unwrap` と `as` を使わない。

## 失敗させるテスト(単体、`explore/graph/tests.rs` と `explore/select/tests.rs`)

共通の準備: 表示領域は 800x600。要素は `role` と名前を持ち、矩形は特に書かない限り (0,0,10,10)(表示領域の中)、「外」と書いたものは (0,700,10,10)。画面は `Redactor::default().screen(Screen { … })` で作る。URL のオリジンは `https://app.example.test`。禁止の集合は書かない限り空。

### `select`(台帳の行)

1. `untouched_before_unfinished_branch`(T-030): 画面 "トップ" `/` に `button "保存"`、`link "ヘルプ"`。画面 "設定" `/settings` に `button "通知"`。両方を `visit` し、`record(トップ, 保存, Click, 設定)`。
   - `memory.finished(トップ, 保存)` は偽("設定" に未操作の "通知" が残る)。
   - `candidates(&memory, トップ, &トップの画面, vp)` は `Candidates::Untouched(vec![Candidate { number: 2, id: link "ヘルプ" nth 0 }])`。"保存" は入らない。
2. `unfinished_branch_before_back_and_scroll`(T-031): 1 の続きで `record(トップ, ヘルプ, Click, トップ)`(画面が変わらない操作)。
   - `memory.finished(トップ, ヘルプ)` は真。
   - `candidates` は `Candidates::Unfinished(vec![Candidate { number: 1, id: button "保存" nth 0 }])`。
3. `back_or_scroll_when_all_finished`(T-032): 画面 "トップ" `/` に `button "保存"`、`link "ヘルプ"`。画面 "設定" `/settings` に `button "通知"`。`record(トップ, 保存, Click, 設定)`、`record(設定, 通知, Click, 設定)`。"トップ" の "ヘルプ" は未操作のまま。
   - "設定" での `candidates` は `Candidates::Moves`("設定" から "トップ" への辺は無いので、"ヘルプ" は "設定" の要素の分岐に数えない)。
   - `memory.finished(設定, 通知)` は真。
   - もう1ケース: "設定" の "通知" が画面 "メール" `/settings/mail`(`button "解除"` を操作済み、遷移なし)へ移る辺だけを持つときも `Candidates::Moves`。
4. `payment_elements_never_candidates`(T-034): 画面 `/cart` に `button "購入する"` だけ。禁止の集合 `{0}` で `visit`。
   - `candidates` は `Candidates::Moves`。
   - もう1ケース: `button "購入する"`、`link "ヘルプ"` で禁止 `{0}` なら `Candidates::Untouched(vec![Candidate { number: 2, id: link "ヘルプ" nth 0 }])`。
5. `forbidden_untouched_excluded`(T-070): 画面 "アカウント" `/account` に `button "保存"`、`button "退会"`。禁止 `{1}` で `visit`。画面 "トップ" `/` に未操作の `link "ヘルプ"` を `visit`(辺は張らない)。`record(アカウント, 保存, Click, アカウント)`。
   - "アカウント" での `candidates` は `Candidates::Moves`。
   - `memory.untouched_reachable(アカウント)` は偽(未操作は禁止要素の "退会" だけ)。
   - `memory.is_operated(アカウント, 退会)` は偽(未操作のまま残る)。

### `graph`(台帳の行)

6. `finished_follows_reachable_screens`(T-033): 画面 "トップ" `/` に `button "設定"`。画面 "設定" `/settings` に `button "通知"`。
   - `record(トップ, 設定, Click, 設定)` のあと "トップ" を同じ画面でもう一度 `visit` すると、返る鍵は最初の "トップ" と等しい。`is_operated(トップ, 設定)` は真、`finished(トップ, 設定)` は偽。
   - `record(設定, 通知, Click, メール)` で画面 "メール" `/settings/mail`(未操作の `button "解除"`)へ移ると、`finished(トップ, 設定)` は偽のまま(2段先に未操作が残る)。
   - "メール" に `link "トップ"` を足した版で、`record(メール, 解除, Click, メール)`、`record(メール, トップ, Click, トップ)` まで済むと、循環("トップ" → "設定" → "メール" → "トップ")があっても止まり、`finished(トップ, 設定)` は真。
   - 別ケース: "メール" の未操作が禁止要素(禁止 `{0}` の `button "解除"`)だけなら、`finished(トップ, 設定)` は真。

### 台帳に無いが、境界と分岐を押さえるテスト

7. `empty_memory_knows_nothing`: `Memory::default()` は `screen_count() == 0`、`edge_count() == 0`。別の `Memory` の `visit` で得た鍵を渡すと、`is_operated`、`finished`、`untouched_reachable` はどれも偽、`record` は偽を返して `edge_count()` は 0 のまま、`candidates` は `Candidates::Moves`。
8. `empty_screen_offers_moves`: 要素0件の画面 `/empty` を `visit` すると `screen_count() == 1`。`candidates` は `Candidates::Moves`、`untouched_reachable` は偽。要素はあるがすべて表示領域の外の画面でも `Candidates::Moves`(未操作の要素は記憶にあり、`untouched_reachable` は真)。
9. `visit_reuses_the_same_screen`: 同じパスと要素の集合で、クエリ、値、矩形だけが違う画面は同じ鍵を返し、`screen_count()` は 1 のまま。要素が1つ増えた画面は別の鍵で `screen_count() == 2`。同じ "トップ" に同名の `button "保存"` が2つになった画面を `visit` すると同じ鍵が返り、`button "保存"` nth 1 が未操作で増える(`candidates` の1段目に出る)。禁止の集合を `{0}` から空に変えて `visit` し直すと、その要素は1段目に戻る。今回の画面に無い要素の `operated` は残る。範囲の外の番号(例 `{9}`)は無視する。
10. `record_reports_new_operation_once`: 最初の `record` は真、同じ引数の2回目は偽で `edge_count()` は 1 のまま。同じ要素を別の `to` へ記録すると偽で `edge_count() == 2`、`Input` で記録すると偽で `edge_count() == 3`。`from` の画面に無い要素を渡すと偽で、`edge_count()` も `is_operated` も変わらない。
11. `untouched_reachable_follows_edges_and_stops_on_cycles`: "トップ" と "設定" を互いに辺で結び(循環)、要素をすべて操作済みにすると、どちらからも偽で、呼び出しが返る。"設定" に未操作の要素を足して `visit` し直すと、"トップ" から真。辺の無い別の画面の未操作の要素は "トップ" からの結果に入らない。
12. `candidates_follow_viewport_numbers`: 画面に `button "上"`(外、矩形 (0,-50,10,10))、`button "保存"`(中)、`link "下"`(外)、`link "ヘルプ"`(中) の順。どれも未操作なら `Candidates::Untouched(vec![{ number: 1, 保存 }, { number: 2, ヘルプ }])`。`visit` した画面と違う要素を持つ画面を同じ鍵で渡すと、記憶に無い要素は候補に入らない(その要素だけの画面なら `Candidates::Moves`)。
13. `memory_debug_hides_names`: 名前 "保存" の要素を持つ記憶の `format!("{memory:?}")` に "保存" が含まれない。

テストは `assert_eq!` で値ごと比べる。`Candidates` は段と中身をまとめて比べる。

## 受け入れ条件

`just check` が終了コード0(カバレッジ100%を含む)。`just mutants --in-place -f crates/webqa-core/src/explore.rs -f crates/webqa-core/src/explore/graph.rs -f crates/webqa-core/src/explore/select.rs` の生き残り0。
`screen.rs`、`redact.rs`、#15 と #4 の受け持ちのファイルを変えていない。
