---
name: implementing
description: >
  このリポジトリでRustのコードを実装するときの手順。
  crates/のコードやテストを書く・直す前、implementerとして
  briefを受け取ったとき、issueを実装するときに必ず使う。
  読む文書の順番と、品質ゲートの下で守る規則をまとめている。
---

# 実装する

## 読む順番

1. `docs/architecture.md`で、触るcrateとモジュールの受け持ちを確かめる。
   「複雑度の上限に耐える書き方」と「テストの層」は必ず読む。
2. `docs/quality-gates.md`で、基準と落ちたときの直し方を確かめる。
3. `tasks/test-design/WebExploratoryQA.md`で、briefが指す行を読む。
   確かめる振る舞い、出典、手法を確かめる。
   手法の欄の技法は、`hymme:test-catalog`の索引から引いて開く。
   テストの名前と期待値はbriefに従い、台帳と食い違えばNEEDS_CONTEXTで返す。
4. Skillツールで`superpowers:test-driven-development`を呼び、その手順で書く。
   テストを先に書き、期待どおりの理由で落ちることを確かめてから実装する。

## 守ること

- ゲートの閾値と、`.config/`・`Cargo.toml`・`justfile`の設定を変えない。
- lintやカバレッジの除外の注釈を足さない。
  例外は、本物の時計を置く`webqa`の`clock.rs`の1か所だけである。
- テストを弱めない。
  assertを消す、期待値を緩める、skipするのはどれも禁止である。
- テストの名前とコードに、台帳の管理番号やTLA+の性質の名前を書かない。
  名前は、確かめる振る舞いの言葉で付ける。
- 手元の絶対パスをファイルに書かない。
- `--no-verify`でgitのフックを飛ばさない。
- コミットの前に`just check`を実行し、終了コード0を確かめる。
- コミットメッセージは`commit-message`のskillに従う。
  分け方は`hymme:micro-commit`に従う。

## 役割分担

誰が何を決めるかは`docs/agents.md`にある。
テストの名前と期待値は親セッションが決め、briefで渡す。
briefに無い設計の判断が要るときは、実装を始めずに親セッションへ返す。
