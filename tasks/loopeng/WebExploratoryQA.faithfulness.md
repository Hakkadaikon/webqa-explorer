# 文面の往復確認: WebExploratoryQA

対象は、`.tla`で状態機械にしたEARS行(トレーサビリティ・マトリクスで「モデル外」でない行)。
読み手はfresh contextのsubagentで、入力は`WebExploratoryQA.faithfulness-input.md`だけ。
入力は現行specからコメントを除いた形式文面で、管理番号とEARS文を含まない。
読み手の言い直しは`WebExploratoryQA.paraphrase.md`にある。
作成は2026-10-18。
要件変更5件(extract.mdの設計メモ10)を入れたv4のspecに対して、全行を作り直した。
性質の名前が変わったので、前回の判定表は使っていない。

前件到達の証跡は`WebExploratoryQA.reach.log`の「v4」節を指す(表では「前件到達はP1」のように節の番号で書く)。
検査の設定は`WebExploratoryQA.cfg`で、`StartsFresh`だけが状態述語の`PROPERTY`、ほかの状態述語はすべて`INVARIANT`。

## 判定表

| 要件 | 式 | 読み手の言い直し | 判定 | 理由 / 直し方 |
|---|---|---|---|---|
| S-002 | `ExecuteNoChange`/`ExecuteChange`/`ExecuteOutOfDomain`、`SafeProposalProceeds`、`ProposalOutcome`、`VisitedChangesOnlyByExecution` | 「ガードに掛からない提案はactingへ進む」「actingの次は、doneならdone、stuckならstuck、それ以外はidleへ」「visitedが変わるのは実行の一歩だけ」 | partial | 許可された提案が実行段階へ進み、実行の帰結が記録されることは検査している。ブラウザ操作という副作用は状態変数に現れない(model.mdの「モデル化で捨てた領域」)。前件到達はP21、P26、P27a〜c。 |
| S-005 | `RequestDecision`、`ProposalWellFormed`、`AwaitingOnlyByDecision` | 「続行中で、固まりでなく、要求予算があり、許可ドメイン内なら、操作をProposableOpsから選び、予算を減らす」「awaitingに入る一歩と判断回数が1増える一歩は同じ」 | partial | 問い合わせ(API呼び出し)そのものはモデル外で、どんな値が返っても扱えることを非決定の選択で表す。前件到達はP15、P23。 |
| S-006 | `RequestDecision`、`ProposalWellFormed` | 「対象を全要素から選ぶ。禁止要素も含む」 | partial | S-005と同じ理由。対象が要らない操作でも対象を持つが、`VisitedAfterOp`はclickとinput以外で対象を使わないので振る舞いに影響しない。 |
| S-010 | `ApplyGuardrails`、`LowConfidenceBackNeverActed`、`SafeProposalProceeds` | 「ペイウォールでなく、backで低信頼なら却下」「actingでペイウォールでなくbackなら低信頼でない」「ガードに掛からない提案は却下しない」 | full | 却下する側と、却下しすぎない側の両方を検査している。ペイウォールの強制backを外すのは、S-013による意図どおりの扱い。0.7の閾値は真偽値に丸めた(設計メモ8)。前件到達はP1、P21。 |
| S-011 | `ExecuteNoChange`、`IdleCounterTracksScreen`、`ObservationChangesOnlyByExecution` | 「画面が変わらなければ無反応カウンタを1増やす」「画面の観測が変わるのは実行の一歩だけ」 | partial | 画面が変わらなかったことの観測は表している。次に確信度の高い候補へ切り替える選択は、`RequestDecision`の非決定の選択に吸収され、モデル外。受け入れシナリオ「画面が変わらないのに同じ要素を選んだら…」で確かめる。前件到達はP28e、P37。 |
| S-012 | `ApplyGuardrails`、`NeverOperatesForbidden`、`DirectOpActedOnlyOnSafeTarget`、`SafeProposalProceeds` | 「clickかinputで対象が禁止要素なら却下」「訪問済みに禁止要素は入らない」「actingでペイウォールでなく直接操作なら、対象は存在し禁止要素でない」 | full | 一致する。禁止要素の分類ロジックの正しさは、設計メモ8とmodel.mdの`ForbiddenElements`節のとおり実装で確かめる。前件到達はP2、P21。 |
| S-013 | `ApplyGuardrails`のペイウォール分岐、`PaywallForcesBack`、`ProposalChangesOnlyByDecision` | 「ペイウォールなら提案に関係なく操作をbackにしてactingへ。doneやstuckの提案もbackになる」 | partial | 判断モデルの提案と無関係に既定の離脱操作をする点は一致する。specはペイウォールの観測が問い合わせの応答と同時に届くので、ペイウォールでも問い合わせて予算を1ずつ使う。「問い合わせない」は検査していない。予算を多めに使う側へのずれで、安全性には影響しない。実装は問い合わせの前に課金画面を判定して問い合わせを省く。受け入れシナリオ「課金画面が出たら判断モデルに問い合わせず…」「課金画面から離脱しても消えないまま3回続いたら…」で確かめる。前件到達はP22、P34。 |
| S-014 | `FinishNormally`、`FinishByProposalDone`、`ExploreGoalOnlyWhenAllExplored`、`ScenarioGoalOnlyAtLastStage`、`ReportedDoneOnlyWithoutDoneWhen`、`GoalPreempts`、`DoneWhenChangesOnlyByExecutionOrAdvance` | 「goal_achievedなら、探索モードは全部訪問済み、シナリオは最終段で完了フラグが真」「reported_doneなら完了条件が成り立っていない」「完了条件が成り立つidleの次は、done・goal_achievedか環境の出来事」 | full | ゴール達成は完了条件だけで決まり、判断モデルのdoneは未検証の終了として分けて記録する。前件到達はP3、P4、P5、P16a、P16b、P38a、P38b。 |
| S-015(段階の定義) | `stage`、`NumStages`、`LastStage` | 「シナリオの段数は1以上。最終段かどうかはstage = NumStages」 | partial | 段階を状態機械に入れたが、YAMLでの定義の書き方と各段のdone_whenの中身はモデル外(model.md)。受け入れシナリオ「テスト対象・入力値・段階をYAMLの設定で定義して実行する」で確かめる。 |
| S-015(段階の切り替え) | `AdvanceStage`、`StageAdvancePreempts`、`StageAdvanceResetsCounters`、`BudgetSharedAcrossStages`、`StagesBoundedByOps`、`TimeNeverRewinds` | 「idleで最終段でない段が完了したら、次の一歩は段を1進めるか環境の出来事」「段が変わる一歩は両カウンタを0にし完了フラグを偽にする」「段が変わる一歩で予算と窓は変わらない」 | partial | 段を進め、固まりと行き詰まりの回数を0に戻し、判断予算と時間窓を戻さないことは要件どおり。ただしspecは、段を進めた直後に次の段の完了フラグを偽にし、操作を1回実行するまで観測しない。次の段のdone_whenが切り替えた時点のページで既に成り立つ場合、要件は直ちに次へ進むかゴール達成とするように読めるが、specは問い合わせを1回挟む。最後の段なら、その問い合わせでdoneが返るとreported_doneで終わりうる。要件はこの時機を決めていないので、下の「ユーザーの判断が要る点」の1に挙げた。前件到達はP17、P30、P31。 |
| S-016(探索モード) | `RequestDecision`の非決定の選択 | 「提案できる操作は、探索モードではdoneを除いたもの。対象は全要素から選ぶ」 | partial | 探索モードを持つことは表しているが、未操作の要素を優先する規則は非決定の選択に吸収した(model.mdの「捨てた領域」)。受け入れシナリオ「未操作の要素を、未探索の分岐が残る操作済み要素より先に選ぶ」で確かめる。 |
| S-016(探索のゴール) | `FinishNormally`、`ExploreDone`、`ExploreGoalOnlyWhenAllExplored`、`GoalPreempts` | 「探索モードで、探索してよい要素をすべて訪問済みならgoal_achievedで終わる。その状態のidleの次はそれか環境の出来事」 | partial | specの「訪問済み」は「操作済み」で、要件の「探索し終えた」(遷移先まで探索し終えた。設計メモ9)とは別の概念。単一画面への簡略化の下では両者が一致する。複数画面での意味は、受け入れシナリオ「遷移先を探索し終えるまで、途中のボタンを探索し終えた要素として扱わない」で確かめる。探索してよい要素が空のときの扱いは、下の「ユーザーの判断が要る点」の2に挙げた。前件到達はP3、P16a。 |
| S-016(制限時間) | `TimeElapses`、`FinishTimeUp`、`LimitStops`、`TimeExceededOnlyWhenTimeUp`、`ExitReasonMatchesPhase`、`TimeNeverRewinds`、`TimeElapsesAlone` | 「時間切れはどの段階でも起きうる」「idleで完了せず新規なしがしきい値未満で、時間切れなら、次はtimed_outか予算切れか環境の出来事」「time_exceededなら時間切れ」 | full | 時間切れは時刻の経過として公平性なしで起き、次のidleで時間切れとして終わる。決め打ちシナリオでも起きる(設計メモ10の(b))。ステップの途中で時間が切れたら次のidleで終わるのは、優先順位の行(S-044の6行目)が待機状態の判定として定めたとおり。前件到達はP7d、P8、P9、P19b、P19c、P32、P33。 |
| S-016(doneを外す) | `ProposableOps`、`ProposalWellFormed`、`ExploreNeverReportedDone` | 「探索モードでは提案操作はdoneでない」「探索モードではreported_doneにならない」 | full | 一致する。`ProposalWellFormed`はシステム側の`ProposableOps`を使わず生の変数で書いてある。前件到達はP6、P15。 |
| S-017 | `RecoverFromFreeze`/`ResumeAfterRecovery`、`RecoveryResetsPageState`、`RecoveryOnlyAfterFreeze`、`FreezePreemptsDecision`、`FreezeRecoveryRecorded`、`MemoryNeverShrinks` | 「続行中で無反応カウンタがしきい値以上ならfrozen_recoveredへ。カウンタだけ0に戻す」「frozen_recoveredならカウンタ0で許可ドメイン内」「固まり中はawaitingに入らない」「訪問済みは減らない」 | partial | 復旧がページの状態だけを戻し、探索の記憶を残すことは検査している。固まりを検出したら必ず復旧することは性質として書いていない。問い合わせは禁止され、弱公平性の下でいずれ復旧か終了が起きるが、これは推論で検査していない。ドメイン外は復帰で、行き詰まりは終了で扱う(設計メモ2。行き詰まりで終えるのはS-044の優先順位による)。「ページが応答しない」は画面が変化しなかった1ケースとして固まりカウンタに合流させた(設計メモ9)。前件到達はP14、P20、P25。 |
| S-021(固まり) | `RecoverFromFreeze`のガード、`IdleCounterTracksScreen`、`CountersBoundedByOps`、`CountersChangeOnlyBySteps` | 「無反応カウンタがしきい値以上」「実行で画面が変わらなければ+1、変われば0」 | full | 一致する。「一定時間」は操作の回数で数える(設計メモ9)。前件到達はP28a、P28e、P35a。 |
| S-021(ドメイン外) | `ExecuteOutOfDomain`、`DomainRecordAccurate`、`DomainChangesOnlyByNavigation` | 「外部ページにいる状態にし、許可ドメイン外にする」「許可ドメイン内と外部ページにいないは常に一致」 | full | 記録が実際のページ位置と一致することを検査する。判定ロジック(URLの比較)の正しさは設計メモ8。前件到達はP36。 |
| S-021(行き詰まり) | `FinishStuck`のガード、`StuckAfterOp`、`StuckCounterTracksNewElements`、`StuckPreempts`、`NewSeenOnlyOnChangedAllowedPage` | 「訪問済みが増えず新規発見もなければ+1、そうでなければ0」「新規なしがしきい値以上のidleの次はstuckか環境の出来事」「新規発見なら、画面変化あり、許可ドメイン内、判断1回以上」 | full | 新しい定義(探索済みの集合が増えず、未操作の要素も新しく見えない)と一致する。許可ドメイン外の要素を新しく見えた要素に数えないこと(設計メモ10の1)も検査している。却下された提案はカウンタを進めないが、設計メモ9のとおり。前件到達はP10、P18、P28b、P28c、P28d、P35b。 |
| S-022 | `VisitedAfterOp`、`MemoryNeverShrinks`、`VisitedOnlyByOps`、`VisitedChangesOnlyByExecution` | 「clickかinputなら対象を訪問済みに加える」「訪問済みは減らない」「訪問済みの数は判断回数以下」 | partial | 1回の実行の中で記憶が失われないことは検査している。グラフ構造は単一の集合で近似し(model.mdの簡略化)、実行をまたぐ永続化は状態変数の外。受け入れシナリオ「探索の記憶を次の実行に引き継ぐ」などで確かめる。前件到達はP26。 |
| S-023 | `RequestDecision`の非決定の選択 | 「提案操作と対象を任意に選ぶ」 | partial | 優先順位の規則は非決定の選択に吸収した(model.mdの「捨てた領域」)。受け入れシナリオ「未操作の要素を…先に選ぶ」ほか3本で確かめる。 |
| S-027 | `Init`の`mode`、`ModeFixed` | 「モードは2種類のどちらかで始まり、途中で変わらない」 | full | 両モードを同じ状態機械で扱う。モードの指定方法はモデル外だが、要件はそれを求めていない。前件到達はP4(シナリオ)、P6(探索)。 |
| S-028(存在確認) | `ApplyGuardrails`、`DirectOpActedOnlyOnSafeTarget`、`SafeProposalProceeds` | 「clickかinputで対象が存在しないなら却下」「actingで直接操作なら対象は存在する」 | full | 一致する。存在の判定(ノードの同一性)は設計メモ8。前件到達はP2、P21。 |
| S-028(決済導線) | `ApplyGuardrails`、`NeverOperatesForbidden` | 「対象が禁止要素なら却下」「訪問済みに禁止要素は入らない」 | partial | specは判断モデルが禁止要素を提案しうるとして、ガードで操作を止めることを検査する。要件の「選択してはならない」(探索の候補に入れない)は非決定の選択に吸収され、モデル外。設計メモ9のとおり、実装は禁止要素全体を候補から外す。受け入れシナリオ「禁止要素は未操作のまま残っていても、探索の候補に入れない」「探索モードでも決済導線の要素は選ばない」で確かめる。 |
| S-033 | `ExecuteOutOfDomain`/`ReturnToAllowedDomain`、`RequestDecision`のガード、`DomainRecordAccurate`、`NoDecisionOutsideDomain`、`DomainChangesOnlyByNavigation` | 「ドメイン外を記録する」「idleからawaitingへの一歩は外部ページにいないときだけ」「続行中でドメイン外なら戻る」 | full | 遷移の記録と、ドメイン外で探索しないことの両方を検査している。前件到達はP29、P36。 |
| S-041 | `RequestDecision`/`ExceedDecisionBudget`/`FinishTimeUp`、`DecisionsWithinCap`、`BudgetAccounting`、`BudgetExceededOnlyWhenSpent`、`DecisionBudgetNeverRestored`、`LimitStops`、`TerminalAbsorbing`、`EventuallyTerminates` | 「判断回数は上限以下」「予算と判断回数の和は上限」「予算0か時間切れのidleの次は、尽きた方の終了か環境の出来事」「判断予算は増えない」「いつか必ず終わる」 | full | 呼び出し回数は上限を超えず、上限か制限時間に達したら次の待機状態で止まり、再開しない。上位の終了条件(ゴール達成、行き詰まり)が先に採られるのは、承認済みの優先順位のとおり。前件到達はP13、P19a、P19b、P19c。 |
| S-042 | `RequestDecision`/`ResetRequestWindow`、`RequestsWithinWindowCap`、`BudgetAccounting`、`BudgetSharedAcrossStages` | 「要求予算が1以上のときだけ問い合わせる」「続行中なら時間の条件なしでいつでも窓を戻せる」「窓内要求数は上限以下」 | partial | 1つの窓で上限を超えないことは検査している。窓が時間の条件なしで満了するのは、要件の「どの時点でも満了しうる」のとおり(設計メモ7)。1窓の実時間の長さはモデル外。数える単位もずれる。specは問い合わせごとに数えるが、実装は対象サーバへのアクセス(ブラウザ操作、課金画面の離脱、固まり復旧の再読み込み、許可ドメインへの戻り)を数え、ガードで止めた提案は数えない(設計メモ9でユーザーが承認)。受け入れシナリオ「ページの再読み込みや許可ドメインへの戻りも…」「時間窓の操作回数が上限に達していたら…」「ガードレールで止めて…」で確かめる。前件到達はP31。 |
| S-044(ゴール達成) | `FinishNormally`、`GoalPreempts`、`ExploreGoalOnlyWhenAllExplored`、`ScenarioGoalOnlyAtLastStage`、`ExitReasonMatchesPhase` | 「idleで目標達成なら、次はdone・goal_achievedか環境の出来事」 | partial | 完了条件の観測が成り立てば、他の終了理由や継続に先取りされずゴール達成で終わる。ただし最後の段に入った直後と実行の開始時は、操作を1回するまで完了条件を観測しない(S-015の段階の切り替えの行と同じ理由)。前件到達はP3、P4、P16a、P16b。 |
| S-044(reported_done) | `FinishByProposalDone`、`ProposalOutcome`、`ReportedDoneOnlyWithoutDoneWhen`、`ExploreNeverReportedDone` | 「actingで提案がdoneならdone・reported_done」「reported_doneなら完了条件が成り立っていない」 | full | 一致する。課金画面のときにdoneの提案がbackに置き換わるのはS-013による。前件到達はP5、P27a。 |
| S-044(行き詰まり) | `FinishByProposalStuck`/`FinishStuck`、`StuckPreempts`、`ProposalOutcome`、`ExitReasonMatchesPhase` | 「提案stuckならstuckで終わる」「新規なしがしきい値以上のidleの次はstuckか環境の出来事」「段階stuckなら終了理由stuck」 | full | `StuckPreempts`は終了理由を見ないが、`ExitReasonMatchesPhase`が段階と終了理由の対応を検査するので、合わせて一致する。前件到達はP7b、P18、P27b。 |
| S-044(中断) | `UserAbort`、`ExitReasonMatchesPhase`、`TerminationKeepsState` | 「終了段階でなければいつでもaborted・user_abortedへ。ほかは変えない」 | partial | どの時点でも中断を受け付け、中断として記録して終わることは検査している。記憶・終了のイベント・実行結果の書き出しはモデル外(設計メモ10の3)。受け入れシナリオで確かめる。前件到達はP7e、P39a、P39b。 |
| S-044(終了状態) | `TerminalAbsorbing`、`Next`の最後の選択肢、`TerminationKeepsState` | 「終了段階なら次の一歩ですべての変数が変わらない」「終了する一歩は段階と終了理由のほかを変えない」 | full | 中断を含め、終了後は状態が変わらない。`TimeElapses`と`UserAbort`も終了段階では起きない。前件到達はP39a、P40。 |
| S-044(優先順位) | `Exploring`/`Continuing`の各ガード、`GoalPreempts`、`StageAdvancePreempts`、`StuckPreempts`、`LimitStops`、`FreezePreemptsDecision`、`NoDecisionOutsideDomain`、`EnvEvent` | 「1. 完了条件(段の前進かゴール) 2. 行き詰まり 3. 予算切れと時間切れ(同順位) 4. 固まり復旧、窓の巻き戻し、ドメイン復帰 5. 問い合わせ。中断と時間切れはどこでも割り込む」 | full | 要件の順序と一致する。予算切れと時間切れが同時に成り立てばどちらでもよいことも、要件の文面どおり(設計メモ10の2)。前件到達はP16a、P16b、P17、P18、P19a〜c、P20、P29。 |

## 読み手が気になった13点の分類

### 1. `StartsFresh`の検査方法

承認済みの扱いどおりで、問題ない。
`WebExploratoryQA.cfg`では`PROPERTY StartsFresh`で、`INVARIANT`には入っていない。
`WebExploratoryQA.tlc-run.log`にもTLCの警告「state-level formulas used as PROPERTY … are only checked in the initial state」が出ている。
変異の検査でも、`StartsFresh`の連言を書き換えた8件が初期状態で性質違反になった。
初期状態だけを見る使い方になっている。

触れていない変数は、ほかの不変条件が初期状態でも縛る。
`decisionsMade`と`requestsInWindow`は、予算が満額なので`BudgetAccounting`から0に決まる。
提案の初期値は`NoProposalBeforeDecision`、`pageOutside`は`DomainRecordAccurate`、`screenChanged`は`NoChangeObservedOnlyAfterOps`が縛る。
`mode`はどちらで始まってもよい(S-027)。

### 2. 予算切れと時間切れが同時に成り立つときの終了理由

承認済みの決定どおりで、問題ない。
S-044の6行目に「同じ段で、両方が成り立てばどちらを採ってもよい」とあり、設計メモ10の2でユーザーが選んだ。
`LimitStops`もどちらの理由も許し、前件到達はP19cにある。

### 3. 時間窓に時間の条件が無い

要件どおりで、問題ない。
S-042は「単位時間の窓は時刻で区切られ、どの時点でも満了しうる」と書き、設計メモ7の承認で明確にした文面である。
時刻をモデルに持たないので、窓はいつ満了してもよい。
そのため`RequestsWithinWindowCap`は「窓を戻すまでの要求数が上限以下」という意味しか持たず、`BudgetAccounting`から導ける。
1窓の実時間の長さはモデル外で、実装のレート制御のテストで確かめる。
数える単位のずれは、判定表のS-042の行のとおり(設計メモ9)。

### 4. 却下された提案をカウンタに数えない

承認済みの決定どおりで、問題ない。
設計メモ9の4つ目に「ガードレールで止めた提案と判断モデルの応答の異常は…固まりにも行き詰まりにも数えない。続いた場合は判断予算切れで止まる」とある。
却下が続いても行き詰まりにはならない。
`RequestDecision`は判断予算を必ず1減らすので、予算が0になれば`ExceedDecisionBudget`が起きる。
`EventuallyTerminates`は`PROPERTY`として検査され、No errorだった。

### 5. ペイウォールと対象の存在が判断ごとに独立に決まる

モデル化の方針どおりで、問題ない。
model.mdの変数表で、`isPaywall`と`targetExists`は「非決定的に観測」と定めている。
同じページで課金画面が続く場合も、独立な観測が続けて真になる振る舞いとして含まれる。
課金画面のときにdoneやstuckの提案もbackに置き換わるのは、S-013の「判断モデルに問い合わせず既定の離脱操作」のとおり。
課金画面が消えないときの終了は、受け入れシナリオ「課金画面から離脱しても消えないまま3回続いたら、行き詰まりで終了する」で確かめる。

### 6. ドメイン外への移動が操作の種類を問わない

モデル化の方針どおりで、問題ない。
S-033は「操作の結果」とだけ書き、操作の種類を限っていない。
waitの間のリダイレクトや、backでの外部ページへの戻りも現実に起こる。
環境の振る舞いを広めに取っているだけで、安全性の性質はより多くの振る舞いの上で成り立っている。

### 7. 画面が変わらなくても訪問済みになる

承認済みの決定どおりで、問題ない。
設計メモ9の1つ目で、specの`visited`は「操作済み」の意味と定めた。
押しても何も起きない要素や、許可ドメインの外へ出る要素は、遷移先に探索すべき要素が無いので、S-024の意味でも探索し終えたことになる。
探索のゴールが「全要素の操作済み」で満たされることの残りは、判定表のS-016(探索のゴール)の行のとおり。

### 8. 訪問済みが段をまたいで引き継がれる

承認済みの決定どおりで、問題ない。
設計メモ10の(c)で「段階を切り替えても探索の記憶(探索済みの要素の集合)は戻さない」とユーザーが承認した。
S-021は新規要素を「探索済みの要素の集合が増えない」と定めているので、前の段で操作した要素を操作し直しても新規要素にならない。
段階を進めたときに行き詰まりの回数を0に戻すのは、設計メモ10の5のとおり。
ただし、前の段と同じボタンを何度も押す決め打ちシナリオでは、新しい要素が見えなければ行き詰まりに近づく。
実装のテストで、この帰結が意図どおりか確かめるとよい。

### 9. `RecoveryResetsPageState`の名前と中身がずれる

モデル化で捨てた領域どおりで、問題ない。
model.mdの「捨てた領域」に、`RecoverFromFreeze`は固まりカウンタを戻すことでページ状態の初期化を表し、ブラウザの再読み込みはモデル外と書いてある。
許可ドメイン内であることは、ドメイン外では問い合わせず固まりカウンタが増えないことから成り立つ。
`.tla`の`RecoverFromFreeze`のコメントにも同じ理由を書いてある。
性質は名前より弱く、復旧直後の状態を確かめるだけである。

### 10. `inAllowedDomain`と`pageOutside`が冗長

モデル化の方針どおりで、問題ない。
model.mdの変数表で、`pageOutside`は記録の正しさを検査するためだけの環境側の真値と定めている。
`DomainRecordAccurate`は、記録を壊す書き換えを捕まえるための性質である。

### 11. 探索してよい要素が空のとき

要件が決めていない点で、ユーザーの判断が要る。
下の「ユーザーの判断が要る点」の2に挙げた。

### 12. 提案のstuckとカウンタのstuckが終了後に区別できない

要件どおりで、問題ない。
S-044の3行目は、判断モデルのstuckと行き詰まりの検出のどちらも「行き詰まり終了として記録」と定めている。
doneのように自己申告を分けて記録したいなら要件の変更になるが、今の要件は求めていない。

### 13. `Operations`にbackが無くてもペイウォールのbackが実行される

要件どおりで、問題ない。
S-005の8種は判断モデルに問い合わせる選択肢で、S-013の離脱操作はツールが判断モデルによらず行う既定の操作である。
設定で判断モデルの選択肢からbackを外しても、課金画面からの離脱は止めない。
`TypeOK`と`ProposalWellFormed`がbackを足した範囲を許すのは、この扱いと整合する。

## 仕様を直す必要があるもの

mismatchとvacuousの行は無い。
読み手の13点にも、要件とのずれは無かった。
仕様の直しは、下の「ユーザーの判断が要る点」の答えによって出る。

## ユーザーの判断が要る点

### 1. 段階に入った時点のdone_when

specは、段を進めた直後と実行の開始時に完了フラグを偽にし、操作を1回実行するまで観測しない。
実装は各ステップの初めにdone_whenを確かめる想定なので、切り替えた直後のページで次の段のdone_whenが成り立てば、問い合わせずに次へ進む。
この振る舞いはspecに無く、`StagesBoundedByOps`(段の数は判断回数以下)にも反する。
最後の段では、specだと問い合わせを1回挟み、doneが返るとreported_doneで終わりうる。

- (a) 今のspecのとおりにする。
  done_whenは操作の後にだけ確かめ、段階に入った直後は確かめない。
  実装もこれに合わせる。
- (b) 段階に入った時点でも確かめる。
  specは`AdvanceStage`と`Init`(決め打ちシナリオ)で完了フラグを非決定に観測する形に直す。
  1回の操作で複数の段が進みうるので、`StagesBoundedByOps`は外すか、別の形に書き直す。

### 2. 探索してよい要素が空のとき

禁止要素しか無いと、探索モードは初期状態で完了条件が成り立ち、1回も判断せずにゴール達成で終わる。
要件の「禁止要素を除くすべての要素を探索し終えたとき」は、空の集合に対しては直ちに成り立つ。
受け入れシナリオ「操作可能な要素が無いページでも問い合わせ…」はモードを書いておらず、探索モードで読むとこの扱いとぶつかる。

- (a) 今のspecのとおり、ゴール達成とする。
  受け入れシナリオには決め打ちシナリオであることを書き足す。
- (b) 探索してよい要素が無ければ、設定エラーかゴール達成とは別の終了理由にする。
  specに`ASSUME Explorable # {}`を足すか、終了理由を足す。
- (c) 最初のページを1回観測するまではゴールを判定しない。
  specでは、探索のゴールの条件に「判断回数が1以上」を足す。

## 空成立の証跡(含意・前提付きの式)

`WebExploratoryQA.reach.log`の「v4」節(P1〜P40)に、v4のspecに対する前件ごとの結果を追記した。
各節に`NotPx`の定義をそのまま書き、どれも反例(前件が成り立つ到達状態または一歩)が出ている。
`LimitStops`は「予算切れだけ」「時間切れだけ」「両方」の3つ(P19a〜c)、`StuckCounterTracksNewElements`は「新しい要素が見えた」場合(P28c)まで分けて確かめた。
前件を持たない式(`NeverOperatesForbidden`、`BudgetAccounting`、`MemoryNeverShrinks`、`ModeFixed`、`DecisionBudgetNeverRestored`、`EventuallyTerminates`など)は対象外である。
同ログのv3以前の節は旧spec向けで、この判定には使っていない。

## 定数の前提

`EventuallyTerminates`は、`ASSUME`で正と定めた閾値と上限の下での主張である。
検査に使う`WebExploratoryQA.cfg`は、`Operations`がclickを含み空でなく、`IdleThreshold`(2)が`MaxDecisions`(4)以下なので、固まり復旧の前件にも届く(P14、P25)。
実装では、設定値の検査で0以下の閾値と空の操作種別を拒否する。

## 変異の検査(v4のspec)

573件の変異を回した。
全件の結果は`WebExploratoryQA.mutants-v4.tsv`(7列目が理由)、値ごとの再検査は`WebExploratoryQA.mutants-v4-recheck.tsv`にある。

- 性質違反でkilledになったもの: 188件。
  内訳は不変条件60、行動性質119、時相性質1、`StartsFresh`の初期状態での違反8。
  最後の8件は、TLCの文言が分類器の正規表現に掛からずエラーに分類されたが、中身は性質違反である。
- 評価時のエラーで止まったもの: 108件。
  killedの証拠には数えず、値を1つずつ代入して再検査した。
  96件はどの値でも性質違反になった。
  12件は一部の値で緑のまま残り、abstraction-limitedとした(下記)。
- equivalent: 82件。
  コメント内の字面13、性質側の変異(`Init`・`Next`・`Spec`は不変)42、連言リストの先頭の`/\`が1項の`\/`になるもの26、時間窓の巻き戻しのガードを`\leq`にするもの1。
  最後の1件は、`-fp 0 -dump dot`の状態グラフから自己ループを除くと原本と一致した(sha256はTSVの7列目)。
- abstraction-limited: 12件。
  どれも要件が許す環境側の振る舞いを減らすだけで、誤った振る舞いを足さない。
  安全性の性質は「ある振る舞いが起こりうること」を要求できないので区別できない。
  - 判断モデルの応答や画面の観測値を1つに固定する書き換え(`RequestDecision`の5変数、6件)。
  - 実行の結果の1つが起こらなくなる書き換え(`ExecuteNoChange`の無反応カウンタ、3つの実行アクションの`UNCHANGED`外し、4件)。
  - 時刻の経過を起こらなくする書き換え(`TimeElapses`、2件)。
- 構文エラーで止まったもの: 195件。
  killにもsurvivorにも数えていない。
  ほとんどは連言リストの途中の`/\`を`\/`にしたもので、SANYが受け付けない。
  内訳は`TypeOK`20、`Init`20、`Next`の各アクション89、性質側51、`Spec`15。
  v3では構文エラーの1件ずつに意味の通る書き換え(連言を外す)での再検査か理由を付けたが、v4の記録にはそれが無い。
  性質側と`TypeOK`の71件は振る舞いを変えないが、`Init`・`Next`・`Spec`の124件(ガードや代入や公平性を外す書き換えに当たる)は再検査が要る。

到達状態を変えるのに緑のまま残った本物のsurvivorは、回せた変異の中では0件。
ただし構文エラーの124件は未検証で、検査の強さの証拠としては欠けている。

## 総括

判定表は全32行(状態機械にした20件の要件のうち、S-015が2行、S-016が4行、S-021が3行、S-028が2行、S-044が6行に分かれる)。

- `full`: 16件(S-010、S-012、S-014、S-016の制限時間とdoneを外す行、S-021×3、S-027、S-028の存在確認、S-033、S-041、S-044のreported_done・行き詰まり・終了状態・優先順位)
- `partial`(理由付き): 16件(S-002、S-005、S-006、S-011、S-013、S-015×2、S-016の探索モードと探索のゴール、S-017、S-022、S-023、S-028の決済導線、S-042、S-044のゴール達成と中断)
- `mismatch`/`vacuous`/`missing`: 0件

partialの行は「検証済み」に含めない。
S-015(段階の切り替え)とS-044(ゴール達成)は、「ユーザーの判断が要る点」の1の答えで判定が変わる。
(b)を選べばspecを直して検査と往復確認をやり直し、(a)を選べば理由欄を「要件がそう決めた」に改めてfullにできる。
