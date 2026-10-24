---- MODULE WebExploratoryQA ----
\* Web向け探索的テストエージェントの1ステップ実行ループ。
\* 元: tasks/loopeng/WebExploratoryQA.extract.md (S-001〜S-044)
\*      tasks/loopeng/WebExploratoryQA.model.md (状態/ドメインモデル)
\*
\* 1ステップは RequestDecision(判断問い合わせ) -> ApplyGuardrails(ガード適用) -> Execute*(実行と結果の観測)。
\* 実行と観測は結果(画面変化なし/変化あり/許可ドメイン外)ごとの1アクションにまとめる。
\* (中ループの mutation oracle で、実行と観測を別アクションにすると実行を飛ばして観測へ進めることが判明したため)
\*
\* モード(S-016, S-027): 決め打ちシナリオ("scenario")、探索("explore")、インタビュー("interview")。初期状態で非決定的に選び、実行中は変わらない。
\*   インタビューは段階と done_when を決め打ちシナリオと同じに扱う。違いは行き詰まりの数え方だけ
\*   (スクロールで見えた要素を進展に数えず、ガードレールで止めた提案を1回に数える)。
\*   決め打ちシナリオとインタビューの done_when は段階ごとの環境の観測(doneWhenMet)。実行の開始時、段階を進めた直後、操作の後に観測する。
\*   探索モードの done_when は、1回以上判断モデルに問い合わせた後で、禁止要素以外を探索し終えたこと。
\*
\* "idle" での判定の優先順位(S-014, S-015, S-044, S-021, S-041, S-016, S-017):
\*   done_when 成立(最後の段階ならゴール達成、そうでなければ段階の切り替え) > 行き詰まり検出
\*   > 判断モデルの失敗の連続 > {判断予算切れ, 時間切れ} > {固まり復旧, ドメイン復帰, 時間窓リセット} > 判断問い合わせ
\* 上位の条件が成立している間は下位のアクションを無効にする(同時成立時の終了理由を一意にするため)。
\* 利用者による中断(S-044)と時刻の経過(S-016, S-041)は環境の事象で、優先順位の外にある。どちらもどのフェーズからでも起こりうる。
\* 制限時間(S-016 の探索時間、S-041 の実行時間の上限)はどちらのモードにもありうる。設定されていなければ時刻の経過が起きないだけ。

EXTENDS Naturals, FiniteSets

CONSTANTS
    \* @type: Int;
    MaxDecisions,        \* S-041: 判断モデルAPI呼び出し回数の上限
    \* @type: Int;
    MaxRequests,         \* S-042: 単位時間あたりのリクエスト数上限
    \* @type: Int;
    IdleThreshold,       \* S-021: 固まり判定の閾値
    \* @type: Int;
    StuckThreshold,      \* S-021: 行き詰まり判定の閾値
    \* @type: Int;
    MaxJudgeFailures,    \* S-046: 判断モデルの失敗が続いたら終える回数
    \* @type: Int;
    NumStages,           \* S-015: 決め打ちシナリオの段階の数
    \* @type: Set(Str);
    Elements,            \* 操作可能要素の有限集合(抽象化)。ページの中身なので空もありうる
    \* @type: Set(Str);
    ForbiddenElements,   \* S-012, S-028: 禁止要素の部分集合(購入/削除/退会/サインイン等)
    \* @type: Set(Str);
    Operations           \* S-005: 判断モデルが返す操作種別9種の抽象化(部分集合でも検査可)

ASSUME ForbiddenElements \subseteq Elements
ASSUME Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "enter", "wait", "done", "stuck"}
\* 終了性(EventuallyTerminates)の前提: 閾値と上限は正。0 だと時間窓リセットや固まり復旧が空回りする。
ASSUME MaxDecisions \in Nat \ {0}
ASSUME MaxRequests \in Nat \ {0}
ASSUME IdleThreshold \in Nat \ {0}
ASSUME StuckThreshold \in Nat \ {0}
ASSUME MaxJudgeFailures \in Nat \ {0}
ASSUME NumStages \in Nat \ {0}

VARIABLES
    \* @type: Str;
    mode,               \* S-016, S-027: 実行モード("scenario" / "explore" / "interview")
    \* @type: Int;
    stage,              \* S-015: 決め打ちシナリオの現在の段階(1..NumStages)。探索モードでは1のまま
    \* @type: Str;
    phase,              \* 実行フェーズ
    \* @type: Str;
    proposedOp,         \* S-005: 判断モデルが返した操作種別
    \* @type: Str;
    proposedTarget,     \* S-006: 判断モデルが返した対象要素
    \* @type: Bool;
    proposedLowConf,    \* S-010: 判断モデルの確信度が0.7未満か(操作種別とは独立に観測)
    \* @type: Bool;
    isPaywall,          \* S-013: 現在画面がpaywall相当か
    \* @type: Bool;
    targetExists,       \* S-028: 実行直前に対象要素が存在するか
    \* @type: Bool;
    screenChanged,      \* S-011, S-021(固まり): 直前の操作後に画面が変化したかの観測結果
    \* @type: Bool;
    newSeen,            \* S-021(行き詰まり): 直前の操作後に未操作の要素が新しく見えたかの観測結果
    \* @type: Bool;
    doneWhenMet,        \* S-014, S-015: 決め打ちシナリオで、現在の段階の done_when を満たしたかの観測結果(段階に入った時点と操作の後に観測)
    \* @type: Set(Str);
    visited,            \* S-022, S-023: 探索済み要素集合
    \* @type: Int;
    idleCounter,        \* S-021(固まり): 画面が変化しなかった操作の連続回数
    \* @type: Int;
    stuckCounter,       \* S-021(行き詰まり): 新規要素が出なかった操作の連続回数
    \* @type: Bool;
    inAllowedDomain,    \* S-033: システムが記録した「現在ページは許可ドメイン内か」
    \* @type: Bool;
    pageOutside,        \* S-033: 環境側の真値。現在ページが実際に許可ドメイン外か(検査用)
    \* @type: Bool;
    timeUp,             \* S-016, S-041: 制限時間に達したか(時刻の経過の抽象化)
    \* @type: Int;
    decisionBudget,     \* S-041: 残りの判断モデルAPI呼び出し回数
    \* @type: Int;
    decisionsMade,      \* S-041: 実際に呼び出した回数(検査用の履歴)
    \* @type: Int;
    requestBudget,      \* S-042: 現在の時間窓で残っているリクエスト数
    \* @type: Int;
    requestsInWindow,   \* S-042: 現在の時間窓で実際に送ったリクエスト数(検査用の履歴)
    \* @type: Int;
    judgeFails,         \* S-046: 判断モデルに届かなかった失敗の連続回数
    \* @type: Str;
    exitReason          \* S-014, S-044, S-041, S-016, S-046: 終了理由

vars == <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists, screenChanged,
          newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside, timeUp,
          decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails, exitReason>>

Modes == {"scenario", "explore", "interview"}
Phases == {"idle", "awaiting_decision", "acting", "done", "stuck", "frozen_recovered", "error_exit", "timed_out", "aborted"}
Terminal == {"done", "stuck", "error_exit", "timed_out", "aborted"}
ExitReasons == {"none", "goal_achieved", "reported_done", "stuck", "budget_exceeded", "judge_unavailable", "time_exceeded",
                "user_aborted"}
DirectOps == {"click", "input"}
Explorable == Elements \ ForbiddenElements

\* S-016: 探索モードでは done を判断モデルの選択肢に入れない(探索モードの終わりはツール側の判定だけで決める)。
ProposableOps == IF mode = "explore" THEN Operations \ {"done"} ELSE Operations

\* S-005, S-006, S-028: 判断モデルが返しうる対象。対象を取らない操作は対象なし("none")。
\* 対象の要る操作(click/input)で要素が1つも無ければ、返る対象は存在しない要素番号("none")になる。
ProposalTargets(op) == IF op \in DirectOps /\ Elements # {} THEN Elements ELSE {"none"}

\* S-014, S-015, S-016: 現在の段階の done_when。探索モードでは、1回以上問い合わせた後で禁止要素以外をすべて探索し終えたこと。
DoneWhen == IF mode = "explore" THEN visited = Explorable /\ decisionsMade > 0 ELSE doneWhenMet

\* S-015: 最後の段階か(探索モードは段階が1つ)。
LastStage == mode = "explore" \/ stage = NumStages

TypeOK ==
    /\ mode \in Modes
    /\ stage \in 1..NumStages
    /\ phase \in Phases
    /\ proposedOp \in Operations \cup {"back", "none"}
    /\ proposedTarget \in Elements \cup {"none"}
    /\ proposedLowConf \in BOOLEAN
    /\ isPaywall \in BOOLEAN
    /\ targetExists \in BOOLEAN
    /\ screenChanged \in BOOLEAN
    /\ newSeen \in BOOLEAN
    /\ doneWhenMet \in BOOLEAN
    /\ visited \subseteq Elements
    /\ idleCounter \in Nat
    /\ stuckCounter \in Nat
    /\ inAllowedDomain \in BOOLEAN
    /\ pageOutside \in BOOLEAN
    /\ timeUp \in BOOLEAN
    /\ decisionBudget \in 0..MaxDecisions
    /\ decisionsMade \in Nat
    /\ requestBudget \in 0..MaxRequests
    /\ requestsInWindow \in Nat
    /\ judgeFails \in 0..MaxJudgeFailures
    /\ exitReason \in ExitReasons

\* S-014, S-015: done_when の観測がとりうる値。決め打ちシナリオとインタビューだけ観測し、探索モードでは常に未成立。
\* 実行の開始時、段階を進めた直後、操作の後の3か所で同じ観測をする。
DoneWhenObservations == IF mode # "explore" THEN BOOLEAN ELSE {FALSE}

Init ==
    /\ mode \in Modes
    /\ stage = 1
    /\ phase = "idle"
    /\ proposedOp = "none"
    /\ proposedTarget = "none"
    /\ proposedLowConf = FALSE
    /\ isPaywall = FALSE
    /\ targetExists = TRUE
    /\ screenChanged = TRUE
    /\ newSeen = FALSE
    /\ doneWhenMet \in DoneWhenObservations
    /\ visited = {}
    /\ idleCounter = 0
    /\ stuckCounter = 0
    /\ inAllowedDomain = TRUE
    /\ pageOutside = FALSE
    /\ timeUp = FALSE
    /\ decisionBudget = MaxDecisions
    /\ decisionsMade = 0
    /\ requestBudget = MaxRequests
    /\ requestsInWindow = 0
    /\ judgeFails = 0
    /\ exitReason = "none"

\* "idle" での優先順位の段(冒頭コメント参照)。
Exploring == phase = "idle" /\ ~DoneWhen                                                   \* done_when 未成立
Running == Exploring /\ stuckCounter < StuckThreshold                                    \* 行き詰まり未検出
Continuing == Running /\ judgeFails < MaxJudgeFailures /\ decisionBudget > 0 /\ ~timeUp   \* 終了条件なし

\* S-005〜S-009, S-041, S-042: 判断モデルへ問い合わせる。1回ごとに判断予算とリクエスト予算を1ずつ消費する。
\* S-017, S-033: 固まり検出中と許可ドメイン外では問い合わせない。
\* 判断モデルの提案と画面の観測結果は非決定的に受け取る(禁止要素も低確信度も提案されうる)。
\* S-046: 判断モデルに届かないこと(APIのエラー、待ち時間切れ、橋渡しの失敗)もある。そのときは提案を受けずに "idle" へ戻り、
\* 失敗の連続回数を1増やす。答えが届けば(使えない答えでも)0に戻す。使えない答えは、要素番号の無い直接操作として表す。
RequestDecision ==
    /\ Continuing
    /\ idleCounter < IdleThreshold
    /\ requestBudget > 0
    /\ inAllowedDomain
    /\ decisionBudget' = decisionBudget - 1
    /\ decisionsMade' = decisionsMade + 1
    /\ requestBudget' = requestBudget - 1
    /\ requestsInWindow' = requestsInWindow + 1
    /\ \/ /\ phase' = "awaiting_decision"
          /\ judgeFails' = 0
          /\ \E op \in ProposableOps, lc \in BOOLEAN, pw \in BOOLEAN :
             \E t \in ProposalTargets(op) : \E ex \in IF op \in DirectOps /\ t = "none" THEN {FALSE} ELSE BOOLEAN :
              /\ proposedOp' = op
              /\ proposedTarget' = t
              /\ proposedLowConf' = lc
              /\ isPaywall' = pw
              /\ targetExists' = ex
       \/ /\ judgeFails' = judgeFails + 1
          /\ UNCHANGED <<phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists>>
    /\ UNCHANGED <<mode, stage, screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter,
                   inAllowedDomain, pageOutside, timeUp, exitReason>>

\* S-042: 時間窓が経過したらリクエスト予算を戻す(時間経過の抽象化。窓の長さそのものはモデル外)。
\* 窓の満了は時刻依存なので、予算を1つでも使っていればいつでも起こりうる。段階をまたいでも窓は共有する。
ResetRequestWindow ==
    /\ Continuing
    /\ requestBudget < MaxRequests
    /\ requestBudget' = MaxRequests
    /\ requestsInWindow' = 0
    /\ UNCHANGED <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, judgeFails, exitReason>>

\* S-021(行き詰まり), S-033, InterviewMode S-047: 提案を止めて "idle" へ戻る。outside は、確かめ直しのために見たページが許可ドメインの外だったこと。
\* 外に着いたら(どのモードでも)行き詰まりを1回数え、記録する。インタビューでは、ガードレールで止めたことも1回に数える
\* (応答の異常=要素番号の無い直接操作は止めたのではないので数えない)。どちらも成り立っても、1ステップにつき1回だけ数える。
Refuse(outside) ==
    /\ phase' = "idle"
    /\ stuckCounter' = IF outside \/ (mode = "interview" /\ ~(proposedOp \in DirectOps /\ proposedTarget = "none"))
                       THEN stuckCounter + 1 ELSE stuckCounter
    /\ pageOutside' = outside
    /\ inAllowedDomain' = ~outside
    /\ newSeen' = (newSeen /\ ~outside)

\* S-010, S-012, S-013, S-028: ガードレール。拒否なら "idle" へ戻し、許可なら "acting" へ進む。
\* S-013: paywall なら判断モデルの提案を使わず既定の離脱操作(back)に置き換える(S-010 の対象外)。
\* S-012, S-028 は要素への直接操作(click/input)にだけ適用する。
\* S-028: 対象の存在は実行の直前にページを見直して確かめる。見直したページが許可ドメインの外のこともある(Refuse(TRUE))。
ApplyGuardrails ==
    /\ phase = "awaiting_decision"
    /\ \/ /\ isPaywall
          /\ phase' = "acting"
          /\ proposedOp' = "back"
          /\ UNCHANGED <<proposedTarget, proposedLowConf, stuckCounter, inAllowedDomain, pageOutside, newSeen>>
       \/ /\ ~isPaywall
          /\ \/ /\ proposedOp = "back"
                /\ proposedLowConf
                /\ Refuse(FALSE)
             \/ /\ proposedOp \in DirectOps
                /\ proposedTarget \in ForbiddenElements
                /\ Refuse(FALSE)
             \/ /\ proposedOp \in DirectOps
                /\ ~targetExists
                /\ Refuse(FALSE)
             \/ /\ proposedOp \in DirectOps
                /\ ~targetExists
                /\ proposedTarget \notin ForbiddenElements \cup {"none"}
                /\ Refuse(TRUE)
             \/ /\ ~(proposedOp = "back" /\ proposedLowConf)
                /\ (proposedOp \notin DirectOps \/ (proposedTarget \notin ForbiddenElements /\ targetExists))
                /\ phase' = "acting"
                /\ UNCHANGED <<stuckCounter, inAllowedDomain, pageOutside, newSeen>>
          /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf>>
    /\ UNCHANGED <<mode, stage, isPaywall, targetExists, screenChanged, doneWhenMet, visited, idleCounter,
                   timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails, exitReason>>

\* S-002, S-022, S-023: 許可された操作を実行した後の探索済み集合(要素への直接操作のときだけ対象要素を加える)。
VisitedAfterOp == IF proposedOp \in DirectOps THEN visited \cup {proposedTarget} ELSE visited

\* S-021(行き詰まり): 実行した操作で探索済み集合が増えず、未操作の要素も新しく見えなければ加算し、どちらかなら0に戻す。
\* InterviewMode S-047: インタビューでは、新しく見えた要素を進展に数えない(スクロールを続けるペルソナは行き詰まる)。
\* (newSeen' はこの式より前の連言で決める)
StuckAfterOp == IF VisitedAfterOp = visited /\ (mode = "interview" \/ ~newSeen') THEN stuckCounter + 1 ELSE 0

\* S-014, S-015: 決め打ちシナリオでは、実行後のページで現在の段階の done_when を満たしたかを観測する(環境の非決定)。
ObserveDoneWhen == doneWhenMet' \in DoneWhenObservations

\* S-002 + S-011 + S-021(固まり): 操作を実行し、画面が変化しなかったことを観測する(新しい要素も見えない)。
ExecuteNoChange ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ visited' = VisitedAfterOp
    /\ phase' = "idle"
    /\ screenChanged' = FALSE
    /\ newSeen' = FALSE
    /\ ObserveDoneWhen
    /\ idleCounter' = idleCounter + 1
    /\ stuckCounter' = StuckAfterOp
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   inAllowedDomain, pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget,
                   requestsInWindow, judgeFails, exitReason>>

\* S-002 + S-021: 操作を実行し、許可ドメイン内で画面が変化したことを観測する。
\* 未操作の要素が新しく見えたか(スクロール等)は環境の観測として非決定的に受け取る。
ExecuteChange ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ visited' = VisitedAfterOp
    /\ phase' = "idle"
    /\ screenChanged' = TRUE
    /\ newSeen' \in BOOLEAN
    /\ ObserveDoneWhen
    /\ idleCounter' = 0
    /\ stuckCounter' = StuckAfterOp
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   inAllowedDomain, pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget,
                   requestsInWindow, judgeFails, exitReason>>

\* S-002 + S-021(ドメイン外) + S-033: 操作を実行した結果、許可ドメイン外へ遷移したことを観測して記録する。
\* ドメイン外のページの要素は探索対象外なので、新しく見えた要素には数えない。
ExecuteOutOfDomain ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ visited' = VisitedAfterOp
    /\ phase' = "idle"
    /\ screenChanged' = TRUE
    /\ newSeen' = FALSE
    /\ ObserveDoneWhen
    /\ idleCounter' = 0
    /\ stuckCounter' = StuckAfterOp
    /\ pageOutside' = TRUE
    /\ inAllowedDomain' = FALSE
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails, exitReason>>

\* S-033: ドメイン外から許可ドメイン内へ戻り、探索を継続する。
ReturnToAllowedDomain ==
    /\ Continuing
    /\ ~inAllowedDomain
    /\ pageOutside' = FALSE
    /\ inAllowedDomain' = TRUE
    /\ UNCHANGED <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, timeUp,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails, exitReason>>

\* S-044(done) + S-014: 判断モデルが done を返したら自己申告による終了(未検証)として記録する。
\* done_when 成立時は問い合わせ前に FinishNormally / AdvanceStage が先取りするので、ここに来るのは done_when 未成立のときだけ。
\* 探索モードでは done を提案させないので、ここに来るのは決め打ちシナリオだけ。
FinishByProposalDone ==
    /\ phase = "acting"
    /\ proposedOp = "done"
    /\ phase' = "done"
    /\ exitReason' = "reported_done"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>

\* S-044(stuck): 判断モデルが stuck を返したら行き詰まり終了として記録する。
FinishByProposalStuck ==
    /\ phase = "acting"
    /\ proposedOp = "stuck"
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>

\* S-017, S-021(固まり): 画面無変化が閾値に到達したらページを初期状態へ戻して探索を継続する。
\* S-022: 戻すのはページ状態(固まりカウンタ)だけで、探索の記憶(探索済み集合)は保持する。
\* S-021: 行き詰まりカウンタは戻さない(復旧は新しい要素を出す操作ではない)。
\* 許可ドメイン外では問い合わせないので固まりカウンタは増えず、復旧は常に許可ドメイン内で起こる(ドメイン状態は戻す必要がない)。
RecoverFromFreeze ==
    /\ Continuing
    /\ idleCounter >= IdleThreshold
    /\ phase' = "frozen_recovered"
    /\ idleCounter' = 0
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, stuckCounter, inAllowedDomain, pageOutside, timeUp,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails, exitReason>>

ResumeAfterRecovery ==
    /\ phase = "frozen_recovered"
    /\ phase' = "idle"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails, exitReason>>

\* S-015: 最後でない段階の done_when を満たしたら次の段階へ進む。
\* 固まりと行き詰まりの回数は0に戻し、判断予算と時間窓は実行全体で共有する(戻さない)。
\* 進んだ直後に次の段階の done_when を観測する。すでに満たしていれば、操作せずにさらに進むかゴール達成になる。
AdvanceStage ==
    /\ phase = "idle"
    /\ DoneWhen
    /\ ~LastStage
    /\ stage' = stage + 1
    /\ doneWhenMet' \in DoneWhenObservations
    /\ idleCounter' = 0
    /\ stuckCounter' = 0
    /\ UNCHANGED <<mode, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, visited, inAllowedDomain, pageOutside, timeUp,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails, exitReason>>

\* S-014, S-015, S-016, S-044(goal): 最後の段階の done_when(探索モードでは禁止要素以外の探索済み)が成立したらゴール達成で終了する。
FinishNormally ==
    /\ phase = "idle"
    /\ DoneWhen
    /\ LastStage
    /\ phase' = "done"
    /\ exitReason' = "goal_achieved"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>

\* S-021(行き詰まり), S-044(stuck): 新規要素が出ない操作が閾値回数続いたら行き詰まり終了する。
FinishStuck ==
    /\ Exploring
    /\ stuckCounter >= StuckThreshold
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>

\* S-046: 判断モデルに届かない失敗が閾値の回数続いたら終える(判断予算切れと時間切れより先の段)。
FinishJudgeUnavailable ==
    /\ Running
    /\ judgeFails >= MaxJudgeFailures
    /\ phase' = "error_exit"
    /\ exitReason' = "judge_unavailable"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>

\* S-041: 判断モデルAPI呼び出し予算が尽きたら強制終了する。
ExceedDecisionBudget ==
    /\ Exploring
    /\ stuckCounter < StuckThreshold
    /\ judgeFails < MaxJudgeFailures
    /\ decisionBudget = 0
    /\ phase' = "error_exit"
    /\ exitReason' = "budget_exceeded"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>

\* S-016, S-041: 制限時間に達したら時間切れで終了する(判断予算切れと同じ段)。
FinishTimeUp ==
    /\ Exploring
    /\ stuckCounter < StuckThreshold
    /\ judgeFails < MaxJudgeFailures
    /\ timeUp
    /\ phase' = "timed_out"
    /\ exitReason' = "time_exceeded"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>

\* S-016, S-041: 時刻の経過(環境の事象)。制限時間は実行中のどの時点でも尽きうる。公平性は課さない。
TimeElapses ==
    /\ ~timeUp
    /\ phase \notin Terminal
    /\ timeUp' = TRUE
    /\ UNCHANGED <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails, exitReason>>

\* S-044(中断): 利用者はいつでも中断できる(環境の事象、公平性は課さない)。中断したら終了状態になる。
\* 記憶・終了のイベント・実行結果の書き出しはモデル外。
UserAbort ==
    /\ phase \notin Terminal
    /\ phase' = "aborted"
    /\ exitReason' = "user_aborted"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>

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
    \/ AdvanceStage
    \/ FinishNormally
    \/ FinishStuck
    \/ FinishJudgeUnavailable
    \/ ExceedDecisionBudget
    \/ FinishTimeUp
    \/ TimeElapses
    \/ UserAbort
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
        => (targetExists /\ proposedTarget \in Explorable)

\* 性質側の定義(システム側の DoneWhen / LastStage とは独立に、要件の言葉で書き直したもの)。
\* S-016: 探索モードの done_when は、1回以上判断モデルに問い合わせた後で、禁止要素以外をすべて探索し終えたこと。
ExploreDone == mode = "explore" /\ visited = Explorable /\ decisionsMade > 0
\* S-015: 決め打ちシナリオ(とインタビュー)の現在の段階の done_when を満たしたこと。
ScenarioStageDone == mode # "explore" /\ doneWhenMet
\* S-014, S-015, S-016: ゴール達成の条件(探索モードは探索し終えたこと、決め打ちシナリオは最後の段階の done_when)。
GoalReached == ExploreDone \/ (ScenarioStageDone /\ stage = NumStages)
\* S-015: 段階の切り替えの条件(決め打ちシナリオとインタビューで、最後でない段階の done_when)。
StageCleared == ScenarioStageDone /\ stage # NumStages

\* S-016: 探索モードでゴール達成で終わるのは、1回以上問い合わせた後で、禁止要素以外をすべて探索し終えたときだけ。
ExploreGoalOnlyWhenAllExplored ==
    (mode = "explore" /\ exitReason = "goal_achieved") => (visited = Explorable /\ decisionsMade > 0)

\* S-014, S-015: 決め打ちシナリオとインタビューでゴール達成で終わるのは、最後の段階の done_when を満たしたときだけ。
ScenarioGoalOnlyAtLastStage ==
    (mode # "explore" /\ exitReason = "goal_achieved") => (stage = NumStages /\ doneWhenMet)

\* S-044: 自己申告の done(reported_done)で終わるのは done_when を満たしていないときだけ。
ReportedDoneOnlyWithoutDoneWhen ==
    exitReason = "reported_done" => ~(ExploreDone \/ ScenarioStageDone)

\* S-016: 探索モードは自己申告の done で終わらない(ゴール達成・行き詰まり・予算切れ・時間切れ・中断のどれかで終わる)。
ExploreNeverReportedDone ==
    mode = "explore" => exitReason # "reported_done"

\* S-044, S-041, S-016, S-046: 終了フェーズと終了理由が対応する。終了前は終了理由を設定しない。
ExitReasonMatchesPhase ==
    /\ phase = "done" => exitReason \in {"goal_achieved", "reported_done"}
    /\ phase = "stuck" => exitReason = "stuck"
    /\ phase = "error_exit" => exitReason \in {"budget_exceeded", "judge_unavailable"}
    /\ phase = "timed_out" => exitReason = "time_exceeded"
    /\ phase = "aborted" => exitReason = "user_aborted"
    /\ phase \notin Terminal => exitReason = "none"

\* S-016, S-041: 時間切れで終わるのは、制限時間に達したときだけ。
TimeExceededOnlyWhenTimeUp ==
    exitReason = "time_exceeded" => timeUp

\* S-015, S-016: 段階と段階ごとの done_when は決め打ちシナリオにだけある。
StagesOnlyInScenario ==
    mode = "explore" => (stage = 1 /\ ~doneWhenMet)

\* S-021, S-033: 未操作の要素が新しく見えたのは、操作の後で、許可ドメイン内の画面が変化したときだけ。
NewSeenOnlyOnChangedAllowedPage ==
    newSeen => (screenChanged /\ inAllowedDomain /\ decisionsMade > 0)

\* S-011, S-021: 画面が変化しなかったと観測するのは、操作の後だけ。
NoChangeObservedOnlyAfterOps ==
    ~screenChanged => decisionsMade > 0

\* S-005, S-006: 判断モデルへ一度も問い合わせていない間は、提案も提案時の観測も無い。
NoProposalBeforeDecision ==
    decisionsMade = 0
        => (proposedOp = "none" /\ proposedTarget = "none" /\ ~proposedLowConf /\ ~isPaywall /\ targetExists)

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

\* S-046: 判断モデルの停止による終了は、失敗が閾値の回数続いたときだけ。
JudgeUnavailableOnlyAfterFailures ==
    exitReason = "judge_unavailable" => judgeFails >= MaxJudgeFailures

\* S-046: 失敗の連続回数は、閾値を超えず、問い合わせの回数も超えない。
JudgeFailsBounded ==
    judgeFails <= MaxJudgeFailures /\ judgeFails <= decisionsMade

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

\* S-005, S-006, S-016: 問い合わせ後の提案は操作種別9種のいずれか(paywall 時は既定の back)と、実在する対象要素。
\* 探索モードでは done を提案させない。
ProposalWellFormed ==
    phase \in {"awaiting_decision", "acting"}
        => (/\ proposedOp \in Operations \cup {"back"}
            /\ proposedTarget \in Elements \cup {"none"}
            /\ (phase = "awaiting_decision" /\ proposedOp \notin DirectOps => proposedTarget = "none")
            /\ (proposedOp \in DirectOps /\ proposedTarget = "none" => ~targetExists)
            /\ (proposedOp \in DirectOps /\ Elements # {} => proposedTarget \in Elements)
            /\ (mode = "explore" => proposedOp # "done"))

\* --- Action properties(優先順位と境界) ---

\* 環境の事象の一歩: 利用者による中断と時刻の経過。どの一歩の代わりにも起こりうる。
EnvEvent == (phase' = "aborted" /\ exitReason' = "user_aborted") \/ (~timeUp /\ timeUp')

\* S-014, S-015, S-044(goal): 最後の段階の done_when が成立した "idle" からの次の一歩は、ゴール達成での終了か環境の事象(中断・時刻の経過)。
GoalPreempts ==
    [][(phase = "idle" /\ GoalReached) => ((phase' = "done" /\ exitReason' = "goal_achieved") \/ EnvEvent)]_vars

\* S-015: 最後でない段階の done_when が成立した "idle" からの次の一歩は、次の段階への切り替えか環境の事象。
StageAdvancePreempts ==
    [][(phase = "idle" /\ StageCleared) => ((phase' = "idle" /\ stage' = stage + 1) \/ EnvEvent)]_vars

\* S-021, S-044(stuck): done_when 未成立で行き詰まりを検出した "idle" からの次の一歩は、行き詰まり終了か環境の事象。
StuckPreempts ==
    [][(phase = "idle" /\ ~ExploreDone /\ ~ScenarioStageDone /\ stuckCounter >= StuckThreshold) => (phase' = "stuck" \/ EnvEvent)]_vars

\* S-046: done_when 未成立で行き詰まりも検出していない "idle" で、判断モデルの失敗が閾値の回数続いていれば、
\* 次の一歩は判断モデルの停止での終了か環境の事象(判断予算切れや時間切れが同時に成り立っても、こちらが先)。
JudgeDownStops ==
    [][(phase = "idle" /\ ~ExploreDone /\ ~ScenarioStageDone /\ stuckCounter < StuckThreshold /\ judgeFails >= MaxJudgeFailures)
        => ((phase' = "error_exit" /\ exitReason' = "judge_unavailable") \/ EnvEvent)]_vars

\* S-041, S-016: 判断予算が尽きたか制限時間に達した "idle" からの次の一歩は(上位の終了条件が無ければ)、
\* 尽きた方の理由での終了か環境の事象。両方なら、どちらの理由でもよい(同じ段)。
LimitStops ==
    [][(phase = "idle" /\ ~ExploreDone /\ ~ScenarioStageDone /\ stuckCounter < StuckThreshold
        /\ judgeFails < MaxJudgeFailures /\ (decisionBudget = 0 \/ timeUp))
        => ((decisionBudget = 0 /\ phase' = "error_exit") \/ (timeUp /\ phase' = "timed_out") \/ EnvEvent)]_vars

\* S-017: 固まりを検出している間は判断モデルへ問い合わせない。
FreezePreemptsDecision ==
    [][(phase = "idle" /\ idleCounter >= IdleThreshold) => (phase' # "awaiting_decision")]_vars

\* S-002, S-010, S-012, S-028: ガードに掛からない提案(paywall でなく、低確信度の back でなく、
\* 直接操作なら対象が禁止要素でなく存在する)は拒否されず実行段階へ進む(環境の事象を除く)。
SafeProposalProceeds ==
    [][(/\ phase = "awaiting_decision"
        /\ ~isPaywall
        /\ ~(proposedOp = "back" /\ proposedLowConf)
        /\ (proposedOp \in DirectOps => (proposedTarget \notin ForbiddenElements /\ targetExists)))
        => (phase' = "acting" \/ EnvEvent)]_vars

\* S-005, S-041, S-046: ガード適用段階へ入るのは判断モデルへ問い合わせたとき(呼び出し回数が1増えるとき)だけ。
\* 逆に、呼び出し回数が1増える一歩は、ガード適用段階へ入るか、判断モデルに届かず "idle" に留まるかのどちらか。
AwaitingOnlyByDecision ==
    [][/\ (phase # "awaiting_decision" /\ phase' = "awaiting_decision") => (decisionsMade' = decisionsMade + 1)
       /\ (decisionsMade' = decisionsMade + 1)
            => (\/ (phase # "awaiting_decision" /\ phase' = "awaiting_decision")
                \/ (phase = "idle" /\ phase' = "idle" /\ judgeFails' = judgeFails + 1))]_vars

\* S-046: 失敗の連続回数が変わるのは問い合わせの一歩だけ。答えが届けば0に戻り、届かなければ1増える。
JudgeFailsTracksReplies ==
    [][(judgeFails' # judgeFails \/ decisionsMade' # decisionsMade)
        => (/\ decisionsMade' = decisionsMade + 1
            /\ phase' \in {"awaiting_decision", "idle"}
            /\ judgeFails' = IF phase' = "awaiting_decision" THEN 0 ELSE judgeFails + 1)]_vars

\* S-010, S-012, S-013, S-028: 実行段階へはガード適用段階からしか入らない(ガードを迂回しない)。
ActingOnlyAfterGuardrails ==
    [][(phase # "acting" /\ phase' = "acting") => (phase = "awaiting_decision")]_vars

\* S-017: done_when 未成立で固まりを検出してカウンタを戻す一歩は、復旧として記録する(frozen_recovered を経由する)。
FreezeRecoveryRecorded ==
    [][(phase = "idle" /\ ~ExploreDone /\ ~ScenarioStageDone /\ idleCounter >= IdleThreshold /\ idleCounter' = 0) => (phase' = "frozen_recovered")]_vars

\* S-002, S-022: 探索済み集合が変わるのは操作を実行した一歩だけ(終了や復旧では変わらない)。
VisitedChangesOnlyByExecution ==
    [][(visited' # visited) => (phase = "acting" /\ phase' = "idle")]_vars

\* S-013: paywall を検出したら、判断モデルの提案を使わず既定の離脱操作(back)を実行段階へ進める(環境の事象を除く)。
PaywallForcesBack ==
    [][(phase = "awaiting_decision" /\ isPaywall) => ((phase' = "acting" /\ proposedOp' = "back") \/ EnvEvent)]_vars

\* S-002, S-044: 実行段階の次の一歩は、done なら done 終了、stuck なら行き詰まり終了、それ以外は操作を実行して "idle" へ戻る(環境の事象を除く)。
ProposalOutcome ==
    [][(phase = "acting")
        => ((phase' = CASE proposedOp = "done" -> "done" [] proposedOp = "stuck" -> "stuck" [] OTHER -> "idle") \/ EnvEvent)]_vars

\* S-021(固まり): 操作を実行し終えた一歩で、画面が変化しなければ固まりカウンタは1増え、変化すれば0に戻る。
IdleCounterTracksScreen ==
    [][(phase = "acting" /\ phase' = "idle")
        => (idleCounter' = IF screenChanged' THEN 0 ELSE idleCounter + 1)]_vars

\* S-022: 探索の記憶は失われない(探索済み集合は減らない。固まり復旧でも保持する)。
MemoryNeverShrinks ==
    [][visited \subseteq visited']_vars

\* S-021(行き詰まり): 操作を実行し終えた一歩で、探索済み集合が増えず未操作の要素も新しく見えなければ
\* 行き詰まりカウンタは1増え、どちらかなら0に戻る。InterviewMode S-047: インタビューでは新しく見えた要素を数えない。
StuckCounterTracksNewElements ==
    [][(phase = "acting" /\ phase' = "idle")
        => (stuckCounter' = IF visited' = visited /\ (mode = "interview" \/ ~newSeen') THEN stuckCounter + 1 ELSE 0)]_vars

\* S-021(行き詰まり), InterviewMode S-011: 提案を止めて "idle" へ戻る一歩で、行き詰まりカウンタは、確かめ直しで許可ドメインの外に着いたか、
\* インタビューでガードレールが止めた(応答の異常でない)ときに1だけ増え、それ以外は変わらない。
RefusalCountsStuck ==
    [][(phase = "awaiting_decision" /\ phase' = "idle")
        => (stuckCounter' = IF ~inAllowedDomain' \/ (mode = "interview" /\ ~(proposedOp \in DirectOps /\ proposedTarget = "none"))
                            THEN stuckCounter + 1 ELSE stuckCounter)]_vars

\* S-033: 実際に許可ドメイン外にいる間は判断モデルへ問い合わせない(探索対象外)。
NoDecisionOutsideDomain ==
    [][(phase = "idle" /\ phase' = "awaiting_decision") => (~pageOutside)]_vars

\* S-015: 段階が変わる一歩は、"idle" で done_when を満たしたときの次の段階への切り替えだけで、
\* 固まりと行き詰まりの回数を0に戻す(次の段階の done_when は入った時点で観測し直す)。
StageAdvanceResetsCounters ==
    [][(stage' # stage)
        => (/\ phase = "idle" /\ phase' = "idle" /\ StageCleared /\ stage' = stage + 1
            /\ idleCounter' = 0 /\ stuckCounter' = 0)]_vars

\* S-015, S-041, S-042: 判断予算と時間窓は段階をまたいで共有する(段階の切り替えで戻さない)。
BudgetSharedAcrossStages ==
    [][(stage' # stage) => UNCHANGED <<decisionBudget, decisionsMade, requestBudget, requestsInWindow>>]_vars

\* S-041: 判断予算は減る一方で、実行中に戻らない。
DecisionBudgetNeverRestored ==
    [][decisionBudget' <= decisionBudget]_vars

\* S-016, S-041: 一度尽きた制限時間は戻らない。
TimeNeverRewinds ==
    [][timeUp => timeUp']_vars

\* S-016, S-041: 時刻の経過は環境の事象で、それだけではシステムの状態を変えない。
TimeElapsesAlone ==
    [][(~timeUp /\ timeUp') => UNCHANGED <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall,
                                           targetExists, screenChanged, newSeen, doneWhenMet, visited, idleCounter,
                                           stuckCounter, inAllowedDomain, pageOutside, decisionBudget, decisionsMade,
                                           requestBudget, requestsInWindow, judgeFails, exitReason>>]_vars

\* S-016, S-027: 実行モードは実行中に変わらない。
ModeFixed ==
    [][mode' = mode]_vars

\* S-005, S-006, S-013: 提案と画面の観測値が変わるのは、判断の問い合わせと、paywall による back への置き換えだけ。
ProposalChangesOnlyByDecision ==
    [][(<<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists>>'
          # <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists>>)
        => (\/ (phase # "awaiting_decision" /\ phase' = "awaiting_decision")
            \/ (/\ phase = "awaiting_decision" /\ isPaywall /\ proposedOp' = "back"
                /\ UNCHANGED <<proposedTarget, proposedLowConf, isPaywall, targetExists>>))]_vars

\* S-021: 固まりカウンタが変わるのは、操作の実行、固まり復旧、段階の切り替えだけ。
\* 行き詰まりカウンタが変わるのは、操作の実行、提案を止めた一歩、段階の切り替えだけ(復旧は新しい要素を出す操作ではない)。
CountersChangeOnlyBySteps ==
    [][/\ (idleCounter' # idleCounter)
            => ((phase = "acting" /\ phase' = "idle") \/ phase' = "frozen_recovered" \/ stage' = stage + 1)
       /\ (stuckCounter' # stuckCounter)
            => ((phase \in {"acting", "awaiting_decision"} /\ phase' = "idle") \/ stage' = stage + 1)]_vars

\* S-033, S-028: 許可ドメイン内外が変わるのは、操作の実行(外へ出る)、対象の確かめ直しで止めた一歩(外に着く)、許可ドメインへの復帰だけ。
DomainChangesOnlyByNavigation ==
    [][(inAllowedDomain' # inAllowedDomain \/ pageOutside' # pageOutside)
        => (\/ (phase = "acting" /\ phase' = "idle")
            \/ (/\ phase = "awaiting_decision" /\ phase' = "idle" /\ ~inAllowedDomain'
                /\ proposedOp \in DirectOps /\ ~targetExists /\ proposedTarget \notin ForbiddenElements \cup {"none"})
            \/ (phase = "idle" /\ phase' = "idle" /\ inAllowedDomain'))]_vars

\* S-011, S-021: 画面の観測結果が変わるのは、操作を実行した一歩だけ。
\* ただし、確かめ直しで許可ドメインの外に着いたら、新しく見えた要素は無くなる。
ObservationChangesOnlyByExecution ==
    [][/\ (screenChanged' # screenChanged) => (phase = "acting" /\ phase' = "idle")
       /\ (newSeen' # newSeen)
            => ((phase = "acting" /\ phase' = "idle") \/ (phase = "awaiting_decision" /\ ~inAllowedDomain' /\ ~newSeen'))]_vars

\* S-014, S-015: done_when の観測が成り立つのは操作を実行した一歩か段階を進めた一歩だけで、取り消されるのは段階の切り替えだけ。
DoneWhenChangesOnlyByExecutionOrAdvance ==
    [][/\ (~doneWhenMet /\ doneWhenMet') => ((phase = "acting" /\ phase' = "idle") \/ stage' = stage + 1)
       /\ (doneWhenMet /\ ~doneWhenMet') => (stage' = stage + 1)]_vars

\* S-044, S-041, S-016: 終了する一歩はフェーズと終了理由だけを変え、その時点の状態(記憶、予算、観測)をそのまま残す。
TerminationKeepsState ==
    [][(phase \notin Terminal /\ phase' \in Terminal)
        => UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                       screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                       pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, judgeFails>>]_vars

\* --- Temporal ---

\* S-015, S-016, S-041, S-042, S-046: 実行は最初の段階から、何も探索していない状態で、予算を満たし、制限時間が残った状態で始まる。
\* 最初の段階の done_when は開始時に観測するので、満たしていてもよい(探索モードでは観測しない)。
\* (状態述語の PROPERTY なので、初期状態だけで検査される)
StartsFresh ==
    /\ phase = "idle" /\ stage = 1 /\ visited = {} /\ ~timeUp /\ exitReason = "none"
    /\ idleCounter = 0 /\ stuckCounter = 0 /\ inAllowedDomain /\ (mode = "explore" => ~doneWhenMet)
    /\ decisionBudget = MaxDecisions /\ requestBudget = MaxRequests /\ judgeFails = 0

\* S-044, S-041: 実行はいつか終了する。
EventuallyTerminates ==
    <>(phase \in Terminal)

\* S-044, S-041, S-016: 一度終了したら(中断を含む)状態は変わらない(終了状態から探索へ戻らない)。
TerminalAbsorbing ==
    [][(phase \in Terminal) => (UNCHANGED vars)]_vars

\* 利用者による中断(UserAbort)と時刻の経過(TimeElapses)には公平性を課さない(起きなくてもよい事象)。
Spec == Init /\ [][Next]_vars
             /\ WF_vars(RequestDecision) /\ WF_vars(ResetRequestWindow) /\ WF_vars(ApplyGuardrails)
             /\ WF_vars(ExecuteNoChange \/ ExecuteChange \/ ExecuteOutOfDomain)  \* 結果はどれか1つが起きればよい
             /\ WF_vars(ReturnToAllowedDomain)
             /\ WF_vars(FinishByProposalDone) /\ WF_vars(FinishByProposalStuck)
             /\ WF_vars(RecoverFromFreeze) /\ WF_vars(ResumeAfterRecovery)
             /\ WF_vars(AdvanceStage) /\ WF_vars(FinishNormally) /\ WF_vars(FinishStuck)
             /\ WF_vars(FinishJudgeUnavailable) /\ WF_vars(ExceedDecisionBudget) /\ WF_vars(FinishTimeUp)
====
