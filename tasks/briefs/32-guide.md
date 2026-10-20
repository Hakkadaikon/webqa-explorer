# brief: issue #32 ペルソナと目的を書かせる手引き

共通の決まりは同じディレクトリの `common.md`。並列に回す手順は skill `parallel-work`。

**#18(ペルソナと目的の書式)と #31(`webqa check-docs`)が入っていることを前提にする。** 33a(インタビューのモードの受け入れの土台)と並列に回る。

## 目的

ペルソナの文書と目的の設定を AI に書かせるための手引きを、Agent Skills の標準の形式で配る。
開発用の `skills/` とは別の `share/skills/` に置く。

## 触ってよいファイル

- `share/skills/writing-persona-purpose/SKILL.md`(新規。手引きの本体)
- `share/skills/writing-persona-purpose/` の下の書式の例(新規。必要なら `examples/persona.yaml`、`examples/purpose.yaml`)
- `crates/webqa/tests/guide.rs`(新規。文書の層のテスト)
- `README.md`(手引きの置き場を1行足すだけ)
- `tasks/test-design/InterviewMode.md`(テスト名の欄を実際に合わせるときだけ)

触らないもの: `skills/`(開発用の手順書)、`crates/` の製品のコード。

## 決めてある設計

### 手引きの中身

- 前付け(YAML front matter)に `name` と `description` を書く。`description` には、いつ使うか(ペルソナと目的を書く・直すとき、インタビューのメモや分析のデータから起こすとき)を書く。
- 本文に次を入れる。
  1. インタビューのメモや分析のデータから、ペルソナの項目(`name`、`profile`、`it_literacy`、`wants`、`avoids`)を埋める手順。
  2. 目的の項目(`goal`、`done_when`)を埋める手順。
  3. 書式の例(ペルソナの文書1つ、目的の設定1つ)。値は架空のもので、実在の人や会社の名前を使わない。
  4. 書いた文書を `webqa check-docs persona <ファイル>` と `webqa check-docs purpose <ファイル>` で確かめる手順。誤りのときは、表示された項目の名前を直して、もう一度確かめる。
- 目的の `done_when` は、確定の操作ができる画面まで来たことで書く。確定の操作そのもの(注文を確定する、送信する)を実行したことで書かない、と例つきで示す。理由は、インタビューのモードが確定の操作を押さずに止まるため(ガードレールの禁止要素)。
- 書式は #18 の実装(`webqa_core::config::persona`、`webqa_core::config::purpose`)と、`crates/webqa/tests/fixtures/docs/` の文書に合わせる。

### 文書の層のテスト(`crates/webqa/tests/guide.rs`)

手引きのファイルを読み、文字列で確かめる。テスト名は台帳(`tasks/test-design/InterviewMode.md` の T-116〜T-118)に合わせる。

| テスト | 期待値 |
|---|---|
| `guide_has_frontmatter_steps_and_examples` | 1行目が `---` で始まる前付けがあり、`name:` と `description:` を持つ。本文に `check-docs persona` と `check-docs purpose` がある。書式の例の YAML を `webqa_core::config::persona` と `purpose` で読むと、どちらも誤りが無い |
| `guide_shows_done_when_at_confirm_screen` | 本文に、`done_when` を確定の操作ができる画面まで来たことで書く、という説明と、その例の `done_when` がある |
| `guide_lives_outside_dev_skills` | 手引きのパスが `share/skills/` の下にあり、`skills/` の下に同じ名前のファイルが無い |

書式の例は、本文に YAML のコードブロックで書くか、`examples/` のファイルに置いて本文から参照する。どちらにしても、テストは例を実際の読み込みの関数に通して、誤りが無いことを確かめる。例が書式から外れたら、テストが落ちる。

### 文書の検査

- `share/skills/` の Markdown は `just docs-check`(mado、typos)の対象になる。1行80桁の上限(MD013)と、SKILL.md が前付けで始まること(MD041 を外してある)に合わせる。
- 文章は skill `writing-docs` と yomiyasu の基準で書く。

## テスト

| テスト | 期待値 |
|---|---|
| `cargo test -p webqa --test guide` | 上の3本が通る |
| `just docs-check` | 通る |
| `just fmt-check`、`just clippy`、`just metrics` | 通る |

## 親に決めてほしいこと

1. **手引きの名前**。推す案: `writing-persona-purpose`。Agent Skills の慣習の動名詞の形で、開発用の `writing-docs` と並べても紛れない。
2. **書式の例の置き場**。推す案: 本文に YAML のコードブロックで書き、テストは本文からコードブロックを取り出して読み込む。手引きが1ファイルで完結し、配るときに迷わない。別案は `examples/` に置く形で、テストは単純になるが、ファイルが増える。
3. **README への載せ方**。推す案: 「使い方」の節に、手引きの置き場を1行足す。
