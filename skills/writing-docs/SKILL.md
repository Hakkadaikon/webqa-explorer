---
name: writing-docs
description: >
  このリポジトリで日本語の文書を書く・直すときの手順。
  docs/、README.md、CONTRIBUTING.md、AGENTS.md、skills/、agents/、
  .omm/ の説明を書くとき、図を描くときに必ず使う。
  文章はyomiyasu、構成図はoh-my-mermaidで作る。
---

# 文書を書く

## 置き場所

文書は`docs/`に置く。
直下に置くのは、ツールや慣習が場所を決めているもの(README.md、CONTRIBUTING.md、AGENTS.md、CLAUDE.md)だけにする。
アーキテクチャの図と説明は、oh-my-mermaidが`.omm/`に書き出す。

## 文章を書く

1. Skill ツールで`yomiyasu`を呼び、引数に`--domain tech`を渡す。
   その規範に従って書く。
2. 1行に1文を書き、1文は短くする。
   mado が1行80桁(全角は2桁)で検査するため、日本語は1行40字ほどが上限になる。
3. 日本語と英単語の間に半角空白を入れない。
   コードやコマンドは`バッククォート`で囲む。
4. 抽出台帳やテスト台帳の管理番号、TLA+の性質の名前を本文に書かない。
   要件は振る舞いの言葉で書く。

yomiyasu が手元に無いときは、<https://github.com/nanaism/yomiyasu>から入れる。

## 図を描く

アーキテクチャ全体の図(構成、データの流れ、状態遷移、コマンドの体系)は、oh-my-mermaidで作る。

1. Claude Codeのプラグイン`oh-my-mermaid`を入れておく(<https://github.com/oh-my-mermaid/oh-my-mermaid>)。
2. Skill ツールで`oh-my-mermaid:omm-scan`を呼び、`.omm/`に図と説明を書き出す。
   言語は`.omm/config.yaml`で日本語に設定してある。
3. `just omm view`で図を開いて確かめる。

`omm`コマンドは`just setup`で`.config/npm/`に入る。
`just`を経由せずに使うときは、`.config/npm/node_modules/.bin`をPATHに通す。

1つの文書の中だけで使う小さな図は、Markdownに`mermaid`のコードブロックで直接書く。
全体の構成に関わる図は`.omm/`に置き、文書からはその要素の名前で参照する。

## 書いたあと

1. `just docs-check`を実行し、終了コード0を確かめる。
   mado、typos、`omm validate`が走る。
2. yomiyasuの検査スクリプトを実行し、指摘を確かめる。
   `python3 <yomiyasuのディレクトリ>/scripts/yomiyasu_lint.py <ファイル>`で動く。
   箇条書きの比率の警告は、手順や規約を並べた文書なら残してよい。
3. コミットは`commit-message`の skill に従い、種類は`docs`にする。
