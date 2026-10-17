---- MODULE WebExploratoryQA ----
\* Web向け探索的テストエージェントの1ステップ実行ループ。
\* 元: tasks/loopeng/WebExploratoryQA.extract.md (S-001〜S-044)
\*      tasks/loopeng/WebExploratoryQA.model.md (状態/ドメインモデル)
\*
\* 1ステップは RequestDecision(判断問い合わせ) -> ApplyGuardrails(ガード適用) -> Execute*(実行と結果の観測)。
\* 実行と観測は結果(画面変化なし/変化あり/許可ドメイン外)ごとの1アクションにまとめる。
\* (中ループの mutation oracle で、実行と観測を別アクションにすると実行を飛ばして観測へ進めることが判明したため)
\*
\* "idle" での判定の優先順位(S-014, S-044, S-021, S-041, S-017):
\*   done_when 成立(ゴール達成) > 行き詰まり検出 > 判断予算切れ > {固まり復旧, ドメイン復帰, 時間窓リセット} > 判断問い合わせ
\* 上位の条件が成立している間は下位のアクションを無効にする(同時成立時の終了理由を一意にするため)。

EXTENDS Naturals, FiniteSets

CONSTANTS
    MaxDecisions,        \* S-041: 判断モデルAPI呼び出し回数の上限
    MaxRequests,         \* S-042: 単位時間あたりのリクエスト数上限
    IdleThreshold,       \* S-021: 固まり判定の閾値
    StuckThreshold,      \* S-021: 行き詰まり判定の閾値
    Elements,            \* 操作可能要素の有限集合(抽象化)
    ForbiddenElements,   \* S-012, S-028: 禁止要素の部分集合(購入/削除/退会/サインイン等)
    Operations           \* S-005: 判断モデルが返す操作種別8種の抽象化(部分集合でも検査可)

ASSUME ForbiddenElements \subseteq Elements
ASSUME Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"}
\* 終了性(EventuallyTerminates)の前提: 閾値と上限は正。0 だと時間窓リセットや固まり復旧が空回りする。
ASSUME MaxDecisions \in Nat \ {0}
ASSUME MaxRequests \in Nat \ {0}
ASSUME IdleThreshold \in Nat \ {0}
ASSUME StuckThreshold \in Nat \ {0}

VARIABLES
    phase,              \* 実行フェーズ
    proposedOp,         \* S-005: 判断モデルが返した操作種別
    proposedTarget,     \* S-006: 判断モデルが返した対象要素
    proposedLowConf,    \* S-010: 判断モデルの確信度が0.7未満か(操作種別とは独立に観測)
    isPaywall,          \* S-013: 現在画面がpaywall相当か
    targetExists,       \* S-028: 実行直前に対象要素が存在するか
    screenChanged,      \* S-011, S-021(固まり): 直前の操作後に画面が変化したかの観測結果
    visited,            \* S-022, S-023: 探索済み要素集合
    idleCounter,        \* S-021(固まり): 画面が変化しなかった操作の連続回数
    stuckCounter,       \* S-021(行き詰まり): 新規要素が出なかった操作の連続回数
    inAllowedDomain,    \* S-033: システムが記録した「現在ページは許可ドメイン内か」
    pageOutside,        \* S-033: 環境側の真値。現在ページが実際に許可ドメイン外か(検査用)
    decisionBudget,     \* S-041: 残りの判断モデルAPI呼び出し回数
    decisionsMade,      \* S-041: 実際に呼び出した回数(検査用の履歴)
    requestBudget,      \* S-042: 現在の時間窓で残っているリクエスト数
    requestsInWindow,   \* S-042: 現在の時間窓で実際に送ったリクエスト数(検査用の履歴)
    exitReason          \* S-014, S-044, S-041: 終了理由

vars == <<phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists, screenChanged,
          visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
          decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

Phases == {"idle", "awaiting_decision", "acting", "done", "stuck", "frozen_recovered", "error_exit"}
Terminal == {"done", "stuck", "error_exit"}
ExitReasons == {"none", "goal_achieved", "reported_done", "stuck", "budget_exceeded"}
DirectOps == {"click", "input"}
Explorable == Elements \ ForbiddenElements

TypeOK ==
    /\ phase \in Phases
    /\ proposedOp \in Operations \cup {"back", "none"}
    /\ proposedTarget \in Elements \cup {"none"}
    /\ proposedLowConf \in BOOLEAN
    /\ isPaywall \in BOOLEAN
    /\ targetExists \in BOOLEAN
    /\ screenChanged \in BOOLEAN
    /\ visited \subseteq Elements
    /\ idleCounter \in Nat
    /\ stuckCounter \in Nat
    /\ inAllowedDomain \in BOOLEAN
    /\ pageOutside \in BOOLEAN
    /\ decisionBudget \in 0..MaxDecisions
    /\ decisionsMade \in Nat
    /\ requestBudget \in 0..MaxRequests
    /\ requestsInWindow \in Nat
    /\ exitReason \in ExitReasons

Init ==
    /\ phase = "idle"
    /\ proposedOp = "none"
    /\ proposedTarget = "none"
    /\ proposedLowConf = FALSE
    /\ isPaywall = FALSE
    /\ targetExists = TRUE
    /\ screenChanged = TRUE
    /\ visited = {}
    /\ idleCounter = 0
    /\ stuckCounter = 0
    /\ inAllowedDomain = TRUE
    /\ pageOutside = FALSE
    /\ decisionBudget = MaxDecisions
    /\ decisionsMade = 0
    /\ requestBudget = MaxRequests
    /\ requestsInWindow = 0
    /\ exitReason = "none"

\* "idle" での優先順位の段(冒頭コメント参照)。
Exploring == phase = "idle" /\ visited # Explorable              \* S-014, S-044: done_when 未成立
Continuing == Exploring /\ stuckCounter < StuckThreshold /\ decisionBudget > 0  \* S-021, S-041: 終了条件なし

\* S-005〜S-009, S-041, S-042: 判断モデルへ問い合わせる。1回ごとに判断予算とリクエスト予算を1ずつ消費する。
\* S-017, S-033: 固まり検出中と許可ドメイン外では問い合わせない。
\* 判断モデルの提案と画面の観測結果は非決定的に受け取る(禁止要素も低確信度も提案されうる)。
RequestDecision ==
    /\ Continuing
    /\ idleCounter < IdleThreshold
    /\ requestBudget > 0
    /\ inAllowedDomain
    /\ phase' = "awaiting_decision"
    /\ decisionBudget' = decisionBudget - 1
    /\ decisionsMade' = decisionsMade + 1
    /\ requestBudget' = requestBudget - 1
    /\ requestsInWindow' = requestsInWindow + 1
    /\ \E op \in Operations, t \in Elements, lc \in BOOLEAN, pw \in BOOLEAN, ex \in BOOLEAN :
        /\ proposedOp' = op
        /\ proposedTarget' = t
        /\ proposedLowConf' = lc
        /\ isPaywall' = pw
        /\ targetExists' = ex
    /\ UNCHANGED <<screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside, exitReason>>

\* S-042: 時間窓が経過したらリクエスト予算を戻す(時間経過の抽象化。窓の長さそのものはモデル外)。
\* 窓の満了は時刻依存なので、予算を1つでも使っていればいつでも起こりうる。
ResetRequestWindow ==
    /\ Continuing
    /\ requestBudget < MaxRequests
    /\ requestBudget' = MaxRequests
    /\ requestsInWindow' = 0
    /\ UNCHANGED <<phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, exitReason>>

\* S-010, S-012, S-013, S-028: ガードレール。拒否なら "idle" へ戻し、許可なら "acting" へ進む。
\* S-013: paywall なら判断モデルの提案を使わず既定の離脱操作(back)に置き換える(S-010 の対象外)。
\* S-012, S-028 は要素への直接操作(click/input)にだけ適用する。
ApplyGuardrails ==
    /\ phase = "awaiting_decision"
    /\ \/ /\ isPaywall
          /\ phase' = "acting"
          /\ proposedOp' = "back"
          /\ UNCHANGED <<proposedTarget, proposedLowConf>>
       \/ /\ ~isPaywall
          /\ \/ /\ proposedOp = "back"
                /\ proposedLowConf
                /\ phase' = "idle"
             \/ /\ proposedOp \in DirectOps
                /\ proposedTarget \in ForbiddenElements
                /\ phase' = "idle"
             \/ /\ proposedOp \in DirectOps
                /\ ~targetExists
                /\ phase' = "idle"
             \/ /\ ~(proposedOp = "back" /\ proposedLowConf)
                /\ (proposedOp \notin DirectOps \/ (proposedTarget \notin ForbiddenElements /\ targetExists))
                /\ phase' = "acting"
          /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf>>
    /\ UNCHANGED <<isPaywall, targetExists, screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, pageOutside, decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

\* S-002, S-022, S-023: 許可された操作を実行した後の探索済み集合(要素への直接操作のときだけ対象要素を加える)。
VisitedAfterOp == IF proposedOp \in DirectOps THEN visited \cup {proposedTarget} ELSE visited

\* S-021(行き詰まり): 実行した操作で新規要素が出なければ加算し、出たら0に戻す。
StuckAfterOp == IF VisitedAfterOp = visited THEN stuckCounter + 1 ELSE 0

\* S-002 + S-011 + S-021(固まり): 操作を実行し、画面が変化しなかったことを観測する。
ExecuteNoChange ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ visited' = VisitedAfterOp
    /\ phase' = "idle"
    /\ screenChanged' = FALSE
    /\ idleCounter' = idleCounter + 1
    /\ stuckCounter' = StuckAfterOp
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   inAllowedDomain, pageOutside, decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

\* S-002: 操作を実行し、許可ドメイン内で画面が変化したことを観測する。
ExecuteChange ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ visited' = VisitedAfterOp
    /\ phase' = "idle"
    /\ screenChanged' = TRUE
    /\ idleCounter' = 0
    /\ stuckCounter' = StuckAfterOp
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   inAllowedDomain, pageOutside, decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

\* S-002 + S-021(ドメイン外) + S-033: 操作を実行した結果、許可ドメイン外へ遷移したことを観測して記録する。
ExecuteOutOfDomain ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ visited' = VisitedAfterOp
    /\ phase' = "idle"
    /\ screenChanged' = TRUE
    /\ idleCounter' = 0
    /\ stuckCounter' = StuckAfterOp
    /\ pageOutside' = TRUE
    /\ inAllowedDomain' = FALSE
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

\* S-033: ドメイン外から許可ドメイン内へ戻り、探索を継続する。
ReturnToAllowedDomain ==
    /\ Continuing
    /\ ~inAllowedDomain
    /\ pageOutside' = FALSE
    /\ inAllowedDomain' = TRUE
    /\ UNCHANGED <<phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

\* S-044(done) + S-014: 判断モデルが done を返したら自己申告による終了(未検証)として記録する。
\* done_when 成立時は問い合わせ前に FinishNormally が先取りするので、ここに来るのは done_when 未成立のときだけ。
FinishByProposalDone ==
    /\ phase = "acting"
    /\ proposedOp = "done"
    /\ phase' = "done"
    /\ exitReason' = "reported_done"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

\* S-044(stuck): 判断モデルが stuck を返したら行き詰まり終了として記録する。
FinishByProposalStuck ==
    /\ phase = "acting"
    /\ proposedOp = "stuck"
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

\* S-017, S-021(固まり): 画面無変化が閾値に到達したらページを初期状態へ戻して探索を継続する。
\* S-022: 戻すのはページ状態(固まりカウンタ)だけで、探索の記憶(探索済み集合)は保持する。行き詰まりカウンタも戻さないが、要件の対象外の選択で性質では検査しない。
\* 許可ドメイン外では問い合わせないので固まりカウンタは増えず、復旧は常に許可ドメイン内で起こる(ドメイン状態は戻す必要がない)。
RecoverFromFreeze ==
    /\ Continuing
    /\ idleCounter >= IdleThreshold
    /\ phase' = "frozen_recovered"
    /\ idleCounter' = 0
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, stuckCounter, inAllowedDomain, pageOutside, decisionBudget, decisionsMade,
                   requestBudget, requestsInWindow, exitReason>>

ResumeAfterRecovery ==
    /\ phase = "frozen_recovered"
    /\ phase' = "idle"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

\* S-014, S-044(goal): done_when 相当(探索可能な要素を全て探索済み)が成立したらゴール達成で終了する。
FinishNormally ==
    /\ phase = "idle"
    /\ visited = Explorable
    /\ phase' = "done"
    /\ exitReason' = "goal_achieved"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

\* S-021(行き詰まり), S-044(stuck): 新規要素が出ない操作が閾値回数続いたら行き詰まり終了する。
FinishStuck ==
    /\ Exploring
    /\ stuckCounter >= StuckThreshold
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

\* S-041: 判断モデルAPI呼び出し予算が尽きたら強制終了する。
ExceedDecisionBudget ==
    /\ Exploring
    /\ stuckCounter < StuckThreshold
    /\ decisionBudget = 0
    /\ phase' = "error_exit"
    /\ exitReason' = "budget_exceeded"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

Next ==
    \/ RequestDecision
    \/ ResetRequestWindow
    \/ ApplyGuardrails
    \/ ExecuteNoChange
    \/ ExecuteChange
    \/ ExecuteOutOfDomain
    \/ ReturnToAllowedDomain
    \/ FinishByProposalDone
    \/ FinishByProposalStuck
    \/ RecoverFromFreeze
    \/ ResumeAfterRecovery
    \/ FinishNormally
    \/ FinishStuck
    \/ ExceedDecisionBudget
    \/ (phase \in Terminal /\ UNCHANGED vars)  \* 終了状態で停止

\* --- Inv ---

\* S-012, S-028: 禁止要素は探索済み集合に入らない(=操作されない)。
NeverOperatesForbidden ==
    visited \cap ForbiddenElements = {}

\* S-010: paywall 以外で実行段階に進んだ back は、低確信度ではない。
LowConfidenceBackNeverActed ==
    (phase = "acting" /\ ~isPaywall /\ proposedOp = "back") => ~proposedLowConf

\* S-012, S-028: paywall 以外で実行段階に進んだ直接操作の対象は、禁止要素ではなく、存在している。
DirectOpActedOnlyOnSafeTarget ==
    (phase = "acting" /\ ~isPaywall /\ proposedOp \in DirectOps)
        => (targetExists /\ proposedTarget \notin ForbiddenElements)

\* S-014: ゴール達成で終わるのは done_when を満たしたときだけ。
GoalOnlyByDoneWhen ==
    exitReason = "goal_achieved" => visited = Explorable

\* S-044: 自己申告の done(reported_done)で終わるのは done_when を満たしていないときだけ。
ReportedDoneOnlyWithoutDoneWhen ==
    exitReason = "reported_done" => visited # Explorable

\* S-044, S-041: 終了フェーズと終了理由が対応する。終了前は終了理由を設定しない。
ExitReasonMatchesPhase ==
    /\ phase = "done" => exitReason \in {"goal_achieved", "reported_done"}
    /\ phase = "stuck" => exitReason = "stuck"
    /\ phase = "error_exit" => exitReason = "budget_exceeded"
    /\ phase \notin Terminal => exitReason = "none"

\* S-042: 1つの時間窓で送るリクエスト数は上限以下。
RequestsWithinWindowCap ==
    requestsInWindow <= MaxRequests

\* S-041: 判断モデルAPIの呼び出し回数は上限以下。
DecisionsWithinCap ==
    decisionsMade <= MaxDecisions

\* S-041, S-042: 残り予算は上限から実際の呼び出し数を引いた値(予算を勝手に増減させない)。
BudgetAccounting ==
    /\ decisionBudget + decisionsMade = MaxDecisions
    /\ requestBudget + requestsInWindow = MaxRequests

\* S-041: 予算超過による終了は、判断予算を使い切ったときだけ。
BudgetExceededOnlyWhenSpent ==
    exitReason = "budget_exceeded" => decisionBudget = 0

\* S-021: 固まり/行き詰まりカウンタは操作の回数を数えるので、実行した判断の回数を超えない。
CountersBoundedByOps ==
    idleCounter <= decisionsMade /\ stuckCounter <= decisionsMade

\* S-002, S-022: 探索済み要素は1回の操作で高々1つ増えるので、その数は実行した判断の回数を超えない。
VisitedOnlyByOps ==
    Cardinality(visited) <= decisionsMade

\* S-017: 固まり復旧は、固まり判定に必要な回数の操作をした後にしか起こらない。
RecoveryOnlyAfterFreeze ==
    phase = "frozen_recovered" => decisionsMade >= IdleThreshold

\* S-017: 固まりから復旧した直後はページ状態が初期化されている(固まりカウンタが0、許可ドメイン内)。
RecoveryResetsPageState ==
    phase = "frozen_recovered" => (idleCounter = 0 /\ inAllowedDomain)

\* S-021(ドメイン外), S-033: システムが記録した許可ドメイン内外は、実際のページ位置と一致する。
DomainRecordAccurate ==
    inAllowedDomain = ~pageOutside

\* S-005, S-006: 問い合わせ後の提案は操作種別8種のいずれか(paywall 時は既定の back)と、実在する対象要素。
ProposalWellFormed ==
    phase \in {"awaiting_decision", "acting"} => (proposedOp \in Operations \cup {"back"} /\ proposedTarget \in Elements)

\* --- Action properties(優先順位と境界) ---

\* S-014, S-044(goal): done_when が成立した "idle" からの次の一歩は必ずゴール達成での終了。
GoalPreempts ==
    [][(phase = "idle" /\ visited = Explorable) => (phase' = "done" /\ exitReason' = "goal_achieved")]_vars

\* S-021, S-044(stuck): done_when 未成立で行き詰まりを検出した "idle" からの次の一歩は必ず行き詰まり終了。
StuckPreempts ==
    [][(phase = "idle" /\ visited # Explorable /\ stuckCounter >= StuckThreshold) => (phase' = "stuck")]_vars

\* S-041: 判断予算が尽きた "idle" からの次の一歩は(上位の終了条件が無ければ)必ず強制終了。
BudgetStops ==
    [][(phase = "idle" /\ visited # Explorable /\ stuckCounter < StuckThreshold /\ decisionBudget = 0)
        => (phase' = "error_exit")]_vars

\* S-017: 固まりを検出している間は判断モデルへ問い合わせない。
FreezePreemptsDecision ==
    [][(phase = "idle" /\ idleCounter >= IdleThreshold) => (phase' # "awaiting_decision")]_vars

\* S-002, S-010, S-012, S-028: ガードに掛からない提案(paywall でなく、低確信度の back でなく、
\* 直接操作なら対象が禁止要素でなく存在する)は拒否されず実行段階へ進む。
SafeProposalProceeds ==
    [][(/\ phase = "awaiting_decision"
        /\ ~isPaywall
        /\ ~(proposedOp = "back" /\ proposedLowConf)
        /\ (proposedOp \in DirectOps => (proposedTarget \notin ForbiddenElements /\ targetExists)))
        => (phase' = "acting")]_vars

\* S-005, S-041: ガード適用段階へ入るのは判断モデルへ問い合わせたとき(呼び出し回数が1増えるとき)だけで、その逆も成り立つ。
AwaitingOnlyByDecision ==
    [][((phase # "awaiting_decision" /\ phase' = "awaiting_decision") <=> (decisionsMade' = decisionsMade + 1))]_vars

\* S-010, S-012, S-013, S-028: 実行段階へはガード適用段階からしか入らない(ガードを迂回しない)。
ActingOnlyAfterGuardrails ==
    [][(phase # "acting" /\ phase' = "acting") => (phase = "awaiting_decision")]_vars

\* S-017: 固まりを検出してカウンタを戻す一歩は、復旧として記録する(frozen_recovered を経由する)。
FreezeRecoveryRecorded ==
    [][(phase = "idle" /\ idleCounter >= IdleThreshold /\ idleCounter' = 0) => (phase' = "frozen_recovered")]_vars

\* S-002, S-022: 探索済み集合が変わるのは操作を実行した一歩だけ(終了や復旧では変わらない)。
VisitedChangesOnlyByExecution ==
    [][(visited' # visited) => (phase = "acting" /\ phase' = "idle")]_vars

\* S-013: paywall を検出したら、判断モデルの提案を使わず既定の離脱操作(back)を実行段階へ進める。
PaywallForcesBack ==
    [][(phase = "awaiting_decision" /\ isPaywall) => (phase' = "acting" /\ proposedOp' = "back")]_vars

\* S-002, S-044: 実行段階の次の一歩は、done なら done 終了、stuck なら行き詰まり終了、それ以外は操作を実行して "idle" へ戻る。
ProposalOutcome ==
    [][(phase = "acting")
        => (phase' = CASE proposedOp = "done" -> "done" [] proposedOp = "stuck" -> "stuck" [] OTHER -> "idle")]_vars

\* S-021(固まり): 操作を実行し終えた一歩で、画面が変化しなければ固まりカウンタは1増え、変化すれば0に戻る。
IdleCounterTracksScreen ==
    [][(phase = "acting" /\ phase' = "idle")
        => (idleCounter' = IF screenChanged' THEN 0 ELSE idleCounter + 1)]_vars

\* S-022: 探索の記憶は失われない(探索済み集合は減らない。固まり復旧でも保持する)。
MemoryNeverShrinks ==
    [][visited \subseteq visited']_vars

\* S-021(行き詰まり): 操作を実行し終えた一歩で、探索済み集合が増えなければ行き詰まりカウンタは1増え、増えれば0に戻る。
StuckCounterTracksNewElements ==
    [][(phase = "acting" /\ phase' = "idle")
        => (stuckCounter' = IF visited' = visited THEN stuckCounter + 1 ELSE 0)]_vars

\* S-033: 実際に許可ドメイン外にいる間は判断モデルへ問い合わせない(探索対象外)。
NoDecisionOutsideDomain ==
    [][(phase = "idle" /\ phase' = "awaiting_decision") => (~pageOutside)]_vars

\* --- Temporal ---

\* S-044, S-041: 実行はいつか終了する。
EventuallyTerminates ==
    <>(phase \in Terminal)

\* S-044, S-041: 一度終了したら状態は変わらない(終了状態から探索へ戻らない)。
TerminalAbsorbing ==
    [][(phase \in Terminal) => (UNCHANGED vars)]_vars

Spec == Init /\ [][Next]_vars
             /\ WF_vars(RequestDecision) /\ WF_vars(ResetRequestWindow) /\ WF_vars(ApplyGuardrails)
             /\ WF_vars(ExecuteNoChange \/ ExecuteChange \/ ExecuteOutOfDomain)  \* 結果はどれか1つが起きればよい
             /\ WF_vars(ReturnToAllowedDomain)
             /\ WF_vars(FinishByProposalDone) /\ WF_vars(FinishByProposalStuck)
             /\ WF_vars(RecoverFromFreeze) /\ WF_vars(ResumeAfterRecovery)
             /\ WF_vars(FinishNormally) /\ WF_vars(FinishStuck) /\ WF_vars(ExceedDecisionBudget)
====
