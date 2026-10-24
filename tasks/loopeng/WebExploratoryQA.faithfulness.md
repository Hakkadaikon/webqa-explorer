# 文面の往復確認: WebExploratoryQA

対象は、`.tla`で状態機械にしたEARS行(トレーサビリティ・マトリクスで「モデル外」でない行)。
読み手はfresh contextのsubagentで、入力は`WebExploratoryQA.faithfulness-input.md`だけ。
入力は現行specからコメントを除いた形式文面で、管理番号とEARS文を含まない。
入力が`.tla`からコメントと区切り線を除いたものと一致することは、行ごとの比較で確かめた(違いはモジュールの見出し行だけ)。
作成は2026-10-18。
設計メモ11の2点(段階に入った時点のdone_whenの観測、探索モードは1回判断してからゴールを判定)を入れたv5のspecに対して、全行を判定し直した。
v4の判定表で「ユーザーの判断が要る点」に挙げた2点は、設計メモ11の回答で決まった。
その後のレビューで、操作可能な要素が1つも無い画面を表せないこと(`RequestDecision`が対象を取らない操作でも`Elements`から対象を選び、`Elements = {}`で問い合わせが起きず`EventuallyTerminates`が破れる)が分かり、v6で直した。
v6では、対象を取らない操作の対象を`"none"`にし、click/inputで要素が無いときの対象を`"none"`(存在しない要素番号、応答の異常)にした(`ProposalTargets`)。
合わせて`ProposalWellFormed`と`DirectOpActedOnlyOnSafeTarget`を直し、`Elements = {}`の`WebExploratoryQA.empty.cfg`を足した。

v6の入力も、`.tla`からコメントと区切り線を除いたものと行ごとに一致することを確かめた(違いはモジュールの見出し行だけ)。
v6の入力を、v5とは別のfresh contextの読み手に渡して、全項目を言い直させ直した。
v6の読み手の言い直しの全文は、v6の判定のときに末尾の付録に写した。今の付録はv7の読み手のもので、v6の全文は`git log`で辿れる(`678289a`で付録を差し替える前の版)。
v6で変えた定義に関わる5行(S-005、S-006、S-012、S-016の探索モード、S-028の存在確認)は、言い直しの欄をv6の読み手の言葉に改めた。
ほかの行の言い直しは、v6の読み手もv5と同じ内容を返したことを、当時の付録(v6の全文)で確かめ、欄はそのままにした。
「読み手が気になった18点の分類」は、v6の読み手の18点で書き直した。
`WebExploratoryQA.paraphrase.md`はv4のspecに対する言い直しで、この判定には使っていない。

前件到達の証跡は`WebExploratoryQA.reach.log`の「v7」節を指す(表では「前件到達はP1」のように節の番号で書く)。v7節はv6節のP1〜P51をv7のspecで回し直し、P52〜P70を足したもの。
P44とP45は`WebExploratoryQA.noexplorable.cfg`(要素1つで、それが禁止要素)、P46〜P50とP63は`WebExploratoryQA.empty.cfg`(要素が無い)、ほかは`WebExploratoryQA.cfg`で回した。
検査の設定は3つとも、`StartsFresh`だけが状態述語の`PROPERTY`、ほかの状態述語はすべて`INVARIANT`。
`WebExploratoryQA.tlc-run.log`で、3つの設定のどれもNo errorだった。

## enter の追加(2026-10-20)

利用者が、キーを押す操作を足すと決めた。操作の種類に、フォーカスのある要素でEnterを押す`enter`を1つ足した(9種)。

- モデルでは、`enter`を対象の無い操作(`wait`や`scroll_down`と同じ)として扱う。`ASSUME Operations \subseteq {...}`に`"enter"`を足しただけで、`DirectOps`には足さない。3つの cfg の`Operations`は変えていない。
- 禁止要素がページにあれば`enter`を選択肢から外すことは、モデルの外(選択肢を組む前の絞り込み)とする。安全性の`NeverOperatesForbidden`は、要素を操作する click/input の性質で、`enter`は対象を取らないため、写せない。受け入れシナリオ「ページに禁止要素があれば、操作種別の選択肢に enter を含めない」で確かめる。
- 探索モードで、表示中に値のある入力欄が無ければ`enter`を外すことも、モデルの外とする。提案の操作は非決定に選ぶので、外した選択肢はその部分集合として含まれる。
- `ASSUME`を変えたので、TLCを3つの cfg で回し直した。結果は変更前と同じ(違反なし、状態数も同じ)。数字は`WebExploratoryQA.tlc-run.log`の末尾にある。

## インタビューと判断モデルの失敗の連続(2026-10-24、v7)

実装に入った3つの変更に、仕様を合わせた。実装を正とした。

- #86: `Modes`に`"interview"`を足した。段階とdone_whenは決め打ちシナリオと同じに扱う(`DoneWhenObservations`、`ScenarioStageDone`、`ScenarioGoalOnlyAtLastStage`を`mode # "explore"`にした)。インタビューでは、スクロールで見えた要素(`newSeen'`)で行き詰まりを数え直さない(`StuckAfterOp`、`StuckCounterTracksNewElements`)。InterviewMode S-047の「判定の方法は変えない」は、利用者の決定で#86によりインタビューで覆した。
- #92: ガードレールで止めた提案を、インタビューでは行き詰まりに1回数える(`ApplyGuardrails`の`Refuse`)。要素番号の無い応答の異常は止めたのではないので数えない。確かめ直しで見たページが許可ドメインの外なら、どのモードでも1回数えて外に出たことを記録する。両方が成り立っても1ステップに1回だけ数える(実装の`refused(settled(..))`)。性質`RefusalCountsStuck`を足し、`CountersChangeOnlyBySteps`、`DomainChangesOnlyByNavigation`、`ObservationChangesOnlyByExecution`を直した。カウンタが判断の数を超えない(`CountersBoundedByOps`)は変えずに成り立つ。
- #85: 判断モデルに届かない失敗を`RequestDecision`の非決定の結果に足した(`judgeFails`、`MaxJudgeFailures`)。失敗で1増え、答えが届けば使えない答えでも0に戻る。閾値に達したら`FinishJudgeUnavailable`(`error_exit`、`judge_unavailable`)で終える。待機の優先順位では、行き詰まりの次、判断予算切れと時間切れより先に採る(実装の`plan.rs`の`Pick::JudgeDown`)。性質`JudgeDownStops`、`JudgeFailsTracksReplies`、`JudgeUnavailableOnlyAfterFailures`、`JudgeFailsBounded`を足し、`AwaitingOnlyByDecision`、`LimitStops`、`ExitReasonMatchesPhase`を直した。検査の閾値は2にした(実装は3)。インタビューでその試行を打ち切り、以後の試行を始めないこと(InterviewMode S-105)は試行の外のループで、このspecの範囲の外。

`WebExploratoryQA.faithfulness-input.md`は、`tools/faithfulness_input.py`でv7の`.tla`から作り直した。

v7の入力を、v6とは別のfresh contextの読み手(general-purposeのsubagent)に渡し、全項目を言い直させた。
読み手は`WebExploratoryQA.faithfulness-input.md`だけを読み、ほかのファイルは開いていない。
読み手を起こしたのは呼び出し元で、仕様を直した担当ではない。
言い直しの全文は、この文書の末尾の付録に写した(v7のspecに対するもの)。

v7で判定し直したのは次の行で、言い直しの欄をv7の読み手の言葉に改めた。

- 新しい行: S-046、InterviewMode S-011(止めた提案を数える)、InterviewMode S-047(見えた要素を数えない)
- 式が変わった行: S-005、S-011(`ObservationChangesOnlyByExecution`)、S-015の段階の切り替えと入った時点のdone_when、S-021のドメイン外と行き詰まり、S-027、S-028の存在確認、S-033、S-041、S-044の優先順位と終了状態

上に挙げていない行は、v7の読み手もv6と同じ内容を返したことを付録で確かめ、欄はそのままにした。
「読み手が気になった26点の分類」は、v7の読み手の26点で書き直した。
`WebExploratoryQA.paraphrase.md`はv4の言い直しのままで、v7の判定には使えない。

前件到達は`WebExploratoryQA.reach.log`の「v7」節にある。
v6のP1〜P51(68本)をv7のspecで回し直し、v7で足したP52〜P70(21本)を加えた。89本すべてで、前件が成り立つ到達状態または一歩が見つかった。
P63(インタビューの応答の異常は数えない)は`WebExploratoryQA.empty.cfg`で回した。要素がある画面では、直接操作の要素番号が無い応答が起きないからである。
足した前件は、判断モデルの停止(P52)、`JudgeDownStops`の前件(P53)と、判断予算の最後の1回や時間切れとの同時(P54a、P54b)、予算が残ったままの停止(P54c)、失敗の一歩(P55)と0に戻る一歩(P56)、閾値に達した状態(P57)、失敗を挟んだ予算切れ(P58)、インタビューで止めた提案(P59)とそれ以外のモード(P60)、確かめ直しで外に着く一歩(P61、P62)、インタビューの応答の異常(P63)、外に着いて見えた要素が無くなる一歩(P64)、インタビューで見えた要素を数えない一歩(P65)、止めた提案で行き詰まりに届く一歩(P66)、インタビューのゴール達成(P67)、段階の切り替え(P68)、申告による終了(P69)、行き詰まりの数が判断の数に並ぶ状態(P70)である。

## 判定表

| 要件 | 式 | 読み手の言い直し | 判定 | 理由 / 直し方 |
|---|---|---|---|---|
| S-002 | `ExecuteNoChange`/`ExecuteChange`/`ExecuteOutOfDomain`、`SafeProposalProceeds`、`ProposalOutcome`、`VisitedChangesOnlyByExecution` | 「安全な提案はactingへ進むか環境事象」「actingの次は、done提案ならdone、stuck提案ならstuck、それ以外はidle」「訪問済みが変わるのはactingからidleへの遷移だけ」 | partial | 許可された提案が実行段階へ進み、実行の帰結が記録されることは検査している。ブラウザ操作という副作用は状態変数に現れない(model.mdの「モデル化で捨てた領域」)。前件到達はP21、P26、P27a〜c。 |
| S-005 | `RequestDecision`、`ProposalWellFormed`、`AwaitingOnlyByDecision`、`JudgeFailsTracksReplies` | 「ガードはContinuing、idleCounter < IdleThreshold、requestBudget > 0、inAllowedDomain。どちらの分岐でも予算を1ずつ減らす。成功の分岐はawaiting_decisionへ進んでjudgeFailsを0にし、opはProposableOpsから、tはProposalTargets(op)から非決定に選ぶ。失敗の分岐はphaseと提案を変えずにjudgeFailsを1増やす」「awaiting_decisionに入る遷移ではdecisionsMadeが1増える。decisionsMadeが1増える遷移は、awaiting_decisionに入るか、idleのままjudgeFailsが1増えるかのどちらか」 | partial | 問い合わせ(API呼び出し)そのものはモデル外で、どんな値が返っても扱えることを非決定の選択で表す。v6で、操作可能な要素が1つも無い画面でも問い合わせが起きるようにした。v7で、判断モデルに届かない失敗の分岐を足した(S-046の行)。前件到達はP15、P23、P46、P55。 |
| S-006 | `RequestDecision`、`ProposalTargets`、`ProposalWellFormed` | 「opがclickかinputで、Elementsが空でなければ、対象の候補は全要素(禁止要素も含む)。それ以外は対象なし(none)だけ」「targetExistsは、直接操作で対象がnoneのときだけ偽に固定」「awaitingで直接操作でなければtargetはnone。直接操作でtargetがnoneなら対象は存在しない。直接操作でElementsが空でなければtargetはElementsに属する」 | partial | S-005と同じ理由。言い直しはv6の`ProposalTargets`と`ProposalWellFormed`の意図(model.mdの`proposedTarget`の行)と一致する。v5は対象を取らない操作にも要素を持たせていたので、要素が無い画面を表せなかった。v6は`ProposalTargets`で、click/inputで要素があれば要素、それ以外は`"none"`を対象にする。`ProposalWellFormed`は、問い合わせの直後に対象を取らない操作の対象が`"none"`であること、click/inputの対象が`"none"`なら存在しない扱いであることを検査する。前件到達はP50(click/inputに要素番号が無い応答)、P51(対象を取らない操作の対象なし)。 |
| S-010 | `ApplyGuardrails`、`LowConfidenceBackNeverActed`、`SafeProposalProceeds` | 「ペイウォールでなく、低確信のbackなら却下してidleへ」「actingで、ペイウォールでなくbackなら低確信ではない」「安全な提案はactingへ進む」 | full | 却下する側と、却下しすぎない側の両方を検査している。ペイウォールの強制backを前件から外すのは、S-013による意図どおりの扱い。0.7の閾値は真偽値に丸めた(設計メモ8)。前件到達はP1、P21。 |
| S-011 | `ExecuteNoChange`、`IdleCounterTracksScreen`、`ObservationChangesOnlyByExecution` | 「ExecuteNoChangeでscreenChanged' = FALSE、idleCounterが1増える」「screenChangedが変わるのは、actingからidleへの遷移だけ」「newSeenが変わるのは、actingからidleへの遷移か、『awaiting_decisionで¬inAllowedDomain'かつ¬newSeen'』の遷移」 | partial | 画面が変わらなかったことの観測は表している。v7で、確かめ直しで許可ドメインの外に着いたら新しく見えた要素が無くなる(B4でnewSeenが偽になる)ことを足した。画面の変化の観測(固まり)には触れない。次に確信度の高い候補へ切り替える選択は、`RequestDecision`の非決定の選択に吸収され、モデル外。受け入れシナリオ「画面が変わらないのに同じ要素を選んだら…」で確かめる。前件到達はP28e、P37。 |
| S-012 | `ApplyGuardrails`、`NeverOperatesForbidden`、`DirectOpActedOnlyOnSafeTarget`、`SafeProposalProceeds` | 「禁止要素への直接操作は退けてidleに戻る」「訪れた要素に禁止要素は含まれない」「ペイウォールでないactingで直接操作なら、対象は存在し、Explorableに属する」 | full | 一致する。v6の`DirectOpActedOnlyOnSafeTarget`は、対象が`Explorable`(要素で、禁止要素でない)にあることを求め、`"none"`への操作も除く。禁止要素の分類ロジックの正しさは、設計メモ8とmodel.mdの`ForbiddenElements`節のとおり実装で確かめる。前件到達はP2、P21。 |
| S-013 | `ApplyGuardrails`のペイウォール分岐、`PaywallForcesBack`、`ProposalChangesOnlyByDecision` | 「ペイウォールなら操作をbackに差し替えてactingへ。doneやstuckの提案もbackになる」 | partial | 判断モデルの提案と無関係に既定の離脱操作をする点は一致する。specはペイウォールの観測が問い合わせの応答と同時に届くので、ペイウォールでも問い合わせて予算を1ずつ使う。「問い合わせない」は検査していない。予算を多めに使う側へのずれで、安全性には影響しない。実装は問い合わせの前に課金画面を判定して問い合わせを省く。受け入れシナリオ「課金画面が出たら判断モデルに問い合わせず…」「課金画面から離脱しても消えないまま3回続いたら…」で確かめる。前件到達はP22、P34。 |
| S-014 | `FinishNormally`、`FinishByProposalDone`、`ExploreGoalOnlyWhenAllExplored`、`ScenarioGoalOnlyAtLastStage`、`ReportedDoneOnlyWithoutDoneWhen`、`GoalPreempts`、`DoneWhenChangesOnlyByExecutionOrAdvance` | 「探索でgoal_achievedなら全部訪問済みで判断1回以上」「シナリオでgoal_achievedなら最後の段でdoneWhenMetが真」「reported_doneならExploreDoneもScenarioStageDoneも偽」「doneWhenMetが偽から真になるのは実行の完了か段の前進」 | full | ゴール達成は完了条件だけで決まり、判断モデルのdoneは未検証の終了として分けて記録する。前件到達はP3、P4、P5、P16a、P16b、P38a、P38b。 |
| S-015(段階の定義) | `stage`、`NumStages`、`LastStage` | 「段の数は1以上。シナリオではstage = NumStagesのときだけ最後の段」 | partial | 段階を状態機械に入れたが、YAMLでの定義の書き方と各段のdone_whenの中身はモデル外(model.md)。受け入れシナリオ「テスト対象・入力値・段階をYAMLの設定で定義して実行する」で確かめる。 |
| S-015(段階の切り替え) | `AdvanceStage`、`StageAdvancePreempts`、`StageAdvanceResetsCounters`、`BudgetSharedAcrossStages`、`TimeNeverRewinds` | 「AdvanceStageはexploreではLastStageが常に真なので、取れるのはscenarioとinterviewだけ」「idleでStageClearedなら、次は段が1進んでidleのままか環境事象」「段が変わるなら2つのカウンタは0」「段が変わる遷移で予算関係の4変数は変わらない」 | full | 段を進め、固まりと行き詰まりの回数を0に戻し、判断予算と時間窓を戻さないことは要件どおり。v4で問題にした切り替え直後の観測は、次の行で扱う。v7でインタビューも段階を持つようにした(InterviewMode S-010)。前件到達はP17、P30、P31、P68。 |
| S-015(入った時点のdone_when) | `Init`の`doneWhenMet`、`AdvanceStage`の`doneWhenMet'`、`DoneWhenObservations`、`StageAdvancePreempts`、`GoalPreempts`、`ScenarioGoalOnlyAtLastStage`、`DoneWhenChangesOnlyByExecutionOrAdvance` | 「scenarioとinterviewではInitでdoneWhenMetが真でも偽でもよい」「AdvanceStageはdoneWhenMetを観測し直す(真になってもよい)」「NumStages = 1なら判断0回でgoal_achievedになる。操作なしで全段を素通りできる」 | full | 実行の開始時と段階を進めた直後にdone_whenを観測し、満たしていれば問い合わせずに次の段階へ進むか、最後の段階ならゴール達成になる。`GoalPreempts`と`StageAdvancePreempts`が、その状態から問い合わせへ進まないことを検査する。読み手が気になった点2で挙げた「操作なしで全段を素通り」は、この行の要件そのもの。前件到達はP41(開始時に満たしている)、P42(進んだ直後に満たしている)、P43(判断0回でゴール達成)。 |
| S-016(探索モード) | `RequestDecision`の非決定の選択 | 「exploreモードではProposableOpsからdoneを除く」「targetはProposalTargets(op)から選ぶ」「直接操作を含まないOperationsでは、Explorableが空でないexploreはゴールに届かず、行き詰まりか予算切れで終わる」 | partial | v6で対象は`ProposalTargets`から選ぶ(S-006の行)。操作可能な要素が無い探索モードでも、1回問い合わせてからゴール達成になる(P47)。探索モードを持つことは表しているが、未操作の要素を優先する規則は非決定の選択に吸収した(model.mdの「捨てた領域」)。受け入れシナリオ「未操作の要素を、未探索の分岐が残る操作済み要素より先に選ぶ」で確かめる。 |
| S-016(探索のゴール) | `DoneWhen`、`ExploreDone`、`FinishNormally`、`ExploreGoalOnlyWhenAllExplored`、`GoalPreempts` | 「探索モードの終了条件は、探索可能な要素を全部訪問し、かつ判断を1回以上したこと」「Explorableが空なら、判断1回で達成になる。禁止要素へのclickを却下された場合も判断数は増えるので同じ」 | partial | 「1回以上問い合わせた後」と「探索してよい要素が無くても、最初の問い合わせの前にゴール達成としない」は一致する(設計メモ11の2)。読み手の気になった点3は、却下された判断も問い合わせ1回に数えることを指す。要件の「問い合わせた後」はそれを含むので、ずれではない。partialの理由はv4と同じで、specの「訪問済み」は「操作済み」で、要件の「探索し終えた」(遷移先まで探索し終えた。設計メモ9)とは別の概念。単一画面への簡略化の下では両者が一致する。複数画面での意味は、受け入れシナリオ「遷移先を探索し終えるまで、途中のボタンを探索し終えた要素として扱わない」で確かめる。前件到達はP3、P16a、P44(空で、問い合わせ前のidle)、P45(空のときのゴール達成)。 |
| S-016(制限時間) | `TimeElapses`、`FinishTimeUp`、`LimitStops`、`TimeExceededOnlyWhenTimeUp`、`ExitReasonMatchesPhase`、`TimeNeverRewinds`、`TimeElapsesAlone` | 「時間切れは終端でなければいつでも起きる」「idleで、終了条件が偽で、閾値未満で、予算切れか時間切れなら、次は対応する終端か環境事象」「time_exceededならtimeUpが真」 | full | 時間切れは時刻の経過として公平性なしで起き、次のidleで時間切れとして終わる。決め打ちシナリオでも起きる(設計メモ10の(b))。ステップの途中で時間が切れたら次のidleで終わるのは、優先順位の行(S-044の6行目)が待機状態の判定として定めたとおり。前件到達はP7d、P8、P9、P19b、P19c、P32、P33。 |
| S-016(doneを外す) | `ProposableOps`、`ProposalWellFormed`、`ExploreNeverReportedDone` | 「探索モードではProposableOpsからdoneを除く」「探索モードなら終了理由はreported_doneでない」 | full | 一致する。`ProposalWellFormed`はシステム側の`ProposableOps`を使わず生の変数で書いてある。前件到達はP6、P15。 |
| S-017 | `RecoverFromFreeze`/`ResumeAfterRecovery`、`RecoveryResetsPageState`、`RecoveryOnlyAfterFreeze`、`FreezePreemptsDecision`、`FreezeRecoveryRecorded`、`MemoryNeverShrinks` | 「Continuingで固まりの閾値以上ならfrozen_recoveredへ。idleCounterだけ0にする」「frozen_recoveredならidleCounterは0で許可ドメイン内」「固まりの閾値以上ならawaiting_decisionへ移らない」「訪問済みは減らない」 | partial | 復旧がページの状態だけを戻し、探索の記憶を残すことは検査している。固まりを検出したら必ず復旧することは性質として書いていない。問い合わせは禁止され、弱公平性の下でいずれ復旧か終了が起きるが、これは推論で検査していない。ドメイン外は復帰で、行き詰まりは終了で扱う(設計メモ2。行き詰まりで終えるのはS-044の優先順位による)。「ページが応答しない」は画面が変化しなかった1ケースとして固まりカウンタに合流させた(設計メモ9)。前件到達はP14、P20、P25。 |
| S-021(固まり) | `RecoverFromFreeze`のガード、`IdleCounterTracksScreen`、`CountersBoundedByOps`、`CountersChangeOnlyBySteps` | 「idleCounterが閾値以上」「実行で画面が変われば0、変わらなければ1増える」 | full | 一致する。「一定時間」は操作の回数で数える(設計メモ9)。前件到達はP28a、P28e、P35a。 |
| S-021(ドメイン外) | `ExecuteOutOfDomain`、`ApplyGuardrails`のB4(`Refuse(TRUE)`)、`DomainRecordAccurate`、`DomainChangesOnlyByNavigation` | 「ExecuteOutOfDomainでpageOutside真、inAllowedDomain偽」「存在しない通常要素への直接操作はB4でドメイン外へ出て拒否でき、モードにかかわらずstuckCounterが増える」「inAllowedDomainは常にpageOutsideの否定」「ドメインの2変数が変わるのは、actingからidle、B4の条件のawaitingからidle、inAllowedDomain'が真のidleからidleだけ」 | full | 記録が実際のページ位置と一致することを検査する。v7で、確かめ直しのために見たページが外だった場合(実装の確認中の観測)を足した。判定ロジック(URLの比較)の正しさは設計メモ8。前件到達はP36、P61、P62。 |
| S-021(行き詰まり) | `FinishStuck`のガード、`StuckAfterOp`、`Refuse`、`StuckCounterTracksNewElements`、`RefusalCountsStuck`、`StuckPreempts`、`NewSeenOnlyOnChangedAllowedPage` | 「StuckAfterOp: VisitedAfterOpがvisitedと変わらず、かつ『modeがinterviewであるかnewSeen'が偽』ならstuckCounter + 1、それ以外は0」「Refuse: outsideが真か、interviewで『直接操作で対象がnone』でないとき1増える。それ以外は据え置き」「awaitingからidleの遷移で、stuckCounter'はRefuseと同じ式(outsideの代わりに¬inAllowedDomain')」「idleで¬DoneWhenかつ閾値以上なら、次はstuckか環境事象」「newSeenが真なら、画面が変わり、許可ドメイン内で、判断1回以上」 | full | 探索済みの集合が増えず、未操作の要素も新しく見えない、という定義と一致する。許可ドメイン外の要素を新しく見えた要素に数えないこと(設計メモ10の1)も検査している。決め打ちシナリオと探索モードでは、ガードレールで止めた提案そのものは数えない(設計メモ9)。ただし確かめ直しのために見たページが許可ドメインの外なら、どのモードでもそのステップを1回数える(S-033の2行目、v7で足したEARS)。この2つはB1〜B3とB4の違いとして式に出ている。インタビューの数え方の違い(InterviewMode S-011、S-047)は下の2行で判定する。前件到達はP10、P18、P28b、P28c、P28d、P35b、P59〜P66。 |
| S-022 | `VisitedAfterOp`、`MemoryNeverShrinks`、`VisitedOnlyByOps`、`VisitedChangesOnlyByExecution` | 「click/inputなら対象を訪問済みに足す」「訪問済みは減らない。段が進んでも減らない」「訪問済みの要素数は判断数以下」 | partial | 1回の実行の中で記憶が失われないことは検査している。グラフ構造は単一の集合で近似し(model.mdの簡略化)、実行をまたぐ永続化は状態変数の外。受け入れシナリオ「探索の記憶を次の実行に引き継ぐ」などで確かめる。何も起きなかった input(入力欄でない対象、値の無い入力欄)は、モデルでは対象の無い実行(wait 相当)として扱う。記憶に足さず、行き詰まりの数は進む。検査する cfg の Operations に input は無い。前件到達はP26。 |
| S-023 | `RequestDecision`の非決定の選択 | 「提案の操作と対象をすべて非決定に選ぶ」 | partial | 優先順位の規則は非決定の選択に吸収した(model.mdの「捨てた領域」)。受け入れシナリオ「未操作の要素を…先に選ぶ」ほか3本で確かめる。 |
| S-027 | `Init`の`mode`、`ModeFixed` | 「modeは3つのモードから非決定に選び、変わらない」 | full | 3つのモードを同じ状態機械で扱う。モードの指定方法はモデル外だが、要件はそれを求めていない。前件到達はP4(シナリオ)、P6(探索)、P67(インタビュー)。 |
| S-028(存在確認) | `ApplyGuardrails`のB3・B4、`DirectOpActedOnlyOnSafeTarget`、`SafeProposalProceeds` | 「B3: 直接操作でtargetExistsが偽ならRefuse(FALSE)」「B4: 直接操作でtargetExistsが偽で、対象が禁止要素でもnoneでもなければRefuse(TRUE)。存在しない通常要素にはB3とB4の両方が取れ、どちらになるかは非決定」「ペイウォールでないactingで直接操作なら、targetExistsが真で、対象はExplorableに入る」 | full | 一致する。要素が無い画面でclick/inputが返った場合も、対象`"none"`で存在しない扱い(応答の異常)になり、却下される。v7で、確かめ直しのために見たページが許可ドメインの外の場合(B4)を足した。存在の判定(ノードの同一性)は設計メモ8。前件到達はP2、P21、P50、P61。 |
| S-028(決済導線) | `ApplyGuardrails`、`NeverOperatesForbidden` | 「禁止要素へのclick/inputは却下」「訪問済みに禁止要素は含まれない」 | partial | specは判断モデルが禁止要素を提案しうるとして、ガードで操作を止めることを検査する。要件の「選択してはならない」(探索の候補に入れない)は非決定の選択に吸収され、モデル外。設計メモ9のとおり、実装は禁止要素全体を候補から外す。受け入れシナリオ「禁止要素は未操作のまま残っていても、探索の候補に入れない」「探索モードでも決済導線の要素は選ばない」で確かめる。 |
| S-033 | `ExecuteOutOfDomain`/`ReturnToAllowedDomain`、`ApplyGuardrails`のB4、`RequestDecision`のガード、`DomainRecordAccurate`、`NoDecisionOutsideDomain`、`DomainChangesOnlyByNavigation` | 「外へ出たことを記録する(ExecuteOutOfDomainとB4)」「idleからawaiting_decisionへ移るとき、pageOutsideは偽」「Continuingで外にいるなら内へ戻る。RequestDecisionはinAllowedDomainを要求するので、先に戻る」 | full | 遷移の記録と、ドメイン外で探索しないことの両方を検査している。2行目(確かめ直しで中止して外に着いたら、どのモードでも記録して行き詰まりを1回数える。インタビューで止めた分と重なっても1回)は、B4と`RefusalCountsStuck`の「¬inAllowedDomain'なら1増える」に当たり、言い直しと一致する。前件到達はP29、P36、P61、P62。 |
| S-041 | `RequestDecision`/`ExceedDecisionBudget`/`FinishTimeUp`、`DecisionsWithinCap`、`BudgetAccounting`、`BudgetExceededOnlyWhenSpent`、`DecisionBudgetNeverRestored`、`LimitStops`、`TerminalAbsorbing`、`EventuallyTerminates` | 「decisionsMade ≤ MaxDecisions」「decisionBudget + decisionsMade = MaxDecisions」「idleで¬DoneWhen、stuckCounter < StuckThreshold、judgeFails < MaxJudgeFailures、かつ予算切れか時間切れなら、次は対応する終端か環境事象」「decisionBudgetは増えない」「いずれphaseが終端になる。TimeElapsesとUserAbortが起きなくても、決定予算が減っていくことだけで成り立つ」 | full | 呼び出し回数は上限を超えず、上限か制限時間に達したら次の待機状態で止まり、再開しない。上位の終了条件(ゴール達成、行き詰まり、判断モデルの停止)が先に採られるのは、承認済みの優先順位のとおり。判断モデルに届かない失敗も1回に数えるのは、実装が問い合わせの前に予算を使うのと同じ(気になった点7)。前件到達はP13、P19a、P19b、P19c、P58。 |
| S-042 | `RequestDecision`/`ResetRequestWindow`、`RequestsWithinWindowCap`、`BudgetAccounting`、`BudgetSharedAcrossStages` | 「リクエスト予算が残っているときだけ問い合わせる」「Continuingなら時間の条件なしでウィンドウを戻せる」「ウィンドウ内のリクエスト数はMaxRequests以下」 | partial | 1つの窓で上限を超えないことは検査している。窓が時間の条件なしで満了するのは、要件の「どの時点でも満了しうる」のとおり(設計メモ7)。1窓の実時間の長さはモデル外。数える単位もずれる。specは問い合わせごとに数えるが、実装は対象サーバへのアクセス(ブラウザ操作、課金画面の離脱、固まり復旧の再読み込み、許可ドメインへの戻り)を数え、ガードで止めた提案は数えない(設計メモ9でユーザーが承認)。受け入れシナリオ「ページの再読み込みや許可ドメインへの戻りも…」「時間窓の操作回数が上限に達していたら…」「ガードレールで止めて…」で確かめる。前件到達はP31。 |
| S-044(ゴール達成) | `FinishNormally`、`GoalPreempts`、`ExploreGoalOnlyWhenAllExplored`、`ScenarioGoalOnlyAtLastStage`、`ExitReasonMatchesPhase` | 「idleでGoalReachedなら、次はdone・goal_achievedか環境事象」 | full | 完了条件の観測が成り立てば、他の終了理由や継続に先取りされずゴール達成で終わる。v4でpartialにした理由(最後の段に入った直後と開始時に観測しない)は、v5の`Init`と`AdvanceStage`の観測で解消した。前件到達はP3、P4、P16a、P16b、P43。 |
| S-044(reported_done) | `FinishByProposalDone`、`ProposalOutcome`、`ReportedDoneOnlyWithoutDoneWhen`、`ExploreNeverReportedDone` | 「actingでdone提案ならdone・reported_done」「reported_doneならExploreDoneもScenarioStageDoneも偽」 | full | 一致する。最後の段でなくてもdoneの提案で実行全体が終わるのは、要件の「実行を未検証の終了として記録」のとおり。課金画面のときにdoneの提案がbackに置き換わるのはS-013による。前件到達はP5、P27a。 |
| S-044(行き詰まり) | `FinishByProposalStuck`/`FinishStuck`、`StuckPreempts`、`ProposalOutcome`、`ExitReasonMatchesPhase` | 「stuck提案ならstuckで終わる」「idleで行き詰まりの閾値以上なら次はstuckか環境事象」「stuckなら終了理由はstuck」 | full | `StuckPreempts`は終了理由を見ないが、`ExitReasonMatchesPhase`が段階と終了理由の対応を検査するので、合わせて一致する。前件到達はP7b、P18、P27b。 |
| S-044(中断) | `UserAbort`、`ExitReasonMatchesPhase`、`TerminationKeepsState` | 「終端でなければaborted・user_abortedで終わる」「終端へ入る遷移では段階と終了理由のほかは変わらない」 | partial | どの時点でも中断を受け付け、中断として記録して終わることは検査している。記憶・終了のイベント・実行結果の書き出しはモデル外(設計メモ10の3)。受け入れシナリオで確かめる。前件到達はP7e、P39a、P39b。 |
| S-044(終了状態) | `TerminalAbsorbing`、`Next`の最後の選択肢、`TerminationKeepsState` | 「終端にいるなら変数は何も変わらない」「終端へ入る遷移では段階と終了理由のほかは変わらない」 | full | 中断を含め、終了後は状態が変わらない。`TimeElapses`と`UserAbort`も終端では起きない。v7で足した判断モデルの停止(`error_exit`、`judge_unavailable`)も同じ終端に入る(`ExitReasonMatchesPhase`の「error_exitならbudget_exceededかjudge_unavailable」)。前件到達はP39a、P40、P52。 |
| S-044(優先順位) | `Exploring`/`Running`/`Continuing`の各ガード、`GoalPreempts`、`StageAdvancePreempts`、`StuckPreempts`、`JudgeDownStops`、`LimitStops`、`FreezePreemptsDecision`、`NoDecisionOutsideDomain`、`EnvEvent` | 読み手は「phaseがidleのときの優先順位(式から読めるもの)」を並べた(付録の4節)。「1. DoneWhenが真ならAdvanceStageかFinishNormallyだけ」「2. 次に閾値以上ならFinishStuckだけ」「3. 次にjudgeFails ≥ MaxJudgeFailuresならFinishJudgeUnavailableだけ。予算切れや時間切れより先」「4. 次に予算切れか時間切れ。両方なら非決定」「5. 残りはContinuingで、固まりなら復旧、外なら復帰、ResetRequestWindowは並んで取れる」「TimeElapsesとUserAbortは終端でなければ割り込める」 | full | 要件の順序と一致する。判断モデルの停止は3段目の先頭で、予算切れと時間切れより先(S-046、実装の`plan.rs`)。予算切れと時間切れが同時に成り立てばどちらでもよいことも、要件の文面どおり(設計メモ10の2)。前件到達はP16a、P16b、P17、P18、P19a〜c、P20、P29、P41、P42、P53、P54a、P54b。 |
| S-046 | `RequestDecision`の失敗の分岐、`FinishJudgeUnavailable`、`JudgeDownStops`、`JudgeFailsTracksReplies`、`JudgeUnavailableOnlyAfterFailures`、`JudgeFailsBounded`、`ExitReasonMatchesPhase` | 「失敗の分岐: judgeFailsが1増え、phaseと提案は変わらずidleのまま。成功の分岐ではjudgeFails' = 0」「FinishJudgeUnavailableのガードはRunningとjudgeFails ≥ MaxJudgeFailures。予算や時間は確かめない」「idleで¬DoneWhen、stuckCounter < StuckThreshold、judgeFails ≥ MaxJudgeFailuresなら、次はerror_exitかつjudge_unavailableか環境事象」「judgeFailsかdecisionsMadeが変わる遷移では、decisionsMadeが1増え、awaiting_decisionならjudgeFails'は0、idleなら1増える」「judge_unavailableならjudgeFails ≥ MaxJudgeFailures」 | partial | 失敗が閾値の回数続いたら、判断予算と制限時間が残っていても判断モデルの停止として終えることは一致する(P54c)。答えが届けば使えない答えでも0に戻るのは、要件の「形の崩れた応答や使えない応答は失敗に数えず、失敗の数を0に戻す」のとおり。使えない答えは要素番号の無い直接操作として表した。partialの理由は3つで、どれもT-123で担保する。失敗の種類(APIのエラー、タイムアウト、橋渡しの失敗)の区別はモデル外で、単体`three_judge_failures_in_a_row_end_the_run`(3種とも数える)と`unusable_answers_are_not_judge_failures`(使えない答えは0に戻す)で確かめる。終了コード3はモデル外で、単体`judge_unavailable_exits_3`と受け入れシナリオ「判断モデルの失敗が3回続いたら、予算を使い切る前に終了コード3で終わる」で確かめる。閾値は検査では2で、実装の3は同じ受け入れシナリオ(問い合わせ回数が3回)と`three_judge_failures_in_a_row_end_the_run`で確かめる。抽象化の記録は抽出台帳の設計メモ8。前件到達はP52〜P58。 |
| InterviewMode S-011(止めた提案を数える) | `ApplyGuardrails`のB1〜B4、`Refuse`、`RefusalCountsStuck`、`CountersChangeOnlyBySteps`、`CountersBoundedByOps` | 「interviewでは拒否(Refuse(FALSE))でもstuckを数える。scenarioとexploreでは数えず、idleCounterも変えない」「interviewの例外: 直接操作で対象がnoneなら数えない。効くのはElementsが空のときだけ」「B4ではモードにかかわらずstuckCounterが増え、両方が成り立っても1回」「stuckCounterが変わるのは、actingかawaiting_decisionからidleへの遷移か、stageが1増える遷移」「stuckCounter ≤ decisionsMade」 | full | インタビューでガードレールが止めた提案と確かめ直しでの中止を行き詰まりに1回数え、固まりの回数は変えない、という#92の決定と一致する。応答の異常を数えないのは実装の`refused`が止めた提案だけに掛かるため。実装のテストは T-143 の`an_unusable_answer_is_not_counted_as_stuck_in_an_interview`。要素の参照を持たない提案も存在の確かめ直しに合流させたのはmodel.mdの「簡略化の明示」のとおりで、数え方は変わらない。前件到達はP59、P60、P62、P63、P66、P70。 |
| InterviewMode S-047(見えた要素を数えない) | `StuckAfterOp`、`StuckCounterTracksNewElements` | 「interviewでは、実行後はnewSeenを見ずにstuckを数える」「VisitedAfterOpがvisitedと変わらなければ、interviewならnewSeen'によらず1増える。visitedが増えれば0」 | full | スクロールで新しく見えた未操作の要素を進展に数えず、未操作の要素を操作したときは今までどおり0に戻す、という#86の決定と一致する。探索モードと決め打ちシナリオは変えていない(P28c)。前件到達はP65。 |

## 読み手が気になった26点の分類

番号はv7の読み手の番号(付録の「気になった点」)。
v6の18点と同じ内容のものは、v6の分類を引き継いだ。

### 1. 決め打ちシナリオとインタビューは判断0回でゴール達成になりうる

要件どおりで、問題ない(v6の2)。
S-015の3行目(設計メモ11の1)は、実行の開始時と段階を進めた直後にdone_whenを確かめ、満たしていれば操作せずに進むと定めた。
インタビューは段階とdone_whenを決め打ちシナリオと同じに扱う(InterviewMode S-044「目的のdone_whenを満たしたら、判断モデルに問い合わせずにゴール達成」)。
前件到達はP41、P42、P43、P67。

### 2. 探索してよい要素が空のとき、判断1回で達成になる

要件どおりで、問題ない(v6の3)。
v7では判断モデルに届かない失敗も問い合わせ1回に数えるので、失敗1回でも達成になる。
要件の「判断モデルに1回以上問い合わせた後に」は問い合わせの回数を言い、答えが届いたかを条件にしていない。
実装の`explore_goal`も`budget.used()`(失敗を含む問い合わせの回数)で判定する。

### 3. 存在しない要素への直接操作で、ドメイン内のまま拒否(B3)とドメイン外へ出て拒否(B4)の両方が取れる

意図どおりで、問題ない。
B4は、実装の確認中に見直したページが許可ドメインの外だった場合を表す(#92、`refused(settled(..))`)。
どちらになるかはページ次第なので、環境の非決定で表した。
外に着いたらモードによらず行き詰まりを1回数えるのは、実装の`settled`がどのモードでも数えるため。
model.mdの「簡略化の明示(InterviewMode S-011)」に書いた。

### 4. ペイウォールではbackへの書き換えで対象と低確信が残る

要件どおりで、問題ない(v6の11)。
S-013の既定の離脱操作は判断モデルの提案によらないので、S-010の確信度は掛からない。
残った対象と低確信は、ペイウォールを前件から外した性質しか読まない。

### 5. backはOperationsに無くてもペイウォールで強制される

要件どおりで、問題ない。
S-013の離脱操作は設定の`leave_paywall`で決まり、判断モデルに見せる操作種別とは別である。

### 6. 時間窓に時間の条件が無く、上限の不変条件が冗長

要件どおりで、問題ない(v6の10)。

### 7. 判断モデルに届かない失敗でも予算が減り、判断の数に入る

実装どおりで、問題ない。
実装は問い合わせの前に判断予算を使う(`waiting.rs`の`ask`)。
答えが届かなくても予算は戻らないので、失敗が続けば予算切れか判断モデルの停止のどちらかで止まる(S-046はその前に止めるための要件)。
時間窓に数える単位がずれることは、S-042の行に書いたとおり(specは問い合わせごとに数える)。

### 8. ゴール達成と段の前進はContinuingを要求しない

要件どおりで、問題ない(v6の7、13)。
S-044の6行目は、待機状態でdone_when成立を最上位に置いている。
v7で足した判断モデルの停止も、done_when成立より下である(実装の`plan.rs`の行の順)。

### 9. 予算切れと時間切れが同時ならどちらでもよい

要件どおりで、問題ない(設計メモ10の2)。
実装は予算切れを先に採る。仕様はその1つを含む。

### 10. 判断モデルの停止は予算切れと時間切れより先、行き詰まりはそのどれよりも先

要件どおりで、問題ない。
S-046と#85の決定は、判断モデルの停止を3段目の先頭に置き、予算の最後の1回で閾値に達したら判断予算切れより先に採るとした(P54a)。

### 11. timeUpを見るのはidleのときだけ

要件どおりで、問題ない(v6の13)。

### 12. 実行の3つの結果は操作の種類によらず選べる

モデル化の方針どおりで、問題ない(v6の7、12)。

### 13. 段の前進は探索の記憶、ドメイン、時間窓を戻さない

承認済みの決定どおりで、問題ない(v6の16、設計メモ10の(c))。

### 14. 直接操作の対象がnoneになるのは要素が空のときだけ

意図どおりで、問題ない(v6の4)。
インタビューの例外(応答の異常は数えない)が効くのも要素が空のときだけになる。
要素がある画面では、範囲の外の要素番号が返る応答の異常を、モデルは要素番号の無い応答として表していない。
実装ではどちらも応答の異常で、行き詰まりに数えない点は同じである。
そのため前件到達のP63は`WebExploratoryQA.empty.cfg`で回した。

### 15. インタビューだけ行き詰まりの数え方が違い、ほかのモードでは却下が続いても予算でしか止まらない

承認済みの決定どおりで、問題ない。
インタビューの数え方は#86と#92の決定(利用者)で、探索モードと決め打ちシナリオは変えないと決めている。
ほかのモードで却下を数えないのは設計メモ9の4つ目のとおり(v6の6)。

### 16. 申告によるdoneとstuckは段階もカウンタも見ずに終わる

要件どおりで、問題ない。
S-044の要件は、doneを未検証の終了、stuckを行き詰まりとして記録して終えることで、段階を条件にしていない。

### 17. `StagesOnlyInScenario`の名前と中身がずれる

名前の話で、要件とのずれではない。
v7でインタビューも段階を持つようにしたので、名前が中身より狭くなった。
式は「探索モードでは段階が1で、done_whenを観測しない」で、要件のとおり。
名前を直すかどうかは、「仕様の直しを検討してよいもの」の2に挙げた。

### 18. `RecoveryResetsPageState`と`RecoveryOnlyAfterFreeze`が弱い

モデル化の方針どおりで、問題ない(v6の8、9)。

### 19. `StartsFresh`は初期状態にしか効かない

承認済みの扱いどおりで、問題ない(v6の15)。
v7で`judgeFails = 0`を足した。`judgeFails`は`JudgeFailsBounded`(問い合わせの回数以下)も初期状態で縛る。

### 20. 冗長な箇所がある

検査の強さは変異の検査で確かめており、問題ない(v6の14)。
v7で足した`RefusalCountsStuck`は、`Refuse`を参照せず式を別に書き、`outside`の代わりに観測後の`~inAllowedDomain'`を使う。
`Refuse`の更新式を書き換える変異はこの性質で捕まった(`WebExploratoryQA.mutants-v7.tsv`の`L231`の各行)。
`JudgeFailsBounded`の第1項が`TypeOK`と重なるのは、実装の全状態の探索のテストが確かめる「失敗の回数は閾値を超えない」を文面に出すため。

### 21. 「Preempts」「Stops」は次に起きる変化を言うだけで、起きることを保証しない

モデル化の方針どおりで、問題ない(v6の18)。
起きることは`Spec`の弱公平性と`EventuallyTerminates`が担う。

### 22. 終了の性質がphaseだけを指定する

問題ない。
終了理由は`ExitReasonMatchesPhase`で検査する。
`error_exit`には2つの理由があるので、`JudgeDownStops`は終了理由まで指定した。
`ExceedDecisionBudget`の終了理由を`judge_unavailable`に変える書き換えは、`JudgeUnavailableOnlyAfterFailures`が捕まえた(`WebExploratoryQA.mutants-v7-recheck.tsv`の`eq->neq#107`)。

### 23. `Elements`の有限性と、`"none"`や`"back"`を含まないことを仮定していない

検査の設定で守っており、問題ない。
3つの`.cfg`の`Elements`はモデル値の有限集合で、文字列`"none"`と`"back"`を含まない。

### 24. 提案できる操作が空でも、判断モデルの停止で終わる

v6の1(提案できる操作が空だと止まる)が、v7で変わった。
v7の`RequestDecision`では、提案を選ぶ`\E op \in ProposableOps`は答えの分岐の中にあり、失敗の分岐は提案を選ばない。
そのため`ProposableOps`が空でも失敗の分岐だけは取れ、判断モデルの停止か予算切れで終わる。
`Operations = {"done"}`の設定(探索モードで`ProposableOps`が空)で、TLCは違反なしだった(`WebExploratoryQA.tlc-run.log`の末尾)。
ただし「提案できる操作が無いと判断モデルは必ず届かない」という振る舞いは実装に無い。
実装は設定値の検査で空の操作種別を拒否するので、この状態に来ない。
要件とのずれではないが、`ASSUME`で除くかどうかを「仕様の直しを検討してよいもの」の1に挙げた。

### 25. ペイウォールと、対象を取らない操作の存在は、提案と独立に選ばれる

要件どおりで、問題ない(v6の11、17)。

### 26. 中断と時刻の経過に公平性が無く、終了は予算だけで成り立つ

モデル化の方針どおりで、問題ない。
S-041の判断予算は、どの経路でも問い合わせごとに減る。v7で足した失敗の分岐も予算を使うので、この構造は変わらない。

## 仕様を直す必要があるもの

v7の読み手の言い直しで判定し直した判定表に、mismatch、vacuous、missingの行は無い。
v7の読み手の26点にも、要件とのずれは無かった。
仕様は直していない。

## 仕様の直しを検討してよいもの

どれも要件とのずれではなく、直さなくても判定は変わらない。

### 1. 提案できる操作が空の場合を`ASSUME`で除く

v6では`EventuallyTerminates`の前提だったが、v7では無くても終わる(気になった点24)。
代わりに、実装に無い「提案できる操作が無いと判断モデルが必ず届かない」振る舞いがモデルに入る。
`ASSUME Operations \ {"done"} # {}`を足せば、この振る舞いを除ける。
`Elements`はページの中身なので、空でない前提を置いてはいけない。

### 2. `StagesOnlyInScenario`の名前を直す

インタビューも段階を持つので、名前を`StagesNotInExplore`のように中身に合わせてよい(気になった点17)。
名前を変えると、`.cfg`と台帳とmodel.mdの対応も直す。

## 空成立の証跡(含意・前提付きの式)

`WebExploratoryQA.reach.log`の「v7」節(P1〜P70、89本)に、v7のspecに対する前件ごとの結果を追記してある。
各節に`NotPx`の定義をそのまま書き、89本すべてで反例(前件が成り立つ到達状態または一歩)が出ている。
P1〜P51(68本)はv6節と同じ前件をv7のspecで回し直したもの。v5とv6で足した前件は次のとおり。

- P41: 開始時に最初の段のdone_whenを満たしている状態(`StartsFresh`、`StageAdvancePreempts`)
- P42: 段を進めた直後に次の段のdone_whenを満たしている一歩(`DoneWhenChangesOnlyByExecutionOrAdvance`、`StageAdvanceResetsCounters`)
- P43: 判断0回でゴール達成になる一歩(`ScenarioGoalOnlyAtLastStage`)
- P44: 探索してよい要素が空で、問い合わせる前のidle(`ExploreGoalOnlyWhenAllExplored`、`GoalPreempts`)
- P45: 探索してよい要素が空のときのゴール達成(`ExploreGoalOnlyWhenAllExplored`)
- P46〜P51: 操作可能な要素が無い場合の問い合わせ、ゴール達成、対象を取らない操作の実行、要素番号の無い応答(v6。P46〜P50は`WebExploratoryQA.empty.cfg`)

v7で足したのは次の21本。P63は`WebExploratoryQA.empty.cfg`で回した。

- P52: 判断モデルの停止での終了(`JudgeUnavailableOnlyAfterFailures`、`ExitReasonMatchesPhase`)
- P53、P54a、P54b: `JudgeDownStops`の前件と、判断予算の最後の1回や時間切れと同時の場合
- P54c: 判断予算が残っているのに停止で終わる状態(`JudgeUnavailableOnlyAfterFailures`)
- P55、P56: 判断モデルに届かない一歩と、失敗の後に答えが届いて0に戻る一歩(`AwaitingOnlyByDecision`、`JudgeFailsTracksReplies`)
- P57: 失敗が閾値に達した状態(`JudgeFailsBounded`)
- P58: 失敗を挟んだ判断予算切れ(`LimitStops`)
- P59、P60: インタビューとほかのモードでガードレールが止めた一歩(`RefusalCountsStuck`)
- P61、P62: 確かめ直しで許可ドメインの外に着いた一歩と、それがインタビューの場合(`RefusalCountsStuck`、`DomainChangesOnlyByNavigation`)
- P63: インタビューの応答の異常(数えない)(`RefusalCountsStuck`)。実装側は T-143 の単体 `an_unusable_answer_is_not_counted_as_stuck_in_an_interview`(インタビューで要素番号の無い click の答えは応答の異常になり、カウンタは (0,0) のまま。同じ規則の疑わしい back では (0,1))で確かめる
- P64: 外に着いて新しく見えた要素が無くなる一歩(`ObservationChangesOnlyByExecution`)
- P65: インタビューで新しく見えた要素を数えない一歩(`StuckCounterTracksNewElements`)
- P66: 止めた提案で行き詰まりの閾値に届く一歩(`StuckPreempts`)
- P67〜P69: インタビューのゴール達成、段階の切り替え、申告による終了(`ScenarioGoalOnlyAtLastStage`、`StageAdvancePreempts`、`ReportedDoneOnlyWithoutDoneWhen`)
- P70: インタビューで行き詰まりの数が判断の数に並ぶ状態(`CountersBoundedByOps`)

前件を持たない式(`NeverOperatesForbidden`、`BudgetAccounting`、`MemoryNeverShrinks`、`ModeFixed`、`DecisionBudgetNeverRestored`、`EventuallyTerminates`など)は対象外である。
同ログのv6以前の節は旧spec向けで、この判定には使っていない。

## 定数の前提

`EventuallyTerminates`は、`ASSUME`で正と定めた閾値と上限の下での主張である。
v6では`ProposableOps`が空でない前提も要ったが、v7では要らない(気になった点24)。
`Elements`はページの中身で、空でもよい。
検査に使う`WebExploratoryQA.cfg`は、`Operations`がclickを含み空でなく、`IdleThreshold`(2)が`MaxDecisions`(4)以下なので、固まり復旧の前件にも届く(P14、P25)。
`MaxJudgeFailures`(2)も`MaxDecisions`(4)以下なので、予算を残したままの停止(P54c)と、予算の最後の1回での停止(P54a)の両方に届く。
`WebExploratoryQA.noexplorable.cfg`は要素1つでそれが禁止要素の場合、`WebExploratoryQA.empty.cfg`は操作可能な要素が1つも無い場合を検査する。
実装では、設定値の検査で0以下の閾値、空の操作種別、探索モードでclickもinputも無い操作種別を拒否する。

## 変異の検査(v7のspec)

691件の変異を、3つの設定で回した。全件の結果は`WebExploratoryQA.mutants-v7.tsv`(列はv6と同じ)、再検査は`WebExploratoryQA.mutants-v7-recheck.tsv`にある。
手元のマシンで、TLCはワーカー2、ヒープ1GBで回した。

- 性質違反でkilledになったもの: 239件。主の設定で225件、`StartsFresh`の初期状態での違反10件、主の設定では緑で`WebExploratoryQA.noexplorable.cfg`か`WebExploratoryQA.empty.cfg`で捕まったもの4件。
- 評価時のエラーか構文エラーで止まり、`Init`・`Next`・`Spec`側にあるもの: 再検査した。224件はすべての書き換えが性質違反になった。値ごとの書き換え(`tools/recheck_values.py`のeq、drop、item)で2306通り、連言を外すと代入が欠ける6件は状態を変えない書き換えか片方の枝だけにする書き換え7通りで回した。
- 構文エラーで止まった性質側の変異: 92件。`Init`・`Next`・`Spec`が原本と同じなので再検査していない。
- `Spec`の`[][Next]_vars`の連言1件: 外すと次状態関係が無くなりTLCで検査できないので、未検査のまま残る(v6と同じ)。
- equivalent: 111件。性質側の変異57件、連言リストの先頭の`/\`が1項の`\/`になるもの30件、コメント内の字面15件、再検査で`.cfg`の定数と同じ値になる書き換え6件、冪等な自己ループが増えるだけの書き換え3件(時間窓の巻き戻しのガード2件、許可ドメインへの戻りのガード1件。どれもdistinctの到達状態数が原本と同じ560893)。
- abstraction-limited: 24件。どれも要件が許す環境側の振る舞いを減らすだけで、誤った振る舞いを足さない。判断モデルの応答や画面・done_whenの観測を1つの値に固定する書き換え、実行の結果の1つを起こらなくする書き換え、時刻の経過を起こらなくする書き換えに加えて、v7では判断モデルに届かない失敗を起こらなくする(または常に起こす)書き換えと、確かめ直しで外に着く結果を起こらなくする書き換えが入った。

到達状態を変えるのに緑のまま残った本物のsurvivorは0件。
239 + 224 + 92 + 1 + 111 + 24 = 691件。

abstraction-limitedのうち、`RequestDecision`の`op \in DirectOps /\ t = "none"`を`\/`にする変異(`and->or#65`)と、対象の存在の観測を偽に固定する変異(`eq->neq#38`、`and->or#70`)は、直接操作が一度も実行されないspecになるのに、どの性質も破れない。
性質はどれも「起きたら正しい」を言い、「安全な直接操作が実行されうる」を言わないからである。前件到達のP2、P21、P26がこの可能性を別に確かめている。
v6の同じ変異(`and->or#60`)は、到達状態が82744に減っていたのに「箇条の先頭」のequivalentに分類されていた。v7でabstraction-limitedに直した。

## 変異の検査(v6のspec)

590件の変異を、3つの設定(`WebExploratoryQA.cfg`、`WebExploratoryQA.noexplorable.cfg`、`WebExploratoryQA.empty.cfg`)で回した。
v6の直しは`RequestDecision`(どの経路も通る)を変え、主の設定の到達状態は307979から232571に変わった。
変えた行だけ回すと、ほかの行の変異がまだ捕まるかを確かめられないので、全件を回し直した。
全件の結果は`WebExploratoryQA.mutants-v6.tsv`(7列目が理由、8列目が`WebExploratoryQA.noexplorable.cfg`、9列目が`WebExploratoryQA.empty.cfg`での結果)、再検査は`WebExploratoryQA.mutants-v6-recheck.tsv`にある。
この節は、往復確認の判定がよって立つ検査の強さを、TSVから数え直して写したもの。

- 性質違反でkilledになったもの: 200件。
  内訳は主の設定で行動性質116、不変条件70、時相性質1、`StartsFresh`の初期状態での違反9、主の設定では緑で`WebExploratoryQA.noexplorable.cfg`で捕まったもの3、`WebExploratoryQA.empty.cfg`だけで捕まったもの1。
  最後の1件は`RequestDecision`の`t = "none"`を`t # "none"`にした変異で、要素が無いときに`ProposalWellFormed`の違反になる。
- 評価時のエラーで止まったもの: 105件。
  killedの証拠には数えず、値を1つずつ代入して再検査した。
  95件はどの値でも性質違反になり、10件は一部の値で緑のまま残ってabstraction-limitedとした。
- 構文エラーで止まったもの: 198件。
  74件は性質側の変異で、`Init`・`Next`・`Spec`が原本と同じなので再検査していない。
  `Init`・`Next`・`Spec`側の124件のうち、`Spec`の`[][Next]_vars`の連言1件は、外すと次状態関係が無くなりTLCで検査できないので再検査していない。
  残る123件は意味の通る書き換え(ガードを外す、代入を値ごとにする、公平性を外す)で再検査した。
  108件はすべての書き換えが性質違反になった。
  15件は緑のまま残る書き換えがあり、そのうち9件がabstraction-limited、6件がequivalent(`.cfg`の定数と同じ値になる4件、冪等な自己ループが増えるだけの2件)だった。
- equivalent: 93件。
  変異のままで緑だった87件(性質側の変異46、連言リストの先頭の`/\`が1項の`\/`になるもの27、コメント内の字面13、時間窓の巻き戻しのガードを変えるもの1)と、上の再検査の6件。
  時間窓の巻き戻しの1件と、自己ループの2件のうち許可ドメインへの戻りのガードを外す1件は、`-fp 0 -dump dot`で自己ループを除いた到達グラフのsha256が原本と一致することを確かめた。
- abstraction-limited: 19件(上の10件と9件)。
  どれも要件が許す環境側の振る舞いを減らすだけで、誤った振る舞いを足さない。
  判断モデルの応答や画面・done_whenの観測を1つの値に固定する書き換え、実行の結果の1つを起こらなくする書き換え、時刻の経過を起こらなくする書き換えに当たる。
  v5では判断モデルの提案の操作と対象の固定も緑で残ったが、v6では`ProposalWellFormed`を強めたのでkilledになった。

到達状態を変えるのに緑のまま残った本物のsurvivorは0件で、TLC で検査できない `[][Next]_vars` の1件を除き、未検証の変異も残っていない。
200 + 93 + 19 + 74 + 1 = 387件に、再検査でkilledにした95 + 108 = 203件を足すと590件になる。

## 総括

判定表は全36行(状態機械にした20件の要件のうち、S-015が3行、S-016が4行、S-021が3行、S-028が2行、S-044が6行に分かれる。v7でS-046と、InterviewModeのS-011、S-047の3行を足した)。

- `full`: 21件(S-010、S-012、S-014、S-015の段階の切り替えと入った時点のdone_when、S-016の制限時間とdoneを外す行、S-021×3、S-027、S-028の存在確認、S-033、S-041、S-044のゴール達成・reported_done・行き詰まり・終了状態・優先順位、InterviewMode S-011、InterviewMode S-047)
- `partial`(理由付き): 15件(S-002、S-005、S-006、S-011、S-013、S-015の段階の定義、S-016の探索モードと探索のゴール、S-017、S-022、S-023、S-028の決済導線、S-042、S-044の中断、S-046)
- `mismatch`/`vacuous`/`missing`: 0件

共有手順の終了条件(全行がfullか理由付きのpartial、入力・判定表・空成立の証跡が`tasks/`に残っている)を満たす。
partialの15行は「検証済み」に含めない。
各行の理由欄に書いたモデル外の部分が、未保証の範囲である。

## 付録: 読み手の言い直し(v7)

以下は、v7の入力を渡した読み手のsubagentが、`WebExploratoryQA.faithfulness-input.md`だけを読んで返した文面を、そのまま写したもの。

````markdown
# WebExploratoryQA の式を日本語で言い直す

## 1. 定数と ASSUME

- **MaxDecisions**、**MaxRequests**、**IdleThreshold**、**StuckThreshold**、**MaxJudgeFailures**、**NumStages**: どれも 1 以上の自然数(`Nat \ {0}`)。
- **Elements**: 要素の集合。有限であることや、文字列 `"none"`・`"back"` を含まないことは仮定されていない。
- **ForbiddenElements**: Elements の部分集合。
- **Operations**: `{"click", "input", "scroll_down", "scroll_up", "back", "enter", "wait", "done", "stuck"}` の部分集合。空でもよく、`"back"` を含まなくてもよい。

変数は次の 23 個。mode、stage、phase、proposedOp、proposedTarget、proposedLowConf、isPaywall、targetExists、screenChanged、newSeen、doneWhenMet、visited、idleCounter、stuckCounter、inAllowedDomain、pageOutside、timeUp、decisionBudget、decisionsMade、requestBudget、requestsInWindow、judgeFails、exitReason。**vars** はこの 23 個を並べた組。

## 2. 補助定義

- **Modes**: `{"scenario", "explore", "interview"}`。
- **Phases**: idle、awaiting_decision、acting、done、stuck、frozen_recovered、error_exit、timed_out、aborted の 9 つ。
- **Terminal**: done、stuck、error_exit、timed_out、aborted の 5 つ。idle、awaiting_decision、acting、frozen_recovered は含まない。
- **ExitReasons**: none、goal_achieved、reported_done、stuck、budget_exceeded、judge_unavailable、time_exceeded、user_aborted の 8 つ。
- **DirectOps**: `{"click", "input"}`。
- **Explorable**: Elements から ForbiddenElements を除いた集合。
- **ProposableOps**: mode が explore なら Operations から `"done"` を除いた集合。それ以外のモードなら Operations そのもの。
- **ProposalTargets(op)**: op が DirectOps に入り、かつ Elements が空でなければ Elements 全体を返す。禁止要素もここに含まれる。それ以外は `{"none"}` を返す。
- **DoneWhen**: explore では「visited が Explorable に等しい」かつ「decisionsMade > 0」。それ以外のモードでは doneWhenMet。
- **LastStage**: mode が explore であるか、stage が NumStages に等しいかのどちらか。
- **DoneWhenObservations**: mode が explore でなければ BOOLEAN。explore なら `{FALSE}`。
- **Exploring**: phase が idle で、DoneWhen が偽。
- **Running**: Exploring で、かつ stuckCounter < StuckThreshold。
- **Continuing**: Running で、かつ judgeFails < MaxJudgeFailures、decisionBudget > 0、timeUp が偽。
- **Refuse(outside)**: アクションの部品で、次の値を決める。
  - phase' = idle。
  - stuckCounter' は、outside が真のとき、または「mode が interview で、しかも『proposedOp が DirectOps に入り proposedTarget が none』ではない」ときに 1 増える。それ以外は据え置き。
  - pageOutside' = outside、inAllowedDomain' = ¬outside。
  - newSeen' = newSeen ∧ ¬outside。outside が真なら newSeen は偽になる。
- **VisitedAfterOp**: proposedOp が DirectOps に入っていれば visited ∪ {proposedTarget}、そうでなければ visited。
- **StuckAfterOp**: VisitedAfterOp が visited と変わらず、かつ「mode が interview であるか newSeen' が偽」のとき stuckCounter + 1。それ以外は 0。newSeen' という次状態の値を参照している。
- **ObserveDoneWhen**: doneWhenMet' を DoneWhenObservations から非決定に選ぶ。explore では常に FALSE になる。
- **ExploreDone**: mode が explore で、visited = Explorable かつ decisionsMade > 0。
- **ScenarioStageDone**: mode が explore でなく、doneWhenMet が真。interview もここに入る。
- **GoalReached**: ExploreDone であるか、ScenarioStageDone かつ stage = NumStages であるか。
- **StageCleared**: ScenarioStageDone かつ stage ≠ NumStages。
- **EnvEvent**: 次のどちらかが起きた遷移。
  - phase' が aborted になり、exitReason' が user_aborted になる。
  - timeUp が偽から真に変わる。
- 補足として、「¬ExploreDone ∧ ¬ScenarioStageDone」は ¬DoneWhen と同じ意味になる。

## 3. TypeOK と Init

**TypeOK**
- mode は Modes の要素、stage は 1..NumStages、phase は Phases の要素。
- proposedOp は Operations ∪ {"back", "none"}。back は Operations に無くても許される。
- proposedTarget は Elements ∪ {"none"}。
- 真偽値の変数は proposedLowConf、isPaywall、targetExists、screenChanged、newSeen、doneWhenMet、inAllowedDomain、pageOutside、timeUp。
- visited は Elements の部分集合。
- idleCounter、stuckCounter、decisionsMade、requestsInWindow は上限のない Nat。
- decisionBudget は 0..MaxDecisions、requestBudget は 0..MaxRequests、judgeFails は 0..MaxJudgeFailures。
- exitReason は ExitReasons の要素。

**Init**
- mode は 3 つのモードから非決定に選ぶ。
- 固定される値は次のとおり。stage = 1、phase = idle、proposedOp = proposedTarget = "none"、proposedLowConf = FALSE、isPaywall = FALSE、targetExists = TRUE、screenChanged = TRUE、newSeen = FALSE、visited = {}、idleCounter = stuckCounter = 0、inAllowedDomain = TRUE、pageOutside = FALSE、timeUp = FALSE、decisionBudget = MaxDecisions、decisionsMade = 0、requestBudget = MaxRequests、requestsInWindow = 0、judgeFails = 0、exitReason = "none"。
- doneWhenMet は DoneWhenObservations から選ぶ。explore では FALSE、scenario と interview では TRUE と FALSE のどちらもありうる。

## 4. Next の各選択肢

**RequestDecision**
- ガード: Continuing、idleCounter < IdleThreshold、requestBudget > 0、inAllowedDomain。
- どちらの分岐でも次の値が変わる。decisionBudget が 1 減り、decisionsMade が 1 増え、requestBudget が 1 減り、requestsInWindow が 1 増える。
- 分岐は 2 つで、どちらになるかは非決定。
  - 成功の分岐: phase' = awaiting_decision、judgeFails' = 0。続けて次の値を非決定に選ぶ。
    - op を ProposableOps から選ぶ。
    - lc と pw を BOOLEAN から選ぶ。
    - t を ProposalTargets(op) から選ぶ。
    - ex は、op が直接操作で t が none のとき FALSE に固定される。それ以外は BOOLEAN から選ぶ。
    - 選んだ値は proposedOp、proposedTarget、proposedLowConf、isPaywall、targetExists にそれぞれ入る。
  - 失敗の分岐: judgeFails が 1 増える。phase と提案の 5 変数は変わらないので、phase は idle のまま。
- 変わらない変数: mode、stage、screenChanged、newSeen、doneWhenMet、visited、idleCounter、stuckCounter、inAllowedDomain、pageOutside、timeUp、exitReason。
- ProposableOps が空なら、成功の分岐は取れず失敗の分岐だけが残る。

**ResetRequestWindow**
- ガード: Continuing、requestBudget < MaxRequests。
- 変わる変数: requestBudget' = MaxRequests、requestsInWindow' = 0。ほかはすべて変わらない。
- 時間やウィンドウの経過を表す条件は無い。リクエストを 1 回でも使えば、いつでも取れる。

**ApplyGuardrails**
- ガード: phase = awaiting_decision。
- 分岐 A(isPaywall が真のとき):
  - phase' = acting、proposedOp' = back。
  - proposedTarget、proposedLowConf、stuckCounter、inAllowedDomain、pageOutside、newSeen は変わらない。
- 分岐 B(isPaywall が偽のとき)には次の 5 つの選択肢があり、どれを取っても proposedOp、proposedTarget、proposedLowConf は変わらない。
  - B1: proposedOp が back で proposedLowConf が真なら、Refuse(FALSE)。
  - B2: 直接操作で、対象が禁止要素なら、Refuse(FALSE)。
  - B3: 直接操作で、targetExists が偽なら、Refuse(FALSE)。
  - B4: 直接操作で、targetExists が偽で、対象が禁止要素でも none でもなければ、Refuse(TRUE)。ページが許可ドメインの外に出て、stuckCounter が 1 増える。
  - B5: 「back かつ低確信度」ではなく、かつ「直接操作でない、または対象が禁止要素でなく存在する」なら、phase' = acting。stuckCounter、ドメイン、newSeen は変わらない。
- どの分岐でも変わらない変数: mode、stage、isPaywall、targetExists、screenChanged、doneWhenMet、visited、idleCounter、timeUp、予算の 4 変数、judgeFails、exitReason。
- 優先順位: ペイウォールが最優先で、A と B は排他になる。B の中では、B5 の条件が B1∨B2∨B3 のちょうど否定なので、拒否と実行は排他になる。存在しない通常要素に対しては B3 と B4 が両方とも取れ、どちらになるかは非決定。B2 と B3 が両方成り立つときは、結果は同じになる。

**ExecuteNoChange**
- ガード: phase = acting、proposedOp が done と stuck のどちらでもない。
- 変わる変数: visited' = VisitedAfterOp、phase' = idle、screenChanged' = FALSE、newSeen' = FALSE、doneWhenMet' は ObserveDoneWhen で選ぶ、idleCounter が 1 増える、stuckCounter' = StuckAfterOp。
- 提案の変数、ドメイン、timeUp、予算、judgeFails、exitReason、mode、stage は変わらない。

**ExecuteChange**
- ガードは ExecuteNoChange と同じ。
- 変わる変数: visited' = VisitedAfterOp、phase' = idle、screenChanged' = TRUE、newSeen' は BOOLEAN から非決定に選ぶ、doneWhenMet' は ObserveDoneWhen で選ぶ、idleCounter' = 0、stuckCounter' = StuckAfterOp。
- 変わらない変数は ExecuteNoChange と同じ。

**ExecuteOutOfDomain**
- ガードは ExecuteNoChange と同じ。
- 変わる変数: visited' = VisitedAfterOp、phase' = idle、screenChanged' = TRUE、newSeen' = FALSE、doneWhenMet' は ObserveDoneWhen で選ぶ、idleCounter' = 0、stuckCounter' = StuckAfterOp、pageOutside' = TRUE、inAllowedDomain' = FALSE。
- 提案の変数、timeUp、予算、judgeFails、exitReason、mode、stage は変わらない。
- 3 つの Execute アクションは、操作の種類(wait や scroll なども含む)にかかわらず、どれでも非決定に選べる。

**ReturnToAllowedDomain**
- ガード: Continuing、¬inAllowedDomain。
- 変わる変数: pageOutside' = FALSE、inAllowedDomain' = TRUE。ほかはすべて変わらない。

**FinishByProposalDone**
- ガード: phase = acting、proposedOp = done。
- 変わる変数: phase' = done、exitReason' = reported_done。ほかは変わらない。
- DoneWhen も stage も確かめない。

**FinishByProposalStuck**
- ガード: phase = acting、proposedOp = stuck。
- 変わる変数: phase' = stuck、exitReason' = stuck。ほかは変わらない。stuckCounter の値は確かめない。

**RecoverFromFreeze**
- ガード: Continuing、idleCounter ≥ IdleThreshold。
- 変わる変数: phase' = frozen_recovered、idleCounter' = 0。ほかは変わらない。ドメインは変えない。

**ResumeAfterRecovery**
- ガード: phase = frozen_recovered。
- 変わる変数: phase' = idle。ほかは変わらない。

**AdvanceStage**
- ガード: phase = idle、DoneWhen、¬LastStage。explore では LastStage が常に真なので、取れるのは scenario と interview だけ。
- 変わる変数: stage が 1 増える、doneWhenMet' は DoneWhenObservations から選び直す、idleCounter' = 0、stuckCounter' = 0。
- visited、newSeen、ドメイン、timeUp、予算、judgeFails は変わらない。
- Continuing を要求しないので、予算切れ、時間切れ、判断の失敗、ドメインの外のどの状況でも取れる。

**FinishNormally**
- ガード: phase = idle、DoneWhen、LastStage。
- 変わる変数: phase' = done、exitReason' = goal_achieved。ほかは変わらない。
- Continuing を要求しない。

**FinishStuck**
- ガード: Exploring(idle かつ ¬DoneWhen)、stuckCounter ≥ StuckThreshold。
- 変わる変数: phase' = stuck、exitReason' = stuck。ほかは変わらない。

**FinishJudgeUnavailable**
- ガード: Running(idle、¬DoneWhen、stuckCounter < StuckThreshold)、judgeFails ≥ MaxJudgeFailures。
- 変わる変数: phase' = error_exit、exitReason' = judge_unavailable。ほかは変わらない。
- 予算や時間は確かめない。

**ExceedDecisionBudget**
- ガード: Exploring、stuckCounter < StuckThreshold、judgeFails < MaxJudgeFailures、decisionBudget = 0。
- 変わる変数: phase' = error_exit、exitReason' = budget_exceeded。ほかは変わらない。

**FinishTimeUp**
- ガード: Exploring、stuckCounter < StuckThreshold、judgeFails < MaxJudgeFailures、timeUp。
- 変わる変数: phase' = timed_out、exitReason' = time_exceeded。ほかは変わらない。

**TimeElapses**
- ガード: ¬timeUp、phase が終端でない。
- 変わる変数: timeUp' = TRUE。ほかはすべて変わらない。awaiting_decision、acting、frozen_recovered のときにも起こりうる。

**UserAbort**
- ガード: phase が終端でない。
- 変わる変数: phase' = aborted、exitReason' = user_aborted。ほかは変わらない。

**終端での足踏み**
- phase が終端のとき、vars 全体を変えない遷移を許す。

**phase が idle のときの優先順位(式から読めるもの)**
1. DoneWhen が真なら、AdvanceStage(最終ステージでないとき)か FinishNormally(最終ステージのとき)だけが取れる。Continuing を前提にするアクションは取れない。
2. DoneWhen が偽で stuckCounter ≥ StuckThreshold なら、FinishStuck だけが取れる。
3. 次に、judgeFails ≥ MaxJudgeFailures なら FinishJudgeUnavailable だけが取れる。予算切れや時間切れより先になる。
4. 次に、decisionBudget = 0 か timeUp なら、ExceedDecisionBudget か FinishTimeUp。両方成り立つときは非決定で、どちらが先かの順位は無い。
5. 残りは Continuing の状態。
   - idleCounter ≥ IdleThreshold なら RecoverFromFreeze を取り、RequestDecision は取れない。
   - ¬inAllowedDomain なら ReturnToAllowedDomain を取る。RequestDecision は inAllowedDomain を要求するので、先に戻る必要がある。
   - ResetRequestWindow は requestBudget < MaxRequests の間、ほかのアクションと並んで常に取れる。
- TimeElapses と UserAbort は、終端でなければどの phase でも割り込める。

## 5. 不変条件と性質

### 状態不変条件

- **NeverOperatesForbidden**: visited と ForbiddenElements は交わらない。
- **LowConfidenceBackNeverActed**: acting で、ペイウォールでなく、op が back なら、proposedLowConf は偽。
- **DirectOpActedOnlyOnSafeTarget**: acting で、ペイウォールでなく、op が直接操作なら、targetExists が真で、対象は Explorable に入る。
- **ExploreGoalOnlyWhenAllExplored**: explore で exitReason が goal_achieved なら、visited = Explorable かつ decisionsMade > 0。
- **ScenarioGoalOnlyAtLastStage**: explore 以外で goal_achieved なら、stage = NumStages かつ doneWhenMet。
- **ReportedDoneOnlyWithoutDoneWhen**: reported_done なら、ExploreDone も ScenarioStageDone も成り立たない。
- **ExploreNeverReportedDone**: explore なら、exitReason は reported_done でない。
- **ExitReasonMatchesPhase**: phase と exitReason が次のように対応する。
  - done なら goal_achieved か reported_done。
  - stuck なら stuck。
  - error_exit なら budget_exceeded か judge_unavailable。
  - timed_out なら time_exceeded。
  - aborted なら user_aborted。
  - 終端でなければ none。
- **TimeExceededOnlyWhenTimeUp**: time_exceeded なら timeUp が真。
- **StagesOnlyInScenario**: explore なら stage = 1 で、doneWhenMet は偽。
- **NewSeenOnlyOnChangedAllowedPage**: newSeen が真なら、screenChanged が真で、inAllowedDomain が真で、decisionsMade > 0。
- **NoChangeObservedOnlyAfterOps**: screenChanged が偽なら decisionsMade > 0。
- **NoProposalBeforeDecision**: decisionsMade = 0 なら、提案の 5 変数は初期値のまま。
- **RequestsWithinWindowCap**: requestsInWindow ≤ MaxRequests。
- **DecisionsWithinCap**: decisionsMade ≤ MaxDecisions。
- **BudgetAccounting**: decisionBudget + decisionsMade = MaxDecisions、かつ requestBudget + requestsInWindow = MaxRequests。
- **BudgetExceededOnlyWhenSpent**: budget_exceeded なら decisionBudget = 0。
- **JudgeUnavailableOnlyAfterFailures**: judge_unavailable なら judgeFails ≥ MaxJudgeFailures。
- **JudgeFailsBounded**: judgeFails ≤ MaxJudgeFailures、かつ judgeFails ≤ decisionsMade。
- **CountersBoundedByOps**: idleCounter ≤ decisionsMade、かつ stuckCounter ≤ decisionsMade。
- **VisitedOnlyByOps**: visited の要素数 ≤ decisionsMade。
- **RecoveryOnlyAfterFreeze**: frozen_recovered なら decisionsMade ≥ IdleThreshold。
- **RecoveryResetsPageState**: frozen_recovered なら idleCounter = 0 かつ inAllowedDomain。
- **DomainRecordAccurate**: inAllowedDomain は常に ¬pageOutside に等しい。
- **ProposalWellFormed**: phase が awaiting_decision か acting のとき、次がすべて成り立つ。
  - op は Operations ∪ {back}、対象は Elements ∪ {none} に入る。
  - awaiting_decision で op が直接操作でなければ、対象は none。
  - 直接操作で対象が none なら、targetExists は偽。
  - 直接操作で Elements が空でなければ、対象は Elements に入る。
  - explore なら op は done でない。
- **StartsFresh**: 次をすべて満たす状態述語。phase = idle、stage = 1、visited = {}、¬timeUp、exitReason = none、idleCounter = stuckCounter = 0、inAllowedDomain、explore なら ¬doneWhenMet、decisionBudget と requestBudget は最大値、judgeFails = 0。

### 行動性質と時相性質

以下の `[][...]_vars` はどれも、vars を変えない足踏みを許す。条件を課すのは、何かが変わる遷移だけ。

- **GoalPreempts**: idle で GoalReached なら、次の遷移は「done かつ goal_achieved」か EnvEvent になる。
- **StageAdvancePreempts**: idle で StageCleared なら、次の遷移は「idle のまま stage が 1 増える」か EnvEvent になる。
- **StuckPreempts**: idle で ¬DoneWhen かつ stuckCounter ≥ StuckThreshold なら、次は phase' = stuck か EnvEvent。exitReason は指定しない。
- **JudgeDownStops**: idle で ¬DoneWhen、stuckCounter < StuckThreshold、judgeFails ≥ MaxJudgeFailures なら、次は「error_exit かつ judge_unavailable」か EnvEvent。
- **LimitStops**: idle で ¬DoneWhen、stuckCounter < StuckThreshold、judgeFails < MaxJudgeFailures、かつ「decisionBudget = 0 または timeUp」なら、次は次のどれか。exitReason は指定しない。
  - decisionBudget = 0 で error_exit に移る。
  - timeUp で timed_out に移る。
  - EnvEvent。
- **FreezePreemptsDecision**: idle で idleCounter ≥ IdleThreshold なら、次の phase は awaiting_decision にならない。
- **SafeProposalProceeds**: awaiting_decision で、ペイウォールでなく、「back かつ低確信度」でなく、直接操作なら対象が禁止要素でなく存在する。このとき次は acting か EnvEvent。
- **AwaitingOnlyByDecision**: 次の 2 つを言う。
  - awaiting_decision に入る遷移では、decisionsMade が 1 増える。
  - decisionsMade が 1 増える遷移は、awaiting_decision に入るか、「idle のまま judgeFails が 1 増える」かのどちらか。
- **JudgeFailsTracksReplies**: judgeFails か decisionsMade が変わる遷移では、次がすべて成り立つ。
  - decisionsMade が 1 増える。
  - phase' は awaiting_decision か idle。
  - judgeFails' は、awaiting_decision なら 0、idle なら 1 増える。
- **ActingOnlyAfterGuardrails**: acting に入るのは awaiting_decision からだけ。
- **FreezeRecoveryRecorded**: idle、¬DoneWhen、idleCounter ≥ IdleThreshold のときに idleCounter' = 0 になるなら、phase' は frozen_recovered。
- **VisitedChangesOnlyByExecution**: visited が変わるのは、acting から idle への遷移だけ。
- **PaywallForcesBack**: awaiting_decision でペイウォールなら、次は「acting かつ op' = back」か EnvEvent。
- **ProposalOutcome**: acting からの次の phase は、op が done なら done、stuck なら stuck、それ以外なら idle。または EnvEvent。
- **IdleCounterTracksScreen**: acting から idle への遷移では、screenChanged' が真なら idleCounter' = 0、偽なら 1 増える。
- **MemoryNeverShrinks**: visited は減らない。
- **StuckCounterTracksNewElements**: acting から idle への遷移では、stuckCounter' は StuckAfterOp と同じ式で決まる。
- **RefusalCountsStuck**: awaiting_decision から idle への遷移では、stuckCounter' は Refuse と同じ式で決まる。outside の代わりに ¬inAllowedDomain' を使う。
- **NoDecisionOutsideDomain**: idle から awaiting_decision への遷移では、pageOutside は偽。
- **StageAdvanceResetsCounters**: stage が変わる遷移は、次をすべて満たす。
  - idle から idle への遷移で、遷移前に StageCleared が成り立つ。
  - stage は 1 だけ増える。
  - idleCounter と stuckCounter は 0 になる。
- **BudgetSharedAcrossStages**: stage が変わる遷移では、予算の 4 変数が変わらない。
- **DecisionBudgetNeverRestored**: decisionBudget は増えない。
- **TimeNeverRewinds**: timeUp は一度真になれば真のまま。
- **TimeElapsesAlone**: timeUp が偽から真になる遷移では、ほかの変数はすべて変わらない。
- **ModeFixed**: mode は変わらない。
- **ProposalChangesOnlyByDecision**: 提案の 5 変数の組が変わるのは、次のどちらかのときだけ。
  - awaiting_decision に入る遷移。
  - awaiting_decision でペイウォールのとき、op' = back になり、ほかの 4 変数が変わらない遷移。
- **CountersChangeOnlyBySteps**: 次の 2 つを言う。
  - idleCounter が変わるのは、acting から idle への遷移、frozen_recovered に入る遷移、stage が 1 増える遷移のどれか。
  - stuckCounter が変わるのは、acting または awaiting_decision から idle への遷移か、stage が 1 増える遷移。
- **DomainChangesOnlyByNavigation**: ドメインの 2 変数が変わるのは、次のどれかのときだけ。
  - acting から idle への遷移。
  - awaiting_decision から idle への遷移で、¬inAllowedDomain'、直接操作、¬targetExists、対象が禁止要素でも none でもないとき。
  - idle から idle への遷移で inAllowedDomain' が真のとき。
- **ObservationChangesOnlyByExecution**: 次の 2 つを言う。
  - screenChanged が変わるのは、acting から idle への遷移だけ。
  - newSeen が変わるのは、acting から idle への遷移か、「awaiting_decision で ¬inAllowedDomain' かつ ¬newSeen'」の遷移。
- **DoneWhenChangesOnlyByExecutionOrAdvance**: 次の 2 つを言う。
  - doneWhenMet が偽から真になるのは、acting から idle への遷移か、stage が 1 増える遷移。
  - 真から偽になるのは、stage が 1 増える遷移だけ。
- **TerminationKeepsState**: 終端に入る遷移では、phase と exitReason 以外の変数がすべて変わらない。
- **EventuallyTerminates**: いずれ phase が終端になる。
- **TerminalAbsorbing**: 終端では vars 全体が変わらない。

## 6. Spec

**Spec** は Init ∧ □[Next]_vars に、次の弱公平性を加えたもの。
- 個別に弱公平性がかかるアクション: RequestDecision、ResetRequestWindow、ApplyGuardrails、ReturnToAllowedDomain、FinishByProposalDone、FinishByProposalStuck、RecoverFromFreeze、ResumeAfterRecovery、AdvanceStage、FinishNormally、FinishStuck、FinishJudgeUnavailable、ExceedDecisionBudget、FinishTimeUp。
- 3 つの Execute は、論理和にまとめて 1 つの弱公平性がかかる。
- TimeElapses と UserAbort には公平性が無い。起きても起きなくてもよい。

## 気になった点

1. scenario と interview では、Init の doneWhenMet が TRUE になりうる。この場合、判断を 1 回もせずに FinishNormally(goal_achieved)や AdvanceStage が起きる。AdvanceStage のあとも doneWhenMet' がまた TRUE になりうるので、判断ゼロのままステージを連続で進められる。explore だけが decisionsMade > 0 を要求しており、モードによって扱いが違う。
2. explore で Explorable が空のときは、判断を 1 回(判断の失敗も含む)した時点で visited = {} = Explorable が成り立ち、ゴールに達する。decisionsMade は判断の失敗でも増えるので、操作を一度もしないまま goal_achieved になりうる。
3. ApplyGuardrails で、存在しない通常要素への直接操作には B3(ドメイン内のまま拒否)と B4(ドメイン外へ出て拒否)の両方が取れ、どちらになるかは非決定。「対象が存在しない」ことと「ドメインの外に出る」ことのつながりは、式からは読み取れない。B4 ではモードにかかわらず stuckCounter が増える。
4. ペイウォールの分岐では、op だけを back に書き換え、proposedTarget と proposedLowConf は元のまま残す。そのため acting で「op = back、対象が要素、低確信度」という状態が生まれる。LowConfidenceBackNeverActed と DirectOpActedOnlyOnSafeTarget はペイウォールを除外し、ProposalWellFormed は「非直接操作なら対象は none」を awaiting_decision に限っている。どれもこの状態を許すように書かれている。
5. back は Operations に入っていなくても、ペイウォールのときに強制される(TypeOK が back を別に許している)。
6. ResetRequestWindow には時間の条件が無く、リクエストを 1 回使えばいつでも予算を戻せる。そのためリクエストの上限は何も制約しない。RequestsWithinWindowCap と DecisionsWithinCap は、BudgetAccounting と TypeOK からそのまま導ける冗長な性質。
7. 判断の失敗(judge 失敗)でも decisionBudget と requestBudget が減り、decisionsMade が増える。BudgetAccounting や VisitedOnlyByOps の「決定数」には失敗した回も入っている。
8. FinishNormally と AdvanceStage は Continuing を要求しない。時間切れ、予算切れ、judge 失敗の上限、stuck の閾値超え、ドメインの外の状態でも、ゴールとステージ進行が優先される。
9. 予算切れと時間切れが同時に成り立つとき、ExceedDecisionBudget と FinishTimeUp のどちらになるかは非決定。LimitStops もどちらでもよいと書いている。
10. FinishJudgeUnavailable は予算と時間を確かめないので、judge の停止が予算切れや時間切れより優先される。FinishStuck は、judge、予算、時間のどれも確かめない。
11. timeUp を見るのは phase が idle のときだけ。awaiting_decision、acting、frozen_recovered の途中で時間切れになっても、操作の実行まで進む。
12. Execute の 3 つの結果は、操作の種類(wait、scroll、back、enter なども)によらず選べる。どの操作でもドメインの外に出たり、doneWhenMet が TRUE になったりしうる。ExecuteOutOfDomain と ExecuteNoChange でも、クリックした要素は visited に入る。
13. AdvanceStage は visited、newSeen、inAllowedDomain を戻さない。ドメインの外にいてもステージは進む。requestBudget もステージの間で引き継がれる。
14. 直接操作の対象が none になるのは、Elements が空のときだけ("none" ∉ Elements と仮定した場合)。したがって、Refuse の interview の例外「直接操作かつ対象が none なら stuck を増やさない」と ProposalWellFormed の第 4 項が効くのは、Elements = {} のときに限られる。
15. interview モードの stuck の数え方は、ほかのモードと違う。実行後は newSeen を見ずに stuck を数え、拒否(Refuse(FALSE))でも数える。scenario と explore では、Refuse(FALSE) は stuck を増やさず、idleCounter も変えない。拒否が繰り返されても、止めるのは決定予算だけになる。
16. FinishByProposalDone はステージを見ずに実行全体を終える。最終ステージでなくても reported_done で終わる。FinishByProposalStuck も stuckCounter を見ずに終える。
17. StagesOnlyInScenario は名前が scenario だが、式は「explore なら」としか言っていない。interview にもステージがある。
18. RecoveryResetsPageState の inAllowedDomain の部分は、RecoverFromFreeze が設定しているのではない。到達可能性からたまたま成り立っている。RecoveryOnlyAfterFreeze は、CountersBoundedByOps と RecoverFromFreeze のガードから導ける弱い性質。
19. StartsFresh は状態述語なので、不変条件として検査すると、phase が idle から離れた時点で偽になる。初期状態の性質としてしか意味を持たない。どちらとして使うかは式からは読めない。proposal、newSeen、screenChanged、decisionsMade、requestsInWindow、pageOutside は条件に入っていない。
20. 冗長な箇所がいくつかある。
    - DomainRecordAccurate が成り立つので、pageOutside と inAllowedDomain は片方で足りる。
    - JudgeFailsBounded の第 1 項と、ProposalWellFormed の第 1・2 項は TypeOK と同じ。
    - DirectOpActedOnlyOnSafeTarget の ¬isPaywall は、ペイウォールなら op = back になるので効いていない。
    - StuckCounterTracksNewElements と RefusalCountsStuck は、StuckAfterOp と Refuse の定義を写したもの。
    - ExploreNeverReportedDone は ProposableOps の定義から直接出る。
    - Next の最後の「終端での足踏み」は、[Next]_vars がもともと足踏みを許すので意味の上では効かない(TLC のデッドロック検査を避ける役目はありうる)。
21. 「Preempts」や「Stops」と名の付く性質はどれも □[A]_vars の形で、足踏みを許す。「次に変化が起きるならこれ」と言うだけで、実際に起きることは保証しない。起きることを保証しているのは、Spec の弱公平性と EventuallyTerminates。FreezePreemptsDecision と FreezeRecoveryRecorded も、回復が起きることまでは言わない。
22. StuckPreempts、LimitStops、ProposalOutcome は、phase だけを指定して exitReason を指定しない。exitReason の整合は、ExitReasonMatchesPhase に頼っている。
23. Elements が有限であることを ASSUME していないが、VisitedOnlyByOps は Cardinality を使う。Elements が "none" や "back" を含まないことも仮定していない。
24. Operations は空でもよい。空のときや、explore で Operations ⊆ {"done"} のときは ProposableOps が空になる。このとき RequestDecision は失敗の分岐しか取れず、結果として judge_unavailable で終わる。
25. isPaywall と、非直接操作のときの targetExists は、提案と独立に非決定に選ばれる。非直接操作では targetExists が何にも使われない。
26. UserAbort と TimeElapses には公平性が無い。EventuallyTerminates は、この 2 つが起きなくても、決定予算が減っていくことだけで成り立つ構造になっている。

---
式の言い直しはファイル 1 本だけから行い、TLC での確認はしていません。到達可能性に頼る判断(18、14 など)は、手で追った推論です。
````
