EXTENDS Naturals, FiniteSets

CONSTANTS
    MaxDecisions,
    MaxRequests,
    IdleThreshold,
    StuckThreshold,
    NumStages,
    Elements,
    ForbiddenElements,
    Operations

ASSUME ForbiddenElements \subseteq Elements
ASSUME Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"}
ASSUME MaxDecisions \in Nat \ {0}
ASSUME MaxRequests \in Nat \ {0}
ASSUME IdleThreshold \in Nat \ {0}
ASSUME StuckThreshold \in Nat \ {0}
ASSUME NumStages \in Nat \ {0}

VARIABLES
    mode,
    stage,
    phase,
    proposedOp,
    proposedTarget,
    proposedLowConf,
    isPaywall,
    targetExists,
    screenChanged,
    newSeen,
    doneWhenMet,
    visited,
    idleCounter,
    stuckCounter,
    inAllowedDomain,
    pageOutside,
    timeUp,
    decisionBudget,
    decisionsMade,
    requestBudget,
    requestsInWindow,
    exitReason

vars == <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists, screenChanged,
          newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside, timeUp,
          decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

Modes == {"scenario", "explore"}
Phases == {"idle", "awaiting_decision", "acting", "done", "stuck", "frozen_recovered", "error_exit", "timed_out", "aborted"}
Terminal == {"done", "stuck", "error_exit", "timed_out", "aborted"}
ExitReasons == {"none", "goal_achieved", "reported_done", "stuck", "budget_exceeded", "time_exceeded", "user_aborted"}
DirectOps == {"click", "input"}
Explorable == Elements \ ForbiddenElements

ProposableOps == IF mode = "explore" THEN Operations \ {"done"} ELSE Operations

DoneWhen == IF mode = "explore" THEN visited = Explorable /\ decisionsMade > 0 ELSE doneWhenMet

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
    /\ exitReason \in ExitReasons

DoneWhenObservations == IF mode = "scenario" THEN BOOLEAN ELSE {FALSE}

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
    /\ exitReason = "none"

Exploring == phase = "idle" /\ ~DoneWhen
Continuing == Exploring /\ stuckCounter < StuckThreshold /\ decisionBudget > 0 /\ ~timeUp

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
    /\ \E op \in ProposableOps, t \in Elements, lc \in BOOLEAN, pw \in BOOLEAN, ex \in BOOLEAN :
        /\ proposedOp' = op
        /\ proposedTarget' = t
        /\ proposedLowConf' = lc
        /\ isPaywall' = pw
        /\ targetExists' = ex
    /\ UNCHANGED <<mode, stage, screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter,
                   inAllowedDomain, pageOutside, timeUp, exitReason>>

ResetRequestWindow ==
    /\ Continuing
    /\ requestBudget < MaxRequests
    /\ requestBudget' = MaxRequests
    /\ requestsInWindow' = 0
    /\ UNCHANGED <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, exitReason>>

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
    /\ UNCHANGED <<mode, stage, isPaywall, targetExists, screenChanged, newSeen, doneWhenMet, visited, idleCounter,
                   stuckCounter, inAllowedDomain, pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget,
                   requestsInWindow, exitReason>>

VisitedAfterOp == IF proposedOp \in DirectOps THEN visited \cup {proposedTarget} ELSE visited

StuckAfterOp == IF VisitedAfterOp = visited /\ ~newSeen' THEN stuckCounter + 1 ELSE 0

ObserveDoneWhen == doneWhenMet' \in DoneWhenObservations

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
                   requestsInWindow, exitReason>>

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
                   requestsInWindow, exitReason>>

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
                   timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

ReturnToAllowedDomain ==
    /\ Continuing
    /\ ~inAllowedDomain
    /\ pageOutside' = FALSE
    /\ inAllowedDomain' = TRUE
    /\ UNCHANGED <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, timeUp,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

FinishByProposalDone ==
    /\ phase = "acting"
    /\ proposedOp = "done"
    /\ phase' = "done"
    /\ exitReason' = "reported_done"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

FinishByProposalStuck ==
    /\ phase = "acting"
    /\ proposedOp = "stuck"
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

RecoverFromFreeze ==
    /\ Continuing
    /\ idleCounter >= IdleThreshold
    /\ phase' = "frozen_recovered"
    /\ idleCounter' = 0
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, stuckCounter, inAllowedDomain, pageOutside, timeUp,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

ResumeAfterRecovery ==
    /\ phase = "frozen_recovered"
    /\ phase' = "idle"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

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
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

FinishNormally ==
    /\ phase = "idle"
    /\ DoneWhen
    /\ LastStage
    /\ phase' = "done"
    /\ exitReason' = "goal_achieved"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

FinishStuck ==
    /\ Exploring
    /\ stuckCounter >= StuckThreshold
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

ExceedDecisionBudget ==
    /\ Exploring
    /\ stuckCounter < StuckThreshold
    /\ decisionBudget = 0
    /\ phase' = "error_exit"
    /\ exitReason' = "budget_exceeded"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

FinishTimeUp ==
    /\ Exploring
    /\ stuckCounter < StuckThreshold
    /\ timeUp
    /\ phase' = "timed_out"
    /\ exitReason' = "time_exceeded"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

TimeElapses ==
    /\ ~timeUp
    /\ phase \notin Terminal
    /\ timeUp' = TRUE
    /\ UNCHANGED <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

UserAbort ==
    /\ phase \notin Terminal
    /\ phase' = "aborted"
    /\ exitReason' = "user_aborted"
    /\ UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                   pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

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
    \/ ExceedDecisionBudget
    \/ FinishTimeUp
    \/ TimeElapses
    \/ UserAbort
    \/ (phase \in Terminal /\ UNCHANGED vars)

NeverOperatesForbidden ==
    visited \cap ForbiddenElements = {}

LowConfidenceBackNeverActed ==
    (phase = "acting" /\ ~isPaywall /\ proposedOp = "back") => ~proposedLowConf

DirectOpActedOnlyOnSafeTarget ==
    (phase = "acting" /\ ~isPaywall /\ proposedOp \in DirectOps)
        => (targetExists /\ proposedTarget \notin ForbiddenElements)

ExploreDone == mode = "explore" /\ visited = Explorable /\ decisionsMade > 0
ScenarioStageDone == mode = "scenario" /\ doneWhenMet
GoalReached == ExploreDone \/ (ScenarioStageDone /\ stage = NumStages)
StageCleared == ScenarioStageDone /\ stage # NumStages

ExploreGoalOnlyWhenAllExplored ==
    (mode = "explore" /\ exitReason = "goal_achieved") => (visited = Explorable /\ decisionsMade > 0)

ScenarioGoalOnlyAtLastStage ==
    (mode = "scenario" /\ exitReason = "goal_achieved") => (stage = NumStages /\ doneWhenMet)

ReportedDoneOnlyWithoutDoneWhen ==
    exitReason = "reported_done" => ~(ExploreDone \/ ScenarioStageDone)

ExploreNeverReportedDone ==
    mode = "explore" => exitReason # "reported_done"

ExitReasonMatchesPhase ==
    /\ phase = "done" => exitReason \in {"goal_achieved", "reported_done"}
    /\ phase = "stuck" => exitReason = "stuck"
    /\ phase = "error_exit" => exitReason = "budget_exceeded"
    /\ phase = "timed_out" => exitReason = "time_exceeded"
    /\ phase = "aborted" => exitReason = "user_aborted"
    /\ phase \notin Terminal => exitReason = "none"

TimeExceededOnlyWhenTimeUp ==
    exitReason = "time_exceeded" => timeUp

StagesOnlyInScenario ==
    mode = "explore" => (stage = 1 /\ ~doneWhenMet)

NewSeenOnlyOnChangedAllowedPage ==
    newSeen => (screenChanged /\ inAllowedDomain /\ decisionsMade > 0)

NoChangeObservedOnlyAfterOps ==
    ~screenChanged => decisionsMade > 0

NoProposalBeforeDecision ==
    decisionsMade = 0
        => (proposedOp = "none" /\ proposedTarget = "none" /\ ~proposedLowConf /\ ~isPaywall /\ targetExists)

RequestsWithinWindowCap ==
    requestsInWindow <= MaxRequests

DecisionsWithinCap ==
    decisionsMade <= MaxDecisions

BudgetAccounting ==
    /\ decisionBudget + decisionsMade = MaxDecisions
    /\ requestBudget + requestsInWindow = MaxRequests

BudgetExceededOnlyWhenSpent ==
    exitReason = "budget_exceeded" => decisionBudget = 0

CountersBoundedByOps ==
    idleCounter <= decisionsMade /\ stuckCounter <= decisionsMade

VisitedOnlyByOps ==
    Cardinality(visited) <= decisionsMade

RecoveryOnlyAfterFreeze ==
    phase = "frozen_recovered" => decisionsMade >= IdleThreshold

RecoveryResetsPageState ==
    phase = "frozen_recovered" => (idleCounter = 0 /\ inAllowedDomain)

DomainRecordAccurate ==
    inAllowedDomain = ~pageOutside

ProposalWellFormed ==
    phase \in {"awaiting_decision", "acting"}
        => (/\ proposedOp \in Operations \cup {"back"}
            /\ proposedTarget \in Elements
            /\ (mode = "explore" => proposedOp # "done"))

EnvEvent == (phase' = "aborted" /\ exitReason' = "user_aborted") \/ (~timeUp /\ timeUp')

GoalPreempts ==
    [][(phase = "idle" /\ GoalReached) => ((phase' = "done" /\ exitReason' = "goal_achieved") \/ EnvEvent)]_vars

StageAdvancePreempts ==
    [][(phase = "idle" /\ StageCleared) => ((phase' = "idle" /\ stage' = stage + 1) \/ EnvEvent)]_vars

StuckPreempts ==
    [][(phase = "idle" /\ ~ExploreDone /\ ~ScenarioStageDone /\ stuckCounter >= StuckThreshold) => (phase' = "stuck" \/ EnvEvent)]_vars

LimitStops ==
    [][(phase = "idle" /\ ~ExploreDone /\ ~ScenarioStageDone /\ stuckCounter < StuckThreshold /\ (decisionBudget = 0 \/ timeUp))
        => ((decisionBudget = 0 /\ phase' = "error_exit") \/ (timeUp /\ phase' = "timed_out") \/ EnvEvent)]_vars

FreezePreemptsDecision ==
    [][(phase = "idle" /\ idleCounter >= IdleThreshold) => (phase' # "awaiting_decision")]_vars

SafeProposalProceeds ==
    [][(/\ phase = "awaiting_decision"
        /\ ~isPaywall
        /\ ~(proposedOp = "back" /\ proposedLowConf)
        /\ (proposedOp \in DirectOps => (proposedTarget \notin ForbiddenElements /\ targetExists)))
        => (phase' = "acting" \/ EnvEvent)]_vars

AwaitingOnlyByDecision ==
    [][((phase # "awaiting_decision" /\ phase' = "awaiting_decision") <=> (decisionsMade' = decisionsMade + 1))]_vars

ActingOnlyAfterGuardrails ==
    [][(phase # "acting" /\ phase' = "acting") => (phase = "awaiting_decision")]_vars

FreezeRecoveryRecorded ==
    [][(phase = "idle" /\ ~ExploreDone /\ ~ScenarioStageDone /\ idleCounter >= IdleThreshold /\ idleCounter' = 0) => (phase' = "frozen_recovered")]_vars

VisitedChangesOnlyByExecution ==
    [][(visited' # visited) => (phase = "acting" /\ phase' = "idle")]_vars

PaywallForcesBack ==
    [][(phase = "awaiting_decision" /\ isPaywall) => ((phase' = "acting" /\ proposedOp' = "back") \/ EnvEvent)]_vars

ProposalOutcome ==
    [][(phase = "acting")
        => ((phase' = CASE proposedOp = "done" -> "done" [] proposedOp = "stuck" -> "stuck" [] OTHER -> "idle") \/ EnvEvent)]_vars

IdleCounterTracksScreen ==
    [][(phase = "acting" /\ phase' = "idle")
        => (idleCounter' = IF screenChanged' THEN 0 ELSE idleCounter + 1)]_vars

MemoryNeverShrinks ==
    [][visited \subseteq visited']_vars

StuckCounterTracksNewElements ==
    [][(phase = "acting" /\ phase' = "idle")
        => (stuckCounter' = IF visited' = visited /\ ~newSeen' THEN stuckCounter + 1 ELSE 0)]_vars

NoDecisionOutsideDomain ==
    [][(phase = "idle" /\ phase' = "awaiting_decision") => (~pageOutside)]_vars

StageAdvanceResetsCounters ==
    [][(stage' # stage)
        => (/\ phase = "idle" /\ phase' = "idle" /\ StageCleared /\ stage' = stage + 1
            /\ idleCounter' = 0 /\ stuckCounter' = 0)]_vars

BudgetSharedAcrossStages ==
    [][(stage' # stage) => UNCHANGED <<decisionBudget, decisionsMade, requestBudget, requestsInWindow>>]_vars

DecisionBudgetNeverRestored ==
    [][decisionBudget' <= decisionBudget]_vars

TimeNeverRewinds ==
    [][timeUp => timeUp']_vars

TimeElapsesAlone ==
    [][(~timeUp /\ timeUp') => UNCHANGED <<mode, stage, phase, proposedOp, proposedTarget, proposedLowConf, isPaywall,
                                           targetExists, screenChanged, newSeen, doneWhenMet, visited, idleCounter,
                                           stuckCounter, inAllowedDomain, pageOutside, decisionBudget, decisionsMade,
                                           requestBudget, requestsInWindow, exitReason>>]_vars

ModeFixed ==
    [][mode' = mode]_vars

ProposalChangesOnlyByDecision ==
    [][(<<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists>>'
          # <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists>>)
        => (\/ (phase # "awaiting_decision" /\ phase' = "awaiting_decision")
            \/ (/\ phase = "awaiting_decision" /\ isPaywall /\ proposedOp' = "back"
                /\ UNCHANGED <<proposedTarget, proposedLowConf, isPaywall, targetExists>>))]_vars

CountersChangeOnlyBySteps ==
    [][/\ (idleCounter' # idleCounter)
            => ((phase = "acting" /\ phase' = "idle") \/ phase' = "frozen_recovered" \/ stage' = stage + 1)
       /\ (stuckCounter' # stuckCounter)
            => ((phase = "acting" /\ phase' = "idle") \/ stage' = stage + 1)]_vars

DomainChangesOnlyByNavigation ==
    [][(inAllowedDomain' # inAllowedDomain \/ pageOutside' # pageOutside)
        => ((phase = "acting" /\ phase' = "idle") \/ (phase = "idle" /\ phase' = "idle" /\ inAllowedDomain'))]_vars

ObservationChangesOnlyByExecution ==
    [][(screenChanged' # screenChanged \/ newSeen' # newSeen) => (phase = "acting" /\ phase' = "idle")]_vars

DoneWhenChangesOnlyByExecutionOrAdvance ==
    [][/\ (~doneWhenMet /\ doneWhenMet') => ((phase = "acting" /\ phase' = "idle") \/ stage' = stage + 1)
       /\ (doneWhenMet /\ ~doneWhenMet') => (stage' = stage + 1)]_vars

TerminationKeepsState ==
    [][(phase \notin Terminal /\ phase' \in Terminal)
        => UNCHANGED <<mode, stage, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                       screenChanged, newSeen, doneWhenMet, visited, idleCounter, stuckCounter, inAllowedDomain,
                       pageOutside, timeUp, decisionBudget, decisionsMade, requestBudget, requestsInWindow>>]_vars

StartsFresh ==
    /\ phase = "idle" /\ stage = 1 /\ visited = {} /\ ~timeUp /\ exitReason = "none"
    /\ idleCounter = 0 /\ stuckCounter = 0 /\ inAllowedDomain /\ (mode = "explore" => ~doneWhenMet)
    /\ decisionBudget = MaxDecisions /\ requestBudget = MaxRequests

EventuallyTerminates ==
    <>(phase \in Terminal)

TerminalAbsorbing ==
    [][(phase \in Terminal) => (UNCHANGED vars)]_vars

Spec == Init /\ [][Next]_vars
             /\ WF_vars(RequestDecision) /\ WF_vars(ResetRequestWindow) /\ WF_vars(ApplyGuardrails)
             /\ WF_vars(ExecuteNoChange \/ ExecuteChange \/ ExecuteOutOfDomain)
             /\ WF_vars(ReturnToAllowedDomain)
             /\ WF_vars(FinishByProposalDone) /\ WF_vars(FinishByProposalStuck)
             /\ WF_vars(RecoverFromFreeze) /\ WF_vars(ResumeAfterRecovery)
             /\ WF_vars(AdvanceStage) /\ WF_vars(FinishNormally) /\ WF_vars(FinishStuck)
             /\ WF_vars(ExceedDecisionBudget) /\ WF_vars(FinishTimeUp)
