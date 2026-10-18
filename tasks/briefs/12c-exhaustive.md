# brief: issue #12c 中核の状態機械の全状態の探索

共通の決まりは同じディレクトリの `common.md`。

issue #12 を3本に分けた3本目。順番は `12a-counters-window.md` の冒頭の表のとおり。
始めてよいのは 12b が main に入ってから。公開の API(`machine::{start, step, abort, Seen, Facts, Heard, Step, Outcome, State, Phase, ExitReason, Rules, Material, Note, Rejection, Anomaly}`、`State` の読み取り)は `12b-machine.md` の「決めてある設計」のとおり。

## タスク

`crates/webqa-core/tests/exhaustive.rs` に、中核の状態機械を小さな定数で総当たりする統合テストを書く。
`Facts` と時刻の取りうる値を有限の候補から選び、`start` から幅優先でたどる。到達したすべての遷移で、仕様の不変条件と行動の性質を Rust の述語として確かめる。乱数は使わない。
設計は `docs/architecture.md` の「決定的なテスト」「アクションの表」「仕様との対応表」「テストの層」。性質の出どころは `tasks/loopeng/WebExploratoryQA.tla` の Inv と Action properties(読むだけ。`tasks/loopeng/` は触らない)。
台帳の行は T-024(`finished_is_absorbing`)と T-078(`budget_and_timeout_tie_yields_one_reason`)。

範囲の外:

- 待機の1段目(ゴール、段階)にかかわる性質(`priority_holds_in_every_reachable_state`、`explore_goal_needs_a_question`、`GoalPreempts`、`StageAdvancePreempts`): #13 が同じファイルに足す。
- TLC の遷移の列との突き合わせ(`tlc_traces.rs`): 別のタスク。

状態機械の振る舞いがおかしいと分かったら、`machine/` を直さずに、反例(たどった `Facts` と時刻の列)を付けて BLOCKED で返す。

## 触ってよいファイル

- `crates/webqa-core/tests/exhaustive.rs`(新規)
- 補助を分けるなら `crates/webqa-core/tests/exhaustive/` の下(`exhaustive.rs` から `mod` で読む)

`src/` の下は触らない。テストに要る公開の読み取りが足りなければ NEEDS_CONTEXT で返す。

## 決めてある設計

### 定数(仕様の検査と同じく小さく)

- 規則: `mode: Scenario`、`ops` は8種すべて、固まり 2、行き詰まり 2、判断予算 3、時間窓の上限 2、`max_duration: Some(5000秒)`、`back_confidence: 0.7`、許可は `https://app.example.test`、`Forbidden::default()`、`Paywall::default()`、`leave_paywall: "back"`。
- 時刻の候補: 10秒(最初の窓の中)、3600秒(次の窓の始まり)、5000秒(制限時間)。時刻は戻さない。今の時刻より前の候補は選ばない。最初は 0秒で `start` する。
- 画面の候補(表示領域 800x600、矩形 (0,0,10,10)、`Material::plain` で材料を作る):
  - トップ `https://app.example.test/`: `button "保存"`、`link "ヘルプ"`、`button "購入する"`(禁止要素)
  - 設定 `https://app.example.test/settings`: `button "保存"`
  - 課金画面 `https://app.example.test/pricing`: 要素なし
  - 外 `https://other.example.net/`: `link "ヘルプ"`
  - 空 `https://app.example.test/empty`: 要素なし
  - 禁止だけ `https://app.example.test/cart`: `button "購入する"`
  - 差し替え: トップと同じ画面で、`material.nodes` の "保存" の参照だけを `"x0"` にしたもの(確認中の `Replaced` を出すため)
- 答えの候補: 操作種別 `click`、`back`、`scroll_down`、`done`、`stuck`、`drag` のそれぞれに、確信度 0.9 と 0.5、対象 `"1"`、`"3"`、`"99"` の組(36通り)と、`Err(JudgeError::Exhausted)`。
- `Facts` の候補は、状態の `phase()` に合う種類だけを使う。`Awaiting` には答え、`Confirming`、`Acting`、`Returning`、`Recovering` には画面、`Sleeping` には `Woke`。食い違いの `Facts` は 12b の単体が受け持つので、ここでは渡さない。

### たどり方

- 節は `(State, 経過時間, 直前の効果)`。`Step::Continue` を節にし、`Step::Finished` は葉として性質だけを確かめる。
- 同じ状態を2度たどらないよう、節の鍵を文字列で作って `BTreeSet` に入れる。鍵に入れるもの: `phase()`、`counters()`、`budget().used()`、`window().index()` と `window().used(経過時間)`、`outside()`、直前の観測が画面の候補のどれか(`last().screen` を候補と `==` で比べた番号)、記憶の `screen_count()` と `edge_count()` と、候補の画面の各要素の `is_operated`(記憶を `clone` して候補の画面を `visit` すれば鍵が得られる)、経過時間、直前の効果。`notes()` は増え続けるので鍵に入れない。
- 節の数が 200,000 を超えたら失敗させる(候補の作り方の誤りで爆発したことを知らせるため)。たどり終えた節の数が 1,000 未満でも失敗させる(候補が少なすぎて何も確かめていないことを知らせるため)。
- テストの関数は複雑度と行数(50行)の上限を受けるので、たどる関数、候補を作る関数、性質ごとの述語に分ける。1回のたどりの結果(遷移の一覧)を `OnceLock` に入れ、テストの関数どうしで使い回してよい。

### 遷移ごとに確かめる性質

遷移は「前の状態 s、渡した `Facts`、時刻 t、返った `Step`」の組。`Continue(s2, e)` なら次の状態 s2 と効果 e、`Finished(o)` なら終了理由と `o.state`。

1. 禁止要素を操作しない(`NeverOperatesForbidden`、`DirectOpActedOnlyOnSafeTarget`): 対象のある `Touch` の対象は "購入する" ではない。どの s2 の記憶でも、トップと禁止だけの画面の "購入する" は操作済みでない。
2. 対象のある `Touch` は確認中からしか出ない(`ActingOnlyAfterGuardrails` の写し): 対象のある `Touch` を返した遷移の s の `phase()` は `Confirming`。`Look` を返した遷移の s の `phase()` は `Awaiting`。
3. 低確信度の「戻る」を実行しない(`LowConfidenceBackNeverActed`): s の `phase()` が `Awaiting` で返った効果が `op: "back"` の `Touch` なら、`confidence >= 0.7`。
4. 問い合わせの前提(`FreezePreemptsDecision`、`NoDecisionOutsideDomain`、課金画面を問い合わせの前に判定すること): `Ask` を返した遷移の s2 は、固まりでなく(`!counters().frozen(2)`)、`outside()` が偽で、直前の観測が課金画面でなく、行き詰まりでなく、t が 5000秒未満で、`window().full(t)` が偽。
5. 予算と窓の上限(`DecisionsWithinCap`、`RequestsWithinWindowCap`): どの s2 と `o.state` でも `budget().used() <= 3`、`window().used(t) <= 2`。
6. 予算を勝手に増減させない(`BudgetAccounting`、`DecisionBudgetNeverRestored`): `budget().used()` は、`Ask` を返した遷移で1増え、ほかの遷移では変わらない。
7. 時間窓に数えるのはアクセスだけ: `window().used(t)` は、`Touch` を返した遷移で1増え(同じ窓の中なら)、`Ask`、`Look`、`Sleep` を返した遷移では増えない。
8. 終了理由の裏付け(`BudgetExceededOnlyWhenSpent`、`TimeExceededOnlyWhenTimeUp`、`ExitReasonMatchesPhase`): `BudgetExceeded` なら `o.state.budget().exhausted()`、`TimeExceeded` なら t が 5000秒以上、`Stuck` なら `o.state.counters().is_stuck(2)` か、渡した答えの操作種別が `stuck`、`ReportedDone` なら渡した答えの操作種別が `done`。`o.state.phase()` は `Ended(o.reason)`。`GoalAchieved` と `UserAborted` は出ない(#12 では1段目が無く、中断は外から来る)。
9. 続く状態は終わっていない: `Continue` の s2 の `phase()` は `Ended(_)` でも `Idle` でもない。
10. 探索の記憶は失われない(`MemoryNeverShrinks`): s で操作済みだった候補の要素は、s2(と `o.state`)でも操作済み。`notes()` は s の `notes()` を先頭に持つ(増えるだけ)。
11. 開き直しは固まりの回数を戻し、行き詰まりの回数を変えない(`RecoveryResetsPageState`、`CountersChangeOnlyBySteps`): `op: "reload"` の `Touch` を返した遷移の s2 は `counters().freeze() == 0`。カウンタが変わるのは、s の `phase()` が `Acting` の遷移か、`reload` を返した遷移だけ。
12. 許可ドメインの外では問い合わせず戻る: s2 の `outside()` が真なら、返った効果は `return` の `Touch` か `Sleep`。

### 終了の吸収と、同じ段の終了

- `finished_is_absorbing`(台帳): 到達したすべての `Finished(o)` について、`o.state` を、時刻の候補(o の時刻以降)と、`Woke`、画面の候補、答えの候補のすべての `Facts` で `step` に渡し直しても、`Finished` で理由は `o.reason` のまま、`budget()`、`window().used(t)`、`counters()`、`notes().len()`、記憶の `screen_count()` と `edge_count()` が変わらない。`abort(o.state)` の理由も `o.reason`。これを確かめた `Finished` が1つ以上あること。
- `budget_and_timeout_tie_yields_one_reason`(台帳): 返った `Step` が `Finished(o)` で、`o.state` が判断予算切れ(`budget().exhausted()`)かつ t が 5000秒以上かつ行き詰まりでないとき、`o.reason` は `BudgetExceeded` か `TimeExceeded` のどちらか。この組み合わせに当たった遷移が1つ以上あること(空振りで通らないように)。さらに、予算切れでも時間切れでもない状態で、どちらかの理由で終わった遷移が無いこと(上の性質8と同じ述語を使ってよい)。

### 空振りの防止

候補が足りずに性質が空振りで通るのを防ぐため、たどり終えたあとに次が1回以上起きたことを確かめる。

- 効果: `Ask`、`Look`、`Sleep`、対象のある `Touch`、`back` の離脱の `Touch`(課金画面から)、`reload`、`return`
- 記録: `Rejected` の4種(`Forbidden`、`LowConfidence`、`Missing`、`Replaced`)、`Anomaly` の3種(`Judge`、`Operation`、`Target`)、`Issue`(許可ドメインの外)
- 終了理由: `Stuck`、`BudgetExceeded`、`TimeExceeded`、`ReportedDone`

## 失敗させるテスト(`crates/webqa-core/tests/exhaustive.rs`)

1. `finished_is_absorbing`(台帳。上の「終了の吸収」)
2. `budget_and_timeout_tie_yields_one_reason`(台帳。上の「同じ段の終了」)
3. `forbidden_and_low_confidence_back_never_act`: 性質1、2、3
4. `no_question_while_frozen_outside_or_on_paywall`: 性質4、12
5. `budgets_and_window_stay_within_caps`: 性質5、6、7
6. `exit_reasons_are_backed_by_state`: 性質8、9
7. `memory_and_notes_only_grow`: 性質10、11
8. `exploration_covers_every_row`: 「空振りの防止」の一覧と、節の数の下限と上限

性質が破れたら、`assert_eq!` の文に、たどった `Facts` と時刻の列(`Debug` で出す。ページの文字列は `Debug` で隠れる)を入れて反例が分かるようにする。そのため、各節は `start` からの列を持つか、親の節の番号を持つ。

## 受け入れ条件

`just check` が終了コード0(カバレッジ100%を含む。テストのコードもカバレッジの対象なので、通らない分岐を残さない。破れたときだけ通る分岐は `assert_eq!` の中に入れる)。テストが10秒以内に終わる(リリースでないビルドの `cargo test -p webqa-core --test exhaustive` で測る。超えたら候補を減らさずに鍵の作り方を見直し、それでも超えたら報告する)。
`src/` の下と `tasks/loopeng/` に差分が無い。
mutation は `src/` の 12b のファイルに対して回し、12b の受け入れのときより生き残りが増えていないこと(このテストで新たに殺せた変異があれば報告に書く)。
