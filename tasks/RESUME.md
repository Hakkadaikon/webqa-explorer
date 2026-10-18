# 再開の起点(2026-10-18 20:10 時点)

## 済み
- main: #1 土台、#2 実物での確認、#3 設定、#5 伏せ字。README、品質ゲート文書、implementing skill、インタビューのモードの設計(docs/superpowers/specs/)、その要件と受け入れシナリオ(9555066)。
- hymme 0.9.4 に書き戻し済み(push 済み)。

## 途中(未コミット。サブエージェントは止まる。worktree と作業ツリーの変更はディスクに残るので、git status と各 worktree の git log で拾い直す)
1. #6 画面のモデル: worktree `.claude/worktrees/agent-acf4075de288d1007`、コミット 769e6c3。直し(cfg_attr(test, Debug)、ScreenText を非公開+as_str、`"` の逃がし)を頼んだところ。→ 直しを確かめ、reviewer に通して main へ merge、`gh issue close 6`。
2. #11 イベントログと TUI: worktree `.claude/worktrees/agent-a665913fab5c2ad6a`。brief は scratchpad の briefs/11-event-tui.md(消えていたら architecture.md の「イベントログとTUI」から作り直す)。→ 完了を確かめ、reviewer、merge。
3. 既存の仕様(tasks/loopeng/WebExploratoryQA.*): 要素0件の画面で問い合わせが起きない穴を modeler が直している(対象を取らない操作は proposedTarget="none"、empty.cfg を足して TLC・変異・到達をやり直し)。docs/architecture.md の v5 への追従(状態の表、対応表、探索のゴール)は親が直して未コミット。→ 終わったら loop-engineering-reviewer と test-design-reviewer を合格まで回し、残り: 探索モードで done だけの操作種別を拒む行(feature と台帳 T-112、設定の検証の直し)、台帳の瑣末な指摘。合格したら一括コミットし、#13 の「仕様待ち」ラベルを外す。
4. インタビューのモード: 抽出台帳と feature は「目的のずれは判断モデルに聞く」を反映済みで未コミット(128本)。テスト項目の台帳 tasks/test-design/InterviewMode.md をそれに追従させているところ。→ test-design-reviewer を合格まで回し、台帳の「構成への追加の案」を architecture.md に反映、issue を切って台帳の issue 列を埋め、コミット。

## 次の順番
- #6 が入ったら #15(縦の一本。今日の目標だった)を最優先。並行で #4、#10、#8、#7(どれも #6 の型を使う)。
- #14 に .env の読み込み(dotenvy)を入れる。キーは利用者が .env に登録済み(.gitignore 済み)。
- JEV の API の仕様を利用者にもらう(橋渡しのスクリプトに要る)。

## 決まりごと
- brief の共通の決まり: scratchpad の briefs/common.md(秘密の値の扱い、値ごとの比較、空の入力、rebase しない)。
- 実装担当は当面 Opus(#3、#5 の比較で Opus が速く質も同等以上)。Sonnet と Opus の総合評価は実装が一通り済んでから出す約束。
- worktree を立てる前に df で空きを見る。終わった worktree は cargo clean。
