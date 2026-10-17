# 状態/ドメインモデル: WebExploratoryQA

loopeng-formalize工程の成果物。抽出台帳(`WebExploratoryQA.extract.md`, S-001〜S-044)の各EARS要件を状態変数・遷移へ対応づける。

**注記(1回目レビュー指摘反映)**: 初版はmodel.mdと`.tla`実装が乖離していた(model.mdに書いた設計の一部が`.tla`へ実装されていない)。本版はmodel.mdを`.tla`の実装内容に完全一致させることを優先し、実装しなかった設計要素は「モデル外」として扱いを明示する側に倒した。

## モデル化の対象範囲

TLA+でモデル検査する価値があるのは「並行・状態遷移・順序」が絡む部分。本システムの核心は**1ステップ実行ループ(判断問い合わせ→ガードレール適用→操作実行→画面観測→終了判定)の繰り返しと、終了条件への到達性、およびガードレールが実際に危険な操作を遮断すること**。

### モデル化で捨てた領域(抽象化の死角の明示)

以下はTLC検査の対象外とし、実装レベルの単体テスト・プロパティテストで別途検証する(トレーサビリティ・マトリクスの「テスト」列で後続のtest-design工程が担う)。

- **S-003, S-009**: 判断モデルへ送信するstateのテキスト表現の内部フォーマット(JSON構造・フィールド名)。状態機械上は「stateを送った」という事実のみを扱い、内容のエンコーディングは外側。
- **S-025, S-026**: 計測・コスト算出は状態遷移に影響を与えない観測用の副作用。状態変数化せず、モデル外のロギングとして扱う。
- **S-004, S-015, S-032, S-038**: YAML設定ファイルの構文・認証情報注入の具体的な実装(ファイルパース、環境変数展開)。状態機械は「設定済みのstage列・認証状態」を所与として受け取る。
- **S-040**: アクセシビリティ情報取得の実装API選択(Playwright `ariaSnapshot()`等)。状態機械は「アクセシビリティツリーを取得した」という抽象化された事実のみ扱う。
- **S-039**: 判断モデルのバックエンド差し替え可能性はインタフェース抽象化の要件であり、状態遷移には影響しない(どのバックエンドでも同じ状態機械が成立することが要件の本質)。
- **S-016, S-023(探索優先順位の具体的な選択規則)**: 「未操作要素→未探索分岐が残る既操作要素→戻る/スクロール」という優先順位の中身は、状態機械上は`RequestDecision`が`\E op \in Operations, t \in Elements : ...`で非決定的に選ぶことに抽象化し、実際にどの順序で選ぶかというヒューリスティックはモデル化しない(安全性・終了性には影響しないため)。優先順位ロジック自体の正しさは実装レベルのテストで検証する。
- **S-018, S-019, S-020, S-035, S-036(問題記録の種類別分類)**: `problemsLog`のような専用の状態変数は持たず、「問題が検出されたら記録する」という事実は状態遷移(特にガードレールでの拒否、ObserveOutOfDomain等)の発生そのものに暗黙に対応づける。問題の種類別分類・命名規則・保存形式は実装レベルで検証する。
- **S-017(固まり検出後の復旧アクションの詳細=ページの再起動)**: `RecoverFromFreeze`で`visited`をリセットすることにより「探索状態の初期化」は表現するが、ブラウザの実際のリロード処理自体はモデル外。

## 状態変数(VARIABLES、`.tla`と一致)

| 変数名 | 型(値域) | 意味 |
|--------|----------|------|
| `phase` | `{"idle", "awaiting_decision", "acting", "done", "stuck", "frozen_recovered", "error_exit"}` | 1ステップ実行ループの進行フェーズ |
| `proposedOp` | `Operations ∪ {"none"}` | 判断モデルが返した操作種別(ガード適用前、S-005) |
| `proposedTarget` | `Elements ∪ {"none"}` | 判断モデルが返した対象要素(ガード適用前、S-006) |
| `proposedLowConfBack` | `BOOLEAN` | 提案がbackでかつ確信度0.7未満か(S-010) |
| `isPaywall` | `BOOLEAN` | 現在画面がpaywall相当か(S-013、非決定的に観測) |
| `targetExists` | `BOOLEAN` | 実行直前に対象要素がアクセシビリティツリーに存在するか(S-028) |
| `screenChanged` | `BOOLEAN` | 直前操作後に画面が変化したか(S-011判定に使用) |
| `visited` | `SUBSET Elements` | 探索済み要素集合(S-022, S-023, S-024。画面ごとのグラフ構造は簡略化し単一の集合で近似) |
| `idleCounter` | `Nat` | 画面変化がない連続回数(S-021固まり判定用) |
| `stuckCounter` | `Nat` | 新規要素が出ない操作の連続回数(S-021行き詰まり判定用) |
| `inAllowedDomain` | `BOOLEAN` | 現在ページが許可ドメイン内か(S-033, S-021ドメイン外判定) |
| `decisionBudget` | `Nat` | 判断モデルAPI呼び出しの残り回数(S-041) |
| `requestBudget` | `Nat` | 探索モードの単位時間あたり残りリクエスト数(S-042) |
| `exitReason` | `{"none", "goal_achieved", "stuck", "budget_exceeded"}` | 終了理由(S-044, S-021, S-041, S-042)。初版では`"frozen"`/`"out_of_domain"`/`"error"`も終了理由に含めていたが、これらは実行を終了させず探索を継続させる復旧可能な中間状態(固まりから復旧=`RecoverFromFreeze`、ドメイン外から復帰=`ReturnToAllowedDomain`)であり、`phase`で表現すれば十分なため終了理由からは除いた(1回目レビュー指摘により修正) |

**簡略化の明示**: `visitedGraph`(画面ごとの既操作要素集合)という初版のグラフ構造は、単一画面を仮定した`visited: SUBSET Elements`に簡略化した。複数画面をまたぐグラフ探索(S-022「画面A→画面Bのグラフ」)の構造自体は、本spec内では状態機械の安全性・終了性の検証に必要な範囲(「全要素が探索済みになれば終了する」)に絞り、画面をまたぐ遷移構造の忠実な再現はモデル外とする。

## 定数(CONSTANTS、`.tla`と一致)

- `Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"}` (S-005)
- `ForbiddenElements \subseteq Elements` (S-012, S-028: 禁止要素の部分集合)
- `Elements`: テスト対象ごとに具体化する有限集合
- `MaxDecisions`, `MaxRequests`, `IdleThreshold`, `StuckThreshold`: 各閾値(S-041, S-042, S-021)

## 初期状態(Init)

- `phase = "idle"`
- `proposedOp = "none"`, `proposedTarget = "none"`, `proposedLowConfBack = FALSE`
- `isPaywall = FALSE`, `targetExists = TRUE`
- `screenChanged = TRUE` (初回は変化とみなす)
- `visited = {}`
- `idleCounter = 0`, `stuckCounter = 0`
- `inAllowedDomain = TRUE`
- `decisionBudget = MaxDecisions`, `requestBudget = MaxRequests`
- `exitReason = "none"`

## 不変条件(Inv / ubiquitous要件、`.tla`と一致)

| 不変条件名 | 対応EARS | 内容 |
|----------|----------|------|
| `TypeOK` | 全変数 | 各変数が値域内 |
| `NeverOperatesForbidden` | S-012, S-028 | `visited ∩ ForbiddenElements = {}`。`RequestDecision`は禁止要素も含めて非決定的に提案するが、`ApplyGuardrails`が拒否するため`visited`には反映されないことを検査する |
| `LowConfidenceBackNeverActed` | S-010 | `(proposedOp = "back" ∧ proposedLowConfBack) ⇒ phase ≠ "acting"`。低確信度backが提案された状態では実行フェーズに進まないことを検査する |
| `BudgetRespected` | S-041 | `decisionBudget ≥ 0` |
| `RequestBudgetRespected` | S-042 | `requestBudget ≥ 0` |
| `ExitReasonSetOnTermination` | S-044 | `phase ∈ {"done","stuck","error_exit"} ⇒ exitReason ≠ "none"` |

## 遷移(Next / event・state・unwanted要件、`.tla`と一致)

1 EARS節 = 1 disjunctの原則を守るため、1回目レビューで指摘された「RequestAndActへの過剰統合」を解消し、判断問い合わせ・ガード適用・操作実行を分離した。

| Next disjunct名 | 対応EARS | 遷移概要 |
|---|---|---|
| `RequestDecision` | S-005, S-006, S-007(簡略化: is_error_shownはモデル外), S-008(簡略化: goal_achievedの自己申告自体はモデル化せずS-014のdone_when判定のみ採用) | `phase="idle"` → `"awaiting_decision"`。判断モデルへ問い合わせ、operation/target/低確信度back/paywall/存在確認の提案を非決定的に受け取る |
| `ApplyGuardrails` | S-010, S-012, S-013, S-028 | `"awaiting_decision"` → `"acting"`(許可)または`"idle"`(拒否)。低確信度back・禁止要素・paywall・存在確認の順にガードし、paywallなら既定の離脱操作(back)へ強制する |
| `ExecuteOperation` | S-002, S-022, S-023 | `"acting"`のまま、`proposedTarget`を`visited`へ加える(許可された操作の実行) |
| `ObserveNoChange` | S-011 | `"acting"` → `"idle"`(`proposedOp \notin {"done","stuck"}`限定)。画面無変化を観測し`idleCounter`/`stuckCounter`を加算 |
| `ObserveChange` | S-002(正常系) | `"acting"` → `"idle"`(同上限定)。画面変化を観測しカウンタをリセット |
| `ObserveOutOfDomain` | S-021(ドメイン外), S-033 | `"acting"` → `"idle"`(同上限定)、`inAllowedDomain' = FALSE`。操作の帰結としてドメイン外遷移を観測する(独立トグルにすると往復だけで有限予算を消費せずliveness(`EventuallyTerminates`)の反例になったため、有限予算を消費する操作の帰結として統合した) |
| `ReturnToAllowedDomain` | S-033 | ドメイン外から許可ドメイン内へ戻り、探索を継続する |
| `RecoverFromFreeze` | S-017, S-021(固まり) | `idleCounter`が閾値到達 → `visited`をリセットして探索継続(ページ再起動の抽象化) |
| `ResumeAfterRecovery` | S-017 | 再起動直後から通常の`"idle"`フェーズへ復帰 |
| `FinishNormally` | S-014 | `visited = Elements \ ForbiddenElements`(done_when相当の構造的到達条件の抽象化) → `phase="done"`, `exitReason="goal_achieved"` |
| `FinishStuck` | S-021(行き詰まり) | `stuckCounter`が閾値到達(構造的な行き詰まり検出) → `phase="stuck"`, `exitReason="stuck"` |
| `FinishByProposalDone` | S-044(done) | `phase="acting"`かつ`proposedOp="done"`(判断モデルの自己申告による終了) → `phase="done"`, `exitReason="goal_achieved"`。done_whenによる構造的終了(`FinishNormally`)とは独立したEARS要件であり、2回目のformalizeレビューでこの経路自体が未実装(`proposedOp`の値を一度も参照しない)だったことを指摘され追加した。TLCの否定不変条件による反証実験で、`visited`が全要素探索済みでない状態からもこの経路で`phase="done"`に到達できることを確認済み(死んだコードでないことの検証) |
| `FinishByProposalStuck` | S-044(stuck) | `phase="acting"`かつ`proposedOp="stuck"` → `phase="stuck"`, `exitReason="stuck"`。`FinishStuck`(stuckCounter閾値による構造的終了)とは独立したEARS要件 |
| `ExceedDecisionBudget` | S-041 | `decisionBudget=0` → `phase="error_exit"`, `exitReason="budget_exceeded"` |

**簡略化の明示(S-007, S-008)**: `is_error_shown`(S-007)・`goal_achieved`の自己申告(S-008)は、判断モデルへの問い合わせ自体は`RequestDecision`で行う前提だが、その返り値を状態変数として個別に持たず、S-019(エラー検出の閾値判定)・S-029(自己申告を信用しない方針)は実装レベルの検証に委ねる。S-014(done_whenによる判定)のみを状態機械の終了条件として採用し、S-008の自己申告は状態機械に影響を与えない値として扱う(これはS-014の要件「自己申告ではなくdone_whenで判定する」の意図と一致する: 自己申告を状態遷移に反映させないこと自体がこの要件の実装である)。

## EARSで状態機械化しない要件(ubiquitous/optional feature、モデル外)

| S-ID | 扱い |
|------|------|
| S-001 | アクセシビリティツリー取得能力。前提条件として所与 |
| S-003, S-004, S-009 | 送信データの形式・設定ファイル定義。モデル化の対象外(上記「捨てた領域」) |
| S-007, S-008 | is_error_shownの値・goal_achievedの自己申告。状態変数化せず、S-007はS-019/S-029の実装レベル検証、S-008は「自己申告を状態機械に反映させない」こと自体がS-014の実装(遷移本文の注記参照) |
| S-015, S-027, S-030, S-031, S-032, S-038, S-039, S-040 | アーキテクチャ・配布・差し替え可能性の要件。状態遷移に現れない |
| S-016, S-023(優先順位の中身) | 非決定的選択に抽象化(上記「捨てた領域」) |
| S-018, S-019, S-020, S-035, S-036 | 問題記録の種類別分類。状態遷移の発生そのものに暗黙対応、分類・保存形式は実装レベル |
| S-024 | `visited`への追加条件(全遷移先探索済みになるまで未探索扱い)は、単一画面への簡略化により`visited`集合への追加=探索済み化と等価になっている。画面をまたぐ遷移先の探索状況追跡はモデル外 |
| S-025, S-026, S-029 | 計測・記録の性質に関する要件。副作用として注記するのみ |
| S-034, S-037, S-043 | 出力形式・マスキング・TLS設定の要件。状態機械の外側(実装レベルの制約) |

## トレーサビリティ・マトリクスへの反映方針

台帳(`WebExploratoryQA.extract.md`)の「形式手法」列へ、上表の対応に従い次を記入する:
- 状態機械化した要件: 対応する`.tla`内のNext disjunct名またはInv名(本モデル文書と完全一致させる)
- モデル化で捨てた領域: 「モデル外(実装レベルで検証)」である旨
