# 引き継ぎ: webqa-explorerのloop-engineering(内ループ完了、テスト設計へ)

発行: 2026-10-17、更新: 2026-10-18。有効条件: `tasks/loopeng/WebExploratoryQA.tla`が更新時点(TLC generated 158363 / distinct 19539)から変更されておらず、`WebExploratoryQA.feature`がシナリオ71本のままであること。
再開の起点はこの文書1つ。ほかの文書は参照先として読む。

凡例: `[ ]`未着手、`[~]`部分完了(残りを同じ行に書く)、`[x]`完了(証跡あり)。

## 1. いまどこにいるか

- リポジトリ: `/home/hakkadaikon/work/jev`(GitHub private `Hakkadaikon/webqa-explorer`)
- 作るもの: Webサービス向けの探索的テストエージェント(判断モデルJev + ブラウザ操作)。設計段階で、実装コードはまだ無い。
- 工程: hymmeのloop-engineering。0段(抽出)、外ループ(EARS/TLA+)、中ループ、内ループ(受け入れシナリオ)まで完了。次はtest-design(`hymme:test-catalog`)。
- 使っているhymme: 0.9.0。

## 2. 済んだこと

- [x] 0段: 抽出台帳S-001〜S-044。証跡: `WebExploratoryQA.extract.md`(reviewer 4回目で合格)
- [x] 外ループ: EARS構造化、状態モデル、TLA+ 初版。証跡: commit `4a936de`(reviewer 11回目で合格)
- [x] 中ループ1回目のmutation oracle(旧spec対象)。254 mutant、survivor 46。証跡: `WebExploratoryQA.mutants-v1.tsv`(全mutantの結果種別と状態数。生成スクリプトは`tools/classify_mutants.py`)
- [x] survivorの分類。コメント内9、到達状態が元と同じ(equivalent)27、到達状態が変わるのに緑(本物の穴の候補)10。
- [x] 旧specで見つかった穴5つを直した新specを書いた(未コミット)。その後の経緯は §4。TLCはNo error(Inv、`EventuallyTerminates`、`TerminalAbsorbing`、generated 108695 / distinct 14537)。証跡: `WebExploratoryQA.tlc-run.log`。残り: mutation oracleの再実行と、関連文書の追従(下の §4)。

### 旧specで見つかった穴と、新specでの直し方

| 穴 | 根拠になったmutant | 新specでの対応 |
|---|---|---|
| `requestBudget`が一度も減らず、S-042の上限ガードが空振り | `gt->ge#3`(L90) | `RequestDecision`で減らし、0になったら`ResetRequestWindow`で戻す。cfgは`MaxRequests=2 < MaxDecisions=4` |
| S-010の判定が操作種別から作った派生フラグに依存し、壊しても検知できない | `eq->neq#21`、`and->or#38`、`and->or#52` | 低確信度を`proposedLowConf`として操作種別と独立に持つ。Invは生の値で書く |
| `ExecuteOperation`を飛ばして観測へ進める | `eq->neq#34`、`#36` | 実行と観測を結果ごとの3アクション(`ExecuteNoChange` / `ExecuteChange` / `ExecuteOutOfDomain`)にまとめた |
| 終了状態から抜け出せても検知できない | `eq->neq#42`、`#67`、`#74`、`#78` | 行動性質`TerminalAbsorbing`を追加 |
| 自己申告の`done`でも`goal_achieved`になる(S-014とS-044が矛盾) | `eq->neq#75`(L243) | done_whenを満たすときだけ`goal_achieved`、満たさなければ`reported_done`。Inv `GoalOnlyByDoneWhen`を追加 |

## 3. 判断待ち(ユーザーに確認すること)

- [ ] 長いページでのscrollと行き詰まり: 操作可能要素の一覧は表示領域内だけなので、scrollで新しい要素が見えても探索済み(操作済み)は増えない。S-021の定義(探索済みが増えない)どおりだと、scrollが3回続くと行き詰まりで終わる(T-018が固定)。意図どおりかを確認する
- [x] 時間窓で数える単位は対象サーバへのアクセス(ブラウザ操作、課金画面の離脱、再読み込み、ドメイン復帰)。2026-10-18に承認。証跡: `extract.md`設計メモ9
- [x] `tasks/test-design/`をgitで追跡する(hymmeの既定は管理外だが、`tasks/loopeng/`と同じ扱いにする)。2026-10-18のClaude Codeセッションで、ユーザーが確認の問い(選択肢「追跡する」/「追跡しない」)に「追跡する」と答えた

- [x] S-044の要件文の変更(done_whenを満たさないdoneは`reported_done`)。2026-10-17に承認。証跡: `WebExploratoryQA.extract.md`設計メモ7
- [x] S-017とS-022の衝突は「探索の記憶を保持する」に決定。S-021(新規要素の定義)、S-042(窓はいつでも満了しうる)、"idle"での優先順位の明確化も承認。2026-10-18。証跡: 同設計メモ7

## 4. 次にやること(順番どおり)

- [x] 新specにmutation oracleを再実行(v2)。証跡: `WebExploratoryQA.mutants-v2.tsv`。ただし評価時エラーをkilledと数え、状態数の一致を等価の根拠にしたため、2回目のレビューで不合格になった
- [x] 2回目のレビューの指摘を受けてspecを強化した(hymme:modeler)。TLCはNo error(generated 158363 / distinct 19539)。証跡: `WebExploratoryQA.tlc-run.log`
- [x] mutation oracleの再実行と再分類(v3)。本物のsurvivor 0、abstraction-limited 1(Hrev)。証跡: `WebExploratoryQA.mutants-v3.tsv`、`WebExploratoryQA.faithfulness.md`の「mutation oracle」節
- [x] 関連文書の追従: `model.md`(変数・不変条件・性質・遷移の各表と優先順位の節)、`extract.md`(マトリクス、S-017/S-021/S-042/S-044の文面、設計メモ7・8、STRIDE/CWE)、`README.md`
- [x] 文面の往復確認をやり直した(fresh reader)。23行、full 14 / partial 9。前件到達は`reach.log`の「v3」「v3b」節
- [x] `hymme:loop-engineering-reviewer`で再レビュー。1回目(2026-10-17)と2回目(2026-10-18)は不合格、3回目(2026-10-18)で合格(`[本質]` 0件)。値ごとの再検査で見つかった穴8件も含めて性質を8本足した。残った`[瑣末]`2件(ログの取り直し、reach.logの時刻表記の注記)も対応済み。未保証の範囲は`WebExploratoryQA.faithfulness.md`のpartial 9行とabstraction-limitedの各行
- [x] コミットしてpushした。証跡: `3f23988` 以降のコミット(`git log 4a936de..`)
- [x] 内ループ(`hymme:loopeng-gherkin`)。TLCは反例を出していないので、受け入れシナリオを要件から書いた(71本、Rule 8つ)。未保証の範囲(partial 9行、abstraction-limited、設計メモ8の判定ロジック)はシナリオに持ち込み、受け持ちの対応表を`tasks/test-design/WebExploratoryQA.md`の「設計検証で保証できなかった範囲の受け持ち」に置いた。`hymme:loop-engineering-reviewer`は4回目で合格(`[本質]` 0件)。証跡: `WebExploratoryQA.feature`、`extract.md`のマトリクス「テスト」列と設計メモ9
- [x] 受け入れシナリオをテスト項目の台帳へ転記(`hymme:test-extract`)。T-001〜T-071、シナリオ1件=1行。`hymme:test-design-reviewer`は3回目で転記段階として合格。証跡: `tasks/test-design/WebExploratoryQA.md`の「機械確認」節
- [ ] テスト手法の割り当て(`hymme:test-catalog`)。台帳の行はすべて`[ ]`のままで、`test-design-extract-gate` hookがテストファイルの作成を止める状態(意図どおり)。その前に実装言語とテストランナーを決める
- [ ] 実装の計画(`superpowers:writing-plans`)と実装。受け入れ条件は手法名・管理番号なしでspecに書く

## 5. 罠(踏んだもの)

- **mutation oracleのkilledを信じない。状態数の一致も等価の証拠にしない。** `tla_mutate_oracle.py`は「TLCが緑でない」ものをすべてkilledと数える。旧specではkilled 208のうち、性質違反で捕まえたのは21だけだった(構文エラー102、評価時エラー84、タイムアウト1)。killedの内訳は`tools/classify_mutants.py`で確かめる。
- **TLCの環境変数。** 先に`export TMPDIR=${TMPDIR:-/tmp}; mkdir -p $TMPDIR/tla; export _JAVA_OPTIONS=-Djava.io.tmpdir=$TMPDIR/tla`を通す。実行後は`states/`と`*_TTrace_*`を消す。
- **TLA+ の書き方。** 行動性質は`[][(P) => (UNCHANGED vars)]_vars`のように括弧で囲む。囲まないとSANYが`]_`で構文エラーを出す。
- **push。** リポジトリのremoteはSSHだが、この環境にはSSH鍵が無い。`git push https://github.com/Hakkadaikon/<repo>.git main`でpushし、追跡情報は`git fetch https://github.com/Hakkadaikon/<repo>.git main:refs/remotes/origin/main`で更新する。
- **hymmeを更新したら。** `/plugin marketplace update hymme`を打ってから`/reload-plugins`を打つ。`/reload-plugins`だけではローカルのマーケットプレイスのコピーが古いままになる。

## 6. 触っていないもの

- `/home/hakkadaikon/work/jev/.claude/`が未追跡で存在する。中身は確認していない。
