# WebExploratoryQA 形式文面の平文への言い直し

入力は `WebExploratoryQA.faithfulness-input.md` だけ。説明文や要件の文書は読んでいない。

## 1. 定数と仮定

| 名前 | 言い直し |
|---|---|
| ForbiddenElements ⊆ Elements | 禁止要素は画面要素全体の一部である。 |
| Operations | 操作の候補は click、input、scroll_down、scroll_up、back、wait、done、stuck の中から選ぶ。どれを含めるかは設定で決める。 |
| MaxDecisions、MaxRequests、IdleThreshold、StuckThreshold、NumStages | どれも 1 以上の自然数。判断回数の上限、時間窓あたりの要求回数の上限、無反応の連続回数のしきい値、新規なしの連続回数のしきい値、シナリオの段数を表す。 |

## 2. 補助の定義

| 名前 | 言い直し |
|---|---|
| vars | 22 個の変数をすべて並べた組。 |
| Modes | 動作モードは scenario(シナリオ実行)と explore(探索)の 2 種類。 |
| Phases | 進行段階は idle、awaiting_decision、acting、done、stuck、frozen_recovered、error_exit、timed_out、aborted の 9 種類。 |
| Terminal | 終了段階は done、stuck、error_exit、timed_out、aborted の 5 つ。frozen_recovered は終了に含まない。 |
| ExitReasons | 終了理由は none、goal_achieved、reported_done、stuck、budget_exceeded、time_exceeded、user_aborted のいずれか。 |
| DirectOps | 要素を直接触る操作は click と input の 2 つ。 |
| Explorable | 探索してよい要素は、全要素から禁止要素を除いたもの。 |
| ProposableOps | 提案できる操作。探索モードでは done を除き、シナリオモードでは Operations 全部。 |
| DoneWhen | 完了条件。探索モードでは「探索してよい要素をすべて訪問済み」、シナリオモードでは「現在の段の完了フラグ doneWhenMet が真」。 |
| LastStage | 最終段かどうか。探索モードでは常に真、シナリオモードでは stage = NumStages のとき真。 |
| Exploring | 進行段階が idle で、完了条件がまだ成り立っていない。 |
| Continuing | Exploring に加えて、新規なしの回数がしきい値未満、判断予算が残っている、時間切れでない、の 3 つが成り立つ。続行してよい状態を表す。 |
| VisitedAfterOp | 実行後の訪問済み集合。提案操作が click か input なら対象要素を加え、それ以外なら変えない。 |
| StuckAfterOp | 実行後の新規なしカウンタ。訪問済み集合が増えず、かつ実行後の newSeen が偽なら 1 増やし、そうでなければ 0 に戻す。 |
| ObserveDoneWhen | 実行後の完了フラグ。シナリオモードでは真偽どちらにもなり得る(環境が決める)。探索モードでは常に偽にする。 |
| ExploreDone | 探索モードで、探索してよい要素をすべて訪問済み。 |
| ScenarioStageDone | シナリオモードで、現在の段の完了フラグが真。 |
| GoalReached | 探索が完了したか、シナリオの最終段が完了した。 |
| StageCleared | シナリオの最終段でない段が完了した。 |
| EnvEvent | 環境側の出来事。利用者による中断(次の段階が aborted で理由が user_aborted)か、時間切れが今の一歩で起きた(timeUp が偽から真)かのどちらか。 |

## 3. 初期状態

### TypeOK

各変数の型を述べる。proposedOp は Operations に back と none を加えた範囲を取る。Operations に back が含まれていなくても back を取れる。proposedTarget は要素か none。visited は要素の部分集合。idleCounter、stuckCounter、decisionsMade、requestsInWindow は自然数。decisionBudget は 0 から MaxDecisions、requestBudget は 0 から MaxRequests の範囲に収まる。

### Init

モードは 2 種類のどちらかで始まる。残りはすべて固定値から始まる。段は 1、段階は idle。提案は空(操作も対象も none)で、低信頼は偽、ペイウォールは偽、対象は存在する扱い。画面は変化あり扱い(screenChanged = TRUE)、新規発見なし、完了フラグは偽。訪問済みは空、両カウンタは 0。許可ドメイン内にいて、外部ページにはいない。時間切れでない。判断予算と要求予算は満額で、使用回数は 0、終了理由は none。

### StartsFresh

段階 idle、段 1、訪問済み空、時間切れでない、終了理由 none、両カウンタ 0、許可ドメイン内、完了フラグ偽、両予算満額、という初期の様子を述べる状態述語。モード、提案、decisionsMade、requestsInWindow、pageOutside、screenChanged には触れていない。

## 4. Next の各アクション

### RequestDecision(判断を求める)

- 起きられる条件: Continuing が成り立つ。加えて、無反応カウンタがしきい値未満、要求予算が 1 以上、許可ドメイン内にいる。
- 変わるもの: 段階が awaiting_decision になる。判断予算と要求予算を 1 ずつ減らし、判断回数と窓内要求数を 1 ずつ増やす。提案操作(ProposableOps から)、対象(全要素から、禁止要素も含む)、低信頼、ペイウォール、対象の存在を、それぞれ任意に選び直す。
- 変わらないもの: モード、段、画面観測、完了フラグ、訪問済み、両カウンタ、ドメイン、時間、終了理由。
- 補足: ペイウォールの有無と対象の存在は、ページの持続的な状態ではなく、判断のたびに独立に決まる。

### ResetRequestWindow(要求の時間窓を戻す)

- 起きられる条件: Continuing が成り立ち、要求予算が満額でない。
- 変わるもの: 要求予算を満額に、窓内要求数を 0 に戻す。
- 変わらないもの: それ以外すべて。段階も idle のまま。
- 補足: 時間の経過を表す条件は無い。続行中ならいつでも窓を戻せる。

### ApplyGuardrails(安全柵を当てる)

- 起きられる条件: 段階が awaiting_decision。
- ペイウォールが真のとき: 提案の内容に関係なく、操作を back に差し替えて acting へ進む。対象と低信頼は変えない。提案が done や stuck でも back に置き換わる。
- ペイウォールが偽のとき、次のどれかになる。提案の 3 項目(操作、対象、低信頼)は変えない。
  - 操作が back で低信頼なら却下し、idle に戻る。
  - 操作が click か input で、対象が禁止要素なら却下し、idle に戻る。
  - 操作が click か input で、対象が存在しないなら却下し、idle に戻る。
  - 上の 3 つのどれにも当たらなければ acting へ進む。
- 却下の場合も実行の場合も、モード、段、ペイウォール、対象の存在、画面観測、完了フラグ、訪問済み、両カウンタ、ドメイン、時間、予算、終了理由は変えない。
- 補足: 却下された判断も判断予算を消費済みである。一方で、却下は無反応カウンタも新規なしカウンタも動かさない。却下と実行の場合分けは漏れなく分かれている。低信頼が効くのは back だけで、低信頼の done や stuck はそのまま通る。

### ExecuteNoChange(実行したが画面が変わらない)

- 起きられる条件: 段階が acting で、提案操作が done でも stuck でもない。
- 変わるもの: 訪問済みを VisitedAfterOp にする。段階を idle に戻す。画面変化なし、新規発見なしにする。完了フラグを ObserveDoneWhen に従って更新する。無反応カウンタを 1 増やす。新規なしカウンタは StuckAfterOp に従う。この場合 newSeen が偽なので、訪問済みが増えなければ 1 増え、増えれば 0 に戻る。
- 変わらないもの: モード、段、提案の 5 項目、ドメイン、時間、予算、終了理由。
- 補足: 画面が変わらなくても、click や input の対象は訪問済みに加わる。

### ExecuteChange(実行して画面が変わる)

- 起きられる条件: ExecuteNoChange と同じ。
- 変わるもの: 訪問済みを更新し、段階を idle に戻す。画面変化ありにし、新規発見は真偽どちらにもなり得る。完了フラグを ObserveDoneWhen に従って更新し、無反応カウンタを 0 に戻す。新規なしカウンタは StuckAfterOp に従う。
- 変わらないもの: ExecuteNoChange と同じ。

### ExecuteOutOfDomain(実行して許可ドメインの外へ出る)

- 起きられる条件: ExecuteNoChange と同じ。
- 変わるもの: 訪問済みを更新し、段階を idle に戻す。画面変化あり、新規発見なしにする。完了フラグを ObserveDoneWhen に従って更新し、無反応カウンタを 0 に戻す。新規なしカウンタは StuckAfterOp に従う。外部ページにいる状態にし、許可ドメイン外にする。
- 変わらないもの: モード、段、提案の 5 項目、時間、予算、終了理由。
- 補足: 操作の種類は問わない。wait や scroll でもドメイン外へ出られる。

### ReturnToAllowedDomain(許可ドメインへ戻る)

- 起きられる条件: Continuing が成り立ち、許可ドメイン外にいる。
- 変わるもの: 外部ページにいない状態にし、許可ドメイン内に戻す。
- 変わらないもの: それ以外すべて。段階は idle のまま、予算も消費しない。

### FinishByProposalDone(提案 done で終わる)

- 起きられる条件: 段階が acting で、提案操作が done。
- 変わるもの: 段階を done に、終了理由を reported_done にする。
- 変わらないもの: それ以外すべて。

### FinishByProposalStuck(提案 stuck で終わる)

- 起きられる条件: 段階が acting で、提案操作が stuck。
- 変わるもの: 段階を stuck に、終了理由を stuck にする。
- 変わらないもの: それ以外すべて。新規なしカウンタがしきい値に届いていなくても終われる。

### RecoverFromFreeze(固まりから回復する)

- 起きられる条件: Continuing が成り立ち、無反応カウンタがしきい値以上。
- 変わるもの: 段階を frozen_recovered にし、無反応カウンタを 0 に戻す。
- 変わらないもの: それ以外すべて。ページやドメインの状態には触れず、予算も消費しない。

### ResumeAfterRecovery(回復後に再開する)

- 起きられる条件: 段階が frozen_recovered。
- 変わるもの: 段階を idle に戻す。
- 変わらないもの: それ以外すべて。

### AdvanceStage(次の段へ進む)

- 起きられる条件: 段階が idle で、完了条件が成り立ち、最終段でない。最終段でないのはシナリオモードに限られる。
- 変わるもの: 段を 1 進める。完了フラグを偽に戻し、両カウンタを 0 に戻す。
- 変わらないもの: 段階(idle のまま)、訪問済み、提案、画面観測、ドメイン、時間、予算、終了理由。訪問済みと予算は段をまたいで引き継がれる。

### FinishNormally(目標達成で終わる)

- 起きられる条件: 段階が idle で、完了条件が成り立ち、最終段である。
- 変わるもの: 段階を done に、終了理由を goal_achieved にする。
- 変わらないもの: それ以外すべて。
- 補足: 時間切れ、予算切れ、新規なしの条件を問わない。

### FinishStuck(新規なしの続きで終わる)

- 起きられる条件: Exploring が成り立ち、新規なしカウンタがしきい値以上。
- 変わるもの: 段階を stuck に、終了理由を stuck にする。
- 変わらないもの: それ以外すべて。

### ExceedDecisionBudget(判断予算切れで終わる)

- 起きられる条件: Exploring が成り立ち、新規なしカウンタがしきい値未満で、判断予算が 0。
- 変わるもの: 段階を error_exit に、終了理由を budget_exceeded にする。
- 変わらないもの: それ以外すべて。

### FinishTimeUp(時間切れで終わる)

- 起きられる条件: Exploring が成り立ち、新規なしカウンタがしきい値未満で、時間切れ。
- 変わるもの: 段階を timed_out に、終了理由を time_exceeded にする。
- 変わらないもの: それ以外すべて。

### TimeElapses(時間が切れる)

- 起きられる条件: まだ時間切れでなく、終了段階にいない。段階が awaiting_decision、acting、frozen_recovered でも起きられる。
- 変わるもの: 時間切れを真にする。
- 変わらないもの: それ以外すべて。

### UserAbort(利用者が中断する)

- 起きられる条件: 終了段階にいない。
- 変わるもの: 段階を aborted に、終了理由を user_aborted にする。
- 変わらないもの: それ以外すべて。

### Next の最後の選択肢

終了段階にいるときは、すべての変数を変えない一歩を明示的に許す。終了後の行き止まりを避けるための足踏み。

### 読み取れる優先順位

idle の段階では、各アクションの前提が互いに排他になっていて、次の順で優先される。

1. 完了条件が成り立つと、AdvanceStage か FinishNormally だけが起きられる。Exploring と Continuing が偽になるので、ほかの idle 系アクションは止まる。
2. 完了していなくて新規なしカウンタがしきい値以上なら、FinishStuck だけが起きられる。予算切れや時間切れの終了は、新規なしカウンタがしきい値未満を前提にしている。
3. 次に、予算切れなら ExceedDecisionBudget、時間切れなら FinishTimeUp。両方成り立つときはどちらも起きられ、順位は無い。
4. 上のどれでもない(Continuing が成り立つ)とき、無反応カウンタがしきい値以上なら RecoverFromFreeze が起き、RequestDecision は起きられない。
5. 無反応カウンタがしきい値未満なら、RequestDecision、ResetRequestWindow、ReturnToAllowedDomain が並んで起きられる。RequestDecision はドメイン内で要求予算が 1 以上のときに限る。

TimeElapses と UserAbort は、終了段階でなければどの段階でも割り込める。

## 5. 不変条件(状態について常に成り立つと主張するもの)

| 名前 | 主張 |
|---|---|
| NeverOperatesForbidden | 訪問済み集合には禁止要素が一つも入らない。禁止要素を click や input で触ることは起きない。 |
| LowConfidenceBackNeverActed | 段階が acting でペイウォールでなく、操作が back なら、その提案は低信頼ではない。前件は「ペイウォール以外の理由で back を実行しようとしている」状況を指す。 |
| DirectOpActedOnlyOnSafeTarget | 段階が acting でペイウォールでなく、操作が click か input なら、対象は存在し、禁止要素でもない。 |
| ExploreGoalOnlyWhenAllExplored | 探索モードで終了理由が goal_achieved なら、探索してよい要素はすべて訪問済み。 |
| ScenarioGoalOnlyAtLastStage | シナリオモードで終了理由が goal_achieved なら、最終段にいて、その段の完了フラグが真。 |
| ReportedDoneOnlyWithoutDoneWhen | 終了理由が reported_done なら、探索完了でもシナリオ段完了でもない。完了条件が成り立たないうちに done を提案して終えた場合だけを reported_done と呼ぶ。 |
| ExploreNeverReportedDone | 探索モードでは終了理由が reported_done にならない。 |
| ExitReasonMatchesPhase | 段階と終了理由が対応する。done なら goal_achieved か reported_done、stuck なら stuck、error_exit なら budget_exceeded、timed_out なら time_exceeded、aborted なら user_aborted。終了段階でなければ none。 |
| TimeExceededOnlyWhenTimeUp | 終了理由が time_exceeded なら、時間切れが真。 |
| StagesOnlyInScenario | 探索モードでは段は常に 1 で、完了フラグは常に偽。 |
| StagesBoundedByOps | 進んだ段の数(stage - 1)は判断回数を超えない。 |
| NewSeenOnlyOnChangedAllowedPage | 新規発見が真なら、画面変化あり、許可ドメイン内、判断回数が 1 以上、がすべて成り立つ。 |
| NoChangeObservedOnlyAfterOps | 画面変化なしが観測されているなら、判断回数は 1 以上。 |
| NoProposalBeforeDecision | 判断回数が 0 なら、提案は初期値のまま。操作も対象も none、低信頼は偽、ペイウォールは偽、対象は存在する扱い。 |
| RequestsWithinWindowCap | 窓内要求数は MaxRequests を超えない。 |
| DecisionsWithinCap | 判断回数は MaxDecisions を超えない。 |
| BudgetAccounting | 判断予算と判断回数の和は常に MaxDecisions に等しい。要求予算と窓内要求数の和は常に MaxRequests に等しい。 |
| BudgetExceededOnlyWhenSpent | 終了理由が budget_exceeded なら、判断予算は 0。 |
| CountersBoundedByOps | 無反応カウンタも新規なしカウンタも、判断回数を超えない。 |
| VisitedOnlyByOps | 訪問済み要素の数は判断回数を超えない。 |
| RecoveryOnlyAfterFreeze | 段階が frozen_recovered なら、判断回数は IdleThreshold 以上。 |
| RecoveryResetsPageState | 段階が frozen_recovered なら、無反応カウンタは 0 で、許可ドメイン内にいる。 |
| DomainRecordAccurate | 「許可ドメイン内」と「外部ページにいない」は常に一致する。 |
| ProposalWellFormed | 段階が awaiting_decision か acting なら、提案操作は Operations か back のどれかで、対象は実在の要素(none でない)。探索モードなら提案操作は done でない。 |

## 6. 行動性質(一歩ごとに成り立つと主張するもの)

どれも `[][…]_vars` の形なので、変数が何も変わらない一歩(足踏み)は常に許される。主張は「何かが変わる一歩」についてだけ効く。

| 名前 | 主張 |
|---|---|
| GoalPreempts | idle で目標達成(探索完了、またはシナリオ最終段の完了)なら、次の一歩は「done になり goal_achieved で終わる」か環境側の出来事のどちらか。前件は、最後の操作の結果で完了条件が成り立った直後の idle を要求する。 |
| StageAdvancePreempts | idle で最終段でない段が完了したなら、次の一歩は「idle のまま段が 1 進む」か環境側の出来事。 |
| StuckPreempts | idle で、完了しておらず新規なしカウンタがしきい値以上なら、次の一歩は stuck への移行か環境側の出来事。 |
| LimitStops | idle で、完了しておらず、新規なしカウンタがしきい値未満で、判断予算が 0 か時間切れなら、次の一歩は「予算 0 のもとで error_exit」「時間切れのもとで timed_out」環境側の出来事、のどれか。 |
| FreezePreemptsDecision | idle で無反応カウンタがしきい値以上なら、次の一歩で awaiting_decision に入らない。 |
| SafeProposalProceeds | awaiting_decision で、ペイウォールでなく、低信頼の back でもなく、click や input なら対象が安全(禁止要素でなく存在する)な提案なら、次の一歩は acting への移行か環境側の出来事。 |
| AwaitingOnlyByDecision | awaiting_decision に新しく入る一歩と、判断回数がちょうど 1 増える一歩は、同じ一歩である(双方向)。 |
| ActingOnlyAfterGuardrails | acting に新しく入るのは、awaiting_decision からだけ。 |
| FreezeRecoveryRecorded | idle で、完了しておらず、無反応カウンタがしきい値以上の状態から、無反応カウンタが 0 になる一歩は、必ず frozen_recovered への移行である。 |
| VisitedChangesOnlyByExecution | 訪問済み集合が変わる一歩は、acting から idle への移行(実行)に限る。 |
| PaywallForcesBack | awaiting_decision でペイウォールなら、次の一歩は「操作を back にして acting へ進む」か環境側の出来事。 |
| ProposalOutcome | acting からの次の一歩は、提案が done なら done、stuck なら stuck、それ以外なら idle へ移る。そうでなければ環境側の出来事。 |
| IdleCounterTracksScreen | acting から idle への一歩では、画面が変わったら無反応カウンタを 0 に戻し、変わらなければ 1 増やす。 |
| MemoryNeverShrinks | 訪問済み集合は減らない。 |
| StuckCounterTracksNewElements | acting から idle への一歩では、訪問済みが増えず新規発見もなければ新規なしカウンタを 1 増やし、そうでなければ 0 に戻す。 |
| NoDecisionOutsideDomain | idle から awaiting_decision への一歩は、外部ページにいないときだけ起きる。 |
| StageAdvanceResetsCounters | 段が変わる一歩は、idle のまま、最終段でない段が完了した状態から、段がちょうど 1 進むものに限る。その一歩で両カウンタを 0 に戻し、完了フラグを偽にする。 |
| BudgetSharedAcrossStages | 段が変わる一歩では、判断予算、判断回数、要求予算、窓内要求数は変わらない。 |
| DecisionBudgetNeverRestored | 判断予算は増えない。 |
| TimeNeverRewinds | 一度時間切れになったら、時間切れのまま。 |
| TimeElapsesAlone | 時間切れが起きる一歩では、ほかの変数は何も変わらない。 |
| ModeFixed | モードは途中で変わらない。 |
| ProposalChangesOnlyByDecision | 提案の 5 項目(操作、対象、低信頼、ペイウォール、対象の存在)が変わる一歩は、次のどちらかに限る。awaiting_decision に新しく入る一歩か、awaiting_decision でペイウォールのときに操作だけを back に差し替える一歩。 |
| CountersChangeOnlyBySteps | 無反応カウンタが変わるのは、実行(acting から idle)、frozen_recovered への移行、段の前進のどれか。新規なしカウンタが変わるのは、実行か段の前進。 |
| DomainChangesOnlyByNavigation | ドメインの記録(inAllowedDomain か pageOutside)が変わるのは、実行か、idle のまま許可ドメインへ戻る一歩に限る。 |
| ObservationChangesOnlyByExecution | 画面変化と新規発見の観測が変わるのは、実行の一歩に限る。 |
| DoneWhenChangesOnlyByExecutionOrAdvance | 完了フラグが偽から真になるのは実行の一歩だけ。真から偽になるのは段が 1 進む一歩だけ。 |
| TerminationKeepsState | 終了段階に入る一歩では、段階と終了理由のほかは何も変わらない。 |
| TerminalAbsorbing | 終了段階にいるなら、次の一歩ですべての変数が変わらない。終了段階から抜けない。 |

## 7. 時相性質と公平性

### Spec

初期状態 Init から始まり、各一歩は Next のどれか(または足踏み)である。そのうえで、次のアクションに弱公平性を課す。

- RequestDecision、ResetRequestWindow、ApplyGuardrails
- 3 種の実行をまとめたもの(ExecuteNoChange、ExecuteChange、ExecuteOutOfDomain のどれか)
- ReturnToAllowedDomain、FinishByProposalDone、FinishByProposalStuck
- RecoverFromFreeze、ResumeAfterRecovery
- AdvanceStage、FinishNormally、FinishStuck、ExceedDecisionBudget、FinishTimeUp

弱公平性は「ずっと起きられ続けるアクションは、いつか必ず起きる」という仮定。TimeElapses と UserAbort には公平性を課していない。時間切れと中断は、起きても起きなくてもよい環境の出来事として扱われている。

### EventuallyTerminates

Spec の公平性の仮定のもとで、いつか必ず終了段階(done、stuck、error_exit、timed_out、aborted のどれか)に達する。時間切れは強制されないので、終了を保証しているのは判断予算の有限性と、各終了アクションへの弱公平性である。判断を消費しない一歩(窓の巻き戻し、ドメインへの復帰、固まりからの回復、段の前進)は、それぞれ判断や実行をはさまないと繰り返せない。

### StartsFresh

状態述語なので、時相性質として検査すれば「初期状態でこの様子が成り立つ」という主張になる。

## 8. 気になった点

1. **StartsFresh の検査方法**: StartsFresh は状態述語で、初期状態の様子だけを述べている。不変条件として検査すると、最初の一歩(例: RequestDecision で段階が awaiting_decision になる)で破れる。性質として初期状態だけを見る使い方になっているかを確かめたい。また、モード、提案の初期値、decisionsMade、requestsInWindow、pageOutside、screenChanged には触れていない。
2. **予算切れと時間切れの間に順位が無い**: 判断予算 0 と時間切れが同時に成り立つと、ExceedDecisionBudget と FinishTimeUp の両方が起きられる。LimitStops もどちらでもよいと書いている。終了理由がどちらになるかは非決定で、意図どおりかを確かめたい。
3. **要求の時間窓に時間の概念が無い**: ResetRequestWindow は、続行中ならいつでも起きられる。1 回要求した直後でも窓を満額に戻せるので、RequestsWithinWindowCap は「時間あたりの要求数の上限」としてはほとんど何も縛らない。BudgetAccounting から自明に導ける。
4. **却下された提案はカウンタに数えられない**: ApplyGuardrails で却下されると予算だけが減り、無反応カウンタも新規なしカウンタも動かない。禁止要素や存在しない要素ばかり提案し続けた場合、stuck にも固まり回復にも入らず、予算切れ(error_exit)まで回り続ける。意図した扱いか確かめたい。
5. **ペイウォールと対象の存在が判断ごとに独立に決まる**: どちらもページの状態ではなく、RequestDecision のたびに任意に選ばれる。同じページで連続してペイウォールが出ることや、back の後でペイウォールが消えることは表現されていない。また、ペイウォールのときは done や stuck の提案も back に置き換わり、低信頼でも back を実行する。LowConfidenceBackNeverActed と DirectOpActedOnlyOnSafeTarget が前件で isPaywall を外しているのは、この扱いと整合する。
6. **ドメイン外への移動が操作の種類を問わない**: ExecuteOutOfDomain は wait や scroll でも起きられる。画面遷移を伴わない操作でドメイン外に出るのは不自然に見える。
7. **画面が変わらなくても訪問済みになる**: ExecuteNoChange でも、click や input の対象は訪問済みに加わる。押しても何も起きなかった要素を探索済みと数えてよいか確かめたい。探索モードの完了条件(全要素訪問)がこれで満たされうる。ExecuteOutOfDomain でも同じく対象が訪問済みに加わる。
8. **訪問済みが段をまたいで引き継がれる**: AdvanceStage は訪問済みを空にしない。シナリオの後の段で前の段に触った要素を操作しても「訪問済みが増えない」扱いになり、新規発見がなければ新規なしカウンタが増える。段ごとの新規性を測りたいのであれば向きが合わない。
9. **RecoveryResetsPageState は名前と中身がずれる**: RecoverFromFreeze はドメインの状態に触れない。frozen_recovered で許可ドメイン内にいることは、ExecuteOutOfDomain が無反応カウンタを 0 に戻すことと、ドメイン外では判断できないことから間接的に成り立っている。名前が示す「回復がページ状態を戻す」という動作は、アクションには書かれていない。
10. **inAllowedDomain と pageOutside が冗長**: DomainRecordAccurate により、2 変数は常に反対の値を取る。NoDecisionOutsideDomain は pageOutside、RequestDecision は inAllowedDomain を見ていて、同じ条件を別の変数で書いている。
11. **探索してよい要素が空のとき**: Elements がすべて禁止要素なら、探索モードでは初期状態で完了条件が成り立ち、判断なしで goal_achieved で終わる。仮定でこの場合を除いていない。
12. **提案 stuck と新規なしカウンタ**: FinishByProposalStuck はカウンタがしきい値未満でも stuck で終われる。終了理由はどちらの経路でも同じ stuck なので、終了後には区別できない。
13. **ProposableOps に back が無い場合**: TypeOK と ProposalWellFormed は Operations に back を足した範囲を許している。Operations から back を外した設定でも、ペイウォール経由の back は実行される。設定上の「使える操作」とずれる可能性がある。
