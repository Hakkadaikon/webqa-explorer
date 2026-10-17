# 文面の往復確認: WebExploratoryQA

対象: `.tla`で状態機械化したEARS要件(トレーサビリティ・マトリクスの「モデル外」以外の行)。
読み手: fresh contextのsubagent(general-purpose)。入力は`WebExploratoryQA.faithfulness-input.md`(形式文面のみ、コメント・S-ID・EARS文は含まない)。

| 要件(S-ID) | EARS文 | 式 | 読み手の言い直し | 判定 | 理由/直し方 |
|---|---|---|---|---|---|
| S-002 | WHEN判断モデルが操作種別と対象要素を返したとき(かつガードレールで禁止されていないとき)、システムはその操作種別・対象要素に従って実際にブラウザ操作を実行しなければならない | `ExecuteOperation` | 「phaseが"acting"のときに成立し、proposedOpがclickかinputかつproposedTargetがElementsに属するならvisitedにそのターゲットを追加し、そうでなければvisitedは変化しない」 | partial | 言い直しは「visitedへの反映」という結果だけを述べており、「実際にブラウザ操作を実行する」というS-002の行為そのもの(タップ・入力・スクロール等の副作用)はモデル化対象外(model.md「モデル化で捨てた領域」に準じる抽象化)。構造的な帰結(探索済み集合の更新)はfullに一致するが、操作実行という行為自体は状態変数に現れないため部分一致とする。 |
| S-005 | WHEN 1ステップを実行するとき、システムは次に取るべき操作の種別を判断モデルに問い合わせなければならない | `RequestDecision`(非決定的選択部分) | 「proposedOp・proposedTarget等を非決定的に選んだ値に更新する」 | partial | 「問い合わせる」という行為(判断モデルへの実際のAPI呼び出し)はモデル外。モデルは「問い合わせた結果として非決定的な値を受け取る」という帰結のみを表現する。これはS-005の意図(どんな値が返ってきても処理できる)を安全側に抽象化したものであり、mismatchではない。 |
| S-006 | WHEN 操作対象が必要な操作種別が選ばれたとき、システムは対象要素を判断モデルに問い合わせなければならない | `RequestDecision`(同上) | 同上 | partial | S-005と同じ式に統合されているため同一の理由。対象要素の問い合わせを操作種別の問い合わせと無条件に同時に行う設計(1回のRequestDecisionでproposedOpとproposedTargetを同時に決定)だが、これは記事原文(A)§4の「1回の呼び出しでoperation/targetを同時に質問する」という仕様と一致しており、分離しないことがむしろ忠実。 |
| S-010 | IF 判断モデルが返した操作がbackかつその確信度が0.7未満のとき、システムはその操作を実行してはならない | `ApplyGuardrails`(該当分岐) / `LowConfidenceBackNeverActed` | 「(1)でproposedOpが"back"でproposedLowConfBackが真なら、phaseを"idle"に戻す」「含意: 前提が成り立つならphaseは"acting"ではない」 | full | 言い直しは「低確信度backが実行(acting)に進まない」ことを正確に述べている。0.7という具体的な閾値は`proposedLowConfBack`というBOOLEANへ抽象化しているが、「閾値未満→拒否」という構造はfullに一致する。空成立でないことはreach.log(P1)で確認済み。 |
| S-011 | IF 直前の操作後に画面が変化せず、同じ要素を再度選択しようとしたとき、システムは次に確信度の高い候補要素へ切り替えなければならない | `ObserveNoChange` | 「phaseが"acting"かつproposedOpがdone/stuckでないときに成立し、phaseを"idle"に戻し、screenChangedをFALSEにし、idleCounterとstuckCounterを両方1増やす」 | partial | 言い直しは「画面無変化の観測」のみを述べており、「次善候補への切替」という能動的な選択ロジックは式に現れない(次のRequestDecisionの非決定的選択に暗黙に吸収される設計)。model.mdの「捨てた領域」に明記した簡略化の範囲内であり、観測部分はfullだが選択ロジックはモデル外、全体としてpartial。 |
| S-012 | IF 対象要素が購入・削除・退会・サインイン等の破壊的/決済操作に分類されるとき、システムはその要素を操作してはならない | `ApplyGuardrails`(該当分岐) / `NeverOperatesForbidden` | 「proposedOpがclick/inputでターゲットがForbiddenElementsに含まれるなら、phaseを"idle"に戻す」「visitedにはForbiddenElementsの要素が一度も含まれない」 | full | 「禁止要素への操作を拒否し、結果として実行(visited追加)に至らない」という構造が正確に一致する。空成立でないことは元の2回目レビューでTLCのバグ発見(paywall経由の混入)という実例でも裏付けられている。 |
| S-013 | IF 課金画面相当のUIが検出されたとき、システムは判断モデルに問い合わせず既定の離脱操作を実行しなければならない | `ApplyGuardrails`(isPaywall分岐) | 「isPaywallが真なら、phaseを"acting"にしproposedOpを"back"に強制的に書き換える」 | full | 「判断モデルに問い合わせず」という部分は、むしろ`RequestDecision`で既に得た提案を`ApplyGuardrails`が上書きする設計であり、「判断モデルの提案を無視して既定操作(back)を強制する」という要件の意図と一致する。問い合わせ自体をスキップする(RequestDecisionを呼ばない)という字面通りの実装ではないが、結果として判断モデルの提案は使われないので意味は一致する。 |
| S-014 | システムはゴール達成を判断モデルの自己申告ではなく、各stageに設定されたdone_when条件で判定しなければならない | `FinishNormally` | 「phaseが"idle"かつvisitedがElementsからForbiddenElementsを除いた集合と等しいときに成立し、phaseを"done"に、exitReasonを"goal_achieved"に遷移させる」 | full | 「視覚的なdone_when条件(全要素探索済み)による構造的終了」という設計が、「自己申告を使わない」というS-014の否定の比重を正確に反映している。判断モデルの自己申告(proposedOp="done")は別の経路(S-044/FinishByProposalDone)として明確に分離されており、混同していない。 |
| S-017 | IF 探索中にページが応答しない・対象ドメイン外に遷移した・行き詰まり状態になったとき、システムは対象ページを初期状態に戻して探索を継続しなければならない | `RecoverFromFreeze` | 「phaseが"idle"かつidleCounterがIdleThreshold以上のときに成立し、phaseを"frozen_recovered"に遷移させ、idleCounterを0にリセットし、visitedを空集合にリセットする」 | partial | 読み手の言い直しは「固まり」(idleCounter到達)のケースのみを正確に捉えているが、S-017は固まり/ドメイン外/行き詰まりの3条件OR→単一アクションのEARS文であり、`RecoverFromFreeze`はそのうち固まりのケースのみを実装している。ドメイン外は`ObserveOutOfDomain`/`ReturnToAllowedDomain`、行き詰まりは`FinishStuck`(終了として扱う、復旧ではない)が別途担っており、「初期状態に戻して継続」という復旧アクションは固まりの場合しか実装されていない。これは台帳のS-017の原注記(初版「結果のアクションが初期状態へ戻して続行1つに収束するため分割対象外」)とTLA+の実装(3条件で別アクションに分岐している)が食い違っていたため、extract.mdのS-017注記を「固まりのみ復旧、ドメイン外は継続、行き詰まりは終了」という実装の実態に合わせて修正済み(トレーサビリティ・マトリクスのS-017行、および台帳末尾の設計メモ2を参照。7回目のformalizeレビューで対応)。 |
| S-021(固まり) | システムは一定時間操作後も画面に変化がない状態を「固まり」として判定しなければならない | `RecoverFromFreeze`(ガード部分) | 「idleCounterがIdleThreshold以上のとき」 | full | 一致。 |
| S-021(ドメイン外) | システムは操作の結果が探索開始時に許可したドメインの外への遷移であることを検出できなければならない | `ObserveOutOfDomain` | 「inAllowedDomainをFALSEにする」 | full | 一致。 |
| S-021(行き詰まり) | システムは既定の試行回数操作しても新規要素が出ない状態を「行き詰まり」として判定しなければならない | `FinishStuck`(ガード部分) | 「stuckCounterがStuckThreshold以上のとき」 | full | 一致。 |
| S-022 | システムは画面状態と操作による遷移を実行をまたいで永続化したグラフとして保持しなければならない | `ExecuteOperation`(visited更新部分) | 「proposedOpがclick/inputかつターゲットがElementsに属するならvisitedに追加」 | partial | 「グラフとして保持」(画面ごとの遷移構造)はmodel.mdで単一画面への簡略化を明示済みであり、`visited`という単一集合はグラフの近似。「実行をまたいで永続化」(プロセス再起動後も保持)という永続化の性質自体はTLA+の状態変数の範囲外(メモリ内モデル)であり実装レベルの検証事項。構造的な「探索済み化」の部分はfullに近いが、永続化とグラフ構造の2点が欠けるためpartial。 |
| S-023 | システムは探索の優先順位として、未操作要素→未探索分岐が残る既操作要素→戻る/スクロールの順で次の操作を選択しなければならない | `ExecuteOperation`(visited更新部分)、優先順位ロジック自体はモデル外 | 同上 | partial | 優先順位の具体的な選択規則(3段階の順序)はmodel.mdで明記した「捨てた領域」であり、`RequestDecision`の非決定的選択に吸収されている。式からは優先順位の存在自体が読み取れないため、この要件の核心(順序付け)はモデル化されていない。理由はmodel.md記載の抽象化方針どおりで、意図的なpartialとして残す。 |
| S-028(存在確認) | IF 操作対象要素として選択されたが、実際に操作を実行する直前に当該要素が存在しないとき、システムは操作を中止しなければならない | `ApplyGuardrails`(targetExists分岐) | 「proposedOpがclick/inputでtargetExistsが偽なら、phaseを"idle"に戻す」 | full | 一致。 |
| S-028(決済導線拒否) | IF 対象要素が決済導線に分類されるとき、探索モードはその要素を選択してはならない | `ApplyGuardrails`(ForbiddenElements分岐、S-012と共有) | 「proposedOpがclick/inputでターゲットがForbiddenElementsに含まれるなら、phaseを"idle"に戻す」 | full | S-012と同一の分岐で担保されており、決済導線が`ForbiddenElements`に含まれる前提のもとで一致する。 |
| S-033 | IF 操作の結果、対象ページが許可ドメインの範囲外に遷移したとき、システムはその遷移を記録し探索対象外として扱わなければならない | `ObserveOutOfDomain` / `ReturnToAllowedDomain` | 「inAllowedDomainをFALSEにする」「phaseが"idle"かつinAllowedDomainが偽のときに成立し、inAllowedDomainだけをTRUEに戻す」 | partial | 「対象外として扱う」(探索しない)ことは`RequestDecision`のガード`inAllowedDomain`で実質的に表現されているが、「遷移を記録する」という記録行為自体は状態変数化されていない(model.mdの計測・記録の簡略化方針と同様)。ドメイン外検出と復帰の構造はfullだが、記録の明示的表現が欠けるためpartial。 |
| S-041 | システムは判断モデルAPIへの呼び出し回数または実行時間に上限を設定でき、上限に達したとき実行を停止しなければならない | `ExceedDecisionBudget` / `BudgetRespected` | 「phaseが"idle"かつdecisionBudgetが0のときに成立し、phaseを"error_exit"に遷移させる」「decisionBudgetが常に0以上である」 | full | 「上限(decisionBudget)に達したら停止する」という構造が正確に一致する。 |
| S-042 | システムは探索モードにおける単位時間あたりの操作回数に上限を設け、対象サーバへの過度な負荷を避けなければならない | `RequestDecision`(requestBudgetガード) / `RequestBudgetRespected` | 「requestBudget > 0」という前提部分、「requestBudgetが常に0以上である」 | partial | 予算が尽きたら`RequestDecision`自体が発火しなくなる(enabledでなくなる)という構造でガードしているが、S-041の`ExceedDecisionBudget`のような「明示的な停止アクション」は無く、単に新規操作が発行されなくなるだけで`phase`が留まり続ける(deadlock状態に近い)。「過度な負荷を避ける」という意図は満たすが、「単位時間あたり」という時間窓の概念はモデル化されておらず、`requestBudget`は累積回数の上限のみを表す。この時間窓の欠落は理由欄に明記し、partialのまま残す。 |
| S-044(done) | WHEN done_when条件が満たされたとき、またはoperation=doneが判断モデルから返されたとき、システムは当該stage/シナリオの実行を正常終了として記録しなければならない | `FinishByProposalDone`(+`FinishNormally`は別要件S-014として分離済み) | 「phaseが"acting"かつproposedOpが"done"のときに成立し、phaseを"done"に、exitReasonを"goal_achieved"に遷移させる」 | full | 「operation=doneが返されたとき正常終了として記録する」という自己申告経路がfullに一致する。空成立でないこと・到達可能であることはreach.log(P2, P3)で確認済み。 |
| S-044(stuck) | WHEN operation=stuckが判断モデルから返されたとき、システムは実行を行き詰まり終了として記録しなければならない | `FinishByProposalStuck` | 「phaseが"acting"かつproposedOpが"stuck"のときに成立し、phaseを"stuck"に、exitReasonを"stuck"に遷移させる」 | full | 一致。 |

## 空成立の証跡(含意・前提付きの式)

- `LowConfidenceBackNeverActed`の前件(`proposedOp="back" /\ proposedLowConfBack`): `WebExploratoryQA.reach.log`のP1セクションに反例を記録済み(到達可能、空成立でない)。
- `ExitReasonSetOnTermination`の前件(`phase \in {"done","stuck","error_exit"}`): 同ログのP2セクションに反例を記録済み(到達可能、空成立でない。`FinishByProposalDone`経由での到達)。
- 補助検査: `FinishByProposalDone`が`visited`の到達度と無関係に機能する(死んだコードでない)ことを同ログのP3セクションで確認済み。

## 総括

判定表は全21行(モデル化した17件のEARS要件のうち、S-021が固まり/ドメイン外/行き詰まりの3行、S-028が存在確認/決済導線拒否の2行に展開されるため合計21行)。表から機械集計(`grep`で判定列を抽出し`sort | uniq -c`)した結果:

- `full`: 12件(S-010, S-012, S-013, S-014, S-021×3, S-028×2, S-041, S-044×2)
- `partial`(理由付き): 9件(S-002, S-005, S-006, S-011, S-017, S-022, S-023, S-033, S-042) — いずれも「行為・記録・優先順位・時間窓等の実装詳細をモデル外とした」という明記済みの理由があり、要件の構造的な核(ガード・終了条件)はモデル化されている。
- `mismatch`/`vacuous`/`missing`: 0件。

(4回目レビューで、初版の総括がfull 11件・partial 8件という誤った集計だったことを指摘された。表の行を`grep`で機械的に数え直し、本節を訂正した。)
- **extract.mdとmodel.mdの間で既存だった食い違い(S-017)に気づいた**: extract.mdのS-017注記は「結果のアクションが初期状態へ戻して続行1つに収束するため分割対象外」としていた。一方`model.md`のNext disjunct対応表(86行目`ObserveOutOfDomain`→S-021ドメイン外/S-033、88行目`RecoverFromFreeze`→S-017,S-021固まり、91行目`FinishStuck`→S-021行き詰まり)を見ると、S-017は`RecoverFromFreeze`(固まりの復旧)にのみ対応づけられ、ドメイン外・行き詰まりの行にはS-017が付与されていない。つまり3条件が別々のdisjunctに実装される構造は、model.mdのこの対応表から1回目レビュー時点以降読み取れる状態だった。今回のfaithfulness.md作成作業(読み手の言い直しと原文を突き合わせる過程)でこの食い違いに気づき、extract.mdのマトリクスS-017行へ実装の実態を追記して揃えた(5回目レビューで、この経路説明に実在しない文言の引用符付き引用が混入していたことを指摘され、model.mdの実際の記述に基づく表現へ訂正した)。

この結果、`mismatch`/`vacuous`/`missing`は無いため式自体の再修正は不要だった。S-017のEARS側注記の記述は実装に合わせて修正済み(台帳末尾の設計メモ2を参照)。
