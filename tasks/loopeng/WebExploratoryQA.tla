---- MODULE WebExploratoryQA ----
\* Web向け探索的テストエージェントの1ステップ実行ループ。
\* 元: tasks/loopeng/WebExploratoryQA.extract.md (S-001〜S-044)
\*      tasks/loopeng/WebExploratoryQA.model.md (状態/ドメインモデル)
\*
\* 1ステップループは3段: RequestDecision(判断問い合わせ) -> ApplyGuardrails(ガード適用) -> ExecuteOperation(実行)。
\* 1 disjunct = 1 EARS節の原則を守るため、判断問い合わせ・ガード判定・実行をそれぞれ別アクションに分離する
\* (1回目のformalizeレビューで、これらを1つのRequestAndActへ統合していたことを指摘され分離した)。

EXTENDS Naturals, FiniteSets

CONSTANTS
    MaxDecisions,        \* S-041: 判断モデルAPI呼び出し回数の上限
    MaxRequests,         \* S-042: 探索モードのリクエスト数上限
    IdleThreshold,        \* S-021: 固まり判定の閾値
    StuckThreshold,       \* S-021: 行き詰まり判定の閾値
    Elements,             \* 操作可能要素の有限集合(抽象化)
    ForbiddenElements,    \* S-012, S-028: 禁止要素の部分集合(購入/削除/退会/サインイン等)
    Operations            \* S-005: 判断モデルが返す操作種別8種の抽象化(全8種を使わず部分集合でも検査可)

ASSUME ForbiddenElements \subseteq Elements
ASSUME Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"}

VARIABLES
    phase,              \* 実行フェーズ
    proposedOp,         \* S-005: 判断モデルが返した操作種別(ガード適用前)
    proposedTarget,     \* S-006: 判断モデルが返した対象要素(ガード適用前、\in Elements \cup {"none"})
    proposedLowConfBack,\* S-010: 提案がbackでかつ確信度0.7未満かどうか
    isPaywall,          \* S-013: 現在画面がpaywall相当かどうか(非決定的に観測)
    targetExists,       \* S-028: 実行直前に対象要素がアクセシビリティツリーに存在するか
    screenChanged,      \* S-011: 直前操作後に画面が変化したか
    visited,            \* S-022, S-023, S-024: 探索済み要素集合
    idleCounter,        \* S-021: 固まり判定用カウンタ
    stuckCounter,       \* S-021: 行き詰まり判定用カウンタ
    inAllowedDomain,    \* S-033: 現在ページが許可ドメイン内か
    decisionBudget,     \* S-041
    requestBudget,      \* S-042
    exitReason          \* S-044, S-021, S-041, S-042: 終了理由

vars == <<phase, proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
          screenChanged, visited, idleCounter, stuckCounter,
          inAllowedDomain, decisionBudget, requestBudget, exitReason>>

Phases == {"idle", "awaiting_decision", "acting",
           "done", "stuck", "frozen_recovered", "error_exit"}
ExitReasons == {"none", "goal_achieved", "stuck", "budget_exceeded"}

\* --- TypeOK: S-001〜S-044 全変数が値域内に収まる(網羅性のアンカー) ---
TypeOK ==
    /\ phase \in Phases
    /\ proposedOp \in Operations \cup {"none"}
    /\ proposedTarget \in Elements \cup {"none"}
    /\ proposedLowConfBack \in BOOLEAN
    /\ isPaywall \in BOOLEAN
    /\ targetExists \in BOOLEAN
    /\ screenChanged \in BOOLEAN
    /\ visited \subseteq Elements
    /\ idleCounter \in Nat
    /\ stuckCounter \in Nat
    /\ inAllowedDomain \in BOOLEAN
    /\ decisionBudget \in Nat
    /\ requestBudget \in Nat
    /\ exitReason \in ExitReasons

\* --- Init ---
Init ==
    /\ phase = "idle"
    /\ proposedOp = "none"
    /\ proposedTarget = "none"
    /\ proposedLowConfBack = FALSE
    /\ isPaywall = FALSE
    /\ targetExists = TRUE
    /\ screenChanged = TRUE
    /\ visited = {}
    /\ idleCounter = 0
    /\ stuckCounter = 0
    /\ inAllowedDomain = TRUE
    /\ decisionBudget = MaxDecisions
    /\ requestBudget = MaxRequests
    /\ exitReason = "none"

\* --- Next: EARS event/state/unwanted を1 disjunctに(1節1アクション) ---

\* S-005〜S-009: 判断モデルへ画面状態を問い合わせ、operation/target/is_error_shown相当の
\* proposedOp・proposedTarget・proposedLowConfBack・isPaywall・targetExistsを非決定的に受け取る。
\* 判断モデルは禁止要素やpaywallの有無を知らず、何を提案してもよい(ガードはApplyGuardrailsが担う)。
RequestDecision ==
    /\ phase = "idle"
    /\ decisionBudget > 0
    /\ requestBudget > 0
    /\ inAllowedDomain
    /\ phase' = "awaiting_decision"
    /\ decisionBudget' = decisionBudget - 1
    /\ \E op \in Operations, t \in Elements, lowConfBack \in BOOLEAN,
          pw \in BOOLEAN, exists \in BOOLEAN :
        /\ proposedOp' = op
        /\ proposedTarget' = t
        /\ proposedLowConfBack' = (op = "back" /\ lowConfBack)
        /\ isPaywall' = pw
        /\ targetExists' = exists
    /\ UNCHANGED <<screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, requestBudget, exitReason>>

\* S-010, S-012, S-013, S-028: ガードレール。禁止操作・低確信度back・paywall・要素不存在を
\* ここで拒否し、許可されたときだけ"acting"へ進む。いずれかに該当すれば操作を実行せず"idle"へ戻す
\* (S-013はpaywallなら既定の離脱操作をacting扱いにして進む)。
\* S-012(禁止要素)・S-028(存在確認)は要素への直接操作(click/input)のときだけ適用するガードであり、
\* back/wait等の要素を伴わない操作には適用しない(1回目の修正でback提案時に禁止要素のまま
\* visitedへ混入する実装ミスが見つかったため、ガード側の前提条件も明示した)。
ApplyGuardrails ==
    /\ phase = "awaiting_decision"
    /\ \/ /\ isPaywall  \* S-013: paywallは判断モデルに問い合わせず既定の離脱操作(S-010の確信度判定の対象外)
          /\ phase' = "acting"
          /\ proposedOp' = "back"
          /\ proposedLowConfBack' = FALSE
          /\ UNCHANGED <<proposedTarget>>
       \/ /\ ~isPaywall
          /\ \/ /\ proposedOp = "back" /\ proposedLowConfBack  \* S-010: 低確信度backを拒否
                /\ phase' = "idle"
                /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack>>
             \/ /\ proposedOp \in {"click", "input"}
                /\ proposedTarget \in ForbiddenElements  \* S-012: 禁止要素を拒否
                /\ phase' = "idle"
                /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack>>
             \/ /\ proposedOp \in {"click", "input"}
                /\ ~targetExists  \* S-028: 実行直前の存在確認で不存在なら中止
                /\ phase' = "idle"
                /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack>>
             \/ /\ ~(proposedOp = "back" /\ proposedLowConfBack)
                /\ (proposedOp \notin {"click", "input"}
                    \/ (proposedTarget \notin ForbiddenElements /\ targetExists))
                /\ phase' = "acting"
                /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack>>
    /\ UNCHANGED <<isPaywall, targetExists, screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget, exitReason>>

\* S-002(正常系の中核要件): ガードレールを通過した操作を実際に実行する。
\* S-022, S-023: 要素への直接操作(click/input)であり、かつ対象が実在の要素のときだけ探索済み集合へ加える。
\* back/scroll_down/scroll_up/wait/done/stuckは対象要素を持たない操作なのでvisitedを変えない
\* (paywall検出時に既定でproposedOpがbackへ上書きされるケース(S-013)もここで正しく除外される)。
ExecuteOperation ==
    /\ phase = "acting"
    /\ visited' = IF proposedOp \in {"click", "input"} /\ proposedTarget \in Elements
                  THEN visited \cup {proposedTarget}
                  ELSE visited
    /\ UNCHANGED <<phase, proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, idleCounter, stuckCounter, inAllowedDomain,
                   decisionBudget, requestBudget, exitReason>>
    /\ phase' = "acting"  \* ObserveXxxが続けて画面の変化を観測し"idle"へ戻す

\* S-011: 画面が変化しなかったとき、次善候補へ切り替える前提としてこの観測を記録する。
\* done/stuckはFinishByProposal*が専用の終了遷移を担うため、ここでは通常操作(click/input/scroll/back/wait)
\* のときだけ観測する(2回目レビューでdone/stuck提案が無視されていた欠落を指摘され分離した)。
ObserveNoChange ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ phase' = "idle"
    /\ screenChanged' = FALSE
    /\ idleCounter' = idleCounter + 1
    /\ stuckCounter' = stuckCounter + 1
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   visited, inAllowedDomain, decisionBudget, requestBudget, exitReason>>

\* S-002(正常系): 操作の結果、画面が変化した(新規要素到達)。許可ドメイン内にとどまる。カウンタをリセット。
ObserveChange ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ phase' = "idle"
    /\ screenChanged' = TRUE
    /\ idleCounter' = 0
    /\ stuckCounter' = 0
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   visited, inAllowedDomain, decisionBudget, requestBudget, exitReason>>

\* S-033, S-021(ドメイン外): 操作の結果、許可ドメイン外へ遷移した。この画面は探索対象外として扱う。
\* (独立したトグルアクションにすると、ドメイン境界を往復するだけで有限予算を使い切らずに
\*  操作を回避し続けliveness(EventuallyTerminates)の反例になったため、
\*  ExecuteOperation後の画面観測の1つ=有限予算を消費する操作の帰結としてモデル化する。)
ObserveOutOfDomain ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ phase' = "idle"
    /\ screenChanged' = TRUE
    /\ idleCounter' = 0
    /\ stuckCounter' = 0
    /\ inAllowedDomain' = FALSE
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   visited, decisionBudget, requestBudget, exitReason>>

\* S-044(event, done): operation=doneが判断モデルから返されたとき、当該実行を正常終了として記録する。
\* done_when条件(S-014)はFinishNormally(visitedの到達度による構造的終了)が別途担うので、
\* この経路は「判断モデルの自己申告による終了」であり、FinishNormallyとは独立したEARS要件(S-044)として分離する。
FinishByProposalDone ==
    /\ phase = "acting"
    /\ proposedOp = "done"
    /\ phase' = "done"
    /\ exitReason' = "goal_achieved"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

\* S-044(event, stuck): operation=stuckが判断モデルから返されたとき、行き詰まり終了として記録する。
\* FinishStuck(stuckCounter閾値による構造的終了)とは独立したEARS要件(S-044)として分離する。
FinishByProposalStuck ==
    /\ phase = "acting"
    /\ proposedOp = "stuck"
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

\* ドメイン外から探索を継続する(対象外ページから離脱し、許可ドメイン内へ戻る)。
ReturnToAllowedDomain ==
    /\ phase = "idle"
    /\ ~inAllowedDomain
    /\ phase' = "idle"
    /\ inAllowedDomain' = TRUE
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   decisionBudget, requestBudget, exitReason>>

\* S-017, S-021(固まり): 画面無変化が閾値に到達 → 初期状態へ戻して探索継続。
RecoverFromFreeze ==
    /\ phase = "idle"
    /\ idleCounter >= IdleThreshold
    /\ phase' = "frozen_recovered"
    /\ idleCounter' = 0
    /\ visited' = {}  \* 再起動により探索状態を初期化
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, stuckCounter, inAllowedDomain, decisionBudget, requestBudget, exitReason>>

ResumeAfterRecovery ==
    /\ phase = "frozen_recovered"
    /\ phase' = "idle"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget, exitReason>>

\* S-014, S-044(正常終了): done_when条件相当 = 全要素が探索済みに到達した状態をゴール達成とみなす抽象化。
FinishNormally ==
    /\ phase = "idle"
    /\ visited = Elements \ ForbiddenElements
    /\ phase' = "done"
    /\ exitReason' = "goal_achieved"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

\* S-021(行き詰まり), S-044(stuck終了): 新規要素が出ない操作が閾値回数続いたら行き詰まり終了。
FinishStuck ==
    /\ phase = "idle"
    /\ stuckCounter >= StuckThreshold
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

\* S-041: 判断モデルAPI呼び出し予算が尽きたら強制終了。
ExceedDecisionBudget ==
    /\ phase = "idle"
    /\ decisionBudget = 0
    /\ phase' = "error_exit"
    /\ exitReason' = "budget_exceeded"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

Next ==
    \/ RequestDecision
    \/ ApplyGuardrails
    \/ ExecuteOperation
    \/ ObserveNoChange
    \/ ObserveChange
    \/ ObserveOutOfDomain
    \/ FinishByProposalDone
    \/ FinishByProposalStuck
    \/ ReturnToAllowedDomain
    \/ RecoverFromFreeze
    \/ ResumeAfterRecovery
    \/ FinishNormally
    \/ FinishStuck
    \/ ExceedDecisionBudget
    \/ (phase \in {"done", "stuck", "error_exit"} /\ UNCHANGED vars)  \* 終了状態で停止

\* --- Inv: ubiquitous要件を安全性として検査 ---

\* S-012, S-028: 禁止要素は決して探索済み集合に入らない(=操作されない)。
\* ApplyGuardrailsが拒否するため、判断モデルが禁止要素やtargetExists=FALSEを提案しても
\* visitedには反映されない。判断モデルの提案は非決定的(RequestDecisionで全組み合わせを許す)なので、
\* このInvはApplyGuardrailsのガード条件が実際に効いていることを検査する。
NeverOperatesForbidden ==
    visited \cap ForbiddenElements = {}

\* S-010: 低確信度のbackが提案された状態では、"acting"(実行)フェーズに留まらない
\* (ApplyGuardrailsが拒否して"idle"へ戻すため、低確信度backが実行に進む経路が無いことの検査)。
LowConfidenceBackNeverActed ==
    (proposedOp = "back" /\ proposedLowConfBack) => phase /= "acting"

\* S-041: 予算を使い切った後は新たな判断モデル呼び出し(RequestDecision)が起きない。
BudgetRespected ==
    decisionBudget >= 0

\* S-044: 終了フェーズに達したら終了理由が必ず設定されている。
ExitReasonSetOnTermination ==
    phase \in {"done", "stuck", "error_exit"} => exitReason /= "none"

\* S-042: リクエスト予算が尽きていたらRequestDecisionは起きない(ガードに内包、TypeOKで非負性も保証)。
RequestBudgetRespected ==
    requestBudget >= 0

Inv ==
    /\ TypeOK
    /\ NeverOperatesForbidden
    /\ LowConfidenceBackNeverActed
    /\ BudgetRespected
    /\ RequestBudgetRespected
    /\ ExitReasonSetOnTermination

\* --- Liveness: S-014/S-044 への到達性(行き詰まらない限りいつか終了する) ---
EventuallyTerminates ==
    []<>(phase \in {"done", "stuck", "error_exit"})

\* 各アクションに個別の弱公平性を与える: enabledのまま無視され続けることを許さない。
Spec == Init /\ [][Next]_vars
             /\ WF_vars(RequestDecision) /\ WF_vars(ApplyGuardrails) /\ WF_vars(ExecuteOperation)
             /\ WF_vars(ObserveNoChange) /\ WF_vars(ObserveChange)
             /\ WF_vars(ObserveOutOfDomain) /\ WF_vars(ReturnToAllowedDomain)
             /\ WF_vars(FinishByProposalDone) /\ WF_vars(FinishByProposalStuck)
             /\ WF_vars(FinishNormally) /\ WF_vars(FinishStuck) /\ WF_vars(ExceedDecisionBudget)
             /\ WF_vars(RecoverFromFreeze) /\ WF_vars(ResumeAfterRecovery)
====
