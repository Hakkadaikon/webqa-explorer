# テスト項目抽出台帳: commit-msg

対象: `scripts/commit_msg.py`(コミットメッセージが `docs/commit-messages.md` の規約に合うかを判定する開発ツール)
走査アンカー: 規約の各条項(ヘッダーの形、種類、範囲、要約、空行、本文の幅、検査の対象外)と、起動の2形態(ファイル1つ、コミットの範囲)
テスト対象: scripts/commit_msg_test.py

## 用語定義(抽出より先に埋める)

| 用語 | 定義(曖昧さを残さない) | この定義が生む境界と異常系 |
|------|--------------------------|----------------------------|
| ヘッダー | メッセージの1行目。`<種類>(<範囲>)!: <要約>` で、`(<範囲>)` と `!` は省ける | 形の崩れ、空のメッセージ |
| 種類 | feat、fix、docs、style、refactor、perf、test、build、ci、chore、revert のどれか | 一覧に無い種類 |
| 範囲 | 英小文字で始まり、英小文字・数字・ハイフンが続く文字列 | 大文字、記号、空の括弧 |
| 日本語を含む | ひらがな、カタカナ、漢字のいずれかを1字以上含む | 英語だけの要約 |
| 要約の長さ | 文字数(コードポイントの数)。50字以内 | 50字、51字 |
| 本文の幅 | 1行の表示幅。全角(East Asian Width が W か F)は2、ほかは1。72以内 | 72、73、全角の混在 |
| 検査しない行 | `#` で始まる行と、`# ------------------------ >8 ------------------------` 以降 | エディタで書いたときのコメント |
| git が作るメッセージ | `Merge `、`Revert "`、`fixup! `、`squash! `、`amend! ` で始まるもの | 検査しない |

## 台帳

- [x] T-001 ヘッダー [event] 「`種類(範囲): 日本語の要約` の形なら合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: HeaderTest.test_a_well_formed_header_passes
- [x] T-002 ヘッダー [event] 「範囲を省いても、`!` を付けても合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: HeaderTest.test_scope_and_bang_are_optional
- [x] T-003 種類 [unwanted] 「一覧に無い種類は不合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: HeaderTest.test_an_unknown_type_fails
- [x] T-004 ヘッダー [unwanted] 「コロンの後の空白が無いなど、形が崩れていれば不合格。空のメッセージも不合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: HeaderTest.test_a_malformed_header_fails
- [x] T-005 範囲 [unwanted] 「大文字や記号を含む範囲、空の括弧は不合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: HeaderTest.test_an_invalid_scope_fails
- [x] T-006 要約 [unwanted] 「日本語を含まない要約は不合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: SummaryTest.test_a_summary_without_japanese_fails
- [x] T-007 要約 [boundary] 「50字なら合格、51字なら不合格」
      → 手法: 境界値分析 (blackbox-partition/SKILL.md) / テスト名: SummaryTest.test_summary_length_boundary
- [x] T-008 要約 [unwanted] 「句点やピリオドで終わる要約は不合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: SummaryTest.test_a_summary_ending_with_a_period_fails
- [x] T-009 本文 [unwanted] 「2行目が空行でなければ不合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: BodyTest.test_the_second_line_must_be_blank
- [x] T-010 本文 [boundary] 「幅72なら合格、73なら不合格。全角は2と数える」
      → 手法: 境界値分析 (blackbox-partition/SKILL.md) / テスト名: BodyTest.test_body_width_boundary
- [x] T-011 本文 [boundary] 「URL を含む行は幅を問わない」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: BodyTest.test_a_line_with_a_url_is_not_measured
- [x] T-012 対象外 [event] 「git が作るメッセージは検査せずに合格」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: SkipTest.test_messages_made_by_git_pass
- [x] T-013 対象外 [event] 「`#` の行と切り取り線から下は無視する」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: SkipTest.test_comments_and_the_scissors_section_are_ignored
- [x] T-014 起動 [event] 「ファイルを1つ渡すと、合格なら 0、不合格なら理由を標準エラーに出して 1 を返す」
      → 手法: 分岐網羅 (whitebox-controlflow-basic/SKILL.md) / テスト名: CliTest.test_a_file_is_checked
- [x] T-015 起動 [event] 「`--range` でコミットの範囲を渡すと各コミットを判定し、落ちたコミットのハッシュと理由を出す」
      → 手法: 分岐網羅 (whitebox-controlflow-basic/SKILL.md) / テスト名: CliTest.test_a_range_of_commits_is_checked
- [x] T-016 起動 [event] 「スクリプトとして実行すると、main の戻り値を終了コードにする」
      → 手法: 分岐網羅 (whitebox-controlflow-basic/SKILL.md) / テスト名: CliTest.test_running_the_script_exits_with_the_result
- [x] T-017 ヘッダー [boundary] 「コメント行だけのメッセージは、空のメッセージとして不合格」
      → 手法: 境界値分析 (blackbox-partition/SKILL.md) / テスト名: EmptyTest.test_a_message_of_only_comments_fails_as_empty

## トレーサビリティマトリクス(消し込み台帳)

| 振る舞い(T番号) | 種別 | 手法(reference) | テスト名 | 設計済 | 緑確認済 | mutation(criticalのみ) |
|------------------|------|------------------|----------|--------|----------|------------------------|
| T-001 | event | 同値分割 | HeaderTest.test_a_well_formed_header_passes | [x] | [x] | — |
| T-002 | event | 同値分割 | HeaderTest.test_scope_and_bang_are_optional | [x] | [x] | — |
| T-003 | unwanted | 同値分割 | HeaderTest.test_an_unknown_type_fails | [x] | [x] | — |
| T-004 | unwanted | 同値分割 | HeaderTest.test_a_malformed_header_fails | [x] | [x] | — |
| T-005 | unwanted | 同値分割 | HeaderTest.test_an_invalid_scope_fails | [x] | [x] | — |
| T-006 | unwanted | 同値分割 | SummaryTest.test_a_summary_without_japanese_fails | [x] | [x] | — |
| T-007 | boundary | 境界値分析 | SummaryTest.test_summary_length_boundary | [x] | [x] | — |
| T-008 | unwanted | 同値分割 | SummaryTest.test_a_summary_ending_with_a_period_fails | [x] | [x] | — |
| T-009 | unwanted | 同値分割 | BodyTest.test_the_second_line_must_be_blank | [x] | [x] | — |
| T-010 | boundary | 境界値分析 | BodyTest.test_body_width_boundary | [x] | [x] | — |
| T-011 | boundary | 同値分割 | BodyTest.test_a_line_with_a_url_is_not_measured | [x] | [x] | — |
| T-012 | event | 同値分割 | SkipTest.test_messages_made_by_git_pass | [x] | [x] | — |
| T-013 | event | 同値分割 | SkipTest.test_comments_and_the_scissors_section_are_ignored | [x] | [x] | — |
| T-014 | event | 分岐網羅 | CliTest.test_a_file_is_checked | [x] | [x] | — |
| T-015 | event | 分岐網羅 | CliTest.test_a_range_of_commits_is_checked | [x] | [x] | — |
| T-016 | event | 分岐網羅 | CliTest.test_running_the_script_exits_with_the_result | [x] | [x] | — |
| T-017 | boundary | 境界値分析 | EmptyTest.test_a_message_of_only_comments_fails_as_empty | [x] | [x] | — |

mutation 列は全行「—」。開発用の検査ツールで、金額・セキュリティ・状態遷移に当たらない。

## 対応する unwanted 行

- T-001、T-002(ヘッダー): T-003〜T-005
- T-012、T-013(対象外): 対象外にしたものは検査しないので、異常系は無い。対象外に当たらないメッセージは T-001〜T-011 で判定する
- T-014〜T-016(起動): 不合格の場合を同じ行の中で確かめる。存在しないファイルや範囲は、Python と git のエラーでそのまま止まれば足りる

## 閉じる前のチェック(設計用: 0段を閉じる条件)

- [x] 用語定義表が埋まり、曖昧な語が残っていない
- [x] 全 T-ID に種別ラベルが振られている
- [x] 各 `event`/`state` 行に対応する `unwanted` 行が最低1つある(無いものは上の節に理由)
- [x] 全 T-ID の「設計済」が `[x]`
- [x] T-001 から連番に欠番なし
- [x] 各 T-ID に手法が割り当たっている
- [x] 正常系だけでなく、境界、異常系が各機能に最低1つある
