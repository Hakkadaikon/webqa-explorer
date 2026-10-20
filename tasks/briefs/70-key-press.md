# issue #70 探索の操作の種類に Enter を押す操作を足す

文中の `ファイル:行` は、この brief を書いた時点の行を指す。

共通の決まりは `common.md`。進め方は skill `parallel-work`(全体のテストとカバレッジは PR の CI で確かめる)。

## 背景

実物の Playwright の TodoMVC デモで探索したら、入力欄に値を打ち込んでも ToDo が追加されなかった。
TodoMVC は、打ち込んだあとに Enter を押して追加する。
今の操作の種類(click、input、scroll_down、scroll_up、back、wait、done、stuck)には、キーを押す操作が無い。
利用者は 2026-10-20 に、キーを押す操作を足すと決めた。

## 推す設計(下の「親に決めてほしいこと」で確定する)

操作の種類に `enter` を1つ足す。
押すキーは Enter だけで、対象の要素を取らない。
押す先は、そのときフォーカスのある要素である。

- `input` は Playwright の `fill` で打つので、打った入力欄にフォーカスが残る。
  次のステップで `enter` を選べば、その入力欄で Enter を押したことになる。
- 対象を取らないので、探索の候補の段(未操作の要素から先に選ぶ)と関わらない。
  打ったあとの入力欄は操作済みになり候補から外れるので、対象を取る形にすると押せなくなる。
- 記憶には数えない(`scroll_down` などと同じく要素の操作ではない)。
  画面が変われば、変わった先の画面が探索の記憶に加わる。

ブラウザは、フォーカスのある要素が入力欄の役割(`screen::fillable`)のときだけ Enter を押す。
そうでなければ何もしない。
画面が変わらなければ、固まりの数え方で扱う。

### 禁止要素との関係

Enter はフォームを送るので、送信のボタンが禁止要素でも送れてしまう。
これを防ぐため、探索モードでもシナリオでも、観測したページに禁止要素が1つでもあれば `enter` を選択肢から外す。
判定は、中核の `machine/material.rs` の `forbidden_indices` と同じ結果を使う(`Seen` の禁止要素)。

### 探索モードで選択肢に出す条件

`input` と同じ条件にする。
その段の候補に、入力欄の役割で設定の inputs に名前のある要素が1つも無ければ、`enter` も外す。
`input` を外す処理(`offer.rs` の `without_input`)を、`input` と `enter` の2つに当てる形にする。
ただし `enter` は、打ったあとの画面で選べないと意味が無い。
打った入力欄は操作済みになり候補の段から外れるので、`enter` の判定だけは候補の段でなく、表示中の要素全体(`visible`)で見る。

### シナリオでの扱い

設定の `ops` の既定(`config/raw.rs` の `all_ops`)に `enter` を足す。
設定に `ops` を書いた利用者は、`enter` を足さない限り使わない。
シナリオでも、禁止要素の判定で外す。

## 触ってよいファイル

- `crates/webqa-core/src/config/raw.rs`(`Op::Enter`、`all_ops`)
- `crates/webqa-core/src/machine/ops.rs`(表の行 `enter`、`element: None`)
- `crates/webqa-core/src/machine/offer.rs`(選択肢の条件、禁止要素で外す)
- `crates/webqa-core/src/machine/rules.rs`(判定の補助が要れば)
- `crates/webqa-core/src/machine/actions/explore_tests/` と `crates/webqa-core/src/machine/tests.rs` 系のテスト
- `crates/webqa-browser/src/ops.rs`(表の行 `enter`、フォーカスのある入力欄だけに Enter)
- `crates/webqa-browser/src/ops/tests.rs`、`crates/webqa-browser/tests/contract/`(契約テスト)
- `crates/webqa/tests/steps/`、`crates/webqa/tests/support/`(受け入れの手順)
- 仕様と記録: 下の「仕様の変更」に挙げたファイル
- `docs/architecture.md`(操作の種類の表と、探索モードの選択肢の説明)

## テスト(名前と期待値)

単体(`webqa-core`):

1. `enter_is_offered_only_with_a_field_that_takes_a_value`
   - 探索モードで、表示中に値のある入力欄がある画面: 選択肢に `"enter"` が含まれる。
   - 値のある入力欄が無い画面: 含まれない。
2. `enter_stays_after_the_field_is_typed`
   - 入力欄に打って操作済みになったあとの画面でも、表示中にその入力欄があれば、選択肢に `"enter"` が含まれる。
3. `enter_is_dropped_when_a_forbidden_element_is_on_the_page`
   - 値のある入力欄があっても、禁止要素が1つでもあれば `"enter"` が含まれない。シナリオでも同じ。
4. `enter_needs_no_target`
   - 判断モデルが `enter` を返したとき、対象の要素を問わずに実行し、記憶に要素の操作を足さない(`screen_count` と操作済みの要素の数が変わらない)。

単体(`webqa-browser`):

5. `enter_presses_only_into_a_focused_input_field`
   - フォーカスが入力欄にあれば Enter を押す(契約テストで、Enter で文字列が追加されるページを使う)。
   - フォーカスがボタンや本文にあれば何もしない(ページが変わらない)。

受け入れ(feature の Scenario、下の「仕様の変更」):

6. 入力欄に打ったあと Enter を押すと、画面が変わり、変わった先の画面が探索の記憶に加わる。
7. 禁止要素がページにあれば、操作種別の選択肢に enter を含めない。
8. 既存の「操作種別の選択肢は … の6種である」(feature 409 行)は、開始の画面に入力欄が無いので変わらない(6種のまま)ことを確かめる。

実物: TodoMVC のデモで、入力欄に打って Enter を押し、ToDo が1件増えることを確かめる(手で回す)。

## 仕様の変更(仕様)

- feature `tasks/loopeng/WebExploratoryQA.feature` と `crates/webqa/tests/features/` の写し:
  - 「探索の記憶と優先順位」の Rule に、上の 6 の Scenario を足す。
  - 「ガードレール」の Rule に、上の 7 の Scenario を足す。
  - 655 行の Outline(値の設定された入力欄が候補にあるときだけ input を含める)の Then は input だけを見ているので、変えない。
- 要件の文書 `tasks/loopeng/WebExploratoryQA.extract.md`:
  - S-005 の「operation(choice)8種」を9種にする。
  - S-016 か S-022 に「表示中に値のある入力欄が無ければ enter を選択肢から外す」を state の文で足す。
  - ガードレールの S-ID に「ページに禁止要素があれば enter を選択肢から外す」を unwanted の文で足す。
  - トレース表の該当の行に、新しい Scenario 名を足す。
- モデルの説明 `WebExploratoryQA.model.md` と `WebExploratoryQA.faithfulness.md`:
  - `enter` はモデルでは対象の無い操作(`wait` や `scroll_down` と同じ)として扱う、と簡略化の節に書く。
  - 禁止要素で外すことは、モデルの外(選択肢を組む前の絞り込み)とする。
- TLA+ `WebExploratoryQA.tla`:
  - 42 行の `ASSUME Operations \subseteq {...}` に `"enter"` を足す。
  - `DirectOps`(104 行)には足さない。対象を取らないため。
  - 3つの cfg(`WebExploratoryQA.cfg`、`.empty.cfg`、`.noexplorable.cfg`)の `Operations` は `{"click", "back", "done", "stuck"}` のまま変えない。
  - 状態空間は変わらないが、`ASSUME` を変えたので、TLC を3つの cfg で回し直し、結果が前と同じ(違反なし、状態数が同じ)ことを `WebExploratoryQA.tlc-run.log` に足す。
- テスト項目の台帳 `tasks/test-design/WebExploratoryQA.md`:
  - 上の 6、7 の行を足し、出典の行番号と sha256 を直す。`check_ledger.py` の Scenario 数を合わせる。
  - 8 は既存の行の備考に1文足す。

仕様の変更は、実装のあとに loop-engineering-reviewer の合格まで通す(親が回す)。

## 親に決めてほしいこと

1. **操作の形**: 推す案は、対象を取らない `enter` 1つ(上の設計)。別案は、`press` に押すキーを選ばせる形(Enter、Tab、Escape など)。キーを選ばせると問いが1つ増え、判断モデルの選択肢も増える。TodoMVC に要るのは Enter だけなので、推す案にする。
2. **禁止要素との関係**: 推す案は、ページに禁止要素が1つでもあれば `enter` を外す。別案は、フォーカスのある入力欄の属するフォームに禁止要素があるときだけ外す(ブラウザがフォームを調べる)。別案のほうが探索は広がるが、ブラウザの処理と契約テストが増える。安全を先にして推す案にする。
3. **何も起きなかった enter の記録**: 推す案は、記録を足さない(画面が変わらなければ固まりで扱う)。別案は、フォーカスが入力欄に無いとき `skipped`(cause `not_focused`)を記録する。別案はブラウザの結果を中核へ返す口が要るので、推す案にする。
4. **シナリオの既定の ops に入れるか**: 推す案は、既定に入れる。別案は、既定に入れず、使う人が設定に書く。探索モードは既定の ops を使うので、入れないと探索で押せない。

## 規模の見込み

コードは中核とブラウザで 100〜150 行、テストは 150〜200 行。
仕様の文書の直しと TLC の回し直しを入れて、実装担当1本で2〜3時間。
