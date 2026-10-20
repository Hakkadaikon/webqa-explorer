# 状態/ドメインモデル: WebExploratoryQA

loopeng-formalize工程の成果物。抽出台帳(`WebExploratoryQA.extract.md`, S-001〜S-044)の各EARS要件を状態変数・遷移へ対応づける。

**注記(1回目レビュー指摘反映)**: 初版はmodel.mdと`.tla`実装が乖離していた(model.mdに書いた設計の一部が`.tla`へ実装されていない)。本版はmodel.mdを`.tla`の実装内容に完全一致させることを優先し、実装しなかった設計要素は「モデル外」として扱いを明示する側に倒した。

## モデル化の対象範囲

TLA+でモデル検査する価値があるのは「並行・状態遷移・順序」が絡む部分。本システムの核心は**1ステップ実行ループ(判断問い合わせ→ガードレール適用→操作実行→画面観測→終了判定)の繰り返しと、終了条件への到達性、およびガードレールが実際に危険な操作を遮断すること**。

### モデル化で捨てた領域(抽象化の死角の明示)

以下はTLC検査の対象外とし、実装レベルの単体テスト・プロパティテストで別途検証する(トレーサビリティ・マトリクスの「テスト」列で後続のtest-design工程が担う)。

- **S-003, S-009**: 判断モデルへ送信するstateのテキスト表現の内部フォーマット(JSON構造・フィールド名)。状態機械上は「stateを送った」という事実のみを扱い、内容のエンコーディングは外側。
- **S-025, S-026**: 計測・コスト算出は状態遷移に影響を与えない観測用の副作用。状態変数化せず、モデル外のロギングとして扱う。
- **S-004, S-015, S-032, S-038**: YAML設定ファイルの構文・認証情報注入の具体的な実装(ファイルパース、環境変数展開)。状態機械は「設定済みのstage列・認証状態」を所与として受け取る。段階の切り替え(S-015)は状態機械に入れたが、各段階のdone_whenの中身は観測の真偽値に抽象化した。
- **S-040**: アクセシビリティ情報取得の実装API選択(Playwright `ariaSnapshot()`等)。状態機械は「アクセシビリティツリーを取得した」という抽象化された事実のみ扱う。
- **S-039**: 判断モデルのバックエンド差し替え可能性はインタフェース抽象化の要件であり、状態遷移には影響しない(どのバックエンドでも同じ状態機械が成立することが要件の本質)。
- **S-016の一部, S-023(探索優先順位の具体的な選択規則)**: 「未操作要素→未探索分岐が残る既操作要素→戻る/スクロール」という優先順位の中身は、状態機械上は`RequestDecision`が`\E op \in Operations, t \in Elements : ...`で非決定的に選ぶことに抽象化し、実際にどの順序で選ぶかというヒューリスティックはモデル化しない(安全性・終了性には影響しないため)。優先順位ロジック自体の正しさは実装レベルのテストで検証する。
- **S-018, S-019, S-020, S-035, S-036(問題記録の種類別分類)**: `problemsLog`のような専用の状態変数は持たず、「問題が検出されたら記録する」という事実は状態遷移(特にガードレールでの拒否、ExecuteOutOfDomain等)の発生そのものに暗黙に対応づける。問題の種類別分類・命名規則・保存形式は実装レベルで検証する。
- **S-017(固まり検出後の復旧アクションの詳細=ページの再起動)**: `RecoverFromFreeze`は固まりカウンタを戻すことでページ状態の初期化を表す。探索の記憶(`visited`)は保持する(S-022)。ブラウザの実際のリロード処理はモデル外。

### `ForbiddenElements`(S-012, S-028)の判定ロジック: 最も危険な抽象化の実装指針

**この節が必要な理由**: `.tla`の`NeverOperatesForbidden`不変条件(`visited ∩ ForbiddenElements = {}`)は「`ForbiddenElements`に分類された要素を操作しない」という**ガードの効き方**しか検査していない。「ある要素が実際に`ForbiddenElements`に分類されるべきか」という**分類ロジック自体の正しさ**は、TLA+の有限集合というモデルの外側に完全に置かれている。TLCが"No error"を返しても、実装の分類ロジックが漏れていれば(例: 「購入する」ボタンを誤って安全要素と判定する)、この不変条件は何も保証しない。モデル検査をしたという事実と、実際に危険な操作を防げることとの間には、このロジック次第で大きな差が生じる。抽象化のレベルで安全性を保証したと言うことはできず、ここは実装レベルの検証を必須とする。

**実装時に`ForbiddenElements`への分類に使うべき判定軸(複数のANDではなくOR、1つでも該当すれば禁止要素とする。安全側に倒す)**:

1. **アクセシブルネーム/テキストのキーワードマッチ**: 「購入」「削除」「退会」「ログイン」「サインイン」「登録」「送信」「決済」「お支払い」等の日本語キーワード、および"buy", "purchase", "delete", "remove", "unsubscribe", "sign in", "log in", "submit", "pay", "checkout"等の英語キーワードを部分一致で判定する。多言語対応のテスト対象では言語ごとのキーワードリストを設定ファイルで追加できるようにする(S-004のYAML設定の一部として)。
2. **ARIA role + 文脈**: `role="button"`または`<button type="submit">`かつ、所属するフォームの`action`属性や周辺のURLパターンに決済/認証/退会関連の語が含まれる場合。
3. **ナビゲーション先のURL/フォームのaction属性パターン**: 遷移先・送信先のURLパスに`/checkout`, `/payment`, `/delete`, `/unsubscribe`, `/signin`, `/login`等が含まれる場合。クリック前には遷移先が確定しないことが多いため、この判定は「クリック後に遷移先が確定してから中止する」というS-028(直前存在確認と同様の事後ガード)と組み合わせる必要がある。
4. **明示的な許可/拒否リスト(設定での上書き)**: 自動判定は過検知・過少検知どちらも起こるため、YAML設定でCSSセレクタ/XPathベースの明示的な禁止リスト・許可リストを利用者が追加できるようにする。自動判定と明示リストが矛盾する場合は、禁止側(安全側)を優先する。

**この判定ロジック自体の検証方法(TLCでは検証できないため、別の手段が必須)**:

- **ゴールデンテスト**: 実在する典型的なECサイト・SaaSのサインアップ/決済ページのHTML断片を固定サンプルとして保存し、各ボタンが正しく禁止要素に分類されるかをスナップショットテストする。
- **ファジング/敵対的サンプル**: 「購入はこちら」のように表記ゆれのあるボタン文言、画像のみでテキストが無いボタン(altテキストに依存)、JavaScriptで動的に生成されるボタン等、判定ロジックの穴を突く入力を生成し、過少検知(本来禁止すべき要素を見逃す)が起きないかを確認する。過検知(安全な要素を誤って禁止する)は機能性の問題だが、過少検知は安全性の問題であり優先度が異なる。
- **差分テスト**: 既知の危険要素データセット(社内の実際のテスト対象アプリから集めた「押してはいけないボタン」一覧)に対して、判定ロジックの新旧バージョンで分類結果が後退していないかを比較する。

この節の存在自体が、「`ForbiddenElements`という抽象化はTLA+レベルでは安全性の“形”しか保証せず、実質的な安全性は実装レベルの判定ロジックとその検証に懸かっている」という限界を明示する。test-design工程(loopeng-gherkin以降)でこの判定ロジックのテストケースを設計する際は、この節を出発点にする。

## 状態変数(VARIABLES、`.tla`と一致)

| 変数名 | 型(値域) | 意味 |
|--------|----------|------|
| `mode` | `{"scenario", "explore"}` | 実行モード(S-016, S-027)。初期状態で非決定的に選び、実行中は変わらない |
| `stage` | `1..NumStages` | 決め打ちシナリオの現在の段階(S-015)。探索モードでは1のまま |
| `phase` | `{"idle", "awaiting_decision", "acting", "done", "stuck", "frozen_recovered", "error_exit", "timed_out", "aborted"}` | 1ステップ実行ループの進行フェーズ |
| `proposedOp` | `Operations ∪ {"back", "none"}` | 判断モデルが返した操作種別(S-005)。paywallのときはガードが`"back"`に書き換える。探索モードでは`"done"`を選択肢に入れない(S-016) |
| `proposedTarget` | `Elements ∪ {"none"}` | 判断モデルが返した対象要素(S-006)。対象を取らない操作(scroll_down、scroll_up、back、enter、wait、done、stuck)では`"none"`。click/inputで要素が1つも無いときも`"none"`で、存在しない要素番号が返った応答の異常として扱う(`targetExists = FALSE`) |
| `proposedLowConf` | `BOOLEAN` | 判断モデルの確信度が0.7未満か(S-010)。操作種別とは独立に観測する |
| `isPaywall` | `BOOLEAN` | 現在画面がpaywall相当か(S-013、非決定的に観測) |
| `targetExists` | `BOOLEAN` | 実行直前に対象要素が存在するか(S-028) |
| `screenChanged` | `BOOLEAN` | 直前の操作後に画面が変化したかの観測結果(S-011, S-021固まり)。ページが応答しない(S-017)場合も「変化なし」(`FALSE`)の1ケースとして同じ変数で表す |
| `newSeen` | `BOOLEAN` | 直前の操作後に未操作の要素が新しく見えたかの観測結果(S-021行き詰まり)。スクロール等で画面が変化したときだけ非決定的に`TRUE`になりうる。許可ドメイン外のページでは`FALSE` |
| `doneWhenMet` | `BOOLEAN` | 決め打ちシナリオで、現在の段階のdone_whenを満たしたかの観測結果(S-014, S-015)。実行の開始時、段階を進めた直後、操作の実行後に非決定的に観測する。探索モードでは常に`FALSE`。3か所とも、とりうる値は補助定義`DoneWhenObservations`(決め打ちシナリオは`BOOLEAN`、探索モードは`{FALSE}`)で共通にした |
| `visited` | `SUBSET Elements` | 探索済み要素集合(S-022, S-023)。探索の記憶で、固まり復旧でも段階の切り替えでも消さない |
| `idleCounter` | `Nat` | 画面が変化しなかった操作の連続回数(S-021固まり) |
| `stuckCounter` | `Nat` | 新規要素が出なかった(`visited`が増えず、未操作の要素も新しく見えなかった)操作の連続回数(S-021行き詰まり) |
| `inAllowedDomain` | `BOOLEAN` | システムが記録した「現在ページは許可ドメイン内か」(S-033) |
| `pageOutside` | `BOOLEAN` | 環境側の真値。現在ページが実際に許可ドメイン外か。記録の正しさを検査するためだけに持つ |
| `timeUp` | `BOOLEAN` | 制限時間に達したか(S-016, S-041)。時刻の経過を環境の事象`TimeElapses`に抽象化し、一度`TRUE`になったら戻らない |
| `decisionBudget` | `0..MaxDecisions` | 判断モデルAPI呼び出しの残り回数(S-041)。段階をまたいで共有する |
| `decisionsMade` | `Nat` | 実際に呼び出した回数(S-041の検査用の履歴) |
| `requestBudget` | `0..MaxRequests` | 現在の時間窓で残っているリクエスト数(S-042)。段階をまたいで共有する |
| `requestsInWindow` | `Nat` | 現在の時間窓で実際に送ったリクエスト数(S-042の検査用の履歴) |
| `exitReason` | `{"none", "goal_achieved", "reported_done", "stuck", "budget_exceeded", "time_exceeded", "user_aborted"}` | 終了理由(S-014, S-044, S-021, S-041, S-016)。`reported_done`はdone_whenを満たさない自己申告doneによる未検証の終了(決め打ちシナリオだけ)。固まり・ドメイン外・段階の切り替えは実行を終わらせない中間状態なので終了理由に含めない |

`pageOutside`、`decisionsMade`、`requestsInWindow`は、システムの記録や予算の変数が正しく動いているかを外から確かめるための変数で、実装に対応する変数はない。`newSeen`、`doneWhenMet`、`timeUp`は環境の観測で、実装では画面の取得結果、done_whenの判定結果、時計に当たる。

**簡略化の明示**: 画面ごとの既操作要素集合というグラフ構造は、単一画面を仮定した`visited: SUBSET Elements`に簡略化した。画面をまたぐ遷移構造(S-022「画面A→画面Bのグラフ」)の再現はモデル外とする。決め打ちシナリオの各段階のdone_whenは、中身(YAMLに書く条件)をモデル外に置き、「満たしたかを観測する」真偽値に抽象化した。観測するのは、実行の開始時、段階を進めた直後、操作の後の3か所(S-015の3行目、設計メモ11の1)。

## 定数(CONSTANTS、`.tla`と一致)

- `Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "enter", "wait", "done", "stuck"}` (S-005)
- `ForbiddenElements \subseteq Elements` (S-012, S-028)
- `Elements`: テスト対象ごとに具体化する有限集合
- `MaxDecisions`, `MaxRequests`, `IdleThreshold`, `StuckThreshold`: 各閾値(S-041, S-042, S-021)。どれも正の整数(`ASSUME`)。0だと時間窓リセットや固まり復旧が空回りし、終了性が成り立たない。実装では設定値の検証で0を拒否する
- `NumStages`: 決め打ちシナリオの段階の数(S-015)。正の整数。検査では2

検査の設定は3つ。主の設定(`.cfg`)は`MaxDecisions = 4`、`MaxRequests = 2`、`IdleThreshold = 2`、`StuckThreshold = 2`、`NumStages = 2`、`Elements = {e1, e2, e3}`、`ForbiddenElements = {e3}`、`Operations = {"click", "back", "done", "stuck"}`。スクロールは`Operations`に入れていないが、要素を対象にしない操作の結果の扱い(新しい要素が見えたかの観測を含む)は`back`と同じ分岐を通るので、`back`で代表させる。もう1つの`WebExploratoryQA.noexplorable.cfg`は`Elements = {e1}`、`ForbiddenElements = {e1}`で、ほかは同じ。探索してよい要素が空の場合(S-016、設計メモ11)を検査するために置く。3つ目の`WebExploratoryQA.empty.cfg`は`Elements = {}`、`ForbiddenElements = {}`で、ほかは同じ。`Elements`はページの中身なので、操作可能な要素が1つも無い画面もありうる。その画面でも問い合わせ、対象を取らない操作を実行し、対象の要る操作は応答の異常として却下することを検査するために置く。

## 初期状態(Init)

- `mode \in {"scenario", "explore"}`、`stage = 1`
- `phase = "idle"`
- `proposedOp = "none"`, `proposedTarget = "none"`, `proposedLowConf = FALSE`
- `isPaywall = FALSE`, `targetExists = TRUE`, `screenChanged = TRUE`, `newSeen = FALSE`
- `doneWhenMet`: 決め打ちシナリオでは`BOOLEAN`のどちらか(開始時の最初の段階のdone_whenの観測)、探索モードでは`FALSE`
- `visited = {}`, `idleCounter = 0`, `stuckCounter = 0`
- `inAllowedDomain = TRUE`, `pageOutside = FALSE`, `timeUp = FALSE`
- `decisionBudget = MaxDecisions`, `decisionsMade = 0`, `requestBudget = MaxRequests`, `requestsInWindow = 0`
- `exitReason = "none"`

## "idle"での優先順位

"idle"では、成り立つ条件のうち最も優先度の高いアクションだけが有効になる(S-044のstate行)。補助定義`DoneWhen`(探索モードは`visited = Explorable`かつ1回以上問い合わせ済み、決め打ちシナリオは`doneWhenMet`)、`LastStage`(探索モードか最後の段階)、`Exploring`(DoneWhen未成立)と`Continuing`(Exploringかつ行き詰まり未検出かつ判断予算ありかつ制限時間内)をガードに使う。

1. done_when成立 → 最後の段階なら`FinishNormally`(ゴール達成)、そうでなければ`AdvanceStage`(段階の切り替え)
2. 行き詰まり検出 → `FinishStuck`
3. 判断予算切れ → `ExceedDecisionBudget`、時間切れ → `FinishTimeUp`。同じ段で、両方が成り立てばどちらが起きてもよい
4. 固まり復旧(`RecoverFromFreeze`)、ドメイン復帰(`ReturnToAllowedDomain`)、時間窓の満了(`ResetRequestWindow`)。この3つは同じ段で、どれが先に起きてもよい
5. 判断の問い合わせ(`RequestDecision`)。固まり検出中と許可ドメイン外では起きない

利用者による中断(`UserAbort`)と時刻の経過(`TimeElapses`)は環境の事象で、この順序の外にある。どちらも終了前ならどのフェーズでも起こりうる。公平性は課さない(起きなくてもよい)。優先順位の行動性質は、次の一歩としてこの2つも許す(`EnvEvent`)。

## 不変条件(INVARIANT、`.tla`と一致)

性質は、システム側の補助定義(`DoneWhen`、`LastStage`、`ProposableOps`)を使わず、要件の言葉で書き直した定義(`ExploreDone`、`ScenarioStageDone`、`GoalReached`、`StageCleared`)か生の変数で書く。システム側の定義を壊したときに、性質も一緒に変わって見逃すのを防ぐため。

| 不変条件名 | 対応EARS | 内容 |
|----------|----------|------|
| `TypeOK` | 全変数 | 各変数が値域内 |
| `NeverOperatesForbidden` | S-012, S-028 | `visited ∩ ForbiddenElements = {}` |
| `DirectOpActedOnlyOnSafeTarget` | S-012, S-028 | `(phase = "acting" ∧ ¬isPaywall ∧ proposedOp ∈ DirectOps) ⇒ (targetExists ∧ proposedTarget ∈ Explorable)` |
| `LowConfidenceBackNeverActed` | S-010 | `(phase = "acting" ∧ ¬isPaywall ∧ proposedOp = "back") ⇒ ¬proposedLowConf`。paywallの強制backはS-013によりS-010の対象外 |
| `ExploreGoalOnlyWhenAllExplored` | S-016, S-044 | 探索モードでゴール達成なら`visited = Explorable`かつ`decisionsMade > 0`(探索してよい要素が無くても1回は問い合わせてから) |
| `ScenarioGoalOnlyAtLastStage` | S-014, S-015, S-044 | 決め打ちシナリオでゴール達成なら、最後の段階で`doneWhenMet` |
| `ReportedDoneOnlyWithoutDoneWhen` | S-044, S-014 | `reported_done`ならdone_when(どちらのモードの定義でも)が未成立 |
| `ExploreNeverReportedDone` | S-016, S-044 | 探索モードは`reported_done`で終わらない |
| `ExitReasonMatchesPhase` | S-044, S-041, S-016 | done ⇒ goal_achievedかreported_done、stuck ⇒ stuck、error_exit ⇒ budget_exceeded、timed_out ⇒ time_exceeded、aborted ⇒ user_aborted、終了前 ⇒ none |
| `TimeExceededOnlyWhenTimeUp` | S-016, S-041 | `exitReason = "time_exceeded" ⇒ timeUp` |
| `StagesOnlyInScenario` | S-015, S-016 | 探索モードでは`stage = 1`かつ`¬doneWhenMet` |
| `NewSeenOnlyOnChangedAllowedPage` | S-021, S-033 | `newSeen ⇒ (screenChanged ∧ inAllowedDomain ∧ decisionsMade > 0)` |
| `NoChangeObservedOnlyAfterOps` | S-011, S-021 | `¬screenChanged ⇒ decisionsMade > 0` |
| `NoProposalBeforeDecision` | S-005, S-006 | 一度も問い合わせていない間は、提案と提案時の観測が初期値のまま |
| `RequestsWithinWindowCap` | S-042 | `requestsInWindow ≤ MaxRequests` |
| `DecisionsWithinCap` | S-041 | `decisionsMade ≤ MaxDecisions` |
| `BudgetAccounting` | S-041, S-042 | 残り予算 + 実際の呼び出し数 = 上限(判断予算、窓のリクエスト予算の両方) |
| `BudgetExceededOnlyWhenSpent` | S-041 | `exitReason = "budget_exceeded" ⇒ decisionBudget = 0` |
| `CountersBoundedByOps` | S-021 | 固まり・行き詰まりカウンタは判断の回数を超えない |
| `RecoveryResetsPageState` | S-017 | `phase = "frozen_recovered" ⇒ (idleCounter = 0 ∧ inAllowedDomain)` |
| `DomainRecordAccurate` | S-021, S-033 | `inAllowedDomain = ¬pageOutside` |
| `VisitedOnlyByOps` | S-002, S-022 | `Cardinality(visited) ≤ decisionsMade` |
| `RecoveryOnlyAfterFreeze` | S-017 | `phase = "frozen_recovered" ⇒ decisionsMade ≥ IdleThreshold` |
| `ProposalWellFormed` | S-005, S-006, S-016, S-028 | 問い合わせ後の提案は操作種別9種のいずれか(paywall時はback)で、対象は要素か`"none"`。問い合わせの直後、対象を取らない操作の対象は`"none"`。click/inputの対象が`"none"`なら対象は存在しない扱いで、要素がある画面ではclick/inputの対象は要素。探索モードではdoneでない |

## 行動性質・時相性質(PROPERTY、`.tla`と一致)

| 性質名 | 対応EARS | 内容 |
|---|---|---|
| `GoalPreempts` | S-014, S-015, S-016, S-044 | ゴール達成の条件(`GoalReached`)が成り立つ"idle"の次の一歩は、ゴール達成での終了か環境の事象 |
| `StageAdvancePreempts` | S-015, S-044 | 最後でない段階のdone_when(`StageCleared`)が成り立つ"idle"の次の一歩は、"idle"のまま段階を1つ進めるか環境の事象 |
| `StuckPreempts` | S-021, S-044 | done_when未成立で行き詰まりを検出した"idle"の次の一歩は、行き詰まり終了か環境の事象 |
| `LimitStops` | S-041, S-016, S-044 | done_when未成立・行き詰まり未検出で、判断予算0か制限時間切れの"idle"の次の一歩は、尽きた方の理由での終了か環境の事象 |
| `FreezePreemptsDecision` | S-017 | 固まりを検出している間は問い合わせない |
| `SafeProposalProceeds` | S-002, S-010, S-012, S-028 | ガードに掛からない提案は却下されず実行段階へ進む(環境の事象を除く) |
| `PaywallForcesBack` | S-013 | paywallを検出した`"awaiting_decision"`の次の一歩は、`"acting"`かつ操作が`"back"`(環境の事象を除く) |
| `AwaitingOnlyByDecision` | S-005, S-041 | `"awaiting_decision"`に入る一歩は、判断の問い合わせの回数がちょうど1増える一歩と一致する |
| `ActingOnlyAfterGuardrails` | S-010, S-012, S-013, S-028 | `"acting"`には`"awaiting_decision"`からしか入れない |
| `FreezeRecoveryRecorded` | S-017 | done_when未成立で固まりを検出した`"idle"`から固まりカウンタを0に戻す一歩は、必ず`"frozen_recovered"`へ進む |
| `VisitedChangesOnlyByExecution` | S-002, S-022 | `visited`が変わるのは、実行段階から`"idle"`へ戻る一歩だけ |
| `ProposalOutcome` | S-002, S-044 | 実行段階の次の一歩は、doneならdone終了、stuckなら行き詰まり終了、それ以外は実行して"idle"へ(環境の事象を除く) |
| `IdleCounterTracksScreen` | S-021 | 実行の一歩で、画面が変化しなければ固まりカウンタは+1、変化すれば0 |
| `StuckCounterTracksNewElements` | S-021 | 実行の一歩で、探索済み集合が増えず未操作の要素も新しく見えなければ行き詰まりカウンタは+1、どちらかなら0 |
| `MemoryNeverShrinks` | S-022, S-017 | 探索済み集合は減らない |
| `NoDecisionOutsideDomain` | S-033 | 実際に許可ドメイン外にいる間は問い合わせない |
| `StageAdvanceResetsCounters` | S-015 | 段階が変わる一歩は、"idle"で`StageCleared`のときに1つ進める一歩だけで、固まりと行き詰まりの回数を0に戻す(次の段階のdone_whenは入った時点で観測し直す) |
| `BudgetSharedAcrossStages` | S-015, S-041, S-042 | 段階が変わる一歩では、判断予算・呼び出し回数・時間窓の予算と履歴を変えない |
| `DecisionBudgetNeverRestored` | S-041 | 判断予算は増えない |
| `TimeNeverRewinds` | S-016, S-041 | 一度尽きた制限時間は戻らない |
| `TimeElapsesAlone` | S-016, S-041 | 時刻の経過の一歩は、ほかの変数を変えない |
| `ModeFixed` | S-016, S-027 | 実行モードは変わらない |
| `ProposalChangesOnlyByDecision` | S-005, S-006, S-013 | 提案と提案時の観測が変わるのは、問い合わせの一歩と、paywallによるbackへの置き換え(操作種別だけ変わる)だけ |
| `CountersChangeOnlyBySteps` | S-021, S-015, S-017 | 固まりカウンタが変わるのは実行・固まり復旧・段階の切り替えだけ。行き詰まりカウンタが変わるのは実行と段階の切り替えだけ(復旧は新しい要素を出す操作ではない) |
| `DomainChangesOnlyByNavigation` | S-033 | 許可ドメイン内外の記録と真値が変わるのは、実行と許可ドメインへの復帰だけ |
| `ObservationChangesOnlyByExecution` | S-011, S-021 | `screenChanged`と`newSeen`が変わるのは実行の一歩だけ |
| `DoneWhenChangesOnlyByExecutionOrAdvance` | S-014, S-015 | `doneWhenMet`が成り立つのは実行の一歩か段階を進めた一歩だけで、取り消されるのは段階の切り替えだけ |
| `TerminationKeepsState` | S-044, S-041, S-016 | 終了する一歩(中断を含む)は、フェーズと終了理由以外を変えない |
| `StartsFresh` | S-015, S-016, S-041, S-042 | 実行は段階1、探索済みなし、カウンタ0、許可ドメイン内、予算満タン、制限時間内、終了理由なしで始まる(最初の段階のdone_whenは開始時に観測するので満たしていてもよい。探索モードでは観測しない)(初期状態だけで検査される状態述語) |
| `EventuallyTerminates` | S-044, S-041 | `<>(phase ∈ Terminal)`。各システムアクションの弱公平性(実行の3結果は和に対して1つ)の下で、いつか終了する。中断と時刻の経過には公平性を課さないので、判断予算だけで終了性が成り立つ。操作可能な要素が1つも無くても成り立つ(`WebExploratoryQA.empty.cfg`で検査)。前提は`ProposableOps`が空でないこと(実装は設定値の検査で、空の操作種別と、探索モードでdoneだけの操作種別を拒否する) |
| `TerminalAbsorbing` | S-044, S-041, S-016 | 一度終了したら(中断を含む)状態は変わらない |

## 遷移(Next、`.tla`と一致)

| Next disjunct名 | 対応EARS | 遷移概要 |
|---|---|---|
| `RequestDecision` | S-005, S-006, S-007(簡略化), S-008(簡略化), S-016, S-017, S-033, S-041, S-042 | `Continuing`かつ固まり未検出かつリクエスト予算ありかつ許可ドメイン内 → `"awaiting_decision"`。判断予算とリクエスト予算を1ずつ減らし、履歴を1ずつ増やす。提案と画面の観測値は非決定的に受け取る。対象は`ProposalTargets(op)`から選ぶ(click/inputで要素があれば要素、それ以外は`"none"`)。探索モードではdoneを選択肢に入れない |
| `ResetRequestWindow` | S-042 | `Continuing`かつ予算を1つ以上使っている → 予算を`MaxRequests`へ戻し、窓の履歴を0にする |
| `ApplyGuardrails` | S-010, S-012, S-013, S-028 | `"awaiting_decision"`から。paywallなら既定のbackへ強制して`"acting"`へ。paywallでなければ低確信度back・禁止要素・存在しない要素のいずれかで`"idle"`へ却下し、どれにも当たらなければ`"acting"`へ |
| `ExecuteNoChange` | S-002, S-011, S-014, S-015, S-021, S-022, S-023 | `"acting"`(done/stuck以外)→`"idle"`。操作を実行し(click/inputなら対象を`visited`へ加える)、画面無変化を観測(新しい要素も見えない)。決め打ちシナリオではdone_whenを満たしたかを非決定的に観測する。固まりカウンタ+1、行き詰まりカウンタは新規要素の有無で更新 |
| `ExecuteChange` | S-002, S-014, S-015, S-021, S-022, S-023 | 同上。画面変化を観測し、固まりカウンタを0にする。未操作の要素が新しく見えたかを非決定的に観測する |
| `ExecuteOutOfDomain` | S-002, S-014, S-015, S-021(ドメイン外), S-033 | 同上。許可ドメイン外への遷移を観測して記録する。ドメイン外の要素は新しく見えた要素に数えない |
| `ReturnToAllowedDomain` | S-033 | `Continuing`かつドメイン外 → 許可ドメイン内へ戻る |
| `FinishByProposalDone` | S-044(reported_done), S-014 | `"acting"`かつdone提案 → `phase="done"`、`exitReason="reported_done"`。探索モードではdoneを提案させないので、決め打ちシナリオでだけ起きる |
| `FinishByProposalStuck` | S-044(stuck) | `"acting"`かつstuck提案 → `phase="stuck"`、`exitReason="stuck"` |
| `RecoverFromFreeze` | S-017, S-021(固まり) | `Continuing`かつ固まりカウンタが閾値以上 → `"frozen_recovered"`。固まりカウンタだけを戻し、探索の記憶と行き詰まりカウンタは保持する |
| `ResumeAfterRecovery` | S-017 | `"frozen_recovered"` → `"idle"` |
| `AdvanceStage` | S-015 | `"idle"`かつdone_when成立かつ最後の段階でない → 段階を1つ進め、固まりと行き詰まりの回数を0に戻し、次の段階のdone_whenを観測し直す(非決定的)。すでに満たしていれば、次の一歩で操作せずにさらに進むかゴール達成になる。判断予算・時間窓・探索の記憶は戻さない |
| `FinishNormally` | S-014, S-016, S-044(goal) | `"idle"`かつdone_when成立かつ最後の段階(探索モードを含む。探索モードのdone_whenは1回以上問い合わせた後だけ成り立つ) → `phase="done"`、`exitReason="goal_achieved"` |
| `FinishStuck` | S-021(行き詰まり), S-044(stuck) | `Exploring`かつ行き詰まりカウンタが閾値以上 → `phase="stuck"`、`exitReason="stuck"` |
| `ExceedDecisionBudget` | S-041 | `Exploring`かつ行き詰まり未検出かつ判断予算0 → `phase="error_exit"`、`exitReason="budget_exceeded"` |
| `FinishTimeUp` | S-016, S-041 | `Exploring`かつ行き詰まり未検出かつ制限時間切れ → `phase="timed_out"`、`exitReason="time_exceeded"` |
| `TimeElapses` | S-016, S-041 | 環境の事象。終了前ならいつでも、制限時間切れ(`timeUp`)にする。公平性なし |
| `UserAbort` | S-044(中断) | 環境の事象。終了前ならいつでも、`phase="aborted"`、`exitReason="user_aborted"`へ。ほかの変数は変えない。記憶・終了のイベント・実行結果の書き出しはモデル外。公平性なし |

**簡略化の明示(S-022)**: 何も起きなかった input(入力欄でない対象、値の無い入力欄)は、モデルでは対象の無い実行(wait 相当)として扱う。記憶に足さず、行き詰まりの数は進む。探索モードで、候補に入力できる要素が無いときに操作の種類から input を外すことも、`ProposableOps`には写さない。提案の操作と対象は非決定に選ぶので、input を外した選択肢はその部分集合として含まれる。

**簡略化の明示(S-005, S-012, S-016)**: `enter`(フォーカスのある要素でEnterを押す。2026-10-20に追加)は、モデルでは対象の無い操作(`wait`や`scroll_down`と同じ)として扱う。`DirectOps`には足さず、記憶にも足さない。実装は、表示中に値のある入力欄が無い探索モードの画面と、ページに禁止要素がある画面で、操作の種類から`enter`を外す。この絞り込みは選択肢を組む前の処理で、モデルの外とする。提案の操作は非決定に選ぶので、`enter`を外した選択肢はその部分集合として含まれる。検査する cfg の`Operations`には`"enter"`を入れていない(`ASSUME`の集合に足しただけで、状態空間は変わらない)。

**簡略化の明示(S-007, S-008)**: `is_error_shown`(S-007)・`goal_achieved`の自己申告(S-008)は、判断モデルへの問い合わせ自体は`RequestDecision`で行う前提だが、その返り値を状態変数として個別に持たず、S-019(エラー検出の閾値判定)・S-029(自己申告を信用しない方針)は実装レベルの検証に委ねる。ゴール達成の判定はS-014(done_when)とS-016(探索モードのゴール条件)だけで行う。判断モデルの自己申告doneは決め打ちシナリオで終了の契機にはなる(S-044)が、done_whenを満たさなければ`reported_done`として記録し、ゴール達成とは扱わない。

## EARSで状態機械化しない要件(ubiquitous/optional feature、モデル外)

| S-ID | 扱い |
|------|------|
| S-001 | アクセシビリティツリー取得能力。前提条件として所与 |
| S-003, S-004, S-009 | 送信データの形式・設定ファイル定義。モデル化の対象外(上記「捨てた領域」) |
| S-007, S-008 | is_error_shownの値・goal_achievedの自己申告。状態変数化せず、S-007はS-019/S-029の実装レベル検証、S-008の自己申告はゴール達成の判定に使わない(`GoalOnlyByDoneWhen`、遷移本文の注記参照) |
| S-030, S-031, S-032, S-038, S-039, S-040 | アーキテクチャ・配布・差し替え可能性の要件。状態遷移に現れない。S-015(段階の定義)とS-027(2モード)は、段階の切り替えと`mode`を状態機械に入れ、定義の書き方だけをモデル外に置く |
| S-016, S-023(優先順位の中身) | 非決定的選択に抽象化(上記「捨てた領域」) |
| S-018, S-019, S-020, S-035, S-036 | 問題記録の種類別分類。状態遷移の発生そのものに暗黙対応、分類・保存形式は実装レベル |
| S-024 | `visited`は「操作済み」の集合で、S-024の「探索済み」(遷移先まで探索し終えた)とは別の概念(extract.md設計メモ9)。単一画面への簡略化により、遷移先の探索状況(探索し終えたかどうか)はモデル外。実装のテストで確かめる |
| S-025, S-026, S-029 | 計測・記録の性質に関する要件。副作用として注記するのみ |
| S-034, S-037, S-043 | 出力形式・マスキング・TLS設定の要件。状態機械の外側(実装レベルの制約) |

## トレーサビリティ・マトリクスへの反映方針

台帳(`WebExploratoryQA.extract.md`)の「形式手法」列へ、上表の対応に従い次を記入する:
- 状態機械化した要件: 対応する`.tla`内のNext disjunct名、INVARIANT名またはPROPERTY名(本モデル文書と完全一致させる)
- モデル化で捨てた領域: 「モデル外(実装レベルで検証)」である旨
