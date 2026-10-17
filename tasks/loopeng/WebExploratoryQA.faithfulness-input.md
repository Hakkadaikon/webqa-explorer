EXTENDS Naturals, FiniteSets

CONSTANTS
    MaxDecisions,
    MaxRequests,
    IdleThreshold,
    StuckThreshold,
    Elements,
    ForbiddenElements,
    Operations

ASSUME ForbiddenElements \subseteq Elements
ASSUME Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"}

ASSUME MaxDecisions \in Nat \ {0}
ASSUME MaxRequests \in Nat \ {0}
ASSUME IdleThreshold \in Nat \ {0}
ASSUME StuckThreshold \in Nat \ {0}

VARIABLES
    phase,
    proposedOp,
    proposedTarget,
    proposedLowConf,
    isPaywall,
    targetExists,
    screenChanged,
    visited,
    idleCounter,
    stuckCounter,
    inAllowedDomain,
    pageOutside,
    decisionBudget,
    decisionsMade,
    requestBudget,
    requestsInWindow,
    exitReason

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

Exploring == phase = "idle" /\ visited # Explorable
Continuing == Exploring /\ stuckCounter < StuckThreshold /\ decisionBudget > 0

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

ResetRequestWindow ==
    /\ Continuing
    /\ requestBudget < MaxRequests
    /\ requestBudget' = MaxRequests
    /\ requestsInWindow' = 0
    /\ UNCHANGED <<phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, exitReason>>

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

VisitedAfterOp == IF proposedOp \in DirectOps THEN visited \cup {proposedTarget} ELSE visited

StuckAfterOp == IF VisitedAfterOp = visited THEN stuckCounter + 1 ELSE 0

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

ReturnToAllowedDomain ==
    /\ Continuing
    /\ ~inAllowedDomain
    /\ pageOutside' = FALSE
    /\ inAllowedDomain' = TRUE
    /\ UNCHANGED <<phase, proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow, exitReason>>

FinishByProposalDone ==
    /\ phase = "acting"
    /\ proposedOp = "done"
    /\ phase' = "done"
    /\ exitReason' = "reported_done"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

FinishByProposalStuck ==
    /\ phase = "acting"
    /\ proposedOp = "stuck"
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

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

FinishNormally ==
    /\ phase = "idle"
    /\ visited = Explorable
    /\ phase' = "done"
    /\ exitReason' = "goal_achieved"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

FinishStuck ==
    /\ Exploring
    /\ stuckCounter >= StuckThreshold
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter, inAllowedDomain, pageOutside,
                   decisionBudget, decisionsMade, requestBudget, requestsInWindow>>

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
    \/ (phase \in Terminal /\ UNCHANGED vars)

NeverOperatesForbidden ==
    visited \cap ForbiddenElements = {}

LowConfidenceBackNeverActed ==
    (phase = "acting" /\ ~isPaywall /\ proposedOp = "back") => ~proposedLowConf

DirectOpActedOnlyOnSafeTarget ==
    (phase = "acting" /\ ~isPaywall /\ proposedOp \in DirectOps)
        => (targetExists /\ proposedTarget \notin ForbiddenElements)

GoalOnlyByDoneWhen ==
    exitReason = "goal_achieved" => visited = Explorable

ReportedDoneOnlyWithoutDoneWhen ==
    exitReason = "reported_done" => visited # Explorable

ExitReasonMatchesPhase ==
    /\ phase = "done" => exitReason \in {"goal_achieved", "reported_done"}
    /\ phase = "stuck" => exitReason = "stuck"
    /\ phase = "error_exit" => exitReason = "budget_exceeded"
    /\ phase \notin Terminal => exitReason = "none"

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
    phase \in {"awaiting_decision", "acting"} => (proposedOp \in Operations \cup {"back"} /\ proposedTarget \in Elements)

GoalPreempts ==
    [][(phase = "idle" /\ visited = Explorable) => (phase' = "done" /\ exitReason' = "goal_achieved")]_vars

StuckPreempts ==
    [][(phase = "idle" /\ visited # Explorable /\ stuckCounter >= StuckThreshold) => (phase' = "stuck")]_vars

BudgetStops ==
    [][(phase = "idle" /\ visited # Explorable /\ stuckCounter < StuckThreshold /\ decisionBudget = 0)
        => (phase' = "error_exit")]_vars

FreezePreemptsDecision ==
    [][(phase = "idle" /\ idleCounter >= IdleThreshold) => (phase' # "awaiting_decision")]_vars

SafeProposalProceeds ==
    [][(/\ phase = "awaiting_decision"
        /\ ~isPaywall
        /\ ~(proposedOp = "back" /\ proposedLowConf)
        /\ (proposedOp \in DirectOps => (proposedTarget \notin ForbiddenElements /\ targetExists)))
        => (phase' = "acting")]_vars

AwaitingOnlyByDecision ==
    [][((phase # "awaiting_decision" /\ phase' = "awaiting_decision") <=> (decisionsMade' = decisionsMade + 1))]_vars

ActingOnlyAfterGuardrails ==
    [][(phase # "acting" /\ phase' = "acting") => (phase = "awaiting_decision")]_vars

FreezeRecoveryRecorded ==
    [][(phase = "idle" /\ idleCounter >= IdleThreshold /\ idleCounter' = 0) => (phase' = "frozen_recovered")]_vars

VisitedChangesOnlyByExecution ==
    [][(visited' # visited) => (phase = "acting" /\ phase' = "idle")]_vars

PaywallForcesBack ==
    [][(phase = "awaiting_decision" /\ isPaywall) => (phase' = "acting" /\ proposedOp' = "back")]_vars

ProposalOutcome ==
    [][(phase = "acting")
        => (phase' = CASE proposedOp = "done" -> "done" [] proposedOp = "stuck" -> "stuck" [] OTHER -> "idle")]_vars

IdleCounterTracksScreen ==
    [][(phase = "acting" /\ phase' = "idle")
        => (idleCounter' = IF screenChanged' THEN 0 ELSE idleCounter + 1)]_vars

MemoryNeverShrinks ==
    [][visited \subseteq visited']_vars

StuckCounterTracksNewElements ==
    [][(phase = "acting" /\ phase' = "idle")
        => (stuckCounter' = IF visited' = visited THEN stuckCounter + 1 ELSE 0)]_vars

NoDecisionOutsideDomain ==
    [][(phase = "idle" /\ phase' = "awaiting_decision") => (~pageOutside)]_vars

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
             /\ WF_vars(FinishNormally) /\ WF_vars(FinishStuck) /\ WF_vars(ExceedDecisionBudget)
