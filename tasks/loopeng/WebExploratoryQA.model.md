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
| `phase` | `{"idle", "awaiting_decision", "acting", "done", "stuck", "frozen_recovered", "error_exit"}` | 1ステップ実行ループの進行フェーズ |
| `proposedOp` | `Operations ∪ {"back", "none"}` | 判断モデルが返した操作種別(S-005)。paywallのときはガードが`"back"`に書き換える |
| `proposedTarget` | `Elements ∪ {"none"}` | 判断モデルが返した対象要素(S-006) |
| `proposedLowConf` | `BOOLEAN` | 判断モデルの確信度が0.7未満か(S-010)。操作種別とは独立に観測する |
| `isPaywall` | `BOOLEAN` | 現在画面がpaywall相当か(S-013、非決定的に観測) |
| `targetExists` | `BOOLEAN` | 実行直前に対象要素が存在するか(S-028) |
| `screenChanged` | `BOOLEAN` | 直前の操作後に画面が変化したかの観測結果(S-011, S-021固まり)。`IdleCounterTracksScreen`が読む |
| `visited` | `SUBSET Elements` | 探索済み要素集合(S-022, S-023)。探索の記憶で、固まり復旧でも消さない |
| `idleCounter` | `Nat` | 画面が変化しなかった操作の連続回数(S-021固まり) |
| `stuckCounter` | `Nat` | 新規要素が出なかった(`visited`が増えなかった)操作の連続回数(S-021行き詰まり) |
| `inAllowedDomain` | `BOOLEAN` | システムが記録した「現在ページは許可ドメイン内か」(S-033) |
| `pageOutside` | `BOOLEAN` | 環境側の真値。現在ページが実際に許可ドメイン外か。記録の正しさを検査するためだけに持つ |
| `decisionBudget` | `0..MaxDecisions` | 判断モデルAPI呼び出しの残り回数(S-041) |
| `decisionsMade` | `Nat` | 実際に呼び出した回数(S-041の検査用の履歴) |
| `requestBudget` | `0..MaxRequests` | 現在の時間窓で残っているリクエスト数(S-042) |
| `requestsInWindow` | `Nat` | 現在の時間窓で実際に送ったリクエスト数(S-042の検査用の履歴) |
| `exitReason` | `{"none", "goal_achieved", "reported_done", "stuck", "budget_exceeded"}` | 終了理由(S-014, S-044, S-021, S-041)。`reported_done`はdone_whenを満たさない自己申告doneによる未検証の終了。固まり・ドメイン外は実行を終わらせない中間状態なので終了理由に含めない |

`pageOutside`、`decisionsMade`、`requestsInWindow`は、システムの記録や予算の変数が正しく動いているかを外から確かめるための変数で、実装に対応する変数はない。

**簡略化の明示**: 画面ごとの既操作要素集合というグラフ構造は、単一画面を仮定した`visited: SUBSET Elements`に簡略化した。画面をまたぐ遷移構造(S-022「画面A→画面Bのグラフ」)の再現はモデル外とする。

## 定数(CONSTANTS、`.tla`と一致)

- `Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"}` (S-005)
- `ForbiddenElements \subseteq Elements` (S-012, S-028)
- `Elements`: テスト対象ごとに具体化する有限集合
- `MaxDecisions`, `MaxRequests`, `IdleThreshold`, `StuckThreshold`: 各閾値(S-041, S-042, S-021)。どれも正の整数(`ASSUME`)。0だと時間窓リセットや固まり復旧が空回りし、終了性が成り立たない。実装では設定値の検証で0を拒否する

## 初期状態(Init)

- `phase = "idle"`
- `proposedOp = "none"`, `proposedTarget = "none"`, `proposedLowConf = FALSE`
- `isPaywall = FALSE`, `targetExists = TRUE`, `screenChanged = TRUE`
- `visited = {}`, `idleCounter = 0`, `stuckCounter = 0`
- `inAllowedDomain = TRUE`, `pageOutside = FALSE`
- `decisionBudget = MaxDecisions`, `decisionsMade = 0`, `requestBudget = MaxRequests`, `requestsInWindow = 0`
- `exitReason = "none"`

## "idle"での優先順位

"idle"では、成り立つ条件のうち最も優先度の高いアクションだけが有効になる(S-044のstate行)。補助定義`Exploring`(done_when未成立)と`Continuing`(Exploringかつ行き詰まり未検出かつ判断予算あり)をガードに使う。

1. done_when成立 → `FinishNormally`
2. 行き詰まり検出 → `FinishStuck`
3. 判断予算切れ → `ExceedDecisionBudget`
4. 固まり復旧(`RecoverFromFreeze`)、ドメイン復帰(`ReturnToAllowedDomain`)、時間窓の満了(`ResetRequestWindow`)。この3つは同じ段で、どれが先に起きてもよい
5. 判断の問い合わせ(`RequestDecision`)。固まり検出中と許可ドメイン外では起きない

## 不変条件(INVARIANT、`.tla`と一致)

| 不変条件名 | 対応EARS | 内容 |
|----------|----------|------|
| `TypeOK` | 全変数 | 各変数が値域内 |
| `NeverOperatesForbidden` | S-012, S-028 | `visited ∩ ForbiddenElements = {}` |
| `DirectOpActedOnlyOnSafeTarget` | S-012, S-028 | `(phase = "acting" ∧ ¬isPaywall ∧ proposedOp ∈ DirectOps) ⇒ (targetExists ∧ proposedTarget ∉ ForbiddenElements)`。実行段階に進んだ直接操作の対象は、存在していて禁止要素でない |
| `LowConfidenceBackNeverActed` | S-010 | `(phase = "acting" ∧ ¬isPaywall ∧ proposedOp = "back") ⇒ ¬proposedLowConf`。paywallの強制backはS-013によりS-010の対象外 |
| `GoalOnlyByDoneWhen` | S-014, S-044 | `exitReason = "goal_achieved" ⇒ visited = Explorable` |
| `ReportedDoneOnlyWithoutDoneWhen` | S-044 | `exitReason = "reported_done" ⇒ visited ≠ Explorable` |
| `ExitReasonMatchesPhase` | S-044, S-041 | done ⇒ goal_achievedかreported_done、stuck ⇒ stuck、error_exit ⇒ budget_exceeded、終了前 ⇒ none |
| `RequestsWithinWindowCap` | S-042 | `requestsInWindow ≤ MaxRequests` |
| `DecisionsWithinCap` | S-041 | `decisionsMade ≤ MaxDecisions` |
| `BudgetAccounting` | S-041, S-042 | 残り予算 + 実際の呼び出し数 = 上限(判断予算、窓のリクエスト予算の両方) |
| `BudgetExceededOnlyWhenSpent` | S-041 | `exitReason = "budget_exceeded" ⇒ decisionBudget = 0` |
| `CountersBoundedByOps` | S-021 | 固まり・行き詰まりカウンタは判断の回数を超えない |
| `RecoveryResetsPageState` | S-017 | `phase = "frozen_recovered" ⇒ (idleCounter = 0 ∧ inAllowedDomain)` |
| `DomainRecordAccurate` | S-021, S-033 | `inAllowedDomain = ¬pageOutside`。記録が実際のページ位置と一致する |
| `VisitedOnlyByOps` | S-002, S-022 | `Cardinality(visited) ≤ decisionsMade`。探索済みの要素は、判断の問い合わせの回数を超えて増えない(1回の実行の中での記憶。実行をまたぐ永続化はモデル外) |
| `RecoveryOnlyAfterFreeze` | S-017 | `phase = "frozen_recovered" ⇒ decisionsMade ≥ IdleThreshold`。累計の判断回数が閾値に届く前に復旧が起きないことだけを言う。直前に固まりを検出したかは見ない(それは`RecoverFromFreeze`のガードと`FreezeRecoveryRecorded`) |
| `ProposalWellFormed` | S-005, S-006 | 問い合わせ後の提案は操作種別8種のいずれか(paywall時はback)と、実在する対象要素 |

## 行動性質・時相性質(PROPERTY、`.tla`と一致)

| 性質名 | 対応EARS | 内容 |
|---|---|---|
| `AwaitingOnlyByDecision` | S-005, S-041 | `"awaiting_decision"`に入る一歩は、判断の問い合わせの回数がちょうど1増える一歩と一致する |
| `ActingOnlyAfterGuardrails` | S-010, S-012, S-013, S-028 | `"acting"`には`"awaiting_decision"`からしか入れない。ガードの判定が正しいかまでは見ない(それは`DirectOpActedOnlyOnSafeTarget`、`LowConfidenceBackNeverActed`、`SafeProposalProceeds`が見る) |
| `FreezeRecoveryRecorded` | S-017 | 固まりを検出した`"idle"`から固まりカウンタを0に戻す一歩は、必ず`"frozen_recovered"`へ進む。固まりを検出したら必ず復旧することまでは言わない |
| `VisitedChangesOnlyByExecution` | S-002, S-022 | `visited`が変わるのは、実行段階から`"idle"`へ戻る一歩(操作の実行)だけ |
| `GoalPreempts` | S-014, S-044 | done_whenが成立した"idle"の次の一歩は、必ずゴール達成での終了 |
| `StuckPreempts` | S-021, S-044 | done_when未成立で行き詰まりを検出した"idle"の次の一歩は、必ず行き詰まり終了 |
| `BudgetStops` | S-041 | 上位の終了条件がなく判断予算が0の"idle"の次の一歩は、必ず強制終了 |
| `FreezePreemptsDecision` | S-017 | 固まりを検出している間は問い合わせない |
| `PaywallForcesBack` | S-013 | paywallを検出した`"awaiting_decision"`の次の一歩は、必ず`"acting"`かつ操作が`"back"` |
| `SafeProposalProceeds` | S-002, S-010, S-012, S-028 | ガードに掛からない提案は却下されず実行段階へ進む(ガードの過剰な拒否を検出する) |
| `ProposalOutcome` | S-002, S-044 | 実行段階の次の一歩は、doneならdone終了、stuckなら行き詰まり終了、それ以外は実行して"idle"へ |
| `IdleCounterTracksScreen` | S-021 | 実行の一歩で、画面が変化しなければ固まりカウンタは+1、変化すれば0 |
| `MemoryNeverShrinks` | S-022, S-017 | 探索済み集合は減らない |
| `StuckCounterTracksNewElements` | S-021 | 実行の一歩で、探索済み集合が増えなければ行き詰まりカウンタは+1、増えれば0 |
| `NoDecisionOutsideDomain` | S-033 | 実際に許可ドメイン外にいる間は問い合わせない |
| `EventuallyTerminates` | S-044, S-041 | `<>(phase ∈ Terminal)`。各アクションの弱公平性(実行の3結果は和に対して1つ)の下で、いつか終了する |
| `TerminalAbsorbing` | S-044, S-041 | 一度終了したら状態は変わらない |

## 遷移(Next、`.tla`と一致)

| Next disjunct名 | 対応EARS | 遷移概要 |
|---|---|---|
| `RequestDecision` | S-005, S-006, S-007(簡略化), S-008(簡略化), S-017, S-033, S-041, S-042 | `Continuing`かつ固まり未検出かつリクエスト予算ありかつ許可ドメイン内 → `"awaiting_decision"`。判断予算とリクエスト予算を1ずつ減らし、履歴を1ずつ増やす。提案と画面の観測値は非決定的に受け取る |
| `ResetRequestWindow` | S-042 | `Continuing`かつ予算を1つ以上使っている → 予算を`MaxRequests`へ戻し、窓の履歴を0にする。窓の満了は時刻依存で、いつでも起こりうる |
| `ApplyGuardrails` | S-010, S-012, S-013, S-028 | `"awaiting_decision"`から。まずpaywallを見て、paywallなら判断モデルの提案を使わず既定のbackへ強制して`"acting"`へ。paywallでなければ低確信度back・禁止要素・存在しない要素のいずれかで`"idle"`へ却下し、どれにも当たらなければ`"acting"`へ |
| `ExecuteNoChange` | S-002, S-011, S-021, S-022, S-023 | `"acting"`(done/stuck以外)→`"idle"`。操作を実行し(click/inputなら対象を`visited`へ加える)、画面無変化を観測。固まりカウンタ+1、行き詰まりカウンタは新規要素の有無で更新 |
| `ExecuteChange` | S-002, S-021, S-022, S-023 | 同上。画面変化を観測し、固まりカウンタを0にする |
| `ExecuteOutOfDomain` | S-002, S-021(ドメイン外), S-033 | 同上。許可ドメイン外への遷移を観測して記録する(`inAllowedDomain`をFALSE、`pageOutside`をTRUE) |
| `ReturnToAllowedDomain` | S-033 | `Continuing`かつドメイン外 → 許可ドメイン内へ戻る |
| `FinishByProposalDone` | S-044(reported_done), S-014 | `"acting"`かつdone提案 → `phase="done"`、`exitReason="reported_done"`。done_when成立時は問い合わせ前に`FinishNormally`が先に起きるので、ここに来るのはdone_when未成立のときだけ |
| `FinishByProposalStuck` | S-044(stuck) | `"acting"`かつstuck提案 → `phase="stuck"`、`exitReason="stuck"` |
| `RecoverFromFreeze` | S-017, S-021(固まり) | `Continuing`かつ固まりカウンタが閾値以上 → `"frozen_recovered"`。ページ状態(固まりカウンタ)だけを戻し、探索の記憶(`visited`)は保持する。行き詰まりカウンタも戻さないが、これは要件の対象外の選択で、性質では検査していない |
| `ResumeAfterRecovery` | S-017 | `"frozen_recovered"` → `"idle"` |
| `FinishNormally` | S-014, S-044(goal) | `"idle"`かつ`visited = Explorable` → `phase="done"`、`exitReason="goal_achieved"` |
| `FinishStuck` | S-021(行き詰まり), S-044(stuck) | `Exploring`かつ行き詰まりカウンタが閾値以上 → `phase="stuck"`、`exitReason="stuck"` |
| `ExceedDecisionBudget` | S-041 | `Exploring`かつ行き詰まり未検出かつ判断予算0 → `phase="error_exit"`、`exitReason="budget_exceeded"` |

**簡略化の明示(S-007, S-008)**: `is_error_shown`(S-007)・`goal_achieved`の自己申告(S-008)は、判断モデルへの問い合わせ自体は`RequestDecision`で行う前提だが、その返り値を状態変数として個別に持たず、S-019(エラー検出の閾値判定)・S-029(自己申告を信用しない方針)は実装レベルの検証に委ねる。ゴール達成の判定はS-014(done_when)だけで行う。判断モデルの自己申告doneは終了の契機にはなる(S-044)が、done_whenを満たさなければ`reported_done`として記録し、ゴール達成とは扱わない(`GoalOnlyByDoneWhen`)。

## EARSで状態機械化しない要件(ubiquitous/optional feature、モデル外)

| S-ID | 扱い |
|------|------|
| S-001 | アクセシビリティツリー取得能力。前提条件として所与 |
| S-003, S-004, S-009 | 送信データの形式・設定ファイル定義。モデル化の対象外(上記「捨てた領域」) |
| S-007, S-008 | is_error_shownの値・goal_achievedの自己申告。状態変数化せず、S-007はS-019/S-029の実装レベル検証、S-008の自己申告はゴール達成の判定に使わない(`GoalOnlyByDoneWhen`、遷移本文の注記参照) |
| S-015, S-027, S-030, S-031, S-032, S-038, S-039, S-040 | アーキテクチャ・配布・差し替え可能性の要件。状態遷移に現れない |
| S-016, S-023(優先順位の中身) | 非決定的選択に抽象化(上記「捨てた領域」) |
| S-018, S-019, S-020, S-035, S-036 | 問題記録の種類別分類。状態遷移の発生そのものに暗黙対応、分類・保存形式は実装レベル |
| S-024 | `visited`への追加条件(全遷移先探索済みになるまで未探索扱い)は、単一画面への簡略化により`visited`集合への追加=探索済み化と等価になっている。画面をまたぐ遷移先の探索状況追跡はモデル外 |
| S-025, S-026, S-029 | 計測・記録の性質に関する要件。副作用として注記するのみ |
| S-034, S-037, S-043 | 出力形式・マスキング・TLS設定の要件。状態機械の外側(実装レベルの制約) |

## トレーサビリティ・マトリクスへの反映方針

台帳(`WebExploratoryQA.extract.md`)の「形式手法」列へ、上表の対応に従い次を記入する:
- 状態機械化した要件: 対応する`.tla`内のNext disjunct名、INVARIANT名またはPROPERTY名(本モデル文書と完全一致させる)
- モデル化で捨てた領域: 「モデル外(実装レベルで検証)」である旨
