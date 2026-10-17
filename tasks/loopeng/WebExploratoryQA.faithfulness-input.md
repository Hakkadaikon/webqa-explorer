CONSTANTS
    MaxDecisions, MaxRequests, IdleThreshold, StuckThreshold,
    Elements, ForbiddenElements, Operations

ASSUME ForbiddenElements \subseteq Elements
ASSUME Operations \subseteq {"click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"}

VARIABLES
    phase, proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
    screenChanged, visited, idleCounter, stuckCounter,
    inAllowedDomain, decisionBudget, requestBudget, exitReason

vars == <<phase, proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
          screenChanged, visited, idleCounter, stuckCounter,
          inAllowedDomain, decisionBudget, requestBudget, exitReason>>

Phases == {"idle", "awaiting_decision", "acting",
           "done", "stuck", "frozen_recovered", "error_exit"}
ExitReasons == {"none", "goal_achieved", "stuck", "budget_exceeded"}

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

ApplyGuardrails ==
    /\ phase = "awaiting_decision"
    /\ \/ /\ isPaywall
          /\ phase' = "acting"
          /\ proposedOp' = "back"
          /\ proposedLowConfBack' = FALSE
          /\ UNCHANGED <<proposedTarget>>
       \/ /\ ~isPaywall
          /\ \/ /\ proposedOp = "back" /\ proposedLowConfBack
                /\ phase' = "idle"
                /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack>>
             \/ /\ proposedOp \in {"click", "input"}
                /\ proposedTarget \in ForbiddenElements
                /\ phase' = "idle"
                /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack>>
             \/ /\ proposedOp \in {"click", "input"}
                /\ ~targetExists
                /\ phase' = "idle"
                /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack>>
             \/ /\ ~(proposedOp = "back" /\ proposedLowConfBack)
                /\ (proposedOp \notin {"click", "input"}
                    \/ (proposedTarget \notin ForbiddenElements /\ targetExists))
                /\ phase' = "acting"
                /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack>>
    /\ UNCHANGED <<isPaywall, targetExists, screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget, exitReason>>

ExecuteOperation ==
    /\ phase = "acting"
    /\ visited' = IF proposedOp \in {"click", "input"} /\ proposedTarget \in Elements
                  THEN visited \cup {proposedTarget}
                  ELSE visited
    /\ UNCHANGED <<phase, proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, idleCounter, stuckCounter, inAllowedDomain,
                   decisionBudget, requestBudget, exitReason>>
    /\ phase' = "acting"

ObserveNoChange ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ phase' = "idle"
    /\ screenChanged' = FALSE
    /\ idleCounter' = idleCounter + 1
    /\ stuckCounter' = stuckCounter + 1
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   visited, inAllowedDomain, decisionBudget, requestBudget, exitReason>>

ObserveChange ==
    /\ phase = "acting"
    /\ proposedOp \notin {"done", "stuck"}
    /\ phase' = "idle"
    /\ screenChanged' = TRUE
    /\ idleCounter' = 0
    /\ stuckCounter' = 0
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   visited, inAllowedDomain, decisionBudget, requestBudget, exitReason>>

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

FinishByProposalDone ==
    /\ phase = "acting"
    /\ proposedOp = "done"
    /\ phase' = "done"
    /\ exitReason' = "goal_achieved"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

FinishByProposalStuck ==
    /\ phase = "acting"
    /\ proposedOp = "stuck"
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

ReturnToAllowedDomain ==
    /\ phase = "idle"
    /\ ~inAllowedDomain
    /\ phase' = "idle"
    /\ inAllowedDomain' = TRUE
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   decisionBudget, requestBudget, exitReason>>

RecoverFromFreeze ==
    /\ phase = "idle"
    /\ idleCounter >= IdleThreshold
    /\ phase' = "frozen_recovered"
    /\ idleCounter' = 0
    /\ visited' = {}
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, stuckCounter, inAllowedDomain, decisionBudget, requestBudget, exitReason>>

ResumeAfterRecovery ==
    /\ phase = "frozen_recovered"
    /\ phase' = "idle"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget, exitReason>>

FinishNormally ==
    /\ phase = "idle"
    /\ visited = Elements \ ForbiddenElements
    /\ phase' = "done"
    /\ exitReason' = "goal_achieved"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

FinishStuck ==
    /\ phase = "idle"
    /\ stuckCounter >= StuckThreshold
    /\ phase' = "stuck"
    /\ exitReason' = "stuck"
    /\ UNCHANGED <<proposedOp, proposedTarget, proposedLowConfBack, isPaywall, targetExists,
                   screenChanged, visited, idleCounter, stuckCounter,
                   inAllowedDomain, decisionBudget, requestBudget>>

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
    \/ (phase \in {"done", "stuck", "error_exit"} /\ UNCHANGED vars)

NeverOperatesForbidden ==
    visited \cap ForbiddenElements = {}

LowConfidenceBackNeverActed ==
    (proposedOp = "back" /\ proposedLowConfBack) => phase /= "acting"

BudgetRespected ==
    decisionBudget >= 0

ExitReasonSetOnTermination ==
    phase \in {"done", "stuck", "error_exit"} => exitReason /= "none"

RequestBudgetRespected ==
    requestBudget >= 0

Inv ==
    /\ TypeOK
    /\ NeverOperatesForbidden
    /\ LowConfidenceBackNeverActed
    /\ BudgetRespected
    /\ RequestBudgetRespected
    /\ ExitReasonSetOnTermination

EventuallyTerminates ==
    []<>(phase \in {"done", "stuck", "error_exit"})

Spec == Init /\ [][Next]_vars
             /\ WF_vars(RequestDecision) /\ WF_vars(ApplyGuardrails) /\ WF_vars(ExecuteOperation)
             /\ WF_vars(ObserveNoChange) /\ WF_vars(ObserveChange)
             /\ WF_vars(ObserveOutOfDomain) /\ WF_vars(ReturnToAllowedDomain)
             /\ WF_vars(FinishByProposalDone) /\ WF_vars(FinishByProposalStuck)
             /\ WF_vars(FinishNormally) /\ WF_vars(FinishStuck) /\ WF_vars(ExceedDecisionBudget)
             /\ WF_vars(RecoverFromFreeze) /\ WF_vars(ResumeAfterRecovery)
