# brief: issue #29 試行ごとに開いたブラウザに前の保存領域が残らないことを確かめる

共通の決まりは同じディレクトリの `common.md`。

## タスク

インタビューのモードは、試行ごとにブラウザを新しく開き、認証状態(`auth.storage_state`)を入れ直す。
新しく開いたブラウザに、前の試行で書いた保存領域(Cookie と localStorage)が残らないことを、実物の Chromium で確かめる契約テストを1つ足す。
台帳は `tasks/test-design/InterviewMode.md` の T-013。テストの名前は `new_context_has_no_previous_storage`。
開き直しを呼ぶ試行のループ(`webqa::interview::trials` の `each_trial_opens_fresh_browser_with_storage_state`)は #30 の受け持ちで、範囲外。

製品のコードは変えない前提。テストを書いてみて、`PlaywrightBrowser` の公開の API だけでは確かめられないと分かったら、製品のコードに手を入れず NEEDS_CONTEXT で返す。

## 触ってよいファイル

- `crates/webqa-browser/tests/contract/pages.rs`(テストを足す。`state_file` と `STATE` を使い回してよい)
- `crates/webqa-browser/tests/contract/serve.rs`(保存領域を読むページの経路を足すときだけ)
- `crates/webqa-browser/tests/fixtures/pages/` の下に新しいページを足してよい。足したら `session.rs` の `PAGES` に行を足す(この1か所だけ `session.rs` を触ってよい)

## 決めてある設計

- 1回目の試行: `storage_state` に `state_file()` を渡して `launch` する。開いたページで、スクリプトを使って Cookie `trial=1` と localStorage の `trial` = `1` を書く。書き方は `launch_with` の `prepare` で受け取る `BrowserContext` を使うか、ページのスクリプトで書く。書けたことを、同じブラウザの観測で確かめる(下の読むページ)。そのあとブラウザを捨てる(drop)。
- 2回目の試行: 同じ `state_file()` で、もう一度 `launch` する。
- 読むページ: 保存領域の中身を見出しかボタンの名前に写すページを1枚足す。例は `/storage` で、`document.cookie` と `localStorage.getItem("trial")` を読み、`button "cookie: <値>"` と `button "local: <値>"` のように出す。名前は観測の `kinds` で比べられる形にする。
- 期待:
  - 1回目の観測は、`session=ok` と `trial=1` の Cookie、localStorage の `trial` = `1` を示す。
  - 2回目の観測は、`session=ok` の Cookie だけを示す。`trial` の Cookie も localStorage の値も無い。
- `assert_eq!` は観測の `kinds` を丸ごと比べる(common.md の決まり)。
- Cookie を `secure: true` で入れているので、ページの経路は既存の `ORIGIN`(https)を使う。`/account` と同じく `route` で答える。

## 失敗させるテスト

1. `new_context_has_no_previous_storage`: 上の2回の試行。先に、2回目の期待を「1回目と同じ」と書いて落ちることを確かめてから、正しい期待に直す。Red の確かめ方を報告に書く。

## 受け入れ条件

`just check` が終了コード0。製品のコード(`crates/webqa-browser/src/`)に差分が無い。テストの実行時間を報告に書く(契約テスト全体の時間が大きく延びないこと)。
