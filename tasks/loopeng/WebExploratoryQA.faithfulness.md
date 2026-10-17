# 文面の往復確認: WebExploratoryQA

対象: `.tla`で状態機械化したEARS要件(トレーサビリティ・マトリクスの「モデル外」以外の行)。
読み手: fresh contextのsubagent(general-purpose)。入力は`WebExploratoryQA.faithfulness-input.md`(現行specからコメントを機械的に除いた形式文面のみ。S-ID・EARS文は含まない)。
作成: 2026-10-18。2回目の中ループレビュー(不合格)を受けて強化したspecに対して、全行をやり直した。

| 要件(S-ID) | EARS文 | 式 | 読み手の言い直し | 判定 | 理由/直し方 |
|---|---|---|---|---|---|
| S-002 | WHEN判断モデルが操作種別と対象要素を返したとき(かつガードレールで禁止されていないとき)、システムはその操作種別・対象要素に従って実際にブラウザ操作を実行しなければならない | `ExecuteNoChange`/`ExecuteChange`/`ExecuteOutOfDomain`、`SafeProposalProceeds`、`ProposalOutcome` | 「ガードに掛からない提案はactingへ進む」「actingの次はdoneならdone、stuckならstuck、それ以外は実行してidleへ。click/inputなら対象をvisitedに加える」 | partial | 許可された提案が必ず実行段階へ進み、実行の帰結が記録されることは検査している。ブラウザ操作という副作用そのものは状態変数に現れない(model.md「モデル化で捨てた領域」)。 |
| S-005 | WHEN 1ステップを実行するとき、システムは次に取るべき操作の種別を判断モデルに問い合わせなければならない | `RequestDecision`、`ProposalWellFormed` | 「Continuingかつ固まり未検出かつリクエスト予算ありかつドメイン内のとき、opをOperationsから非決定に選び、予算を減らして履歴を増やす」「問い合わせ後の提案は8種のいずれか(paywall時はback)」 | partial | 問い合わせ行為(API呼び出し)はモデル外。どんな値が返っても処理できることを非決定的選択で表す。 |
| S-006 | WHEN 操作対象が必要な操作種別が選ばれたとき、システムは対象要素を判断モデルに問い合わせなければならない | `RequestDecision`、`ProposalWellFormed` | 「targetをElementsから非決定に選ぶ。非Direct操作でもtargetは選ばれる」 | partial | S-005と同じ理由。対象不要の操作でもtargetを持つが、`VisitedAfterOp`はclick/input以外でtargetを使わないため振る舞いに影響しない。 |
| S-010 | IF 判断モデルが返した操作がbackかつその確信度が0.7未満のとき、システムはその操作を実行してはならない | `ApplyGuardrails`(B1)、`LowConfidenceBackNeverActed`、`SafeProposalProceeds` | 「非paywallでback∧lowConfならidleへ却下」「acting・非paywall・backならlowConfでない。paywall由来は対象外」「ガードに掛からない提案は却下しない」 | full | 拒否する側と、拒否しすぎない側の両方を検査している。paywall除外はS-013による意図どおりの非対称。0.7の閾値はBOOLEANへの抽象化(設計メモ8)。前件到達はreach.log v3b-rerun。 |
| S-011 | IF 直前の操作後に画面が変化せず、同じ要素を再度選択しようとしたとき、システムは次に確信度の高い候補要素へ切り替えなければならない | `ExecuteNoChange`、`IdleCounterTracksScreen` | 「screenChanged=FALSE、idleCounter+1」「実行の一歩で画面が変化しなければidleCounter+1、変化すれば0」 | partial | 画面無変化の観測は表現されているが、次善候補への切替という選択ロジックは`RequestDecision`の非決定的選択に吸収され、モデル外。 |
| S-012 | IF 対象要素が購入・削除・退会・サインイン等の破壊的/決済操作に分類されるとき、システムはその要素を操作してはならない | `ApplyGuardrails`(B2)、`NeverOperatesForbidden`、`SafeProposalProceeds` | 「Directでforbiddenならidleへ却下」「visited∩Forbidden={}。paywall分岐はbackにするので追加しない」 | full | 一致。禁止要素の分類ロジックの正しさは設計メモ8と model.md の`ForbiddenElements`節のとおり実装で検証する。 |
| S-013 | IF 課金画面相当のUIが検出されたとき、システムは判断モデルに問い合わせず既定の離脱操作を実行しなければならない | `ApplyGuardrails`(A) | 「isPaywallならactingへ進み、opをbackに上書きする。lowConfでも、元の提案がdone/stuckでもback」 | partial | 既定の離脱操作を、判断モデルの提案と無関係に実行する点は一致する。ただしspecでは、paywallの観測が問い合わせの応答と同時に届くため、paywallでも`RequestDecision`で問い合わせて判断予算とリクエスト予算を1ずつ消費する。「問い合わせない」は検査していない。予算を多めに使う側にずれるだけで、安全性(禁止操作をしない・上限を超えない)には影響しない。実装では問い合わせの前にpaywallを判定し、問い合わせを省く。 |
| S-014 | システムはゴール達成を判断モデルの自己申告ではなく、各stageに設定されたdone_when条件で判定しなければならない | `FinishNormally`、`FinishByProposalDone`、`GoalOnlyByDoneWhen`、`GoalPreempts` | 「visited=ExplorableならidleからdoneでGoal」「done提案は常にreported_done」「goal_achievedならvisited=Explorable」「visited=Explorableのidleからの次の一歩は必ずdone∧goal_achieved」 | full | ゴール達成はdone_whenだけで判定し、done_whenが成り立ったら他の終了理由や継続に先取りされない。 |
| S-017 | IF 探索中にページが応答しない・対象ドメイン外に遷移した・行き詰まり状態になったとき、システムは対象ページを初期状態に戻して探索を継続しなければならない。初期状態に戻すのはページの状態だけで、探索の記憶(S-022)は保持する | `RecoverFromFreeze`/`ResumeAfterRecovery`、`RecoveryResetsPageState`、`FreezePreemptsDecision`、`MemoryNeverShrinks` | 「Continuing∧idleCounter≥閾値ならfrozen_recoveredへ、idleCounter=0、他は不変」「frozen_recoveredならidleCounter=0∧ドメイン内」「固まり検出中はawaitingへ行かない(frozen_recoveredへの遷移までは要求しない)」「visitedは減らない」 | partial | 復旧はページ状態だけを戻し、記憶を保持することは検査している。固まりを検出したら必ず復旧することは、性質として書いていない。問い合わせは禁止され(`FreezePreemptsDecision`)、"idle"に残るアクションは時間窓の満了・ドメイン復帰・復旧だけなので、弱公平性の下でいずれ復旧が起きる。ただしこれは推論で、TLCの性質としては検査していない。ドメイン外と行き詰まりは別のdisjunctで扱う(extract.md設計メモ2)。 |
| S-021(固まり) | システムは一定時間操作後も画面に変化がない状態を「固まり」として判定しなければならない | `RecoverFromFreeze`のガード、`IdleCounterTracksScreen`、`CountersBoundedByOps` | 「idleCounter≥IdleThreshold」「画面無変化なら+1、変化なら0」 | full | 一致。 |
| S-021(ドメイン外) | システムは操作の結果が許可ドメインの外への遷移であることを検出できなければならない | `ExecuteOutOfDomain`、`DomainRecordAccurate` | 「pageOutside=TRUE、inAllowedDomain=FALSE」「inAllowedDomain=¬pageOutside(同時に更新されるので冗長)」 | full | 記録が実際のページ位置と一致することを検査する。読み手が指摘した冗長さは意図どおりで、記録を壊すmutantを捕まえるための性質。検出の判定ロジック(URLの比較)の正しさは設計メモ8。 |
| S-021(行き詰まり) | システムは既定の試行回数操作しても新規要素が出ない状態を「行き詰まり」として判定しなければならない。新規要素が出ないとは、操作の結果、探索済みの要素の集合が増えないことをいう | `FinishStuck`のガード、`StuckAfterOp`、`StuckCounterTracksNewElements`、`StuckPreempts` | 「Exploring∧stuckCounter≥閾値ならstuck終了」「visitedが増えなければ+1、増えれば0」「行き詰まり検出時の次の一歩は必ずstuck」 | full | 一致。前回のmismatch(画面変化で数えていた)は解消した。ガードで却下された提案はカウンタを進めないが、却下は「操作」ではないので要件と矛盾しない。 |
| S-022 | システムは画面状態と操作による遷移を実行をまたいで永続化したグラフとして保持しなければならない | `VisitedAfterOp`、`MemoryNeverShrinks` | 「Directなら対象をvisitedに加える」「visitedは減らない」 | partial | 記憶が失われないことは検査している。グラフ構造は単一集合`visited`で近似し(model.mdの簡略化)、実行をまたぐ永続化は状態変数の外。 |
| S-023 | システムは探索の優先順位として、未操作要素→未探索分岐が残る既操作要素→戻る/スクロールの順で次の操作を選択しなければならない | `VisitedAfterOp`、優先順位はモデル外 | 同上 | partial | 優先順位の規則は`RequestDecision`の非決定的選択に吸収(model.md「捨てた領域」)。 |
| S-028(存在確認) | IF 操作対象要素として選択されたが、実際に操作を実行する直前に当該要素が存在しないとき、システムは操作を中止しなければならない | `ApplyGuardrails`(B3)、`SafeProposalProceeds` | 「Directで¬targetExistsならidleへ却下」 | full | 一致。存在判定のロジックは設計メモ8。 |
| S-028(決済導線拒否) | IF 対象要素が決済導線に分類されるとき、探索モードはその要素を選択してはならない | `ApplyGuardrails`(B2)、`NeverOperatesForbidden` | S-012と同じ | full | 決済導線が`ForbiddenElements`に含まれる前提で一致する。 |
| S-033 | IF 操作の結果、対象ページが許可ドメインの範囲外に遷移したとき、システムはその遷移を記録し探索対象外として扱わなければならない | `ExecuteOutOfDomain`/`ReturnToAllowedDomain`、`RequestDecision`のガード、`DomainRecordAccurate`、`NoDecisionOutsideDomain` | 「ドメイン外を記録する」「記録は真値と一致」「実際にドメイン外にいる間はidle→awaitingの一歩が起きない」「Continuing∧ドメイン外なら戻る」 | full | 遷移を記録すること(`inAllowedDomain`)と、ドメイン外で探索しないことの両方を検査している。前回partialにした「記録」は、記録変数と真値の一致として検査するようになった。 |
| S-041 | システムは判断モデルAPIへの呼び出し回数または実行時間に上限を設定でき、上限に達したとき実行を停止しなければならない | `RequestDecision`/`ExceedDecisionBudget`、`DecisionsWithinCap`、`BudgetAccounting`、`BudgetExceededOnlyWhenSpent`、`BudgetStops`、`TerminalAbsorbing` | 「呼び出しごとに予算−1・履歴+1」「履歴≤上限」「予算+履歴=上限」「budget_exceededなら予算0」「上位条件なしで予算0のidleの次は必ずerror_exit」「終端からは変化しない」 | full | 呼び出し回数が上限を超えず、上限到達で必ず停止し、再開しない。実行時間の上限はモデル外(回数で代表)。 |
| S-042 | システムは探索モードにおける単位時間あたりの操作回数に上限を設け、対象サーバへの過度な負荷を避けなければならない。単位時間の窓は時刻で区切られ、どの時点でも満了しうる | `RequestDecision`/`ResetRequestWindow`、`RequestsWithinWindowCap`、`BudgetAccounting` | 「リクエスト予算>0のときだけ問い合わせ、予算−1・窓履歴+1」「予算を1つでも使っていればいつでも満タンに戻し窓履歴を0に」「窓履歴≤上限」 | partial | 1つの窓で上限を超えないことは検査している。窓の満了は時刻依存で、モデルは「いつでも満了しうる」として扱う(要件の文面どおり)。1窓の実時間の長さはモデル外で、実装のレート制御のテストで確かめる。 |
| S-044(goal) | WHEN done_when条件が満たされたとき、システムは実行をゴール達成として記録しなければならない | `FinishNormally`、`GoalPreempts`、`ExitReasonMatchesPhase` | 「visited=Explorableのidleからの次の一歩は必ずdone∧goal_achieved」 | full | 前回の反例(done_whenを満たしたままstuckや予算超過で終わる)は`GoalPreempts`で排除した。 |
| S-044(reported_done) | WHEN operation=doneが返されたがdone_when条件が満たされていないとき、システムは実行を未検証の終了(reported_done)として記録しなければならない | `FinishByProposalDone`、`ReportedDoneOnlyWithoutDoneWhen`、`ProposalOutcome` | 「acting∧done提案ならdone∧reported_done」「reported_doneならvisited≠Explorable(到達可能性で成り立つ)」 | full | 一致。 |
| S-044(stuck) | WHEN operation=stuckが返されたとき、またはS-021の行き詰まり検出条件が成立したとき、システムは実行を行き詰まり終了として記録しなければならない | `FinishByProposalStuck`/`FinishStuck`、`StuckPreempts`、`ExitReasonMatchesPhase` | 「stuck提案ならstuck∧stuck」「行き詰まり検出時の次の一歩はstuck(終了理由は検査しない)」「stuckならexitReason=stuck」 | full | `StuckPreempts`は終了理由を見ないが、`ExitReasonMatchesPhase`がphaseと終了理由の対応を検査するので、合わせて一致する。 |
| S-044(優先順位) | WHILE 判断を問い合わせる前の待機状態にあるとき、システムはゴール達成、行き詰まり、判断予算切れ、固まり復旧・ドメイン復帰・リクエスト窓の満了、判断の問い合わせの順に、成り立つ条件のうち最も優先度の高いものだけを実行しなければならない | `Exploring`/`Continuing`の各ガード、`GoalPreempts`、`StuckPreempts`、`BudgetStops`、`FreezePreemptsDecision`、`NoDecisionOutsideDomain` | 「1.FinishNormally 2.FinishStuck 3.ExceedDecisionBudget 4.Continuing群(内部に完全な優先順位は無い。復旧と問い合わせ、ドメイン復帰と問い合わせは排他。窓の満了は他と同時に有効になりうる)」 | full | 1〜3の順と、4の段の中で「固まり中・ドメイン外では問い合わせない」ことは要件どおり。4の段の中で窓の満了と他が同時に有効になるのは、要件で同じ段に置いたとおり。 |

## 追加した性質の言い直し(3回目のレビューの後)

3回目のレビュー後に足した8本の性質を、別の fresh reader に言い直させた。どれも上の判定表の行を補強するもので、行の判定は変えない(S-017 は「必ず復旧する」を性質として書いていないので partial のまま)。

| 性質 | 支える行 | 読み手の言い直し | 判定 | 理由 |
|---|---|---|---|---|
| `DirectOpActedOnlyOnSafeTarget` | S-012, S-028 | 「acting・非paywall・直接操作なら、対象は存在して禁止要素でない」 | full | 存在確認のガードを外した書き換えを捕まえる(2回目のレビューの指摘X)。 |
| `PaywallForcesBack` | S-013 | 「awaiting・paywall からの一歩は acting かつ back。対象や確信度は縛らない」 | full | paywall を却下にする書き換えを捕まえる(指摘V)。対象や確信度を縛らないのは、paywall の back が要素を操作せず、S-013 で S-010 の対象外だから。 |
| `VisitedOnlyByOps` | S-002, S-022 | 「visited の要素数は判断の回数以下。中身が正しいかは見ない」 | full(名前より弱い) | 個数の上限だけ。中身の正しさは`NeverOperatesForbidden`と`VisitedChangesOnlyByExecution`が見る。1回の実行の中の記憶として読み、実行をまたぐ永続化は S-022 の行のとおりモデル外。 |
| `RecoveryOnlyAfterFreeze` | S-017 | 「frozen_recovered なら累計の判断回数が閾値以上。直前の固まりの検出は見ない」 | full(名前より弱い) | 初期状態から直接復旧状態へ入る書き換えを捕まえるための性質。固まりの検出の条件は`RecoverFromFreeze`のガードが持つ。 |
| `AwaitingOnlyByDecision` | S-005, S-041 | 「awaiting に入る一歩と、判断回数がちょうど1増える一歩が同値」 | full | 予算を消費しない問い合わせと、問い合わせ無しの予算消費の両方を捕まえる。 |
| `ActingOnlyAfterGuardrails` | S-010, S-012, S-013, S-028 | 「acting に入るのは awaiting からだけ。ガードの判定の正しさは見ない」 | full(名前より弱い) | ガードを飛ばす遷移だけを捕まえる。判定の正しさは`DirectOpActedOnlyOnSafeTarget`・`LowConfidenceBackNeverActed`・`SafeProposalProceeds`が見る。 |
| `FreezeRecoveryRecorded` | S-017 | 「固まり検出中の idle から固まりカウンタを0にする一歩は frozen_recovered へ。必ず復旧することは言わない」 | full(名前より弱い) | 復旧を記録せずにカウンタだけ戻す書き換えを捕まえる。必ず復旧することは S-017 の行のとおり推論にとどまる。 |
| `VisitedChangesOnlyByExecution` | S-002, S-022 | 「visited が変わるのは acting→idle の一歩だけ。変化の中身は縛らない」 | full | 終了遷移などで visited を書き換える書き換えを捕まえる。 |

読み手は、`RecoveryOnlyAfterFreeze`と`FreezeRecoveryRecorded`が`IdleThreshold > MaxDecisions`の設定で、`DirectOpActedOnlyOnSafeTarget`が`Operations`に click も input も無い設定で空成立すると指摘した。検査に使う cfg(`IdleThreshold=2`、`MaxDecisions=4`、`Operations`に click を含む)では、どの前件も到達できることを reach.log の「v3c」節で確かめた。

## 空成立の証跡(含意・前提付きの式)

`WebExploratoryQA.reach.log`の「=== v3」「=== v3b」「v3c」節に、現行specに対する前件ごとの結果を追記した。各節に`NotPx`の定義文をそのまま書き、反例(前件が成り立つ到達状態または一歩)が出ている。性質の追加・変更後に、v3の全エントリを現行specで再実行した(v3b-rerun)。補助のAUX1(成り立つことの確認)はNo errorで、期待どおり。

同ログの先頭の節(v2以前)は旧spec向けで、現行specの判定には使っていない。

## 定数の前提

`EventuallyTerminates`は、`ASSUME`で正と定めた閾値・上限(`MaxDecisions`、`MaxRequests`、`IdleThreshold`、`StuckThreshold`)の下での主張。読み手は、`Operations`が空で`Explorable`が空でないと、"idle"でデッドロックすることも指摘した。検査に使う`WebExploratoryQA.cfg`では`Operations`は空でない。実装では、設定値の検証で閾値の0と空の操作種別を拒否する。

## mutation oracle(現行spec)

372 mutant、survivor 53。全mutantの結果種別・状態数・理由は`WebExploratoryQA.mutants-v3.tsv`(`tools/classify_mutants.py`で生成。7列目が理由)。

- 性質違反で捕まえたもの: 102(INV 35、行動性質 64、時相性質 3)
- エラーで止まったもの: 217(構文エラー129、評価時エラー88)。killedの証拠には数えていない。すべて意味の通る書き換えで再検査した。値を差し替える行は、取りうる別の値を1つずつ代入して回した(`x' \in DOM \ {c}`のように集合でまとめると、無関係な値で性質違反が出て穴を隠すため。3回目のレビューの指摘)。約395回の実行で、まとめた形では killed に見えていた14行が値ごとでは緑になり、うち8行は本物の穴だった。性質を6本足して塞いだ(`VisitedOnlyByOps`、`RecoveryOnlyAfterFreeze`、`AwaitingOnlyByDecision`、`ActingOnlyAfterGuardrails`、`FreezeRecoveryRecorded`、`VisitedChangesOnlyByExecution`)。
- 緑のまま残ったもの53件は、コメント内の字面12、性質側の変異(Init/Nextは不変)18、連言リスト先頭の`/\`が1項の`\/`になるもの22、時間窓リセットのガードを`\leq`にするもの1。最後の1件と、再検査で緑だったガード外し2件は、`-fp 0 -dump dot`の状態グラフから自己ループを除くと元と一致した(sha256はTSVの7列目)。状態数の一致は等価の根拠にしていない。
- 2回目と3回目のレビューで指摘された意味のある書き換え(A〜N、X、V)は、どれも性質違反で捕まる。
- 振る舞いを変えるのに緑のままのもの(abstraction-limited): 次の3種類。どれも要件が許す環境側の振る舞いを減らすだけで、誤った振る舞いを足さない。安全性の性質は「ある振る舞いが起こりうること」を要求できないので区別できない。
  - 判断モデルの応答や画面の観測値を1つの値に固定する書き換え(`RequestDecision`の`op`、`t`、`lc`、`pw`、`ex`)。応答の多様さは実装のテストで確かめる。
  - 値ごとの再検査で実行の結果の1つが起こらなくなるもの(`ExecuteNoChange`の`idleCounter' = 1`と、3つのExecuteの`exitReason`を外した行)。
  - 時間窓のリセットを「予算が0のときだけ」に戻す書き換え(Hrev)。窓の満了は時刻依存でモデルの外にある。満了のタイミングは実装のレート制御のテストで確かめる。
- 到達状態を変えるのに緑になる本物のsurvivorは0件。

## 総括

判定表は全23行(状態機械化した17件のうち、S-021が3行、S-028が2行、S-044が4行に展開される)。判定列を`grep '^| S-' | awk -F'|' '{print $6}' | sort | uniq -c`で集計した結果は次のとおり。

- `full`: 14件(S-010, S-012, S-014, S-021×3, S-028×2, S-033, S-041, S-044×4)
- `partial`(理由付き): 9件(S-002, S-005, S-006, S-011, S-013, S-017, S-022, S-023, S-042)
- `mismatch`/`vacuous`/`missing`: 0件

partialの行は「検証済み」に含めない。
