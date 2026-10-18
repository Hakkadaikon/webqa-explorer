# brief: issue #6 画面のモデル

共通の決まりは同じディレクトリの `common.md`。

## タスク

`webqa-core::screen` に、要素の範囲、画面の識別、名前の正規化、画面の変化の判定を書く。
設計は `docs/architecture.md` の「要素の範囲」「画面と要素の識別」。
受け入れシナリオは `tasks/loopeng/WebExploratoryQA.feature` の T-001、T-072、T-093〜T-096(台帳 `tasks/test-design/WebExploratoryQA.md` の issue #6 の行)。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod screen;` を足すだけ)
- `crates/webqa-core/src/screen.rs` と `crates/webqa-core/src/screen/` の下(新規)
- `crates/webqa-core/Cargo.toml`(依存を足すなら `.workspace = true` で)、`Cargo.lock`

## 決めてある設計

- `Rect { x: i64, y: i64, width: i64, height: i64 }`(CSS px、表示領域が基準)、`Viewport { width: i64, height: i64 }`。
- `Element { role: String, name: String, value: Option<String>, rect: Option<Rect> }`。並びはページの順。
- `Screen { url: url::Url, elements: Vec<Element> }`。`elements` はページ全体の操作可能要素。
- `in_viewport(&Element, Viewport) -> bool`: 矩形と表示領域(0,0,width,height)が1px以上重なれば真。`rect` が無ければ偽。幅か高さが0の矩形は偽。
- `visible(&Screen, Viewport) -> Vec<&Element>`: 表示領域の中の要素だけ、ページの順で。
- `ScreenText(String)` と `screen_text(&Screen, Viewport) -> ScreenText`: 表示領域の中の要素を1行に1つ、1から番号を振って並べる。形は `1: button "保存"`、値があれば `2: textbox "メール" value="a@example.test"`。要素が0件なら空の文字列。伏せ字はここでは掛けない(判断モデルに送る前に別の処理が掛ける)。
- `changed(prev: &Screen, next: &Screen, viewport: Viewport) -> bool`: 2つの `screen_text` か URL(文字列として)の少なくとも一方が違えば真。
- `normalize_name(&str) -> String`: 前後の空白を除き、空白の連なりを半角空白1つに、数字(ASCII の 0-9 と全角の0-9)の連なりを `#` 1つに置き換える。
- `ScreenId { path: String, elements: BTreeSet<(String, String)> }` と `screen_id(&Screen) -> ScreenId`: URL のパスと、ページ全体の要素の(role, 正規化した名前)の集合。クエリと値は使わない。
- `ElementId { role: String, name: String, nth: usize }` と `element_ids(&Screen) -> Vec<ElementId>`: ページ全体の要素ごとに、正規化した名前と、同じ role と正規化した名前の中での0から始まる順番。

## 失敗させるテスト(単体、`screen` の `#[cfg(test)]` か `screen/tests.rs`)

1. `screen_text_lists_role_name_value`: 表示領域の中の `button "保存"` と値 `a@example.test` の `textbox "メール"` から、`"1: button \"保存\"\n2: textbox \"メール\" value=\"a@example.test\""` ができる(区切りは改行、末尾に改行なし)。要素0件なら空の文字列。
2. `viewport_filter_by_box`: 表示領域 800x600 で、(0,0,10,10) は中、(790,590,20,20) は中(重なりあり)、(800,0,10,10) は外(重なり0)、(0,-20,10,20) は外、(0,-20,10,21) は中、矩形なしは外、幅0は外。`visible` と `screen_text` は外の要素を含まない。
3. `scroll_changing_viewport_is_change`: 同じ URL、同じページ全体の要素で、表示領域の中の要素だけが変わった2つの画面は `changed` が真。
4. `scroll_at_bottom_is_unchanged`: 表示領域の中の要素も値も URL も同じなら `changed` は偽。値だけが変われば真、URL だけが変われば真。
5. `scroll_keeps_screen_identity`: 表示領域の中だけが違い、ページ全体の要素とパスが同じ2つの画面は、`screen_id` が等しい。クエリ文字列と値が違っても等しい。
6. `page_element_change_is_new_screen`: ページ全体の要素が1つ増えた、または名前が変わった画面は `screen_id` が違う。パスが違っても違う。
7. `normalize_collapses_digits_and_spaces`: `"通知 (3)"` と `"通知 (12)"` はどちらも `"通知 (#)"`。`"  保存  する "` は `"保存 する"`。全角の数字も `#`。空の文字列は空のまま。
8. `element_ids_count_same_role_and_name`: `button "保存"`、`link "保存"`、`button "保存"` の順なら、nth は 0、0、1。名前が `"通知 (3)"` と `"通知 (4)"` の2つの button は、同じ名前 `"通知 (#)"` で nth が 0 と 1。

## 受け入れ条件

`just check` が終了コード0。`screen` 配下の mutation の生き残り0。
