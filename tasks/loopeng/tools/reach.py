#!/usr/bin/env python3
"""Append antecedent-reachability evidence to WebExploratoryQA.reach.log.

Usage: reach.py <version label> <section heading>

For each antecedent P: define NotP == ~P (state predicate -> INVARIANT) or
NotP == [][~(P)]_vars (action predicate -> PROPERTY) in MCWebExploratoryQA.tla
(EXTENDS WebExploratoryQA), copy the cfg keeping CONSTANTS/SPECIFICATION and
only that one check, and run TLC. A check runs on WebExploratoryQA.cfg unless it
names another cfg. A violation (= a reachable state/step where P
holds) is the expected outcome. The MC files are removed afterwards.
"""
import datetime, os, re, subprocess, sys

D = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LOG = os.path.join(D, "WebExploratoryQA.reach.log")

VER, HEADING = sys.argv[1:3]
EMPTY = "WebExploratoryQA.noexplorable.cfg"
# (id, property whose antecedent is checked, kind S=state / A=action, predicate[, cfg])
CHECKS = [
    ("P1", "LowConfidenceBackNeverActed", "S", 'phase = "acting" /\\ ~isPaywall /\\ proposedOp = "back"'),
    ("P2", "DirectOpActedOnlyOnSafeTarget", "S", 'phase = "acting" /\\ ~isPaywall /\\ proposedOp \\in DirectOps'),
    ("P3", "ExploreGoalOnlyWhenAllExplored", "S", 'mode = "explore" /\\ exitReason = "goal_achieved"'),
    ("P4", "ScenarioGoalOnlyAtLastStage", "S", 'mode = "scenario" /\\ exitReason = "goal_achieved"'),
    ("P5", "ReportedDoneOnlyWithoutDoneWhen", "S", 'exitReason = "reported_done"'),
    ("P6", "ExploreNeverReportedDone / StagesOnlyInScenario (explore の終了状態)", "S", 'mode = "explore" /\\ phase \\in Terminal'),
    ("P7a", "ExitReasonMatchesPhase (done)", "S", 'phase = "done"'),
    ("P7b", "ExitReasonMatchesPhase (stuck)", "S", 'phase = "stuck"'),
    ("P7c", "ExitReasonMatchesPhase (error_exit)", "S", 'phase = "error_exit"'),
    ("P7d", "ExitReasonMatchesPhase (timed_out)", "S", 'phase = "timed_out"'),
    ("P7e", "ExitReasonMatchesPhase (aborted)", "S", 'phase = "aborted"'),
    ("P7f", "ExitReasonMatchesPhase (non-terminal, 操作の後)", "S", 'phase \\notin Terminal /\\ decisionsMade > 0'),
    ("P8", "TimeExceededOnlyWhenTimeUp", "S", 'exitReason = "time_exceeded"'),
    ("P9", "TimeExceededOnlyWhenTimeUp (決め打ちシナリオでの時間切れ)", "S", 'mode = "scenario" /\\ exitReason = "time_exceeded"'),
    ("P10", "NewSeenOnlyOnChangedAllowedPage", "S", "newSeen"),
    ("P11", "NoChangeObservedOnlyAfterOps", "S", "~screenChanged"),
    ("P12", "NoProposalBeforeDecision (問い合わせ前の idle 以外の状態)", "S", 'decisionsMade = 0 /\\ phase # "idle"'),
    ("P13", "BudgetExceededOnlyWhenSpent", "S", 'exitReason = "budget_exceeded"'),
    ("P14", "RecoveryResetsPageState / RecoveryOnlyAfterFreeze", "S", 'phase = "frozen_recovered"'),
    ("P15", "ProposalWellFormed (探索モードの提案)", "S", 'mode = "explore" /\\ phase \\in {"awaiting_decision", "acting"}'),
    ("P16a", "GoalPreempts (探索モード)", "S", 'phase = "idle" /\\ GoalReached /\\ mode = "explore"'),
    ("P16b", "GoalPreempts (決め打ちシナリオ、最後の段階)", "S", 'phase = "idle" /\\ GoalReached /\\ mode = "scenario"'),
    ("P17", "StageAdvancePreempts", "S", 'phase = "idle" /\\ StageCleared'),
    ("P18", "StuckPreempts", "S", 'phase = "idle" /\\ ~ExploreDone /\\ ~ScenarioStageDone /\\ stuckCounter >= StuckThreshold'),
    ("P19a", "LimitStops (判断予算切れだけ)", "S", 'phase = "idle" /\\ ~ExploreDone /\\ ~ScenarioStageDone /\\ stuckCounter < StuckThreshold /\\ decisionBudget = 0 /\\ ~timeUp'),
    ("P19b", "LimitStops (時間切れだけ)", "S", 'phase = "idle" /\\ ~ExploreDone /\\ ~ScenarioStageDone /\\ stuckCounter < StuckThreshold /\\ decisionBudget > 0 /\\ timeUp'),
    ("P19c", "LimitStops (両方)", "S", 'phase = "idle" /\\ ~ExploreDone /\\ ~ScenarioStageDone /\\ stuckCounter < StuckThreshold /\\ decisionBudget = 0 /\\ timeUp'),
    ("P20", "FreezePreemptsDecision", "S", 'phase = "idle" /\\ idleCounter >= IdleThreshold'),
    ("P21", "SafeProposalProceeds", "S", 'phase = "awaiting_decision" /\\ ~isPaywall /\\ ~(proposedOp = "back" /\\ proposedLowConf) /\\ (proposedOp \\in DirectOps => (proposedTarget \\notin ForbiddenElements /\\ targetExists))'),
    ("P22", "PaywallForcesBack", "S", 'phase = "awaiting_decision" /\\ isPaywall'),
    ("P23", "AwaitingOnlyByDecision", "A", 'phase # "awaiting_decision" /\\ phase\' = "awaiting_decision"'),
    ("P24", "ActingOnlyAfterGuardrails", "A", 'phase # "acting" /\\ phase\' = "acting"'),
    ("P25", "FreezeRecoveryRecorded", "A", 'phase = "idle" /\\ ~ExploreDone /\\ ~ScenarioStageDone /\\ idleCounter >= IdleThreshold /\\ idleCounter\' = 0'),
    ("P26", "VisitedChangesOnlyByExecution", "A", "visited' # visited"),
    ("P27a", "ProposalOutcome (done、決め打ちシナリオ)", "S", 'phase = "acting" /\\ proposedOp = "done"'),
    ("P27b", "ProposalOutcome (stuck)", "S", 'phase = "acting" /\\ proposedOp = "stuck"'),
    ("P27c", "ProposalOutcome (other)", "S", 'phase = "acting" /\\ proposedOp \\notin {"done", "stuck"}'),
    ("P28a", "IdleCounterTracksScreen / StuckCounterTracksNewElements (実行の一歩)", "A", 'phase = "acting" /\\ phase\' = "idle"'),
    ("P28b", "StuckCounterTracksNewElements (探索済みが増えず、新しい要素も見えない)", "A", 'phase = "acting" /\\ phase\' = "idle" /\\ visited\' = visited /\\ ~newSeen\' /\\ stuckCounter > 0'),
    ("P28c", "StuckCounterTracksNewElements (探索済みは増えないが、新しい要素が見えた: スクロールの数え直し)", "A", 'phase = "acting" /\\ phase\' = "idle" /\\ visited\' = visited /\\ newSeen\' /\\ stuckCounter > 0'),
    ("P28d", "StuckCounterTracksNewElements (探索済みが増えた)", "A", 'phase = "acting" /\\ phase\' = "idle" /\\ visited\' # visited /\\ stuckCounter > 0'),
    ("P28e", "IdleCounterTracksScreen (画面が変化しない)", "A", 'phase = "acting" /\\ phase\' = "idle" /\\ ~screenChanged\' /\\ idleCounter > 0'),
    ("P29", "NoDecisionOutsideDomain", "A", 'phase = "idle" /\\ phase\' = "awaiting_decision"'),
    ("P30", "StageAdvanceResetsCounters (カウンタが0でないときの切り替え)", "A", "stage' # stage /\\ (idleCounter > 0 \\/ stuckCounter > 0)"),
    ("P31", "BudgetSharedAcrossStages (予算を使った後の切り替え)", "A", "stage' # stage /\\ decisionBudget < MaxDecisions /\\ requestBudget < MaxRequests"),
    ("P32", "TimeNeverRewinds", "S", "timeUp"),
    ("P33", "TimeElapsesAlone", "A", "~timeUp /\\ timeUp'"),
    ("P34", "ProposalChangesOnlyByDecision", "A", "<<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists>>' # <<proposedOp, proposedTarget, proposedLowConf, isPaywall, targetExists>>"),
    ("P35a", "CountersChangeOnlyBySteps (固まりカウンタ)", "A", "idleCounter' # idleCounter"),
    ("P35b", "CountersChangeOnlyBySteps (行き詰まりカウンタ)", "A", "stuckCounter' # stuckCounter"),
    ("P36", "DomainChangesOnlyByNavigation", "A", "inAllowedDomain' # inAllowedDomain"),
    ("P37", "ObservationChangesOnlyByExecution", "A", "screenChanged' # screenChanged \\/ newSeen' # newSeen"),
    ("P38a", "DoneWhenChangesOnlyByExecutionOrAdvance (成り立つ)", "A", "~doneWhenMet /\\ doneWhenMet'"),
    ("P38b", "DoneWhenChangesOnlyByExecutionOrAdvance (取り消される)", "A", "doneWhenMet /\\ ~doneWhenMet'"),
    ("P39a", "TerminationKeepsState (終了する一歩)", "A", "phase \\notin Terminal /\\ phase' \\in Terminal"),
    ("P39b", "TerminationKeepsState (実行段階の途中での中断)", "A", 'phase = "acting" /\\ phase\' = "aborted"'),
    ("P40", "TerminalAbsorbing (中断後の状態)", "S", 'phase = "aborted" /\\ visited # {}'),
    ("P41", "StartsFresh / StageAdvancePreempts (開始時に最初の段階の done_when を満たしている)", "S", 'decisionsMade = 0 /\\ ScenarioStageDone'),
    ("P42", "DoneWhenChangesOnlyByExecutionOrAdvance / StageAdvanceResetsCounters (進んだ直後に次の段階の done_when を満たしている)", "A", "stage' # stage /\\ doneWhenMet'"),
    ("P43", "ScenarioGoalOnlyAtLastStage (操作せずにゴール達成)", "A", 'decisionsMade = 0 /\\ exitReason\' = "goal_achieved"'),
    ("P44", "ExploreGoalOnlyWhenAllExplored / GoalPreempts (探索してよい要素が空で、問い合わせる前の idle)", "S", 'mode = "explore" /\\ phase = "idle" /\\ decisionsMade = 0 /\\ visited = Explorable', EMPTY),
    ("P45", "ExploreGoalOnlyWhenAllExplored (探索してよい要素が空のときのゴール達成)", "S", 'mode = "explore" /\\ exitReason = "goal_achieved"', EMPTY),
]

env = dict(os.environ, _JAVA_OPTIONS="-Djava.io.tmpdir=/tmp/tla")
with open(LOG, "a") as log:
    log.write(f"\n# {HEADING}\n")
    for pid, prop, kind, pred, *c in CHECKS:
        cfgname = c[0] if c else "WebExploratoryQA.cfg"
        head = open(os.path.join(D, cfgname)).read().split("SPECIFICATION")[0] + "SPECIFICATION Spec\n"
        name = "Not" + pid
        defn = f"{name} == ~({pred})" if kind == "S" else f"{name} == [][~({pred})]_vars"
        open(os.path.join(D, "MCWebExploratoryQA.tla"), "w").write(
            f"---- MODULE MCWebExploratoryQA ----\nEXTENDS WebExploratoryQA\n{defn}\n====\n")
        open(os.path.join(D, "MCWebExploratoryQA.cfg"), "w").write(
            head + ("INVARIANT " if kind == "S" else "PROPERTY ") + name + "\n")
        cmd = ["tlc", "-workers", "auto", "-deadlock", "-config", "MCWebExploratoryQA.cfg", "MCWebExploratoryQA.tla"]
        p = subprocess.run(cmd, cwd=D, capture_output=True, text=True, env=env)
        out = p.stdout + p.stderr
        out = out.replace(os.path.dirname(os.path.dirname(D)) + os.sep, "")  # repo-relative paths only
        out = "\n".join(l for l in out.splitlines() if not re.match(r"(Parsing file|Semantic processing|Linting of module)", l))
        hit = re.search(r"(Invariant|Action property) Not\w+ is violated", out)
        log.write(f"=== {VER} {pid}: {prop} の前件の到達性 ===\n")
        log.write(f"実行日時: {datetime.datetime.now().astimezone().isoformat(timespec='seconds')}\n")
        log.write(f"MCWebExploratoryQA.tla: EXTENDS WebExploratoryQA / 定数: {cfgname}\n")
        log.write(f"定義: {defn}\n{'INVARIANT' if kind == 'S' else 'PROPERTY'}: {name}\n$ {' '.join(cmd)}\n")
        log.write(out + "\n")
        log.write(f"判定: {'到達する(前件が成り立つ状態/一歩がある)' if hit else '到達しない(vacuous の疑い)'}\n\n")
        print(pid, "REACHED" if hit else "NOT REACHED")
for f in ["MCWebExploratoryQA.tla", "MCWebExploratoryQA.cfg"]:
    os.unlink(os.path.join(D, f))
subprocess.run("rm -rf states MCWebExploratoryQA_TTrace_*", shell=True, cwd=D)
