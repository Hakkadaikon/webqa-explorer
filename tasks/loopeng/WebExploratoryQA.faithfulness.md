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
読み手の言い直しの全文は、この文書の末尾の付録に写した(v6のspecに対するもの)。
v6で変えた定義に関わる5行(S-005、S-006、S-012、S-016の探索モード、S-028の存在確認)は、言い直しの欄をv6の読み手の言葉に改めた。
ほかの行の言い直しは、v6の読み手もv5と同じ内容を返したことを付録で確かめ、欄はそのままにした。
「読み手が気になった18点の分類」は、v6の読み手の18点で書き直した。
`WebExploratoryQA.paraphrase.md`はv4のspecに対する言い直しで、この判定には使っていない。

前件到達の証跡は`WebExploratoryQA.reach.log`の「v6」節を指す(表では「前件到達はP1」のように節の番号で書く)。
P44とP45は`WebExploratoryQA.noexplorable.cfg`(要素1つで、それが禁止要素)、P46〜P50は`WebExploratoryQA.empty.cfg`(要素が無い)、ほかは`WebExploratoryQA.cfg`で回した。
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

読み手の言い直しは、まだ取っていない。
仕様を直した担当は、読み手のsubagentを起こさない決まりなので、呼び出し元に任せた。
v7の入力を新しい読み手に渡し、次の行を判定し直す必要がある。

- 新しい行: S-046、InterviewMode S-011(止めた提案を数える)、InterviewMode S-047(見えた要素を数えない)
- 式が変わった行: S-005(`RequestDecision`の失敗の枝、`AwaitingOnlyByDecision`)、S-021の行き詰まりとドメイン外(`StuckAfterOp`、`Refuse`)、S-027(モードが3つ)、S-028の存在確認(確かめ直しで外に着く)、S-033(`DomainChangesOnlyByNavigation`)、S-041(`LimitStops`)、S-044の優先順位と終了状態(`JudgeDownStops`、`ExitReasonMatchesPhase`)、S-015の段階の行(インタビューも段階を持つ)

下の判定表のうち、この2つに当たる行は、読み手の言い直しが届くまで「v6の判定」として読む。
`WebExploratoryQA.paraphrase.md`はv4の言い直しのままで、v7の判定には使えない。

前件到達は`WebExploratoryQA.reach.log`の「v7」節にある。
v6のP1〜P51(68本)をv7のspecで回し直し、v7で足したP52〜P70(21本)を加えた。89本すべてで、前件が成り立つ到達状態または一歩が見つかった。
P63(インタビューの応答の異常は数えない)は`WebExploratoryQA.empty.cfg`で回した。要素がある画面では、直接操作の要素番号が無い応答が起きないからである。
足した前件は、判断モデルの停止(P52)、`JudgeDownStops`の前件(P53)と、判断予算の最後の1回や時間切れとの同時(P54a、P54b)、予算が残ったままの停止(P54c)、失敗の一歩(P55)と0に戻る一歩(P56)、閾値に達した状態(P57)、失敗を挟んだ予算切れ(P58)、インタビューで止めた提案(P59)とそれ以外のモード(P60)、確かめ直しで外に着く一歩(P61、P62)、インタビューの応答の異常(P63)、外に着いて見えた要素が無くなる一歩(P64)、インタビューで見えた要素を数えない一歩(P65)、止めた提案で行き詰まりに届く一歩(P66)、インタビューのゴール達成(P67)、段階の切り替え(P68)、申告による終了(P69)、行き詰まりの数が判断の数に並ぶ状態(P70)である。

## 判定表

| 要件 | 式 | 読み手の言い直し | 判定 | 理由 / 直し方 |
|---|---|---|---|---|
| S-002 | `ExecuteNoChange`/`ExecuteChange`/`ExecuteOutOfDomain`、`SafeProposalProceeds`、`ProposalOutcome`、`VisitedChangesOnlyByExecution` | 「安全な提案はactingへ進むか環境事象」「actingの次は、done提案ならdone、stuck提案ならstuck、それ以外はidle」「訪問済みが変わるのはactingからidleへの遷移だけ」 | partial | 許可された提案が実行段階へ進み、実行の帰結が記録されることは検査している。ブラウザ操作という副作用は状態変数に現れない(model.mdの「モデル化で捨てた領域」)。前件到達はP21、P26、P27a〜c。 |
| S-005 | `RequestDecision`、`ProposalWellFormed`、`AwaitingOnlyByDecision` | 「Continuing、idleCounterが閾値未満、requestBudgetが残り、許可ドメインの中なら、awaiting_decisionへ移り、予算を1ずつ減らす。opはProposableOpsから、targetはProposalTargets(op)から非決定に選ぶ」「awaitingに入ることと判断回数が1増えることは同値」 | partial | 問い合わせ(API呼び出し)そのものはモデル外で、どんな値が返っても扱えることを非決定の選択で表す。v6で、操作可能な要素が1つも無い画面でも問い合わせが起きるようにした。前件到達はP15、P23、P46(要素が無いときの問い合わせ)。 |
| S-006 | `RequestDecision`、`ProposalTargets`、`ProposalWellFormed` | 「opがclickかinputで、Elementsが空でなければ、対象の候補は全要素(禁止要素も含む)。それ以外は対象なし(none)だけ」「targetExistsは、直接操作で対象がnoneのときだけ偽に固定」「awaitingで直接操作でなければtargetはnone。直接操作でtargetがnoneなら対象は存在しない。直接操作でElementsが空でなければtargetはElementsに属する」 | partial | S-005と同じ理由。言い直しはv6の`ProposalTargets`と`ProposalWellFormed`の意図(model.mdの`proposedTarget`の行)と一致する。v5は対象を取らない操作にも要素を持たせていたので、要素が無い画面を表せなかった。v6は`ProposalTargets`で、click/inputで要素があれば要素、それ以外は`"none"`を対象にする。`ProposalWellFormed`は、問い合わせの直後に対象を取らない操作の対象が`"none"`であること、click/inputの対象が`"none"`なら存在しない扱いであることを検査する。前件到達はP50(click/inputに要素番号が無い応答)、P51(対象を取らない操作の対象なし)。 |
| S-010 | `ApplyGuardrails`、`LowConfidenceBackNeverActed`、`SafeProposalProceeds` | 「ペイウォールでなく、低確信のbackなら却下してidleへ」「actingで、ペイウォールでなくbackなら低確信ではない」「安全な提案はactingへ進む」 | full | 却下する側と、却下しすぎない側の両方を検査している。ペイウォールの強制backを前件から外すのは、S-013による意図どおりの扱い。0.7の閾値は真偽値に丸めた(設計メモ8)。前件到達はP1、P21。 |
| S-011 | `ExecuteNoChange`、`IdleCounterTracksScreen`、`ObservationChangesOnlyByExecution` | 「画面が変わらなければidleCounterを1増やす」「screenChangedとnewSeenが変わるのは実行の完了だけ」 | partial | 画面が変わらなかったことの観測は表している。次に確信度の高い候補へ切り替える選択は、`RequestDecision`の非決定の選択に吸収され、モデル外。受け入れシナリオ「画面が変わらないのに同じ要素を選んだら…」で確かめる。前件到達はP28e、P37。 |
| S-012 | `ApplyGuardrails`、`NeverOperatesForbidden`、`DirectOpActedOnlyOnSafeTarget`、`SafeProposalProceeds` | 「禁止要素への直接操作は退けてidleに戻る」「訪れた要素に禁止要素は含まれない」「ペイウォールでないactingで直接操作なら、対象は存在し、Explorableに属する」 | full | 一致する。v6の`DirectOpActedOnlyOnSafeTarget`は、対象が`Explorable`(要素で、禁止要素でない)にあることを求め、`"none"`への操作も除く。禁止要素の分類ロジックの正しさは、設計メモ8とmodel.mdの`ForbiddenElements`節のとおり実装で確かめる。前件到達はP2、P21。 |
| S-013 | `ApplyGuardrails`のペイウォール分岐、`PaywallForcesBack`、`ProposalChangesOnlyByDecision` | 「ペイウォールなら操作をbackに差し替えてactingへ。doneやstuckの提案もbackになる」 | partial | 判断モデルの提案と無関係に既定の離脱操作をする点は一致する。specはペイウォールの観測が問い合わせの応答と同時に届くので、ペイウォールでも問い合わせて予算を1ずつ使う。「問い合わせない」は検査していない。予算を多めに使う側へのずれで、安全性には影響しない。実装は問い合わせの前に課金画面を判定して問い合わせを省く。受け入れシナリオ「課金画面が出たら判断モデルに問い合わせず…」「課金画面から離脱しても消えないまま3回続いたら…」で確かめる。前件到達はP22、P34。 |
| S-014 | `FinishNormally`、`FinishByProposalDone`、`ExploreGoalOnlyWhenAllExplored`、`ScenarioGoalOnlyAtLastStage`、`ReportedDoneOnlyWithoutDoneWhen`、`GoalPreempts`、`DoneWhenChangesOnlyByExecutionOrAdvance` | 「探索でgoal_achievedなら全部訪問済みで判断1回以上」「シナリオでgoal_achievedなら最後の段でdoneWhenMetが真」「reported_doneならExploreDoneもScenarioStageDoneも偽」「doneWhenMetが偽から真になるのは実行の完了か段の前進」 | full | ゴール達成は完了条件だけで決まり、判断モデルのdoneは未検証の終了として分けて記録する。前件到達はP3、P4、P5、P16a、P16b、P38a、P38b。 |
| S-015(段階の定義) | `stage`、`NumStages`、`LastStage` | 「段の数は1以上。シナリオではstage = NumStagesのときだけ最後の段」 | partial | 段階を状態機械に入れたが、YAMLでの定義の書き方と各段のdone_whenの中身はモデル外(model.md)。受け入れシナリオ「テスト対象・入力値・段階をYAMLの設定で定義して実行する」で確かめる。 |
| S-015(段階の切り替え) | `AdvanceStage`、`StageAdvancePreempts`、`StageAdvanceResetsCounters`、`BudgetSharedAcrossStages`、`TimeNeverRewinds` | 「idleでStageClearedなら、次は段が1進んでidleのままか環境事象」「段が変わるなら2つのカウンタは0」「段が変わる遷移で予算関係の4変数は変わらない」 | full | 段を進め、固まりと行き詰まりの回数を0に戻し、判断予算と時間窓を戻さないことは要件どおり。v4で問題にした切り替え直後の観測は、次の行で扱う。前件到達はP17、P30、P31。 |
| S-015(入った時点のdone_when) | `Init`の`doneWhenMet`、`AdvanceStage`の`doneWhenMet'`、`DoneWhenObservations`、`StageAdvancePreempts`、`GoalPreempts`、`ScenarioGoalOnlyAtLastStage`、`DoneWhenChangesOnlyByExecutionOrAdvance` | 「シナリオではInitでdoneWhenMetが真でも偽でもよい」「AdvanceStageはdoneWhenMetを観測し直す(真になってもよい)」「NumStages = 1なら判断0回でgoal_achievedになる。操作なしで全段を素通りできる」 | full | 実行の開始時と段階を進めた直後にdone_whenを観測し、満たしていれば問い合わせずに次の段階へ進むか、最後の段階ならゴール達成になる。`GoalPreempts`と`StageAdvancePreempts`が、その状態から問い合わせへ進まないことを検査する。読み手が気になった点2で挙げた「操作なしで全段を素通り」は、この行の要件そのもの。前件到達はP41(開始時に満たしている)、P42(進んだ直後に満たしている)、P43(判断0回でゴール達成)。 |
| S-016(探索モード) | `RequestDecision`の非決定の選択 | 「exploreモードではProposableOpsからdoneを除く」「targetはProposalTargets(op)から選ぶ」「直接操作を含まないOperationsでは、Explorableが空でないexploreはゴールに届かず、行き詰まりか予算切れで終わる」 | partial | v6で対象は`ProposalTargets`から選ぶ(S-006の行)。操作可能な要素が無い探索モードでも、1回問い合わせてからゴール達成になる(P47)。探索モードを持つことは表しているが、未操作の要素を優先する規則は非決定の選択に吸収した(model.mdの「捨てた領域」)。受け入れシナリオ「未操作の要素を、未探索の分岐が残る操作済み要素より先に選ぶ」で確かめる。 |
| S-016(探索のゴール) | `DoneWhen`、`ExploreDone`、`FinishNormally`、`ExploreGoalOnlyWhenAllExplored`、`GoalPreempts` | 「探索モードの終了条件は、探索可能な要素を全部訪問し、かつ判断を1回以上したこと」「Explorableが空なら、判断1回で達成になる。禁止要素へのclickを却下された場合も判断数は増えるので同じ」 | partial | 「1回以上問い合わせた後」と「探索してよい要素が無くても、最初の問い合わせの前にゴール達成としない」は一致する(設計メモ11の2)。読み手の気になった点3は、却下された判断も問い合わせ1回に数えることを指す。要件の「問い合わせた後」はそれを含むので、ずれではない。partialの理由はv4と同じで、specの「訪問済み」は「操作済み」で、要件の「探索し終えた」(遷移先まで探索し終えた。設計メモ9)とは別の概念。単一画面への簡略化の下では両者が一致する。複数画面での意味は、受け入れシナリオ「遷移先を探索し終えるまで、途中のボタンを探索し終えた要素として扱わない」で確かめる。前件到達はP3、P16a、P44(空で、問い合わせ前のidle)、P45(空のときのゴール達成)。 |
| S-016(制限時間) | `TimeElapses`、`FinishTimeUp`、`LimitStops`、`TimeExceededOnlyWhenTimeUp`、`ExitReasonMatchesPhase`、`TimeNeverRewinds`、`TimeElapsesAlone` | 「時間切れは終端でなければいつでも起きる」「idleで、終了条件が偽で、閾値未満で、予算切れか時間切れなら、次は対応する終端か環境事象」「time_exceededならtimeUpが真」 | full | 時間切れは時刻の経過として公平性なしで起き、次のidleで時間切れとして終わる。決め打ちシナリオでも起きる(設計メモ10の(b))。ステップの途中で時間が切れたら次のidleで終わるのは、優先順位の行(S-044の6行目)が待機状態の判定として定めたとおり。前件到達はP7d、P8、P9、P19b、P19c、P32、P33。 |
| S-016(doneを外す) | `ProposableOps`、`ProposalWellFormed`、`ExploreNeverReportedDone` | 「探索モードではProposableOpsからdoneを除く」「探索モードなら終了理由はreported_doneでない」 | full | 一致する。`ProposalWellFormed`はシステム側の`ProposableOps`を使わず生の変数で書いてある。前件到達はP6、P15。 |
| S-017 | `RecoverFromFreeze`/`ResumeAfterRecovery`、`RecoveryResetsPageState`、`RecoveryOnlyAfterFreeze`、`FreezePreemptsDecision`、`FreezeRecoveryRecorded`、`MemoryNeverShrinks` | 「Continuingで固まりの閾値以上ならfrozen_recoveredへ。idleCounterだけ0にする」「frozen_recoveredならidleCounterは0で許可ドメイン内」「固まりの閾値以上ならawaiting_decisionへ移らない」「訪問済みは減らない」 | partial | 復旧がページの状態だけを戻し、探索の記憶を残すことは検査している。固まりを検出したら必ず復旧することは性質として書いていない。問い合わせは禁止され、弱公平性の下でいずれ復旧か終了が起きるが、これは推論で検査していない。ドメイン外は復帰で、行き詰まりは終了で扱う(設計メモ2。行き詰まりで終えるのはS-044の優先順位による)。「ページが応答しない」は画面が変化しなかった1ケースとして固まりカウンタに合流させた(設計メモ9)。前件到達はP14、P20、P25。 |
| S-021(固まり) | `RecoverFromFreeze`のガード、`IdleCounterTracksScreen`、`CountersBoundedByOps`、`CountersChangeOnlyBySteps` | 「idleCounterが閾値以上」「実行で画面が変われば0、変わらなければ1増える」 | full | 一致する。「一定時間」は操作の回数で数える(設計メモ9)。前件到達はP28a、P28e、P35a。 |
| S-021(ドメイン外) | `ExecuteOutOfDomain`、`DomainRecordAccurate`、`DomainChangesOnlyByNavigation` | 「許可ドメインの外へ出る(pageOutside真、inAllowedDomain偽)」「inAllowedDomainは常にpageOutsideの否定」 | full | 記録が実際のページ位置と一致することを検査する。判定ロジック(URLの比較)の正しさは設計メモ8。前件到達はP36。 |
| S-021(行き詰まり) | `FinishStuck`のガード、`StuckAfterOp`、`StuckCounterTracksNewElements`、`StuckPreempts`、`NewSeenOnlyOnChangedAllowedPage` | 「訪問済みが増えず新規発見もなければ1増やし、それ以外は0」「idleで、終了条件が偽で、行き詰まりの閾値以上なら、次はstuckか環境事象」「newSeenが真なら、画面が変わり、許可ドメイン内で、判断1回以上」 | full | 探索済みの集合が増えず、未操作の要素も新しく見えない、という定義と一致する。許可ドメイン外の要素を新しく見えた要素に数えないこと(設計メモ10の1)も検査している。却下された提案はカウンタを進めないが、設計メモ9のとおり。前件到達はP10、P18、P28b、P28c、P28d、P35b。 |
| S-022 | `VisitedAfterOp`、`MemoryNeverShrinks`、`VisitedOnlyByOps`、`VisitedChangesOnlyByExecution` | 「click/inputなら対象を訪問済みに足す」「訪問済みは減らない。段が進んでも減らない」「訪問済みの要素数は判断数以下」 | partial | 1回の実行の中で記憶が失われないことは検査している。グラフ構造は単一の集合で近似し(model.mdの簡略化)、実行をまたぐ永続化は状態変数の外。受け入れシナリオ「探索の記憶を次の実行に引き継ぐ」などで確かめる。何も起きなかった input(入力欄でない対象、値の無い入力欄)は、モデルでは対象の無い実行(wait 相当)として扱う。記憶に足さず、行き詰まりの数は進む。検査する cfg の Operations に input は無い。前件到達はP26。 |
| S-023 | `RequestDecision`の非決定の選択 | 「提案の操作と対象をすべて非決定に選ぶ」 | partial | 優先順位の規則は非決定の選択に吸収した(model.mdの「捨てた領域」)。受け入れシナリオ「未操作の要素を…先に選ぶ」ほか3本で確かめる。 |
| S-027 | `Init`の`mode`、`ModeFixed` | 「モードは2つのどちらで始まってもよく、変わらない」 | full | 両モードを同じ状態機械で扱う。モードの指定方法はモデル外だが、要件はそれを求めていない。前件到達はP4(シナリオ)、P6(探索)。 |
| S-028(存在確認) | `ApplyGuardrails`、`DirectOpActedOnlyOnSafeTarget`、`SafeProposalProceeds` | 「存在しない対象への直接操作は退けてidleに戻る」「ペイウォールでないactingで直接操作なら、対象は存在する」「Elementsが空だと、clickとinputの対象はnoneでtargetExistsは偽に固定され、毎回退けられる」 | full | 一致する。要素が無い画面でclick/inputが返った場合も、対象`"none"`で存在しない扱い(応答の異常)になり、却下される。存在の判定(ノードの同一性)は設計メモ8。前件到達はP2、P21、P50。 |
| S-028(決済導線) | `ApplyGuardrails`、`NeverOperatesForbidden` | 「禁止要素へのclick/inputは却下」「訪問済みに禁止要素は含まれない」 | partial | specは判断モデルが禁止要素を提案しうるとして、ガードで操作を止めることを検査する。要件の「選択してはならない」(探索の候補に入れない)は非決定の選択に吸収され、モデル外。設計メモ9のとおり、実装は禁止要素全体を候補から外す。受け入れシナリオ「禁止要素は未操作のまま残っていても、探索の候補に入れない」「探索モードでも決済導線の要素は選ばない」で確かめる。 |
| S-033 | `ExecuteOutOfDomain`/`ReturnToAllowedDomain`、`RequestDecision`のガード、`DomainRecordAccurate`、`NoDecisionOutsideDomain`、`DomainChangesOnlyByNavigation` | 「外へ出たことを記録する」「idleからawaiting_decisionへ移るとき、ページは許可ドメインの内」「Continuingで外にいるなら内へ戻る」 | full | 遷移の記録と、ドメイン外で探索しないことの両方を検査している。前件到達はP29、P36。 |
| S-041 | `RequestDecision`/`ExceedDecisionBudget`/`FinishTimeUp`、`DecisionsWithinCap`、`BudgetAccounting`、`BudgetExceededOnlyWhenSpent`、`DecisionBudgetNeverRestored`、`LimitStops`、`TerminalAbsorbing`、`EventuallyTerminates` | 「判断数はMaxDecisions以下」「判断予算と判断数の和は常にMaxDecisions」「予算切れか時間切れのidleの次は、対応する終端か環境事象」「判断予算は増えない」「システム側の遷移が弱公平なら、いつかは終端に至る」 | full | 呼び出し回数は上限を超えず、上限か制限時間に達したら次の待機状態で止まり、再開しない。上位の終了条件(ゴール達成、行き詰まり)が先に採られるのは、承認済みの優先順位のとおり。`EventuallyTerminates`は定数の前提の下での主張で、下の「定数の前提」に書いた。前件到達はP13、P19a、P19b、P19c。 |
| S-042 | `RequestDecision`/`ResetRequestWindow`、`RequestsWithinWindowCap`、`BudgetAccounting`、`BudgetSharedAcrossStages` | 「リクエスト予算が残っているときだけ問い合わせる」「Continuingなら時間の条件なしでウィンドウを戻せる」「ウィンドウ内のリクエスト数はMaxRequests以下」 | partial | 1つの窓で上限を超えないことは検査している。窓が時間の条件なしで満了するのは、要件の「どの時点でも満了しうる」のとおり(設計メモ7)。1窓の実時間の長さはモデル外。数える単位もずれる。specは問い合わせごとに数えるが、実装は対象サーバへのアクセス(ブラウザ操作、課金画面の離脱、固まり復旧の再読み込み、許可ドメインへの戻り)を数え、ガードで止めた提案は数えない(設計メモ9でユーザーが承認)。受け入れシナリオ「ページの再読み込みや許可ドメインへの戻りも…」「時間窓の操作回数が上限に達していたら…」「ガードレールで止めて…」で確かめる。前件到達はP31。 |
| S-044(ゴール達成) | `FinishNormally`、`GoalPreempts`、`ExploreGoalOnlyWhenAllExplored`、`ScenarioGoalOnlyAtLastStage`、`ExitReasonMatchesPhase` | 「idleでGoalReachedなら、次はdone・goal_achievedか環境事象」 | full | 完了条件の観測が成り立てば、他の終了理由や継続に先取りされずゴール達成で終わる。v4でpartialにした理由(最後の段に入った直後と開始時に観測しない)は、v5の`Init`と`AdvanceStage`の観測で解消した。前件到達はP3、P4、P16a、P16b、P43。 |
| S-044(reported_done) | `FinishByProposalDone`、`ProposalOutcome`、`ReportedDoneOnlyWithoutDoneWhen`、`ExploreNeverReportedDone` | 「actingでdone提案ならdone・reported_done」「reported_doneならExploreDoneもScenarioStageDoneも偽」 | full | 一致する。最後の段でなくてもdoneの提案で実行全体が終わるのは、要件の「実行を未検証の終了として記録」のとおり。課金画面のときにdoneの提案がbackに置き換わるのはS-013による。前件到達はP5、P27a。 |
| S-044(行き詰まり) | `FinishByProposalStuck`/`FinishStuck`、`StuckPreempts`、`ProposalOutcome`、`ExitReasonMatchesPhase` | 「stuck提案ならstuckで終わる」「idleで行き詰まりの閾値以上なら次はstuckか環境事象」「stuckなら終了理由はstuck」 | full | `StuckPreempts`は終了理由を見ないが、`ExitReasonMatchesPhase`が段階と終了理由の対応を検査するので、合わせて一致する。前件到達はP7b、P18、P27b。 |
| S-044(中断) | `UserAbort`、`ExitReasonMatchesPhase`、`TerminationKeepsState` | 「終端でなければaborted・user_abortedで終わる」「終端へ入る遷移では段階と終了理由のほかは変わらない」 | partial | どの時点でも中断を受け付け、中断として記録して終わることは検査している。記憶・終了のイベント・実行結果の書き出しはモデル外(設計メモ10の3)。受け入れシナリオで確かめる。前件到達はP7e、P39a、P39b。 |
| S-044(終了状態) | `TerminalAbsorbing`、`Next`の最後の選択肢、`TerminationKeepsState` | 「終端にいるなら変数は何も変わらない」「終端へ入る遷移では段階と終了理由のほかは変わらない」 | full | 中断を含め、終了後は状態が変わらない。`TimeElapses`と`UserAbort`も終端では起きない。前件到達はP39a、P40。 |
| S-044(優先順位) | `Exploring`/`Continuing`の各ガード、`GoalPreempts`、`StageAdvancePreempts`、`StuckPreempts`、`LimitStops`、`FreezePreemptsDecision`、`NoDecisionOutsideDomain`、`EnvEvent` | 読み手は各アクションのガードと各性質を個別に言い直した(付録の4節と5.2節)。並べると「完了条件(段の前進かゴール)、行き詰まり、予算切れと時間切れ(どちらでもよい)、固まり復旧・窓の巻き戻し・ドメイン復帰、問い合わせ。中断と時間切れの発生はどこでも起きる」 | full | 要件の順序と一致する。予算切れと時間切れが同時に成り立てばどちらでもよいことも、要件の文面どおり(設計メモ10の2)。v5で開始時と段の前進直後にdone_whenを観測するようになっても、完了条件が最上位であることは変わらない。前件到達はP16a、P16b、P17、P18、P19a〜c、P20、P29、P41、P42。 |

## 読み手が気になった18点の分類

番号はv6の読み手の番号(付録の「気になった点」)。

### 1. 提案できる操作が空だと止まる

定数の前提の話で、要件とのずれではない。
`Operations`が空か、探索モードで`Operations`がdoneだけのとき、`ProposableOps`が空になり`RequestDecision`が有効にならない。
公平性のある遷移が1つも無く、`EventuallyTerminates`が破れる。
検査に使う3つの`.cfg`は、どれも`Operations`がclickを含む。
下の「定数の前提」に、実装の設定値の検査で拒否する組み合わせとして書いた。
`.tla`の`ASSUME`に足すかどうかは、「仕様の直しを検討してよいもの」の1に挙げた。
v5の読み手は要素が空の場合も止まると指摘し、v6で直した(要素が空の画面を表せなかった)。
v6の読み手は、要素が空の場合を止まる例に挙げていない。

### 2. 決め打ちシナリオは判断0回でゴール達成になりうる

要件どおりで、問題ない。
S-015の3行目(設計メモ11の1)は、実行の開始時と段階を進めた直後にdone_whenを確かめ、満たしていれば操作せずに進むと定めた。
探索モードとの扱いの違いも、設計メモ11の2つの回答がそれぞれ決めたもの。
前件到達はP41、P42、P43。

### 3. 探索してよい要素が空のとき、判断1回で達成になる

要件どおりで、問題ない。
設計メモ11の2で、ユーザーは「1回は判断してから」を選んだ。
却下された判断も問い合わせ1回に数えるのは、要件の「判断モデルに1回以上問い合わせた後に」のとおり。
判定表のS-016(探索のゴール)の行に書いた。

### 4. 要素が空か、全要素が禁止要素だと、click/inputは毎回却下される

要件どおりで、問題ない。
v6で意図した扱いで、model.mdの`proposedTarget`の行に「click/inputで要素が1つも無いときも`"none"`で、存在しない要素番号が返った応答の異常として扱う」とある。
対象を取らない操作は実行できる(P49)。
却下が続けば判断予算切れで止まるのは、設計メモ9の4つ目のとおり。

### 5. 直接操作を含まない操作種別では、探索モードがゴールに届かない

設定の話で、要件とのずれではない。
探索モードのゴールは禁止要素以外をすべて操作することなので、clickもinputも選べない設定ではゴールに届かず、行き詰まりか予算切れで終わる。
安全性の性質には影響しない。
実装の設定値の検査で、探索モードにclickかinputを含まない操作種別を拒否するかどうかは、実装の判断に任せる。

### 6. 却下された提案をカウンタに数えない

承認済みの決定どおりで、問題ない。
設計メモ9の4つ目に「ガードレールで止めた提案と判断モデルの応答の異常は…固まりにも行き詰まりにも数えない。続いた場合は判断予算切れで止まる」とある。
`RequestDecision`は判断予算を必ず1減らすので、予算が0になれば`ExceedDecisionBudget`が起きる。

### 7. 許可ドメインの外にいるままゴール達成になりうる

要件どおりで、問題ない。
S-044の6行目は、待機状態でdone_when成立をドメイン復帰より上に置いている。
S-033が禁じているのはドメイン外での問い合わせで、done_whenの観測は問い合わせではない。
探索モードで外へ出る要素を最後に押した場合も、設計メモ9のとおり、遷移先に探索すべき要素が無いので探索し終えた扱いになる。
どの操作でも外へ出られるのは、S-033が「操作の結果」とだけ書き、操作の種類を限っていないため(waitの間のリダイレクトなど)。
ただし決め打ちシナリオで、done_whenを外のページで判定してよいかは、要件の文面からは読み取れない。
実装のテストで、この帰結が意図どおりか確かめるとよい。

### 8. `RecoveryResetsPageState`の名前と中身がずれる

モデル化で捨てた領域どおりで、問題ない。
model.mdの「捨てた領域」に、`RecoverFromFreeze`は固まりカウンタを戻すことでページ状態の初期化を表し、ブラウザの再読み込みはモデル外と書いてある。
許可ドメイン内であることは、ドメイン外では問い合わせず固まりカウンタが増えないことから成り立つ。
性質は名前より弱く、復旧直後の状態を確かめるだけである。

### 9. `RecoveryOnlyAfterFreeze`が弱く、閾値によっては前件が立たない

検査の設定で前件に届いており、問題ない。
固まりを検出したことは`FreezeRecoveryRecorded`と`RecoverFromFreeze`のガードが担う。
`WebExploratoryQA.cfg`は`IdleThreshold`と`StuckThreshold`が2、`MaxDecisions`が4、`NumStages`が2なので、固まり・行き詰まり・段の前進の前件にどれも届く。
証跡はP14、P18、P20、P25、P17、P30、P31。

### 10. 時間窓に時間の条件が無く、上限の不変条件が冗長

要件どおりで、問題ない。
S-042は「単位時間の窓は時刻で区切られ、どの時点でも満了しうる」と書き、設計メモ7の承認で明確にした文面である。
時刻をモデルに持たないので、窓はいつ満了してもよい。
そのため`RequestsWithinWindowCap`は「窓を戻すまでの要求数が上限以下」という意味しか持たない。
1窓の実時間の長さはモデル外で、実装のレート制御のテストで確かめる。
`RequestsWithinWindowCap`と`DecisionsWithinCap`が`BudgetAccounting`から導けるのは、要件の文面に近い形で別に書いているためで、検査を弱めない。

### 11. ペイウォールではdoneやstuckの提案もbackに差し替わる

要件どおりで、問題ない。
S-013の「判断モデルに問い合わせず既定の離脱操作」のとおり、離脱操作は判断モデルの提案によらない。
S-010の確信度は判断モデルが提案したbackに対する条件で、既定の離脱操作には掛からない。
ペイウォールが提案と一緒に決まるのは、model.mdの変数表の「非決定的に観測」のとおり。
`isPaywall`が次の問い合わせまで真のまま残っても、読むのは`awaiting_decision`と`acting`だけなので振る舞いに影響しない。

### 12. 画面が変わらなくても完了条件が真になりうる

モデル化の方針どおりで、問題ない。
done_whenの中身はモデル外で、操作の後に満たしたかを観測する真偽値に抽象化した(model.mdの「簡略化の明示」)。
waitの後に時間で表示が変わる場合のように、画面の変化の観測とdone_whenの成立は独立に起こりうる。

### 13. 時間切れや予算切れは操作の途中では割り込まない

要件どおりで、問題ない。
ステップの途中で時間が切れたら次のidleで終わるのは、優先順位の行(S-044の6行目)が待機状態の判定として定めたとおり。
時間切れの後に段の前進やゴール達成が起きるのも、同じ行がdone_when成立を最上位に置いているため。

### 14. 性質が定義を写している、条件や変数が冗長

検査の強さは変異の検査で確かめており、問題ない。
`StuckCounterTracksNewElements`と`IdleCounterTracksScreen`は、アクションの更新式を参照せず、式を別に書いている。
そのため更新式を書き換える変異は、これらの性質で捕まる。
`GoalReached`も`DoneWhen`と`LastStage`を参照せず、`ExploreDone`と`ScenarioStageDone`から別に組み立てている。
`ExploreDone`の`decisionsMade > 0`を書き換える変異は、`WebExploratoryQA.noexplorable.cfg`で`ExploreGoalOnlyWhenAllExplored`か`GoalPreempts`の違反として捕まった(`WebExploratoryQA.mutants-v6.tsv`の`gt->ge#6`、`gt->ge#9`)。
`DirectOpActedOnlyOnSafeTarget`の`~isPaywall`は、ペイウォールの経路で`proposedOp`がbackになるので実質は余分だが、S-013の経路を前件から外すことを文面で示している。
`pageOutside`は記録の正しさを検査するための環境側の真値で、model.mdの変数表で定めている。
終端の足踏みの選択肢については、TLCを`-deadlock`(デッドロックを検査しない)で回しており、終端でないまま止まる振る舞いは`EventuallyTerminates`で捕まえる。
この性質は3つの設定でNo errorだった。

### 15. `StartsFresh`は初期状態にしか効かない

承認済みの扱いどおりで、問題ない。
3つの`.cfg`とも`PROPERTY StartsFresh`で、`INVARIANT`には入っていない。
変異の検査でも、`StartsFresh`の連言を書き換えた9件が初期状態で性質違反になった。
触れていない変数は、ほかの不変条件が初期状態でも縛る。
`decisionsMade`と`requestsInWindow`は`BudgetAccounting`、提案の初期値は`NoProposalBeforeDecision`、`screenChanged`は`NoChangeObservedOnlyAfterOps`が縛る。

### 16. 訪問済みが段をまたいで引き継がれる

承認済みの決定どおりで、問題ない。
設計メモ10の(c)で「段階を切り替えても探索の記憶(探索済みの要素の集合)は戻さない」とユーザーが承認した。
前の段と同じボタンを何度も押す決め打ちシナリオでは、新しい要素が見えなければ行き詰まりに近づく。
実装のテストで、この帰結が意図どおりか確かめるとよい。

### 17. 低確信が意味を持つのはbackだけ

要件どおりで、問題ない。
S-010は「確信度の低い「戻る」は実行しない」で、ほかの操作の確信度を条件にしていない。
対象を取らない操作の`targetExists`がどこからも読まれないのは、`ApplyGuardrails`が存在の確認をclickとinputに限るため(S-028)。

### 18. ゴール達成を保証する活性が無い

モデル化の方針どおりで、問題ない。
判断モデルの提案を非決定の選択で表すので、ゴールに必ず届くことは書けない。
要件もゴールに必ず届くことは求めておらず、S-041とS-044が求めるのは、どれかの理由でいつか止まることで、`EventuallyTerminates`が担う。

## 仕様を直す必要があるもの

v5では、v5の読み手の気になった点1の「要素が空だと止まる」が要件とのずれだった(操作可能な要素が無い画面を表せない)。v6で直した。
v6の読み手の言い直しで判定し直した判定表に、mismatch、vacuous、missingの行は無い。
v6の読み手の18点にも、要件とのずれは無かった。

## 仕様の直しを検討してよいもの

どれも要件とのずれではなく、直さなくても判定は変わらない。

### 1. 定数の前提を`ASSUME`に書く

`EventuallyTerminates`は、`ProposableOps`が空でない前提の下でしか成り立たない(気になった点1)。
今は`.cfg`の値と、実装の設定値の検査でこの前提を守っている。
`ASSUME Operations \ {"done"} # {}`を足せば、前提がspecの文面に出る。
`Elements`はページの中身なので、空でない前提を置いてはいけない。

## 空成立の証跡(含意・前提付きの式)

`WebExploratoryQA.reach.log`の「v6」節(P1〜P51、68本)に、v6のspecに対する前件ごとの結果を追記してある。
各節に`NotPx`の定義をそのまま書き、68本すべてで反例(前件が成り立つ到達状態または一歩)が出ている。
v5で足したのは次の5本。

- P41: 開始時に最初の段のdone_whenを満たしている状態(`StartsFresh`、`StageAdvancePreempts`)
- P42: 段を進めた直後に次の段のdone_whenを満たしている一歩(`DoneWhenChangesOnlyByExecutionOrAdvance`、`StageAdvanceResetsCounters`)
- P43: 判断0回でゴール達成になる一歩(`ScenarioGoalOnlyAtLastStage`)
- P44: 探索してよい要素が空で、問い合わせる前のidle(`ExploreGoalOnlyWhenAllExplored`、`GoalPreempts`)
- P45: 探索してよい要素が空のときのゴール達成(`ExploreGoalOnlyWhenAllExplored`)

v6で足したのは次の6本。P46〜P50は`WebExploratoryQA.empty.cfg`で回した。

- P46: 操作可能な要素が無いときの問い合わせ(`ProposalWellFormed`)
- P47: 操作可能な要素が無い探索モードのゴール達成(`ExploreGoalOnlyWhenAllExplored`)
- P48: 操作可能な要素が無い決め打ちシナリオのゴール達成(`ScenarioGoalOnlyAtLastStage`)
- P49: 操作可能な要素が無く、対象を取らない操作を実行する状態(`SafeProposalProceeds`)
- P50: click/inputに要素番号が無い応答を受けた状態(`DirectOpActedOnlyOnSafeTarget`、`ProposalWellFormed`)
- P51: 要素がある画面で、対象を取らない操作の対象が`"none"`の状態(`ProposalWellFormed`)

前件を持たない式(`NeverOperatesForbidden`、`BudgetAccounting`、`MemoryNeverShrinks`、`ModeFixed`、`DecisionBudgetNeverRestored`、`EventuallyTerminates`など)は対象外である。
同ログのv5以前の節は旧spec向けで、この判定には使っていない。

## 定数の前提

`EventuallyTerminates`は、`ASSUME`で正と定めた閾値と上限に加えて、`ProposableOps`が空でない前提の下での主張である。
`Elements`はページの中身で、空でもよい。
検査に使う`WebExploratoryQA.cfg`は、`Operations`がclickを含み空でなく、`IdleThreshold`(2)が`MaxDecisions`(4)以下なので、固まり復旧の前件にも届く(P14、P25)。
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

判定表は全33行(状態機械にした20件の要件のうち、S-015が3行、S-016が4行、S-021が3行、S-028が2行、S-044が6行に分かれる)。

- `full`: 19件(S-010、S-012、S-014、S-015の段階の切り替えと入った時点のdone_when、S-016の制限時間とdoneを外す行、S-021×3、S-027、S-028の存在確認、S-033、S-041、S-044のゴール達成・reported_done・行き詰まり・終了状態・優先順位)
- `partial`(理由付き): 14件(S-002、S-005、S-006、S-011、S-013、S-015の段階の定義、S-016の探索モードと探索のゴール、S-017、S-022、S-023、S-028の決済導線、S-042、S-044の中断)
- `mismatch`/`vacuous`/`missing`: 0件

共有手順の終了条件(全行がfullか理由付きのpartial、入力・判定表・空成立の証跡が`tasks/`に残っている)を満たす。
partialの14行は「検証済み」に含めない。
各行の理由欄に書いたモデル外の部分が、未保証の範囲である。

## 付録: 読み手の言い直し(v6)

以下は、v6の入力を渡した読み手のsubagentが、`WebExploratoryQA.faithfulness-input.md`だけを読んで返した文面を、そのまま写したもの。

````markdown
# WebExploratoryQA の式を日本語で言い直す

## 1. 定数と ASSUME

| 名前 | 言い直し |
|---|---|
| MaxDecisions | エージェントが判断を求めてよい回数の上限。1以上の自然数。 |
| MaxRequests | 1つの窓の中で出してよい要求数の上限。1以上。 |
| IdleThreshold | 画面が変わらない実行がこの回数続くと「固まった」とみなす閾値。1以上。 |
| StuckThreshold | 新しい要素も新しい発見もない実行がこの回数続くと「行き詰まった」とみなす閾値。1以上。 |
| NumStages | シナリオの段の数。1以上。 |
| Elements | 画面上の要素の集合。空でもよい(制約がない)。 |
| ForbiddenElements | 操作してはいけない要素。Elements の部分集合。 |
| Operations | エージェントが提案できる操作。click、input、scroll_down、scroll_up、back、enter、wait、done、stuck の部分集合。**空でもよい**。done や back を含まなくてもよい。 |

## 2. 補助定義

| 名前 | 言い直し |
|---|---|
| vars | 22個の変数をすべて並べた組。VARIABLES 宣言と過不足なく一致している。 |
| Modes | 動作モードは scenario(シナリオ実行)と explore(探索)の2つ。 |
| Phases | 状態は idle、awaiting_decision、acting、done、stuck、frozen_recovered、error_exit、timed_out、aborted の9つ。 |
| Terminal | 終了状態は done、stuck、error_exit、timed_out、aborted。frozen_recovered は終了状態に入らない。 |
| ExitReasons | 終了理由は none、goal_achieved、reported_done、stuck、budget_exceeded、time_exceeded、user_aborted。 |
| DirectOps | 要素を対象に取る操作は click と input。 |
| Explorable | 探索してよい要素。Elements から ForbiddenElements を除いたもの。 |
| ProposableOps | 提案できる操作。explore モードでは done を除き、scenario モードでは Operations すべて。 |
| ProposalTargets(op) | op が click か input で、かつ Elements が空でなければ、対象の候補は全要素(**禁止要素も含む**)。それ以外は対象なし("none")だけ。 |
| DoneWhen | 完了条件。explore では「訪れた要素が Explorable と一致し、判断を1回以上した」こと。scenario では観測値 doneWhenMet が真であること。 |
| LastStage | 最終段にいるか。explore なら常に真。scenario なら stage = NumStages のとき。 |
| DoneWhenObservations | 完了条件の観測値が取りうる値。scenario では真偽どちらも、explore では偽だけ。 |
| Exploring | idle にいて、まだ完了条件を満たしていない。 |
| Continuing | Exploring であり、さらに行き詰まりカウンタが閾値未満、判断予算が残っている、時間切れでない、の3つも満たす。 |
| VisitedAfterOp | 提案操作が click か input なら、訪問済み集合に対象を足したもの。それ以外の操作なら訪問済み集合のまま。実装では、何も起きなかった input(入力欄でない対象、値の無い入力欄)は記憶に足さず、モデルでは対象の無い実行(wait 相当)として扱う。行き詰まりの数は進む。 |
| StuckAfterOp | 操作で訪問済みが増えず、しかも次状態で新しい発見(newSeen')もなければ、行き詰まりカウンタを1増やす。どちらかがあれば0に戻す。次状態の値を参照するので、newSeen' が決まった後で評価する必要がある。 |
| ObserveDoneWhen | 次状態の doneWhenMet を観測値の集合から自由に選ぶ。 |
| ExploreDone | explore モードで全 Explorable を訪れ、判断を1回以上した。 |
| ScenarioStageDone | scenario モードで現在の段の完了条件を満たしている。 |
| GoalReached | 探索を終えた、または scenario の最終段で完了条件を満たした。 |
| StageCleared | scenario で段の完了条件を満たしたが、最終段ではない。 |
| EnvEvent | 環境から起きる出来事。ユーザーが中断した(次状態が aborted で、理由が user_aborted)か、時間切れになった(timeUp が偽から真に変わった)かのどちらか。 |

## 3. TypeOK と Init

| 名前 | 言い直し |
|---|---|
| TypeOK | 各変数の型。stage は 1..NumStages。proposedOp は Operations に back と none を足した集合から取る。proposedTarget は Elements か none。visited は Elements の部分集合。idleCounter、stuckCounter、decisionsMade、requestsInWindow は自然数で、上限は型では決めていない。decisionBudget は 0..MaxDecisions、requestBudget は 0..MaxRequests。残りは真偽値か、決められた列挙の値。 |
| Init | モードはどちらでもよい。stage=1、phase=idle、提案は空(op と target は none、低確信なし、ペイウォールなし、targetExists=真)。screenChanged=真、newSeen=偽。doneWhenMet は観測値から選ぶので、**scenario では最初から真の場合もある**。visited は空、各カウンタは0、許可ドメインの中にいる(pageOutside=偽)、時間切れでない。予算は満タン、判断回数と窓内の要求数は0、終了理由は none。 |

## 4. Next の各選択肢

| 名前 | ガード | 効果 |
|---|---|---|
| RequestDecision | Continuing、idleCounter &lt; IdleThreshold、requestBudget &gt; 0、許可ドメインの中にいる。 | awaiting_decision へ移る。判断予算と要求予算を1ずつ減らし、判断回数と窓内の要求数を1ずつ増やす。提案を非決定的に選ぶ。op は ProposableOps から、target は ProposalTargets(op) から選ぶ。低確信とペイウォールは任意。targetExists は、直接操作で対象が none のときだけ偽に固定され、それ以外は任意。 |
| ResetRequestWindow | Continuing で、要求予算が満タンでない。 | 要求予算を満タンに戻し、窓内の要求数を0にする。時間の経過とは結びついておらず、条件を満たせばいつでも起きる。 |
| ApplyGuardrails | awaiting_decision にいる。 | ペイウォールなら、op を back に差し替えて acting へ進む(target と低確信はそのまま)。ペイウォールでなければ、次のどれかに当たると提案を退けて idle に戻る。低確信の back、禁止要素への直接操作、存在しない対象への直接操作。どれにも当たらなければ acting へ進む。退ける条件と進む条件はちょうど補集合になっていて、遷移は決定的。退けてもカウンタは変わらず、予算だけが減ったまま残る。 |
| ExecuteNoChange | acting にいて、op が done でも stuck でもない。 | visited を VisitedAfterOp にし、idle へ戻る。画面は変わらず(screenChanged=偽)、新しい発見もない。完了条件を観測し直す。idleCounter を1増やし、stuckCounter は StuckAfterOp にする。 |
| ExecuteChange | 同上。 | 同様に idle へ戻る。画面は変わり、newSeen は任意、idleCounter は0、stuckCounter は StuckAfterOp。 |
| ExecuteOutOfDomain | 同上。 | 画面は変わるが新しい発見はなく、idleCounter は0、stuckCounter は StuckAfterOp。許可ドメインの外に出る(pageOutside=真、inAllowedDomain=偽)。**どの操作でも**外に出られる。 |
| ReturnToAllowedDomain | Continuing で、ドメインの外にいる。 | ドメインの中に戻る。phase は idle のまま。予算は使わない。 |
| FinishByProposalDone | acting にいて、op が done。 | done 状態で終わり、理由は reported_done。 |
| FinishByProposalStuck | acting にいて、op が stuck。 | stuck 状態で終わり、理由は stuck。 |
| RecoverFromFreeze | Continuing で、idleCounter ≥ IdleThreshold。 | frozen_recovered へ移り、idleCounter を0にする。画面もドメインもほかの変数は変えない。 |
| ResumeAfterRecovery | frozen_recovered にいる。 | idle へ戻る。 |
| AdvanceStage | idle にいて、DoneWhen が真で、最終段ではない。 | 段を1つ進め、完了条件を観測し直し(**直後にまた真になってもよい**)、idleCounter と stuckCounter を0にする。visited と予算は持ち越す。 |
| FinishNormally | idle にいて、DoneWhen が真で、最終段。 | done 状態で終わり、理由は goal_achieved。予算切れ、時間切れ、行き詰まり、ドメインの外にいるかどうかは問わない。 |
| FinishStuck | Exploring で、stuckCounter ≥ StuckThreshold。 | stuck 状態で終わる。時間切れや予算切れより優先する。 |
| ExceedDecisionBudget | Exploring で、行き詰まりカウンタが閾値未満、判断予算が0。 | error_exit で終わり、理由は budget_exceeded。 |
| FinishTimeUp | Exploring で、行き詰まりカウンタが閾値未満、時間切れ。 | timed_out で終わり、理由は time_exceeded。予算も0のときは ExceedDecisionBudget とどちらが起きてもよい。 |
| TimeElapses | 時間切れでなく、終了状態でもない。 | timeUp を真にする。ほかは変えない。acting や awaiting の最中にも起きる。 |
| UserAbort | 終了状態でない。 | aborted で終わり、理由は user_aborted。 |
| 終了状態での足踏み | 終了状態にいる。 | すべての変数を変えない。 |

## 5. 不変条件と性質

### 状態不変条件

以下の「成立」は、この式を読んだうえでの判断で、TLC の実行結果ではない。

| 名前 | 言い直し / 前提が真になりうるか |
|---|---|
| NeverOperatesForbidden | 訪れた要素に禁止要素は含まれない。禁止要素への直接操作はガードレールが退け、ペイウォールで差し替える back は visited を増やさないので成立する。 |
| LowConfidenceBackNeverActed | ペイウォールでない acting で op が back なら、低確信ではない。前提は真になりうる。ペイウォールで差し替えた back は除外しているので、低確信の back がそのまま acting に入る場合もある。 |
| DirectOpActedOnlyOnSafeTarget | ペイウォールでない acting で直接操作なら、対象は存在し、Explorable に属する。前提は真になりうる。ペイウォールの acting では op が必ず back なので、前提の ~isPaywall は余分。 |
| ExploreGoalOnlyWhenAllExplored | explore で goal_achieved なら、全 Explorable を訪れ、判断を1回以上した。前提は真になりうる。 |
| ScenarioGoalOnlyAtLastStage | scenario で goal_achieved なら、最終段にいて完了条件を満たしている。前提は真になりうる。0回の判断のまま達成する場合もある(気になった点の2)。 |
| ReportedDoneOnlyWithoutDoneWhen | reported_done で終わったなら、完了条件はどちらも満たしていない。前提は scenario で op=done を提案したときに真になる。RequestDecision が ~DoneWhen を要求し、acting の間は doneWhenMet が変わらないので成立する。 |
| ExploreNeverReportedDone | explore では reported_done にならない。ProposableOps から done を除いているので成立する。 |
| ExitReasonMatchesPhase | 終了状態と終了理由が対応し、終了していない間の理由は none。 |
| TimeExceededOnlyWhenTimeUp | time_exceeded なら時間切れ。 |
| StagesOnlyInScenario | explore では stage=1 で、doneWhenMet は偽。 |
| NewSeenOnlyOnChangedAllowedPage | 新しい発見があるのは、画面が変わり、ドメインの中にいて、判断を1回以上した後に限る。acting に入れるのはドメインの中にいるときだけで、外に出る遷移は newSeen を偽にするので成立する。 |
| NoChangeObservedOnlyAfterOps | 画面が変わらなかったと記録されるのは、判断を1回以上した後に限る。 |
| NoProposalBeforeDecision | 判断が0回なら、提案は初期値のまま。 |
| RequestsWithinWindowCap | 窓内の要求数は MaxRequests 以下。BudgetAccounting から直接出るので余分。 |
| DecisionsWithinCap | 判断回数は MaxDecisions 以下。これも BudgetAccounting から出るので余分。 |
| BudgetAccounting | 残りの判断予算と判断回数の和は MaxDecisions。残りの要求予算と窓内の要求数の和は MaxRequests。 |
| BudgetExceededOnlyWhenSpent | budget_exceeded なら判断予算は0。 |
| CountersBoundedByOps | 2つのカウンタはどちらも判断回数以下。 |
| VisitedOnlyByOps | 訪問済み要素の数は判断回数以下。 |
| RecoveryOnlyAfterFreeze | frozen_recovered にいるなら、判断回数は IdleThreshold 以上。主張としては弱い(気になった点の9)。IdleThreshold &gt; MaxDecisions なら前提が真にならず、空虚になる。 |
| RecoveryResetsPageState | frozen_recovered にいるなら、idleCounter=0 で、ドメインの中にいる。ドメインの中にいることは、RecoverFromFreeze が戻すのではなく、到達可能性から成り立つ(気になった点の8)。 |
| DomainRecordAccurate | inAllowedDomain は pageOutside の否定と常に等しい。 |
| ProposalWellFormed | awaiting か acting にいる間、op は Operations か back、target は Elements か none。awaiting で直接操作でなければ target は none。直接操作で target が none なら対象は存在しない。直接操作で Elements が空でなければ target は Elements に属する。explore では done にならない。 |
| StartsFresh | 状態述語。性質として与えるなら初期状態だけを検査する。中身は Init の一部を言い直したもので、Init から自明に出る。不変条件として与えると、phase が idle から離れた時点で破れる。 |

### 行動性質と時相性質

| 名前 | 言い直し / 前提が真になりうるか |
|---|---|
| GoalPreempts | idle でゴールに達しているなら、次は goal_achieved で done になるか、環境の出来事が起きる。到達できる。 |
| StageAdvancePreempts | idle で段をクリアしたなら、idle のまま段が1つ進むか、環境の出来事が起きる。NumStages ≥ 2 のときだけ到達でき、NumStages=1 では空虚。 |
| StuckPreempts | idle で、完了しておらず、行き詰まりカウンタが閾値以上なら、次は stuck になるか、環境の出来事が起きる。StuckThreshold ≤ MaxDecisions でないと到達できない。 |
| LimitStops | idle で、完了しておらず、行き詰まっていないが、予算切れか時間切れなら、該当する終了状態へ進むか、環境の出来事が起きる。到達できる。 |
| FreezePreemptsDecision | idle で固まり判定に達しているなら、次に awaiting へは進まない。言っているのは「判断を求めない」ことだけで、回復することまでは言っていない。 |
| SafeProposalProceeds | awaiting で、ペイウォールでなく、安全な提案なら、次は acting か環境の出来事。 |
| AwaitingOnlyByDecision | awaiting に入ることと判断回数が1増えることは同値。 |
| ActingOnlyAfterGuardrails | acting へは awaiting からしか入らない。 |
| FreezeRecoveryRecorded | idle で、完了しておらず、固まり判定に達していて、idleCounter が0に戻るなら、次は frozen_recovered。 |
| VisitedChangesOnlyByExecution | visited が変わるのは acting から idle への遷移に限る。 |
| PaywallForcesBack | awaiting でペイウォールなら、次は op を back にして acting へ進むか、環境の出来事。 |
| ProposalOutcome | acting の次は、op が done なら done、stuck なら stuck、それ以外なら idle。または環境の出来事。 |
| IdleCounterTracksScreen | acting から idle へ戻るとき、画面が変われば idleCounter は0、変わらなければ1増える。 |
| MemoryNeverShrinks | visited は減らない。 |
| StuckCounterTracksNewElements | acting から idle へ戻るとき、stuckCounter は StuckAfterOp と同じ式で更新される。定義をもう一度書いたもので、同語反復に近い。 |
| NoDecisionOutsideDomain | idle から awaiting へ進むとき、ドメインの外にはいない。 |
| StageAdvanceResetsCounters | 段が変わるのは、段をクリアした idle から idle への遷移で、1つだけ進み、2つのカウンタが0になる場合に限る。 |
| BudgetSharedAcrossStages | 段が変わっても予算と判断回数は変わらない。 |
| DecisionBudgetNeverRestored | 判断予算は増えない。 |
| TimeNeverRewinds | 時間切れは元に戻らない。 |
| TimeElapsesAlone | 時間切れになる遷移では、ほかの変数は変わらない。 |
| ModeFixed | モードは変わらない。 |
| ProposalChangesOnlyByDecision | 提案の5つ組が変わるのは、awaiting に入るときか、ペイウォールで op だけを back に差し替えるときに限る。 |
| CountersChangeOnlyBySteps | idleCounter が変わるのは、実行、固まりからの回復、段の前進のどれか。stuckCounter が変わるのは、実行か段の前進。 |
| DomainChangesOnlyByNavigation | ドメインの状態が変わるのは、実行か、idle から idle でドメインの中へ戻る遷移に限る。 |
| ObservationChangesOnlyByExecution | screenChanged と newSeen が変わるのは実行のときだけ。 |
| DoneWhenChangesOnlyByExecutionOrAdvance | doneWhenMet が偽から真になるのは実行か段の前進のとき。真から偽になるのは段の前進のときだけ。 |
| TerminationKeepsState | 終了状態に入る遷移では、phase と exitReason 以外は変わらない。 |
| EventuallyTerminates | いつかは必ず終了状態に入る。 |
| TerminalAbsorbing | 終了状態に入ったら、すべての変数が変わらない。終了状態で有効な遷移は足踏みだけなので成立する。 |

## 6. Spec

Spec は、Init から始まり、毎ステップ Next か足踏みを取る振る舞い全体。そのうえで次の遷移に弱公平性(WF)を仮定している。RequestDecision、ResetRequestWindow、ApplyGuardrails、3つの Execute をまとめた選択、ReturnToAllowedDomain、FinishByProposalDone、FinishByProposalStuck、RecoverFromFreeze、ResumeAfterRecovery、AdvanceStage、FinishNormally、FinishStuck、ExceedDecisionBudget、FinishTimeUp。

TimeElapses と UserAbort には公平性を課していない。したがって EventuallyTerminates は、時間切れやユーザーの中断に頼らず、システム自身の遷移が弱公平に進むことだけを仮定して主張している。その論拠は次のとおり。

- 終了していない各状態で、システムの遷移が少なくとも1つ有効になっている。idle の場合を順に見る。DoneWhen が真なら FinishNormally か AdvanceStage。偽で行き詰まっていれば FinishStuck。予算が0なら ExceedDecisionBudget。時間切れなら FinishTimeUp。どれでもなければ Recover、Return、Reset、RequestDecision のどれか。awaiting では ApplyGuardrails が必ず有効になる。acting では Execute か Finish が有効になる。
- 予算を使わずに起きる遷移(Reset、Return、Recover と Resume、Advance)は、どれも続けて何度も起きられない。

このため判断予算が単調に減り、いずれ終わる。ただし、これは Operations が空でない場合に限る(気になった点の1)。

## 気になった点

1. **ProposableOps が空になりうる(EventuallyTerminates)**。Operations が空でもよいことになっていて、explore で Operations={"done"} の場合も ProposableOps は空になる。すると RequestDecision が有効にならない。初期状態では要求予算が満タンなので ResetRequestWindow も有効でない。ほかにはシステムの遷移がなく、公平性のない TimeElapses と UserAbort しか残らないので、足踏みが永久に続き、EventuallyTerminates が破れる。ASSUME に Operations の非空、さらに explore 用に Operations \ {"done"} の非空が抜けているように見える。
2. **scenario は判断0回でもゴールに達する(Init、AdvanceStage、FinishNormally、ScenarioGoalOnlyAtLastStage)**。Init で doneWhenMet が真になりうるうえ、AdvanceStage の観測し直しでもまた真になりうる。このため、一度も判断せずに全段を素通りして goal_achieved で終われる。explore の DoneWhen が decisionsMade &gt; 0 を要求しているのと非対称になっている。
3. **Explorable が空の explore**。visited={}=Explorable が初めから成り立つので、最初の判断を1回した時点で DoneWhen が真になる。その判断がガードレールに退けられた提案であってもよい。行き詰まりや予算切れより先に goal_achieved で終わる。
4. **Elements が空、または全要素が禁止要素**。Elements が空だと、click と input の対象は none になり、targetExists が偽に固定されるので、毎回ガードレールに退けられる。全要素が禁止の場合も、対象は必ず禁止要素なので退けられる。どちらの場合も予算を消費するだけになる。
5. **直接操作を含まない Operations**。visited は増えないので、Explorable が空でない explore はゴールに届かず、行き詰まりか予算切れで終わるしかない。StuckAfterOp は newSeen' だけで0に戻る。
6. **退けた提案は行き詰まりにも固まりにも数えない(ApplyGuardrails)**。禁止要素への操作を延々と提案し続けても、2つのカウンタは動かず、予算切れでしか止まらない。意図したものかは確認したほうがよい。
7. **ドメインの外にいてもゴールに達する(ExecuteOutOfDomain、FinishNormally)**。ExecuteOutOfDomain でも doneWhenMet を観測し、visited を増やす。FinishNormally はドメインの中にいることを要求しない。このため、許可ドメインの外でゴールに達して終われる。また、scroll や wait、ペイウォールで差し替えた back でもドメインの外に出られる。
8. **RecoveryResetsPageState は名前と中身が合わない**。RecoverFromFreeze は idleCounter しか戻さず、画面もドメインも触らない。inAllowedDomain が真なのは「ドメインの外では idleCounter=0 のままで、判断も求められない」ことから間接的に出る結論にすぎない。
9. **RecoveryOnlyAfterFreeze は弱い**。言っているのは「判断回数 ≥ IdleThreshold」だけで、直前に固まっていたことまでは言っていない。IdleThreshold &gt; MaxDecisions の設定では、これと FreezeRecoveryRecorded、FreezePreemptsDecision の前提がどれも真にならず、空虚になる。同じように、StuckThreshold &gt; MaxDecisions なら StuckPreempts も空虚になる。NumStages=1 なら StageAdvancePreempts、StageCleared、AdvanceStage がどれも空虚になる。定数の値はこの文書には書かれていないので確かめられない。
10. **要求の窓が実質的に制限になっていない(ResetRequestWindow、RequestsWithinWindowCap)**。窓のリセットは時間と結びついておらず、要求を1回出した直後でも起きる。このため requestBudget &gt; 0 の条件が判断を遅らせることはない。RequestsWithinWindowCap と DecisionsWithinCap は BudgetAccounting の系で、余分になっている。
11. **ペイウォールは done と stuck の提案も back に差し替える(ApplyGuardrails)**。エージェントが done や stuck を報告しても、isPaywall が真なら back が実行され、終了しない。ペイウォールは画面の状態ではなく、提案と一緒に決まる値として表している。さらに isPaywall は次の RequestDecision まで真のまま残る。
12. **画面が変わらなくても完了条件が真になる(ExecuteNoChange)**。ExecuteNoChange でも ObserveDoneWhen が働くので、画面が変わらないのに完了条件だけが満たされる場合がある。
13. **時間切れは操作の途中では割り込まない(TimeElapses、FinishTimeUp)**。awaiting や acting の最中に時間切れになっても、実行が済んで idle に戻るまで止まらない。予算切れも同じで、最後の判断までは実行される。また、時間切れの後も AdvanceStage と FinishNormally は起きる。
14. **同語反復と余分な条件**。StuckCounterTracksNewElements と IdleCounterTracksScreen は、各アクションの更新式をそのまま書き直したもの。DirectOpActedOnlyOnSafeTarget の ~isPaywall は余分(ペイウォールの acting では op が必ず back)。pageOutside は常に ~inAllowedDomain と等しく、変数として冗長。Next の最後にある終了状態の足踏みは [][Next]_vars でもともと許されている。デッドロック検出を避けるためのものだと読める。
15. **StartsFresh の扱い**。状態述語なので、性質として与えれば初期状態しか検査しない。Init の一部を言い直しているだけで、必ず成り立つ。不変条件として登録すると破れる。
16. **visited を段をまたいで持ち越す(AdvanceStage)**。scenario で visited は stuckCounter の計算にしか使わない。そのため前の段で訪れた要素を新しい段で操作しても「新しい要素」と数えられず、行き詰まりカウンタが増えやすい。
17. **低確信は back のときだけ意味を持つ**。低確信の click や input は退けられない。直接操作以外の op の targetExists も自由に選ばれるが、どこからも参照されない。
18. **ゴール到達を保証する活性はない**。EventuallyTerminates はどの終了状態に入るかを問わない。「達成できるゴールには必ず達する」という性質はない。エージェントの判断を非決定的に表しているので、書けないのは当然とも言える。
````
