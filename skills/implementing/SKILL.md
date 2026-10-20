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

1. `docs/arch/overview.md`で、触るcrateとモジュールの受け持ちを確かめる。
   `docs/arch/code-style.md`と`docs/arch/testing.md`は必ず読む。
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
- コミットの前に、触ったモジュールのテストを回し、フックの検査を通す。
  全体のテストとカバレッジは、PRのCIで確かめる。
- コミットメッセージは`commit-message`のskillに従う。
  分け方は`hymme:micro-commit`に従う。

## 落とし穴

- git worktreeで作業するときは、最初に`just worktree-tools`を実行する。
  worktreeには`.aqua`と`.config/npm/node_modules`が無く、整形とフックが落ちる。
- 複雑度はクロージャの中の分岐も外側の関数に数える。
  テスト関数も複雑度と行数(50行)の上限を受ける。
  長いテストは、確かめる組を表にして補助関数で回す。
- ファイルごとのHalsteadの量にも上限がある。
  超えたら、話題ごとにファイルを分ける(例: 秘密の値の読み取りを別ファイルにする)。
- テストの`matches!`や`if let`は、通らない側がリージョンとして残り、カバレッジが100%に届かない。
  期待値は`assert_eq!`で値ごと比べる。
- 包むだけのエラー型は、中身を誰も読まないとdead_codeになる。
  まず元のエラー型をそのまま返し、要るようになってから自前の型を足す。
- 空の文字列や空の一覧を受け取る関数は、空のときの振る舞いを先に決める。
  例えば、空の文字列で伏せ字を作ると、文字の間すべてに一致する。
- ディスクの空きが少ないときは、mutationを`just mutants --in-place`で回す。
  worktreeの中なら、ほかの作業を壊さない。

## 役割分担

誰が何を決めるかは`docs/agents.md`にある。
テストの名前と期待値は親セッションが決め、briefで渡す。
briefに無い設計の判断が要るときは、実装を始めずに親セッションへ返す。
