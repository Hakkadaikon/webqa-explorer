# テスト項目抽出台帳: InterviewMode

対象: インタビューのモード(ペルソナと目的の入力、試行と試行の外のループ、判断モデルへの問い、試行の終わり方、課題の見つけ方、ペルソナから外れないこと、台帳の書式、課題の選択とTUI、再実行と照らし合わせ、手引きと形式の検査、既存のモードとの関係)
走査アンカー: 受け入れ条件(`tasks/loopeng/InterviewMode.feature` の Scenario 1件 = T-ID 1行で転記)
テスト対象: crates/*
転記元の版: コミット 7e4329b の `tasks/loopeng/InterviewMode.feature` に、見えにくい要素の問いを最大3つに絞り、答えの書き方の揺れを読むことに直した決定(T-132、T-133 を足し、題名は変えず、行番号を合わせた)と、issue #33 の決定(操作種別を9種と書き、画像が要る2本のシナリオに画像を扱えるという申告の前提を足し、課題を直さないシナリオに正常に終わる場面の前提を足し、1冊の台帳にまとめるシナリオに目的を達成する前提を足し、窓待ちの制限時間を50分にし、台帳の書式の2本に課題が2件出る場面の前提を足した)と、issue #79 の決定(期待の問いを、中核が選んだ操作と要素を問いの文に入れて2回目の問い合わせで問う。T-135〜T-137 を足し、T-048 の題名と表から期待を外し、T-051 の期待の例を T-137 に分け、目的に届かない課題のシナリオから最後のステップの期待を外した)を反映した版。出典欄の行番号はこの版の行を指す。
転記元の sha256: 6faa85dab3c149414b59a843886954293ebff2521b9ee0dc8086c316a4c97ac4
feature が変わったら、台帳の行を追従させてから、この sha256 を新しい版の値に書き換える。`tasks/test-design/tools/check_ledger.py InterviewMode` は、この値と実物の feature の sha256 が違えば失敗する。
抽出台帳は `tasks/loopeng/InterviewMode.extract.md`、設計は `docs/superpowers/specs/2026-10-18-interview-mode-design.md` にある。

各 ID は「テストすべき振る舞い」1つ。test-catalog で全行に手法とテスト名を割り当てた。
チェックボックスの意味は、ここで1回だけ定義する。台帳の行、マトリクスの「設計済」列、閉じる前のチェックで同じ意味に使う。
- `[ ]` 未着手、または未確認。
- `[~]` 部分完了。何が残るかを同じ項目に「残り:」で書く。T-ID の行では、手法とテスト名は割り当て済みで、仕様の直しで何が変わりうるかを `→ 残り:` に書く。
- `[x]` 完了。T-ID の行では設計済、すなわち手法とテスト名が決まり、仕様の直しを待たないことを表す。テストが緑になったことは表さない。緑の確認は、トレーサビリティマトリクスの「緑確認済」列で test-verify が消し込む。

手法の欄には、手法名と、割り当てるときに開いた技法スキルの名前を括弧で書いた。
テストの置き場所は `docs/architecture.md` の「テストの層」と「ディレクトリ構成」に従い、構成図に無いモジュールは下の「構成への追加の案」に沿って割り当てた。
どの行も受け入れの層を持つ。`webqa/tests/acceptance.rs` が cucumber で、題名が同じ Scenario を走らせる。
テスト名の欄には、受け入れのほかに、その振る舞いを受け持つ下の層のテストを書いた。単体は `crate::モジュール` の形で、ほかの層はテストのファイルで場所を示す。
既存の台帳 `tasks/test-design/WebExploratoryQA.md` と同じテスト名を書いた行(T-055、T-120、T-121)は、共有する中核の道筋を既存のテストが受け持つことを表す。新しく書くテストではない。そのテストは既存の台帳の T-005、T-007、T-008、T-015、T-020 の行で、まだ実装していない。この3行の緑は、それらの行の実装に依存する。
テスト関数の名前には T-ID などの管理番号を入れない。
issue の欄は、その行を実装する GitHub の issue の番号である。下の層のテストを持つ issue(#18〜#32、T-055、T-120、T-121 は既存の #13、#4、#12)と、受け入れを配線する #33 の両方を書く。案の番号との対応は `tasks/briefs/interview-issues.md` にある。照合スクリプトは、この台帳では issue の欄が空でも通す設定のままにしてある。
仕様の直しで変わりうる行は `[~]` にする。今は該当する行が無い。
直し待ちの行: なし
仕様待ちの issue: なし
抽出台帳で【推測】としていた29件と設計メモ1-c は、親が1つずつ確かめて【親】に確定した(7e4329b)。【推測】は0件なので、それに頼っていた行も仕様の直しを待たず、`[x]` のままでよい。
Scenario Outline は1行にまとめ、Examples の各行をそのテストのケースにする。
T-ID は一度振ったら変えない。T-124〜T-131 は、親が目的のずれの判定と目的のずれの課題の項目を決めて feature に足したシナリオで、番号を保つために末尾へ追加した。feature の中では T-006 と T-008 の間にあり、T-006、T-131、T-129、T-007、T-124〜T-128、T-130 の順に並ぶ。T-006、T-124〜T-126 は同じ判断で題名を言い直した。

## 構成への追加の案

`docs/architecture.md` の構成図に無いモジュールとテストのファイルを、次のように足す案にした。台帳のテスト名はこの案に沿う。architecture.md への反映は親が行う。
`webqa-core` は今の方針どおり同期の純粋なコードだけにし、時計・子プロセス・ブラウザを使う試行のループは `webqa` に置く。

| crate | 足すもの | 受け持つもの | 割り当てた T-ID の数 |
|---|---|---|---|
| `webqa-core` | `config/persona.rs` | ペルソナの文書の型と検証(5項目、書式) | 3 |
| `webqa-core` | `config/purpose.rs` | 目的(書いた目的か推測か)、目的を1段階の段階に写すこと、画面ごとの目的と URL の型の検証 | 4 |
| `webqa-core` | `config/interview.rs` | インタビューのモードの設定(試行の回数、閾値、自然さの問いの上限、モード別の行き詰まりの閾値、アクセスの上限が2以上、初期化のコマンドの引数の列と待ち時間) | 2 |
| `webqa-core` | `interview.rs` と `interview/` | このモードの純粋な部品を下の9つにまとめる親 | — |
| `webqa-core` | `interview/ask.rs` | 問いの組み立て(ペルソナの添付、足した4つの問い、done を選択肢に入れる、候補を絞らない、画像を添えるかの判断、伏せ字)と、試行の後のずれの問いの組み立て(画面ごとの目的の1項目(URL の型1つ)につき1回、その型に合う画面で終えた試行に出た推測を前後の空白を除いて重複をまとめ、書いた目的と一緒に渡す。終えた試行が届かない項目と、使える推測が無い項目では問わない) | 17 |
| `webqa-core` | `interview/answer.rs` | 足した問いの答えの検証(欠け、範囲の外、画像を扱えないときの免除)と、ずれの問いの答えの検証(欠け、選択肢の外は使えない) | 6 |
| `webqa-core` | `interview/clue.rs` | 迷いの手がかり(度合い、確信度、引き返し、同じ画面の行き来) | 5 |
| `webqa-core` | `interview/issue.rs` | 課題の型、種類、番号、検出(視認性、目的に届かない、ずれの問いの答えからの目的のずれ)、試行をまたいだ集計(目的のずれの課題の画面・N・k・スクリーンショットの決め方を含む) | 14 |
| `webqa-core` | `interview/outcome.rs` | 試行の結末(終えた、打ち切り)と、目的に届かない試行の数え方 | 6 |
| `webqa-core` | `interview/natural.rs` | 自然さの問いと区分(本体、確かめられなかった、ペルソナから外れる)、ずれの問いと自然さの問いで分け合う問いの回数の上限(ずれの問いが先、自然さの問いが後。ずれの課題も自然さの問いにかける) | 7 |
| `webqa-core` | `interview/ledger.rs` | 台帳の Markdown の書き出しと読み戻し、並び、書式の無害化、選択の書き換え | 19 |
| `webqa-core` | `interview/compare.rs` | 前回の台帳との照らし合わせ、選択の引き継ぎ、ペルソナと目的の食い違い | 10 |
| `webqa-core` | `interview/schedule.rs` | 次の試行を始めるかの判定(中断、期限、回数、窓の残り)、中核に渡す時間窓の上限、実行全体の時間窓の数え方 | 4 |
| `webqa-core` | `tests/interview_golden.rs` | ゴールデン。問いの文面と台帳の形 | 4 |
| `webqa-core` | 既存の `ports.rs`、`judge.rs` を直す | `Judge` に画像を扱えるかの申告を足す。`Question` に自由な文の問いを足す | (`ask`、`answer` の行が受け持つ) |
| `webqa-judge` | 既存の `scripted.rs`、`command.rs` と `tests/command.rs` を直す | アダプタの画像の申告、橋渡しの JSON の画像の項目と版の項目での申告 | 4 |
| `webqa-browser` | 既存の `tests/contract.rs` にテストを足す | 試行ごとに新しく開いたブラウザに前の保存領域が残らないこと | 1 |
| `webqa-tui` | `ledger.rs` | 台帳の画面の状態(一覧、種類での絞り込み、中身、付け外し、外での書き換えの知らせ)。イベントを畳み込む純粋な関数にする | 3 |
| `webqa-tui` | `ledger_file.rs` と `tests/ledger_file.rs` | 台帳のファイルへの書き戻しと、開いたあとの外での書き換えの検出。テストは保存の層で `CARGO_TARGET_TMPDIR` を使う | 3 |
| `webqa-tui` | 既存の `tests/render.rs` にテストを足す | 台帳の画面のゴールデン。実行中の画面のゴールデンは今のまま残す | 3 |
| `webqa` | `interview.rs` と `interview/trials.rs` | 試行の外のループ(初期化のコマンド、ブラウザの起動、中核の実行、試行の間の待ち、期限と中断、ずれの問いと自然さの問いの呼び出し、台帳の書き出し) | 21 |
| `webqa` | `interview/reset.rs` と `tests/reset.rs` | 初期化のコマンドの実行(シェルを通さない、API キーを渡さない、待ち時間、出力の伏せ字)。`tests/reset.rs` は試験用コマンドが受け取った引数と環境変数を書き出す契約の層 | 4 |
| `webqa` | 既存の `cli.rs`、`exit.rs`、`run.rs`、`store/screenshots.rs` を直す | サブコマンド(インタビューの実行、形式の検査)、終了コード、既存のモードの問いの中身、課題のスクリーンショットの名前 | 7 |
| `webqa` | `tests/trials_props.rs` | 新しい層「性質」。試行の外のループを偽の時計・ブラウザ・初期化のコマンドで回し、乱数で作った入力で性質を確かめる | 16 |
| `webqa` | `tests/guide.rs` | 新しい層「文書」。配布する手引きの前付け、手順、例、置き場所を確かめる | 3 |
| `webqa` | `tests/features/interview_mode.feature`、`tests/steps/` の Rule ごとのファイル、`tests/support/` の偽の初期化のコマンド | 受け入れ。`tasks/` の feature を写し、既存の feature と同じ `acceptance.rs` で走らせる | 133 |
| 配布物 | `share/skills/` の下の手引き | ペルソナと目的を書かせる Agent Skills の手引き。開発用の `skills/` の外に置く | (`tests/guide.rs` の行が受け持つ) |

「割り当てた T-ID の数」は、そのモジュールかファイルのテストを1つ以上持つ行の数で、1行が複数の場所を持てば重ねて数えた。既存のモジュールで既存のテストを指す行(T-055、T-120、T-121)は数えていない。

テストの層の表に足す行の案は次の2つである。

| 層 | 置き場所 | 対象 | 実行環境 |
|---|---|---|---|
| 性質 | `webqa/tests/` | 試行の外のループの安全性(実行全体の窓、期限、試行の数) | 偽の時計、ブラウザ、初期化のコマンド、判断モデル。乱数の seed と試行回数を固定する |
| 文書 | `webqa/tests/` | 配布する手引きの形 | なし |

外部crateの表には、性質テストのために `proptest` を dev の依存として足す。版を crates.io で確かめて表に足すのは、それを最初に使う #30 の範囲とした。
受け入れシナリオの数は、テストの層の表の「111本」に、このモードの133本を足して244本になる。

## 用語定義(抽出より先に埋める)

既存の台帳の用語定義(1ステップ、固まり、行き詰まり、done_when、時間窓、判断予算、時間切れ、利用者による中断、終了コードなど)は、そのまま使う。ここには、このモードで足す語と、意味が変わる語だけを書く。

| 用語 | 定義(曖昧さを残さない) | この定義が生む境界と異常系 |
|------|--------------------------|----------------------------|
| 試行 | 1つのペルソナと1つの目的の組を1回試すこと。中核の1回の実行で、中核の初期状態から始める | 試行どうしで判断予算・記憶・ブラウザの状態を持ち越さない |
| 試行の結末 | 終えた試行か打ち切りか。打ち切りは、利用者による中断か実行全体の期限で終わった試行で、1回の実行で多くても1回 | 打ち切りを試行の回数Nにも目的に届かない試行にも数えない。Nが0になりうる |
| 終えた試行の数 N | 打ち切りでない試行の数。課題の出現の割合の分母 | N = 0 で割合を出さない |
| 目的に届かない試行 | 終えた試行のうち、ゴール達成と申告による終了(未検証の終了)のほかで終わったもの。行き詰まりと判断予算切れ | 打ち切り、ゴール達成、申告による終了は数えない |
| 目的を書いた / 推測させた | 設定に目的の done_when を書いたかどうか。書いた目的は段階1つの決め打ちシナリオとして中核に渡す。推測させたときは done_when が成り立たない段階1つとして渡す | 宣言が0個の done_when を作らない。目的の設定の done_when に宣言が無ければ設定エラー |
| 画面ごとの目的 | URL の型と目的の文の組。URL がその型に合う画面でだけ、推測した目的とのずれを比べる | 型の書式の誤り、型に合わない画面、推測が合っている画面 |
| 目的のずれ | 画面ごとの目的の1項目(URL の型1つ)について、その型に合う画面で出た推測が書いた目的と同じことを指さないと、判断モデルがずれの問いで答えたこと。文字列の一致では決めない | 文字列は違うが同じと答える、書いていない画面 |
| ずれの問い | すべての試行を終えたあと、画面ごとの目的の1項目(URL の型1つ)につき1回だけ聞く別の問い合わせ。その型に合う画面で終えた試行に出た推測を全部(前後の空白を除いた文字列で重複をまとめたもの)と書いた目的を渡し、「同じ」「同じでない」で答えさせる。その型に合う画面に届いた終えた試行が無い項目と、使える推測が1つも無い項目では聞かない。打ち切りの試行の推測は数えない。自然さの問いと同じ上限に数え、ずれの問いを先に、自然さの問いを後に聞く。判断予算に数えない | 同じ型に合う複数の画面、空白だけ違う推測、届いたのが打ち切りの試行だけ、推測がどれも応答の異常、答えの欠け、選択肢の外の答え、上限を超えた項目 |
| 迷いの手がかり | 迷いの度合いが閾値以上、確信度が閾値未満の操作を選んだ、back を実行した、同じ2つの画面を A→B→A→B と行き来した、のどれか。行き詰まりの回数には入れない | 度合いの閾値ちょうど(記録する)、確信度の閾値ちょうど(記録しない)、A→B→A まで |
| 迷いの度合いの閾値 / 迷いとみなす確信度の閾値 | どちらも0より大きく1以下。既定値は設定で決める | 0、1.5 は設定エラー、1 は受け付ける |
| 画像の申告 | 判断モデルのアダプタが起動のときに、画像を扱えるかを真偽で申告すること。橋渡しは JSON の版の項目で申告する | 扱えないとき、見えにくい要素の答えを求めず、視認性の課題を出さず、スクリーンショットを送らず、そのことを台帳に書く |
| 足した問い | 迷った度合い(0以上1以下の値)、期待(自由な文)、見えにくい要素(要素の番号の一覧と理由の文)、画面の目的の推測(自由な文)。今の4つの問いと同じ1回の問い合わせで問う | 欠け、範囲の外、送った一覧に無い番号は応答の異常 |
| 課題 | 種類(判断に迷う、視認性が悪い、目的に届かない、目的のずれ)、画面、要素(無いこともある)、指示の文、出現回数 k/N、その画面に届いた試行の数などを持つ台帳の1件。目的のずれの課題だけは、画面を画面ごとの目的の URL の型とし、要素を持たず、N とその画面に届いた試行の数をその型に合う画面に届いた終えた試行の数とし、k = N とする。スクリーンショットはその型に最初に合った観測のもので、名前はその試行とステップから作る | 0 ≦ k ≦ N。目的のずれは型に届かない試行を N に数えない(3回のうち2回届けば2回のうち2回)、同じ型に合う画面が複数の試行にある |
| 課題の番号 | 要素を持つ課題は(画面の識別、要素の識別、種類)、目的に届かない課題は(画面の識別、種類)、目的のずれの課題は(画面ごとの目的の URL の型、種類)から決める決定的な値。実行をまたいで同じ組なら同じ番号 | 組の1つだけが違う、要素を持たない2種類の課題 |
| 区分 | 本体、確かめられなかった、ペルソナから外れる、の3つ。自然と答えた課題だけが本体。ずれの問いの答えが使えないか上限を超えた画面も、確かめられなかった区分に入れる。台帳では本体、確かめられなかった、ペルソナから外れる、の順に節を並べる | 応答が使えない、問いの回数の上限を超えた |
| 自然さの問い | すべての試行を終えて課題をまとめたあと、課題の番号ごとに1回だけ問う別の問い合わせ。判断予算に数えない。上限(既定50)から、先に聞いたずれの問いの回数を引いた残りまでを、台帳の並びの順に問う。ずれの問いで出た目的のずれの課題も対象にする | 上限ちょうど、上限を超えた残り、ずれの問いで上限を使い切った場合 |
| 台帳 | Markdown の1ファイル。ペルソナの名前、目的(書いた目的か、推測の印と推測した文か)、開始の URL、終えた試行の回数、課題の本体、確かめられなかった節、ペルソナから外れる節を持つ。書いて読み戻すと同じ値に戻る | 判断モデルの文に入った見出し・チェックボックス・改行、N = 0 |
| 台帳の並び | 本体は出現の割合 k/N の降順、同じ割合なら番号の昇順 | 同率 |
| 照らし合わせ | 新しく出た、残っている、解消した、確かめられなかった、の4つ。解消したは、前回の課題が今回1回も起きず、今回その画面に届いた試行の数が前回の数以上のときだけ | 届いた数が同じ、1少ない、0 |
| 選択 | 課題のチェックボックス。読み戻しで選んだ課題を区別する。残っている課題は前回の選択を引き継ぎ、新しく出た課題は選ばない | エディタでの直接の書き換え、TUI での付け外し、外での書き換えとの競合 |
| 実行全体の時間窓 | 実行の開始から1時間ごとに区切り、試行をまたいで対象サーバーへのアクセスを数える窓。試行の最初のページの表示も1回数え、初期化のコマンドのアクセスは数えない | 窓の切り替わり、上限ちょうど |
| 中核に渡す時間窓の上限 | 次の試行を始めるとき、上限 − u − 1(u は実行全体の今の窓のアクセスの数)。その2倍が上限以上なら待たずに渡し、下回れば窓が明けるまで待って上限 − 1 を渡す。試行の間は変えない | 上限10で u = 4(5を渡す、ちょうど半分)、u = 5(待つ)、u = 10 |
| 実行全体の期限 | 制限時間を実行の開始から測った時刻。各試行の中核に同じ期限を渡す | 試行の間の待ち、初期化のコマンドの最中、ブラウザの起動の最中 |
| 初期化のコマンド | 設定に引数の列で書く外部のコマンド。各試行でブラウザを開く前に、シェルを通さずに実行する。API キーを渡さず、出力をログに残すときは伏せる | 0以外の終了コード、待ち時間切れ、期限の後の待ち時間切れ |
| 当たり前要求 | 初期化のコマンドへのシェルの注入を防ぐ、API キーを子プロセスに渡さない、台帳とログに機密情報を出さない、スクリーンショットの名前にパスの区切りを入れない、台帳の構造を判断モデルの文で偽らせない。スクリーンショットは画面のまま判断モデルに送る(設計の判断) | 各セキュリティの行 |

## 台帳

- [x] T-001 InterviewMode.feature:21(ペルソナと目的の入力) [event] 「形式の正しいペルソナの文書を読み込んで実行を始める」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::persona` の `reads_all_required_fields` / issue: #18、#33
- [x] T-002 InterviewMode.feature:26(ペルソナと目的の入力) [unwanted] 「ペルソナの文書に必須の項目が欠けていたら設定エラーにする」(Examples の各行を1ケースとする)
      → 手法: ネガティブテスト(process-static)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::persona` の `missing_field_is_config_error`、実行ファイル `webqa/tests/cli.rs` の `interview_config_error_exits_two_before_opening` / issue: #18、#31、#33
- [x] T-003 InterviewMode.feature:41(ペルソナと目的の入力) [unwanted] 「ペルソナの文書が決まった形式で読めなければ設定エラーにする」
      → 手法: 構文テスト(experience-scenario)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::persona` の `malformed_persona_rejected` / issue: #18、#33
- [x] T-004 InterviewMode.feature:47(ペルソナと目的の入力) [event] 「目的を書いたときは、目的の done_when を満たしたかで達成を判定する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::purpose` の `written_purpose_becomes_single_stage` / issue: #18、#33
- [x] T-005 InterviewMode.feature:53(ペルソナと目的の入力) [event] 「目的を書かなかったときは、判断モデルが推測した目的を台帳に書く」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `inferred_purpose_is_written` / issue: #24、#33
- [x] T-006 InterviewMode.feature:59(ペルソナと目的の入力) [unwanted] 「画面ごとの目的の項目で、推測が書いた目的と同じでないと判断モデルが答えたら、目的のずれの課題を出す」
      → 手法: デシジョンテーブル(blackbox-partition)、テストダブル(test-doubles)、非決定的出力システムのテスト(ai-nondeterministic) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `asks_mismatch_once_per_url_pattern`、単体 `webqa-core::interview::ask` の `mismatch_question_passes_all_guesses_once`、単体 `webqa-core::interview::issue` の `judge_says_different_purpose_is_mismatch`、単体 `webqa-core::interview::issue` の `mismatch_issue_screen_is_url_pattern` / issue: #20、#22、#33
- [x] T-007 InterviewMode.feature:85(ペルソナと目的の入力) [event] 「推測の文字列が書いた目的と違っても、同じことを指すと判断モデルが答えたら、目的のずれの課題は出さない」
      → 手法: デシジョンテーブル(blackbox-partition)、テストダブル(test-doubles)、非決定的出力システムのテスト(ai-nondeterministic) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `different_wording_is_left_to_judge`、単体 `webqa-core::interview::issue` の `judge_says_same_purpose_no_mismatch` / issue: #20、#22、#33
- [x] T-008 InterviewMode.feature:138(ペルソナと目的の入力) [state] 「画面ごとの目的を書いていない画面では、推測した目的をずれとして比べない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `no_mismatch_question_for_undeclared_screen`、単体 `webqa-core::interview::issue` の `undeclared_screen_not_compared` / issue: #20、#22、#33
- [x] T-009 InterviewMode.feature:144(ペルソナと目的の入力) [unwanted] 「画面ごとの目的の URL の型が書式に合わなければ設定エラーにする」
      → 手法: 構文テスト(experience-scenario)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::purpose` の `malformed_url_pattern_rejected` / issue: #18、#33
- [x] T-010 InterviewMode.feature:150(ペルソナと目的の入力) [boundary] 「試行の回数や閾値が範囲の外なら設定エラーにする」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::interview` の `rejects_out_of_range_settings`、単体 `webqa-core::config::interview` の `accepts_range_edges` / issue: #18、#33
- [x] T-011 InterviewMode.feature:168(ペルソナと目的の入力) [optional] 「行き詰まりの閾値は、インタビューのモードだけ別の値にできる」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::interview` の `stuck_threshold_overrides_for_interview` / issue: #18、#33
- [x] T-012 InterviewMode.feature:176(試行) [ubiquitous] 「1回の実行で、1つのペルソナと1つの目的の組を決めた回数だけ試し、1冊の台帳にまとめる」
      → 手法: 同値分割(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `runs_configured_trials_into_one_ledger` / issue: #30、#33
- [x] T-013 InterviewMode.feature:182(試行) [event] 「試行ごとにブラウザを新しく開き、認証状態を入れ直す」
      → 手法: テストダブル(test-doubles)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `each_trial_opens_fresh_browser_with_storage_state`、契約 `webqa-browser/tests/contract.rs` の `new_context_has_no_previous_storage` / issue: #29、#30、#33
- [x] T-014 InterviewMode.feature:189(試行) [optional] 「初期化のコマンドを設定に書けば、各試行の前に実行する」
      → 手法: テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `reset_runs_before_each_browser_open` / issue: #30、#33
- [x] T-015 InterviewMode.feature:195(試行) [state] 「初期化のコマンドを書かなければ、試行の前に何も実行しない」
      → 手法: 同値分割(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `no_reset_command_runs_nothing` / issue: #30、#33
- [x] T-016 InterviewMode.feature:201(試行) [unwanted] 「初期化のコマンドが失敗したら、その試行をせずに実行を止めて終了コード3で終わる」(Examples の各行を1ケースとする)
      → 手法: ネガティブテスト(process-static)、テストダブル(test-doubles)、時刻依存の封じ込め(flakiness-value) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::reset` の `nonzero_or_timeout_is_failure`、単体 `webqa::interview::trials` の `reset_failure_writes_ledger_then_exits_three`、単体 `webqa::interview::trials` の `failed_reset_before_any_trial_is_a_run_that_got_nowhere` / issue: #28、#30、#33
- [x] T-017 InterviewMode.feature:213(試行) [ubiquitous] 「初期化のコマンドはシェルを通さず、設定に書いた引数のまま実行する」
      → 手法: セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、契約 `webqa/tests/reset.rs` の `args_passed_verbatim_without_shell` / issue: #28、#33
- [x] T-018 InterviewMode.feature:219(試行) [unwanted] 「初期化のコマンドに、判断モデルの API キーを渡さない」
      → 手法: セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、契約 `webqa/tests/reset.rs` の `api_key_env_not_inherited` / issue: #28、#33
- [x] T-019 InterviewMode.feature:224(試行) [unwanted] 「初期化のコマンドの出力をログに残すときも、設定で指定した機密情報を伏せる」
      → 手法: セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::reset` の `logged_output_is_redacted` / issue: #28、#33
- [x] T-020 InterviewMode.feature:230(試行) [event] 「試行ごとに探索の記憶を空で始め、保存しない」
      → 手法: CRUD/ライフサイクルテスト(blackbox-state)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `trials_start_with_empty_memory_and_never_save` / issue: #30、#33
- [x] T-021 InterviewMode.feature:236(試行) [event] 「1つの試行が判断予算切れで終わっても、次の試行は判断予算を使っていない状態で始める」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `budget_resets_per_trial` / issue: #30、#33
- [x] T-022 InterviewMode.feature:242(試行) [event] 「試行の最初のページの表示も、実行全体の時間窓のアクセスに1回数える」
      → 手法: プロパティベーステスト(generative-property)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::schedule` の `first_page_counts_one_access`、性質 `webqa/tests/trials_props.rs` の `every_run_window_stays_within_limit` / issue: #26、#30、#33
- [x] T-023 InterviewMode.feature:247(試行) [ubiquitous] 「初期化のコマンドが対象サーバーへ送るアクセスは、実行全体の時間窓に数えない」
      → 手法: プロパティベーステスト(generative-property)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `reset_accesses_not_counted`、性質 `webqa/tests/trials_props.rs` の `every_run_window_stays_within_limit` / issue: #30、#33
- [x] T-024 InterviewMode.feature:253(試行) [boundary] 「残りから最初の表示の1回を引いた数が上限の半分以上なら、その数を中核の時間窓の上限にして次の試行を始める」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::schedule` の `starts_with_remaining_minus_one_at_half`、性質 `webqa/tests/trials_props.rs` の `every_run_window_stays_within_limit` / issue: #26、#30、#33
- [x] T-025 InterviewMode.feature:265(試行) [boundary] 「残りから最初の表示の1回を引いた数が上限の半分を下回るなら、窓が明けるまで待ち、上限から1を引いた数で次の試行を始める」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition)、プロパティベーステスト(generative-property)、時刻依存の封じ込め(flakiness-value) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::schedule` の `waits_when_below_half`、性質 `webqa/tests/trials_props.rs` の `every_run_window_stays_within_limit` / issue: #26、#30、#33
- [x] T-026 InterviewMode.feature:277(試行) [event] 「次の試行を始めるかは、初期化のコマンドの前に判定する」
      → 手法: 禁止仕様からの導出(blackbox-state)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `start_check_precedes_reset`、性質 `webqa/tests/trials_props.rs` の `no_reset_while_waiting_for_window` / issue: #30、#33
- [x] T-027 InterviewMode.feature:284(試行) [ubiquitous] 「試行の中核の時間窓は、実行の開始からの経過時間で、実行全体の時間窓と同じ時刻に区切る」
      → 手法: 時刻依存の封じ込め(flakiness-value)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::schedule` の `core_window_aligns_with_run_window`、性質 `webqa/tests/trials_props.rs` の `every_run_window_stays_within_limit` / issue: #26、#30、#33
- [x] T-028 InterviewMode.feature:290(試行) [state] 「試行に渡した中核の時間窓の上限は、その試行の間は後の窓でも変えない」
      → 手法: 状態遷移テスト(blackbox-state)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `core_limit_fixed_for_trial`、性質 `webqa/tests/trials_props.rs` の `every_run_window_stays_within_limit` / issue: #30、#33
- [x] T-029 InterviewMode.feature:296(試行) [unwanted] 「上限を小さくして始めた試行は、その回数に達したら窓が明けるまで問い合わせも操作もしない」
      → 手法: 境界値分析(blackbox-partition)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `reduced_limit_blocks_core_until_window`、性質 `webqa/tests/trials_props.rs` の `every_run_window_stays_within_limit` / issue: #30、#33
- [x] T-030 InterviewMode.feature:304(試行) [event] 「窓を待ってから始めた試行も、中核の時間窓の操作回数を0から数える」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `core_window_count_starts_at_zero` / issue: #30、#33
- [x] T-031 InterviewMode.feature:310(試行) [state] 「試行の間で窓を待っている間に中断したら、窓を待たずに、終えた試行までの結果で台帳を書いて終わる」
      → 手法: 並行・競合の封じ込め(flakiness-concurrency)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `abort_while_waiting_writes_ledger`、性質 `webqa/tests/trials_props.rs` の `wait_ends_on_abort_or_deadline` / issue: #30、#33
- [x] T-032 InterviewMode.feature:317(試行) [state] 「試行の間で窓を待っている間に制限時間に達したら、窓を待たずに時間切れで終わる」
      → 手法: 時刻依存の封じ込め(flakiness-value)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `deadline_while_waiting_times_out`、性質 `webqa/tests/trials_props.rs` の `wait_ends_on_abort_or_deadline` / issue: #30、#33
- [x] T-033 InterviewMode.feature:325(試行) [ubiquitous] 「制限時間は実行の開始から測り、試行をまたいで引き継ぐ」
      → 手法: 時刻依存の封じ込め(flakiness-value)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `deadline_measured_from_run_start`、性質 `webqa/tests/trials_props.rs` の `nothing_starts_after_deadline` / issue: #30、#33
- [x] T-034 InterviewMode.feature:333(試行) [state] 「初期化のコマンドの最中に期限が来たら、終わるのを待ってから、試行を始めずに打ち切る」
      → 手法: 時刻依存の封じ込め(flakiness-value)、状態遷移テスト(blackbox-state)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `deadline_during_reset_waits_then_cuts`、性質 `webqa/tests/trials_props.rs` の `nothing_starts_after_deadline` / issue: #30、#33
- [x] T-035 InterviewMode.feature:341(試行) [unwanted] 「初期化のコマンドの最中に期限が来て、そのあと待ち時間も超えたら、コマンドを止めて打ち切りとして時間切れで終える」
      → 手法: 時刻依存の封じ込め(flakiness-value)、デシジョンテーブル(blackbox-partition)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `reset_timeout_after_deadline_is_time_up`、性質 `webqa/tests/trials_props.rs` の `nothing_starts_after_deadline` / issue: #30、#33
- [x] T-036 InterviewMode.feature:350(試行) [unwanted] 「打ち切りの試行で見た手がかりは、出現回数にも課題にも入れない」
      → 手法: 禁止仕様からの導出(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `cut_trial_clues_are_dropped` / issue: #22、#33
- [x] T-037 InterviewMode.feature:357(試行) [state] 「ブラウザの起動や最初の表示の最中に期限が来たら、待たずに打ち切る」
      → 手法: 並行・競合の封じ込め(flakiness-concurrency)、時刻依存の封じ込め(flakiness-value)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `deadline_during_browser_start_cuts_without_waiting`、性質 `webqa/tests/trials_props.rs` の `nothing_starts_after_deadline` / issue: #30、#33
- [x] T-038 InterviewMode.feature:364(試行) [unwanted] 「期限や中断で打ち切った試行は、試行の回数にも目的に届かない試行にも数えない」(Examples の各行を1ケースとする)
      → 手法: デシジョンテーブル(blackbox-partition)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::outcome` の `cut_trials_not_counted`、性質 `webqa/tests/trials_props.rs` の `finished_plus_cut_equals_started` / issue: #21、#30、#33
- [x] T-039 InterviewMode.feature:375(試行) [boundary] 「終えた試行が0回なら、課題を出さず、そのことを台帳に書く」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `zero_finished_trials_has_no_issues` / issue: #24、#33
- [x] T-040 InterviewMode.feature:381(試行) [event] 「実行中に中断したら、残りの試行をせず、終えた試行までの結果で台帳を書いて終わる」
      → 手法: 並行・競合の封じ込め(flakiness-concurrency)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `abort_mid_trial_skips_rest`、単体 `webqa::interview::trials` の `interrupt_stops_the_command_and_what_it_started`、単体 `webqa::interview::trials` の `interrupt_during_reset_cuts`、性質 `webqa/tests/trials_props.rs` の `finished_plus_cut_equals_started`、性質 `webqa/tests/trials_props.rs` の `finished_trials_never_exceed_setting` / issue: #30、#33
- [x] T-041 InterviewMode.feature:390(判断モデルへの問い) [state] 「インタビューのモードでは、画像を扱えると申告したアダプタに、アクセシビリティ情報のテキストとスクリーンショットを渡す」
      → 手法: 同値分割(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `attaches_screenshot_when_judge_reads_images` / issue: #20、#33
- [x] T-042 InterviewMode.feature:395(判断モデルへの問い) [state] 「探索モードと決め打ちシナリオでは、これまでどおりテキストだけを渡す」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition)、回帰テスト(levels-operational) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::run` の `other_modes_send_text_only` / issue: #30、#33
- [x] T-043 InterviewMode.feature:404(判断モデルへの問い) [ubiquitous] 「外部コマンドへの問いの JSON に、特定のモデルに依らない形で画像を入れる」
      → 手法: コントラクトテスト(levels-service)、API スキーマ検証(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-judge::command` の `image_field_shape_is_model_neutral`、契約 `webqa-judge/tests/command.rs` の `bridge_receives_image_field` / issue: #19、#33
- [x] T-044 InterviewMode.feature:410(判断モデルへの問い) [event] 「判断モデルのアダプタは、どれも起動のときに画像を扱えるかを申告する」(Examples の各行を1ケースとする)
      → 手法: コントラクトテスト(levels-service)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-judge::scripted` の `scripted_declares_image_support`、契約 `webqa-judge/tests/command.rs` の `bridge_declares_image_support`、単体 `webqa-core::interview::ask` の `questions_follow_declared_capability` / issue: #19、#20、#33
- [x] T-045 InterviewMode.feature:420(判断モデルへの問い) [event] 「橋渡しは起動のときに、JSON の版の項目で画像を扱えるかを申告する」
      → 手法: コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、契約 `webqa-judge/tests/command.rs` の `capability_read_from_version_line` / issue: #19、#33
- [x] T-046 InterviewMode.feature:425(判断モデルへの問い) [ubiquitous] 「スクリーンショットは伏せずに渡す」
      → 手法: セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `screenshot_sent_unredacted_text_redacted` / issue: #20、#33
- [x] T-047 InterviewMode.feature:433(判断モデルへの問い) [event] 「毎ステップの問いにペルソナの説明を添えて、この人ならどうするかを問う」
      → 手法: 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `prompt_includes_persona_fields`、ゴールデン `webqa-core/tests/interview_golden.rs` の `interview_prompt_snapshot` / issue: #20、#33
- [x] T-048 InterviewMode.feature:438(判断モデルへの問い) [event] 「1ステップの1回目の問い合わせで、今の4つの問いに加えて、迷い・見えにくい要素・画面の目的を問う」
      → 手法: 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `first_request_asks_seven_items`、ゴールデン `webqa-core/tests/interview_golden.rs` の `interview_prompt_snapshot` / issue: #20、#33、#79
- [x] T-049 InterviewMode.feature:471(判断モデルへの問い) [event] 「目的を書いたときも、画面の目的の推測を問う」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `asks_purpose_guess_even_when_written` / issue: #20、#33
- [x] T-050 InterviewMode.feature:476(判断モデルへの問い) [event] 「期待や理由は、自由な文の答えとしてそのまま受け取る」
      → 手法: 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::answer` の `free_text_kept_verbatim`、単体 `webqa-core::interview::answer` の `expectation_alone_is_read_as_written` / issue: #20、#33、#79
- [x] T-051 InterviewMode.feature:481(判断モデルへの問い) [unwanted] 「足した問いの答えが欠けていたら、応答の異常として扱う」(Examples の各行を1ケースとする)
      → 手法: 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::answer` の `missing_added_answer_is_anomaly`、単体 `webqa::run::tests::interviewing::unusable` の `interview_passes_on_a_malformed_reply_when_an_added_answer_is_missing`、単体 `webqa::run::tests::interviewing::second` の `second_call_is_not_made_when_no_operation_is_carried_out` / issue: #20、#33、#79
- [x] T-052 InterviewMode.feature:501(判断モデルへの問い) [boundary] 「足した問いの答えが範囲の外なら、応答の異常として扱う」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition)、非決定的出力システムのテスト(ai-nondeterministic) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::answer` の `added_answer_out_of_range_is_anomaly`、単体 `webqa-core::interview::answer` の `hesitation_range_edges_accepted`、単体 `webqa::run::tests::interviewing::unusable` の `interview_passes_on_a_malformed_reply_when_an_added_answer_is_out_of_range` / issue: #20、#33
- [x] T-053 InterviewMode.feature:513(判断モデルへの問い) [state] 「操作の対象は候補を絞らず、表示領域の操作可能要素から判断モデルに選ばせる」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `target_options_are_all_viewport_elements` / issue: #20、#33
- [x] T-054 InterviewMode.feature:519(判断モデルへの問い) [state] 「インタビューのモードでは、操作種別の選択肢に done を入れる」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `interview_offers_done_option` / issue: #20、#33
- [x] T-055 InterviewMode.feature:525(試行の終わり方) [event] 「目的の done_when を満たしたら、判断モデルに問い合わせずにゴール達成で試行を終える」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `goal_met_finishes_without_asking`(WebExploratoryQA 台帳の T-015 に依存。未実装) / issue: #13、#33
- [x] T-056 InterviewMode.feature:531(試行の終わり方) [unwanted] 「目的の確定のボタンが禁止要素でも押さず、確定の操作ができる画面に来たところでゴール達成にする」
      → 手法: 禁止仕様からの導出(blackbox-state)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::purpose` の `forbidden_confirm_button_counts_as_reached` / issue: #18、#33
- [x] T-057 InterviewMode.feature:538(試行の終わり方) [event] 「目的を推測させたとき、判断モデルの done は推測した目的に届いたという申告として記録する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::outcome` の `inferred_done_is_claimed_reach` / issue: #21、#33
- [x] T-058 InterviewMode.feature:544(試行の終わり方) [unwanted] 「目的を書いたのに done_when を満たさないまま done が返ったら、未検証の終了として記録する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::outcome` の `written_purpose_done_is_unverified_not_goal` / issue: #21、#33
- [x] T-059 InterviewMode.feature:550(試行の終わり方) [event] 「判断モデルが stuck を返したら、ペルソナが諦めたものとして行き詰まりで試行を終える」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::outcome` の `stuck_answer_records_gave_up` / issue: #21、#33
- [x] T-060 InterviewMode.feature:555(試行の終わり方) [event] 「打ち切りでない試行のうち、ゴール達成と申告による終了のほかで終わった試行は、目的に届かない試行に数える」(Examples の各行を1ケースとする)
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::outcome` の `stuck_and_budget_count_as_unreached` / issue: #21、#33
- [x] T-061 InterviewMode.feature:565(試行の終わり方) [unwanted] 「ゴール達成や申告による終了で終わった試行は、目的に届かない試行に数えない」(Examples の各行を1ケースとする)
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::outcome` の `goal_and_claim_not_unreached` / issue: #21、#33
- [x] T-062 InterviewMode.feature:577(課題の見つけ方) [boundary] 「迷いの度合いが閾値以上なら、判断に迷う課題の手がかりにする」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::clue` の `hesitation_threshold_boundary` / issue: #21、#33
- [x] T-063 InterviewMode.feature:588(課題の見つけ方) [boundary] 「確信度が閾値に届かない操作を選んだら、判断に迷う課題の手がかりにする」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::clue` の `low_confidence_threshold_boundary` / issue: #21、#33
- [x] T-064 InterviewMode.feature:598(課題の見つけ方) [event] 「引き返したら、判断に迷う課題の手がかりにする」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::clue` の `executed_back_is_clue` / issue: #21、#33
- [x] T-065 InterviewMode.feature:603(課題の見つけ方) [event] 「同じ画面を行き来したら、行き詰まりとは別に迷いの手がかりとして記録する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::clue` の `ping_pong_between_two_screens_is_clue` / issue: #21、#33
- [x] T-066 InterviewMode.feature:609(課題の見つけ方) [unwanted] 「同じ画面を行き来しても、試行は行き詰まりで終わらずに続く」
      → 手法: 禁止仕様からの導出(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::clue` の `ping_pong_leaves_stuck_counter` / issue: #21、#33
- [x] T-067 InterviewMode.feature:615(課題の見つけ方) [event] 「判断モデルが見えにくい要素を挙げたら、視認性が悪い課題として記録する」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `listed_hard_to_see_element_is_visibility_issue` / issue: #22、#33
- [x] T-068 InterviewMode.feature:638(課題の見つけ方) [unwanted] 「判断モデルが見えにくい要素を挙げなければ、視認性が悪い課題は出さない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `no_listed_element_no_visibility_issue` / issue: #22、#33
- [x] T-069 InterviewMode.feature:643(課題の見つけ方) [event] 「目的に届かなかった試行は、止まった画面と理由を目的に届かない課題として記録する」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `unreached_trial_records_last_screen_and_reason` / issue: #22、#33
- [x] T-070 InterviewMode.feature:649(課題の見つけ方) [unwanted] 「橋渡しが画像を扱えないと申告したら、視認性が悪い課題を出さず、そのことを台帳に書く」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `no_visibility_issue_without_image_support`、単体 `webqa-core::interview::ledger` の `visibility_skipped_note_when_no_image_support` / issue: #22、#24、#33
- [x] T-071 InterviewMode.feature:655(課題の見つけ方) [unwanted] 「橋渡しが画像を扱えないと申告したら、問い合わせにスクリーンショットを含めない」
      → 手法: デシジョンテーブル(blackbox-partition)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `no_screenshot_when_judge_cannot_read_images`、契約 `webqa-judge/tests/command.rs` の `no_image_sent_to_text_only_bridge` / issue: #19、#20、#33
- [x] T-072 InterviewMode.feature:662(課題の見つけ方) [unwanted] 「橋渡しが画像を扱えないと申告したら、見えにくい要素の答えが無くても応答の異常にしない」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::answer` の `hard_to_see_optional_without_image_support` / issue: #20、#33
- [x] T-073 InterviewMode.feature:669(課題の見つけ方) [unwanted] 「橋渡しが画像を扱えると申告したのに見えにくい要素の答えが無ければ、応答の異常として扱う」
      → 手法: デシジョンテーブル(blackbox-partition)、非決定的出力システムのテスト(ai-nondeterministic) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::answer` の `hard_to_see_required_with_image_support` / issue: #20、#33
- [x] T-074 InterviewMode.feature:678(ペルソナから外れないこと) [event] 「試行をまとめたあと、課題の番号ごとに1回だけ、そのペルソナとして自然かを別の問いで確かめる」
      → 手法: テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::natural` の `asks_once_per_issue_outside_budget` / issue: #23、#33
- [x] T-075 InterviewMode.feature:685(ペルソナから外れないこと) [boundary] 「自然さの問いの回数の上限を超えた課題は、確かめられなかった区分に入れる」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::natural` の `cap_asks_in_ledger_order_rest_unchecked`、単体 `webqa-core::interview::ledger` の `unchecked_section_between_body_and_off_persona` / issue: #23、#24、#33
- [x] T-076 InterviewMode.feature:693(ペルソナから外れないこと) [event] 「自然だと判断した課題は、台帳の本体に入れる」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::natural` の `natural_answer_goes_to_body` / issue: #23、#33
- [x] T-077 InterviewMode.feature:698(ペルソナから外れないこと) [unwanted] 「自然でないと判断した課題は、本体に入れず、最後の別の節にまとめる」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::natural` の `unnatural_answer_goes_to_off_persona`、単体 `webqa-core::interview::ledger` の `off_persona_section_is_last` / issue: #23、#24、#33
- [x] T-078 InterviewMode.feature:704(ペルソナから外れないこと) [unwanted] 「自然さの問いの応答が使えないときは、本体に入れず確かめられなかった区分に入れる」
      → 手法: 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::natural` の `unusable_answer_goes_to_unchecked` / issue: #23、#33
- [x] T-079 InterviewMode.feature:710(ペルソナから外れないこと) [ubiquitous] 「台帳の各課題に、ペルソナのどの性質からそう判断したかを書く」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `issue_cites_persona_trait` / issue: #24、#33
- [x] T-080 InterviewMode.feature:717(台帳の書式) [ubiquitous] 「台帳は Markdown の1ファイルで、各課題は直す指示の文を1行目に持つ」
      → 手法: ゴールデンテスト(oracle-past-output) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `first_line_is_fix_instruction`、ゴールデン `webqa-core/tests/interview_golden.rs` の `ledger_snapshot` / issue: #24、#33
- [x] T-081 InterviewMode.feature:723(台帳の書式) [ubiquitous] 「各課題は、決まった項目をすべて持つ」
      → 手法: ゴールデンテスト(oracle-past-output)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `issue_has_all_fields`、単体 `webqa-core::interview::ledger` の `mismatch_issue_fields_without_element`、ゴールデン `webqa-core/tests/interview_golden.rs` の `ledger_snapshot` / issue: #24、#33
- [x] T-082 InterviewMode.feature:744(台帳の書式) [ubiquitous] 「課題をソースコードの場所に結び付けず、画面の要素の手がかりまでを書く」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `no_source_location_fields` / issue: #24、#33
- [x] T-083 InterviewMode.feature:750(台帳の書式) [boundary] 「3回のうち1回だけ起きた課題も、その回数を添えて台帳に出す」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `single_occurrence_kept_with_count` / issue: #22、#33
- [x] T-084 InterviewMode.feature:755(台帳の書式) [ubiquitous] 「課題は起きた割合の高い順に並べる」
      → 手法: プロパティベーステスト(generative-property)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `body_order_is_rate_desc_then_number`、単体 `webqa-core::interview::ledger` の `mismatch_issue_sorted_by_pattern_reach_rate` / issue: #24、#33
- [x] T-085 InterviewMode.feature:760(台帳の書式) [boundary] 「起きた割合が同じ課題は、課題の番号の順に並べる」
      → 手法: 境界値分析(blackbox-partition)、プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `ties_ordered_by_number` / issue: #24、#33
- [x] T-086 InterviewMode.feature:765(台帳の書式) [ubiquitous] 「同じ画面の同じ要素の同じ種類の課題は、実行をまたいで同じ番号になる」
      → 手法: プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `same_screen_element_kind_same_number` / issue: #22、#33
- [x] T-087 InterviewMode.feature:770(台帳の書式) [unwanted] 「画面、要素、種類のどれかが違えば、別の番号になる」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `different_key_different_number` / issue: #22、#33
- [x] T-088 InterviewMode.feature:781(台帳の書式) [ubiquitous] 「要素を持たない課題は、画面と種類で番号を決める」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `elementless_number_from_screen_and_kind`、単体 `webqa-core::interview::issue` の `mismatch_number_from_url_pattern_and_kind` / issue: #22、#33
- [x] T-089 InterviewMode.feature:787(台帳の書式) [unwanted] 「スクリーンショットの名前は、課題の番号・試行の番号・ステップの番号から作り、パスの区切りを含めない」
      → 手法: セキュリティテスト(nonfunctional-attributes)、エラー推測(experience-checklist) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::store::screenshots` の `issue_screenshot_name_from_numbers_only`、単体 `webqa::interview::trials` の `mismatch_screenshot_named_from_first_match` / issue: #30、#33
- [x] T-090 InterviewMode.feature:794(台帳の書式) [ubiquitous] 「台帳を読み戻すと、書いた課題の項目と選択がそのまま戻る」
      → 手法: プロパティベーステスト(generative-property) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `write_then_read_round_trips` / issue: #24、#33
- [x] T-091 InterviewMode.feature:800(台帳の書式) [unwanted] 「判断モデルの自由な文に台帳の書式が入っていても、台帳の構造を崩さない」(Examples の各行を1ケースとする)
      → 手法: プロパティベーステスト(generative-property)、エラー推測(experience-checklist)、セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `free_text_cannot_forge_structure`、単体 `webqa-core::interview::ledger` の `write_then_read_round_trips` / issue: #24、#33
- [x] T-092 InterviewMode.feature:812(台帳の書式) [unwanted] 「判断モデルの自由な文に改行が入っていても、直す指示の文は1行のままにする」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `fix_instruction_stays_one_line` / issue: #24、#33
- [x] T-093 InterviewMode.feature:817(台帳の書式) [unwanted] 「台帳に書く文にも、設定で指定した機密情報の伏せ字を通す」
      → 手法: セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `ledger_text_passes_redaction` / issue: #24、#33
- [x] T-094 InterviewMode.feature:823(台帳の書式) [ubiquitous] 「台帳に、ペルソナの名前、目的、開始の URL、終えた試行の回数を書く」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `header_has_persona_purpose_url_count` / issue: #24、#33
- [x] T-095 InterviewMode.feature:830(台帳の書式) [event] 「インタビューの課題だけを見つけて終わったら、終了コードは0になる」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::exit` の `interview_issues_exit_zero` / issue: #31、#33
- [x] T-096 InterviewMode.feature:837(課題を選ぶことと TUI) [event] 「Markdown のチェックボックスを直接書き換えて選んだ課題は、選んだ課題として読み戻せる」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `edited_checkbox_reads_as_selected` / issue: #24、#33
- [x] T-097 InterviewMode.feature:842(課題を選ぶことと TUI) [event] 「TUI で課題の選択を付け外しすると、同じ Markdown に書き戻す」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-tui::ledger` の `toggle_selects_issue`、保存 `webqa-tui/tests/ledger_file.rs` の `toggle_writes_back_to_same_file` / issue: #27、#33
- [x] T-098 InterviewMode.feature:847(課題を選ぶことと TUI) [unwanted] 「TUI で書き戻しても、選択のほかの中身は変えない」
      → 手法: 冪等性テスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `set_selection_changes_only_checkbox`、保存 `webqa-tui/tests/ledger_file.rs` の `toggle_twice_restores_bytes` / issue: #24、#27、#33
- [x] T-099 InterviewMode.feature:852(課題を選ぶことと TUI) [unwanted] 「TUI で開いたあとに台帳が外で書き換えられていたら、上書きせずに知らせる」
      → 手法: 並行・競合の封じ込め(flakiness-concurrency)、禁止仕様からの導出(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-tui::ledger` の `shows_external_edit_notice`、保存 `webqa-tui/tests/ledger_file.rs` の `external_edit_blocks_overwrite` / issue: #27、#33
- [x] T-100 InterviewMode.feature:859(課題を選ぶことと TUI) [state] 「TUI で課題を種類で絞り込める」
      → 手法: 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-tui::ledger` の `filter_by_kind`、ゴールデン `webqa-tui/tests/render.rs` の `ledger_list_filtered_snapshot` / issue: #27、#33
- [x] T-101 InterviewMode.feature:864(課題を選ぶことと TUI) [event] 「TUI で課題を選ぶと、その中身を読める」
      → 手法: ゴールデンテスト(oracle-past-output) / テスト名: 受け入れ(同題の Scenario)、ゴールデン `webqa-tui/tests/render.rs` の `ledger_issue_detail_snapshot` / issue: #27、#33
- [x] T-102 InterviewMode.feature:869(課題を選ぶことと TUI) [ubiquitous] 「実行中のイベントログを追う画面は、今のまま使える」
      → 手法: 回帰テスト(levels-operational)、ゴールデンテスト(oracle-past-output) / テスト名: 受け入れ(同題の Scenario)、ゴールデン `webqa-tui/tests/render.rs` の `watch_view_snapshot` / issue: #27、#33
- [x] T-103 InterviewMode.feature:876(再実行と照らし合わせ) [event] 「前回も今回も出た課題は、残っているとして示す」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `seen_again_is_remaining` / issue: #25、#33
- [x] T-104 InterviewMode.feature:881(再実行と照らし合わせ) [event] 「今回だけ出た課題は、新しく出たとして示す」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `only_now_is_new` / issue: #25、#33
- [x] T-105 InterviewMode.feature:886(再実行と照らし合わせ) [boundary] 「その画面に届いた試行の数が前回以上で、1回も起きなければ、解消したとして示す」(Examples の各行を1ケースとする)
      → 手法: ドメイン分析テスト(blackbox-partition)、境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `resolved_when_reach_not_fewer`、単体 `webqa-core::interview::compare` の `mismatch_reach_counts_pattern_trials` / issue: #25、#33
- [x] T-106 InterviewMode.feature:897(再実行と照らし合わせ) [boundary] 「その画面に届いた試行の数が前回より1少なければ、1回も起きなくても解消したとせず、確かめられなかったとして示す」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `one_fewer_reach_is_unverified` / issue: #25、#33
- [x] T-107 InterviewMode.feature:902(再実行と照らし合わせ) [unwanted] 「前回課題が出た画面に届かなければ、確かめられなかったとして示す」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `unreached_screen_is_unverified` / issue: #25、#33
- [x] T-108 InterviewMode.feature:908(再実行と照らし合わせ) [event] 「前回選んだ課題が残っていれば、選択を引き継ぐ」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `selection_carries_to_remaining` / issue: #25、#33
- [x] T-109 InterviewMode.feature:913(再実行と照らし合わせ) [event] 「新しく出た課題は、選ばれていない状態で台帳に出す」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `new_issue_unselected` / issue: #25、#33
- [x] T-110 InterviewMode.feature:917(再実行と照らし合わせ) [boundary] 「前回の台帳を渡さなければ、すべての課題を新しく出たとして示す」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `no_previous_all_new` / issue: #25、#33
- [x] T-111 InterviewMode.feature:922(再実行と照らし合わせ) [unwanted] 「前回の台帳が決まった書式で読めなければ、照らし合わせずに設定エラーにする」
      → 手法: 構文テスト(experience-scenario)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ledger` の `broken_ledger_read_fails`、実行ファイル `webqa/tests/cli.rs` の `broken_previous_ledger_exits_two` / issue: #24、#31、#33
- [x] T-112 InterviewMode.feature:929(再実行と照らし合わせ) [unwanted] 「前回の台帳のペルソナか目的が今回と違えば、照らし合わせずに設定エラーにする」(Examples の各行を1ケースとする)
      → 手法: デシジョンテーブル(blackbox-partition)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `persona_or_purpose_mismatch_rejected`、単体 `webqa-core::interview::compare` の `previous_head_checked_before_trials`、実行ファイル `webqa/tests/cli.rs` の `mismatched_previous_ledger_exits_two` / issue: #25、#31、#33
- [x] T-113 InterviewMode.feature:941(再実行と照らし合わせ) [event] 「目的を推測させた実行どうしは、推測した文が違っても、開始の URL が同じなら照らし合わせる」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::compare` の `inferred_runs_compare_by_start_url` / issue: #25、#33
- [x] T-114 InterviewMode.feature:949(手引きと形式の検査) [event] 「形式の検査のサブコマンドは、正しい文書を通す」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、実行ファイル `webqa/tests/cli.rs` の `check_accepts_valid_documents` / issue: #31、#33
- [x] T-115 InterviewMode.feature:959(手引きと形式の検査) [unwanted] 「形式の検査のサブコマンドは、誤りのある文書の誤りの場所を示して失敗する」(Examples の各行を1ケースとする)
      → 手法: 構文テスト(experience-scenario)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::purpose` の `done_when_without_declarations_rejected`、実行ファイル `webqa/tests/cli.rs` の `check_reports_field_and_exits_two` / issue: #18、#31、#33
- [x] T-116 InterviewMode.feature:970(手引きと形式の検査) [ubiquitous] 「手引きは Agent Skills の標準の形式で、形式の検査のサブコマンドを呼ぶ手順と書式の例を持つ」
      → 手法: チェックリストベーステスト(experience-checklist) / テスト名: 受け入れ(同題の Scenario)、文書 `webqa/tests/guide.rs` の `guide_has_frontmatter_steps_and_examples` / issue: #32、#33
- [x] T-117 InterviewMode.feature:976(手引きと形式の検査) [ubiquitous] 「手引きは、目的の done_when を確定の操作ができる画面まで来たことで書くよう示す」
      → 手法: チェックリストベーステスト(experience-checklist) / テスト名: 受け入れ(同題の Scenario)、文書 `webqa/tests/guide.rs` の `guide_shows_done_when_at_confirm_screen` / issue: #32、#33
- [x] T-118 InterviewMode.feature:980(手引きと形式の検査) [ubiquitous] 「手引きは、このリポジトリの開発用の手順書とは別の場所に置く」
      → 手法: チェックリストベーステスト(experience-checklist) / テスト名: 受け入れ(同題の Scenario)、文書 `webqa/tests/guide.rs` の `guide_lives_outside_dev_skills` / issue: #32、#33
- [x] T-119 InterviewMode.feature:986(既存のモードとの関係) [ubiquitous] 「探索モードと決め打ちシナリオの受け入れシナリオは、そのまま通る」
      → 手法: 回帰テスト(levels-operational) / テスト名: 受け入れ(同題の Scenario)、受け入れ `webqa/tests/acceptance.rs` の `web_exploratory_qa_feature` / issue: #33
- [x] T-120 InterviewMode.feature:991(既存のモードとの関係) [unwanted] 「インタビューのモードでも、既存のガードレールで止まる操作は実行しない」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition)、回帰テスト(levels-operational) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::forbidden` の `forbidden_by_keyword`、単体 `webqa-core::guard::confidence` の `back_confidence_boundary`、単体 `webqa-core::guard::identity` の `missing_element_aborts`(WebExploratoryQA 台帳の T-007、T-005、T-008 に依存。未実装) / issue: #4、#33
- [x] T-121 InterviewMode.feature:1002(既存のモードとの関係) [unwanted] 「インタビューのモードでも、判断モデルの呼び出しが上限に達したら試行を止める」
      → 手法: 境界値分析(blackbox-partition)、回帰テスト(levels-operational) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `budget_exhausted_finishes`(WebExploratoryQA 台帳の T-020 に依存。未実装) / issue: #12、#33
- [x] T-122 InterviewMode.feature:1007(既存のモードとの関係) [unwanted] 「インタビューのモードでも、設定で指定した機密情報は判断モデルに送るテキストで伏せる」
      → 手法: セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `screen_text_redacted_in_interview` / issue: #20、#33
- [x] T-123 InterviewMode.feature:1013(既存のモードとの関係) [ubiquitous] 「インタビューのモードは課題を直さず、台帳と実行結果のほかにファイルを書き換えない」
      → 手法: 禁止仕様からの導出(blackbox-state)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interview::trials` の `writes_only_ledger_events_report_screenshots`、単体 `webqa::interview::trials` の `detected_problem_screenshots_do_not_overwrite_across_trials` / issue: #30、#33
- [x] T-124 InterviewMode.feature:92(ペルソナと目的の入力) [unwanted] 「画面ごとの目的の URL の型に合う画面に届いた試行が無ければ、ずれの問いを聞かない」
      → 手法: デシジョンテーブル(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `no_mismatch_question_without_reaching_trial` / issue: #20、#33
- [x] T-125 InterviewMode.feature:99(ペルソナと目的の入力) [boundary] 「その URL の型に合う画面に届いたのが打ち切りの試行だけなら、ずれの問いを聞かない」
      → 手法: 境界値分析(blackbox-partition)、禁止仕様からの導出(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `cut_trial_reach_does_not_trigger_mismatch_question` / issue: #20、#33
- [x] T-126 InterviewMode.feature:105(ペルソナと目的の入力) [unwanted] 「その URL の型に合う画面に使える推測が1つも無ければ、ずれの問いを聞かない」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `no_mismatch_question_without_usable_guess` / issue: #20、#33
- [x] T-127 InterviewMode.feature:111(ペルソナと目的の入力) [unwanted] 「ずれの問いの答えが使えなければ、ずれの課題を出さず確かめられなかった区分に入れる」(Examples の各行を1ケースとする)
      → 手法: 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition)、デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::answer` の `mismatch_answer_missing_or_off_choice_is_unusable`、単体 `webqa-core::interview::issue` の `unusable_mismatch_answer_goes_to_unchecked` / issue: #20、#22、#33
- [x] T-128 InterviewMode.feature:123(ペルソナと目的の入力) [boundary] 「ずれの問いは自然さの問いと同じ上限に数え、超えた分は確かめられなかった区分に入れる」
      → 手法: 境界値分析(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::natural` の `mismatch_questions_take_cap_before_natural`、単体 `webqa::interview::trials` の `mismatch_questions_leave_trial_budget` / issue: #23、#30、#33
- [x] T-129 InterviewMode.feature:78(ペルソナと目的の入力) [boundary] 「同じ URL の型に合う複数の画面の推測を、重複をまとめて1回だけ聞く」
      → 手法: 境界値分析(blackbox-partition)、同値分割(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `guesses_across_matching_screens_merged_trimmed` / issue: #20、#33
- [x] T-130 InterviewMode.feature:131(ペルソナと目的の入力) [event] 「目的のずれの課題も、自然さの問いにかける」
      → 手法: 同値分割(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::natural` の `mismatch_issue_also_gets_natural_question` / issue: #23、#33
- [x] T-131 InterviewMode.feature:69(ペルソナと目的の入力) [event] 「目的のずれの課題は、URL の型を画面とし、出現回数と届いた試行の数をその型に届いた終えた試行の数で書く」
      → 手法: 同値分割(blackbox-partition)、境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::issue` の `mismatch_issue_counts_trials_reaching_pattern`、単体 `webqa-core::interview::issue` の `mismatch_screenshot_from_first_matching_observation` / issue: #22、#33
- [x] T-132 InterviewMode.feature:620(課題の見つけ方) [boundary] 「判断モデルが見えにくい要素を4つ挙げても、視認性が悪い課題は先頭の3つにとどめる」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::answer` の `hard_to_see_cut_to_the_first_three` / issue:
- [x] T-133 InterviewMode.feature:625(課題の見つけ方) [event] 「判断モデルの答えの書き方が揺れていても、見えにくい要素として読む」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition)、非決定的出力システムのテスト(ai-nondeterministic) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::answer` の `hard_to_see_required_with_image_support`、単体 `webqa::run` の `interview_passes_on_a_malformed_reply_when_hard_to_see_is_not_an_option` / issue:
- [x] T-135 InterviewMode.feature:452(判断モデルへの問い) [event] 「期待は、中核が選んだ操作と要素を問いの文に入れて、2回目の問い合わせで問う」
      → 手法: 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::interview::ask` の `expectation_question_names_the_operation_and_the_element`、単体 `webqa-core::machine::chosen` の `an_operation_on_an_element_names_it`、単体 `webqa-core::machine::chosen` の `the_element_passed_over_for_being_unmoved_is_the_one_named`、ゴールデン `webqa-core/tests/interview_golden.rs` の `expectation_prompt_snapshot`、単体 `webqa::run::tests::interviewing::second` の `second_call_asks_only_the_expectation_of_the_chosen_operation`、単体 `webqa::run::tests::interviewing::unmoved` の `second_call_names_the_element_the_core_picked`、単体 `webqa::run::tests::interviewing::unmoved` の `each_call_is_logged_and_recorded_in_order` / issue: #79
- [x] T-136 InterviewMode.feature:459(判断モデルへの問い) [unwanted] 「操作を行わないステップでは、期待を問わない」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::chosen` の `nothing_is_chosen_when_the_core_carries_out_nothing`、単体 `webqa-core::interview::answer` の `expectation_is_empty_when_it_was_not_asked`、単体 `webqa-core::interview::ledger::write` の `an_empty_expectation_leaves_no_parentheses`、単体 `webqa::run::tests::interviewing::second` の `second_call_is_not_made_when_no_operation_is_carried_out` / issue: #79
- [x] T-137 InterviewMode.feature:494(判断モデルへの問い) [unwanted] 「期待の問いの答えが欠けていたら、応答の異常として扱う」
      → 手法: 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::run::tests::interviewing::second` の `missing_expectation_is_unreadable_after_two_calls`、単体 `webqa-core::interview::answer` の `expectation_alone_is_read_as_written` / issue: #79

## 試行の外のループの性質テスト

抽出台帳の「状態機械の外の新しいループ」の方針どおり、試行の外のループは TLA+ でなく性質テストで確かめる。
`webqa/tests/trials_props.rs` は、試行の長さ、試行を始める時刻、各試行のアクセスの数と時刻、初期化のコマンドとブラウザの起動にかかる時間を乱数で作り、偽の時計で回す。seed と試行回数は固定する。
抽出台帳に挙げた5つの性質に、待っている間に初期化のコマンドを始めない性質を1つ足した。

| 性質のテスト | 確かめること | 受け持つ T-ID |
|---|---|---|
| `every_run_window_stays_within_limit` | どの実行全体の窓でも、数えたアクセス(最初の表示を含み、初期化のコマンドの分を除く)は上限以下 | T-022〜T-025、T-027〜T-029 |
| `nothing_starts_after_deadline` | 期限を過ぎた時刻に、初期化のコマンドもブラウザの起動も始まっていない | T-033〜T-035、T-037 |
| `finished_plus_cut_equals_started` | 終えた試行と打ち切りの数の和は始めた試行の数に等しく、打ち切りは多くても1回 | T-038、T-040 |
| `wait_ends_on_abort_or_deadline` | 試行の間で待っている間に中断か期限が来たら、次の窓を待たずに終える | T-031、T-032 |
| `finished_trials_never_exceed_setting` | 終えた試行の数は設定の試行の回数以下 | T-040 |
| `no_reset_while_waiting_for_window` | 窓が明けるのを待っている間に初期化のコマンドを始めない(判定は初期化のコマンドの前) | T-026 |

各行の単体のテストは、性質テストで反例が出たときに切り分けるための具体例である。性質テストで見つかった最小の反例は、その行の単体のテストのケースに足して固定する。

## トレーサビリティマトリクス(消し込み台帳)

mutation 列は、設定エラー、試行と試行の外のループ、終わり方、迷いの手がかり、課題の番号と区分、台帳の構造と読み戻し、照らし合わせ、選択の書き戻し、セキュリティ、終了コード、既存のモードとの関係の行を「要」にした。
「—」は、問いの文面と画像の受け渡し(T-041〜T-045、T-047〜T-050、T-053、T-054、T-135)、目的の推測を書くこと(T-005)、視認性と目的に届かない課題の中身(T-067〜T-069)、台帳の見た目と並び(T-079〜T-085、T-094)、TUI の表示(T-100〜T-102)、手引き(T-116〜T-118)の行で、セキュリティ・状態遷移・終了コードに当たらない。

| 振る舞い(T番号) | 種別 | 手法(reference) | テスト名 | issue | 設計済 | 緑確認済 | mutation(criticalのみ) |
|------------------|------|------------------|----------|-------|--------|----------|------------------------|
| T-001 | event | 同値分割(blackbox-partition) | `reads_all_required_fields` | #18、#33 | [x] | [ ] | 要 |
| T-002 | unwanted | ネガティブテスト(process-static)、同値分割(blackbox-partition) | `missing_field_is_config_error`、`interview_config_error_exits_two_before_opening` | #18、#31、#33 | [x] | [ ] | 要 |
| T-003 | unwanted | 構文テスト(experience-scenario)、ネガティブテスト(process-static) | `malformed_persona_rejected` | #18、#33 | [x] | [ ] | 要 |
| T-004 | event | 状態遷移テスト(blackbox-state) | `written_purpose_becomes_single_stage` | #18、#33 | [x] | [ ] | 要 |
| T-005 | event | 同値分割(blackbox-partition) | `inferred_purpose_is_written` | #24、#33 | [x] | [ ] | — |
| T-006 | unwanted | デシジョンテーブル(blackbox-partition)、テストダブル(test-doubles)、非決定的出力システムのテスト(ai-nondeterministic) | `asks_mismatch_once_per_url_pattern`、`mismatch_question_passes_all_guesses_once`、`judge_says_different_purpose_is_mismatch`、`mismatch_issue_screen_is_url_pattern` | #20、#22、#33 | [x] | [ ] | 要 |
| T-007 | event | デシジョンテーブル(blackbox-partition)、テストダブル(test-doubles)、非決定的出力システムのテスト(ai-nondeterministic) | `different_wording_is_left_to_judge`、`judge_says_same_purpose_no_mismatch` | #20、#22、#33 | [x] | [ ] | 要 |
| T-008 | state | 同値分割(blackbox-partition) | `no_mismatch_question_for_undeclared_screen`、`undeclared_screen_not_compared` | #20、#22、#33 | [x] | [ ] | 要 |
| T-009 | unwanted | 構文テスト(experience-scenario)、ネガティブテスト(process-static) | `malformed_url_pattern_rejected` | #18、#33 | [x] | [ ] | 要 |
| T-010 | boundary | 境界値分析(blackbox-partition)、ネガティブテスト(process-static) | `rejects_out_of_range_settings`、`accepts_range_edges` | #18、#33 | [x] | [ ] | 要 |
| T-011 | optional | 境界値分析(blackbox-partition) | `stuck_threshold_overrides_for_interview` | #18、#33 | [x] | [ ] | 要 |
| T-012 | ubiquitous | 同値分割(blackbox-partition)、テストダブル(test-doubles) | `runs_configured_trials_into_one_ledger` | #30、#33 | [x] | [ ] | 要 |
| T-013 | event | テストダブル(test-doubles)、コントラクトテスト(levels-service) | `each_trial_opens_fresh_browser_with_storage_state`、`new_context_has_no_previous_storage` | #29、#30、#33 | [x] | [ ] | 要 |
| T-014 | optional | テストダブル(test-doubles) | `reset_runs_before_each_browser_open` | #30、#33 | [x] | [ ] | 要 |
| T-015 | state | 同値分割(blackbox-partition)、テストダブル(test-doubles) | `no_reset_command_runs_nothing` | #30、#33 | [x] | [ ] | 要 |
| T-016 | unwanted | ネガティブテスト(process-static)、テストダブル(test-doubles)、時刻依存の封じ込め(flakiness-value) | `nonzero_or_timeout_is_failure`、`reset_failure_writes_ledger_then_exits_three`、`failed_reset_before_any_trial_is_a_run_that_got_nowhere` | #28、#30、#33 | [x] | [ ] | 要 |
| T-017 | ubiquitous | セキュリティテスト(nonfunctional-attributes) | `args_passed_verbatim_without_shell` | #28、#33 | [x] | [ ] | 要 |
| T-018 | unwanted | セキュリティテスト(nonfunctional-attributes) | `api_key_env_not_inherited` | #28、#33 | [x] | [ ] | 要 |
| T-019 | unwanted | セキュリティテスト(nonfunctional-attributes) | `logged_output_is_redacted` | #28、#33 | [x] | [ ] | 要 |
| T-020 | event | CRUD/ライフサイクルテスト(blackbox-state)、テストダブル(test-doubles) | `trials_start_with_empty_memory_and_never_save` | #30、#33 | [x] | [ ] | 要 |
| T-021 | event | 状態遷移テスト(blackbox-state) | `budget_resets_per_trial` | #30、#33 | [x] | [ ] | 要 |
| T-022 | event | プロパティベーステスト(generative-property)、同値分割(blackbox-partition) | `first_page_counts_one_access`、`every_run_window_stays_within_limit` | #26、#30、#33 | [x] | [ ] | 要 |
| T-023 | ubiquitous | プロパティベーステスト(generative-property)、同値分割(blackbox-partition) | `reset_accesses_not_counted`、`every_run_window_stays_within_limit` | #30、#33 | [x] | [ ] | 要 |
| T-024 | boundary | 境界値分析(blackbox-partition)、プロパティベーステスト(generative-property) | `starts_with_remaining_minus_one_at_half`、`every_run_window_stays_within_limit` | #26、#30、#33 | [x] | [ ] | 要 |
| T-025 | boundary | 境界値分析(blackbox-partition)、プロパティベーステスト(generative-property)、時刻依存の封じ込め(flakiness-value) | `waits_when_below_half`、`every_run_window_stays_within_limit` | #26、#30、#33 | [x] | [ ] | 要 |
| T-026 | event | 禁止仕様からの導出(blackbox-state)、プロパティベーステスト(generative-property) | `start_check_precedes_reset`、`no_reset_while_waiting_for_window` | #30、#33 | [x] | [ ] | 要 |
| T-027 | ubiquitous | 時刻依存の封じ込め(flakiness-value)、プロパティベーステスト(generative-property) | `core_window_aligns_with_run_window`、`every_run_window_stays_within_limit` | #26、#30、#33 | [x] | [ ] | 要 |
| T-028 | state | 状態遷移テスト(blackbox-state)、プロパティベーステスト(generative-property) | `core_limit_fixed_for_trial`、`every_run_window_stays_within_limit` | #30、#33 | [x] | [ ] | 要 |
| T-029 | unwanted | 境界値分析(blackbox-partition)、プロパティベーステスト(generative-property) | `reduced_limit_blocks_core_until_window`、`every_run_window_stays_within_limit` | #30、#33 | [x] | [ ] | 要 |
| T-030 | event | 状態遷移テスト(blackbox-state) | `core_window_count_starts_at_zero` | #30、#33 | [x] | [ ] | 要 |
| T-031 | state | 並行・競合の封じ込め(flakiness-concurrency)、プロパティベーステスト(generative-property) | `abort_while_waiting_writes_ledger`、`wait_ends_on_abort_or_deadline` | #30、#33 | [x] | [ ] | 要 |
| T-032 | state | 時刻依存の封じ込め(flakiness-value)、プロパティベーステスト(generative-property) | `deadline_while_waiting_times_out`、`wait_ends_on_abort_or_deadline` | #30、#33 | [x] | [ ] | 要 |
| T-033 | ubiquitous | 時刻依存の封じ込め(flakiness-value)、プロパティベーステスト(generative-property) | `deadline_measured_from_run_start`、`nothing_starts_after_deadline` | #30、#33 | [x] | [ ] | 要 |
| T-034 | state | 時刻依存の封じ込め(flakiness-value)、状態遷移テスト(blackbox-state)、プロパティベーステスト(generative-property) | `deadline_during_reset_waits_then_cuts`、`nothing_starts_after_deadline` | #30、#33 | [x] | [ ] | 要 |
| T-035 | unwanted | 時刻依存の封じ込め(flakiness-value)、デシジョンテーブル(blackbox-partition)、プロパティベーステスト(generative-property) | `reset_timeout_after_deadline_is_time_up`、`nothing_starts_after_deadline` | #30、#33 | [x] | [ ] | 要 |
| T-036 | unwanted | 禁止仕様からの導出(blackbox-state) | `cut_trial_clues_are_dropped` | #22、#33 | [x] | [ ] | 要 |
| T-037 | state | 並行・競合の封じ込め(flakiness-concurrency)、時刻依存の封じ込め(flakiness-value)、プロパティベーステスト(generative-property) | `deadline_during_browser_start_cuts_without_waiting`、`nothing_starts_after_deadline` | #30、#33 | [x] | [ ] | 要 |
| T-038 | unwanted | デシジョンテーブル(blackbox-partition)、プロパティベーステスト(generative-property) | `cut_trials_not_counted`、`finished_plus_cut_equals_started` | #21、#30、#33 | [x] | [ ] | 要 |
| T-039 | boundary | 境界値分析(blackbox-partition) | `zero_finished_trials_has_no_issues` | #24、#33 | [x] | [ ] | 要 |
| T-040 | event | 並行・競合の封じ込め(flakiness-concurrency)、プロパティベーステスト(generative-property) | `abort_mid_trial_skips_rest`、`interrupt_stops_the_command_and_what_it_started`、`interrupt_during_reset_cuts`、`finished_plus_cut_equals_started`、`finished_trials_never_exceed_setting` | #30、#33 | [x] | [ ] | 要 |
| T-041 | state | 同値分割(blackbox-partition)、テストダブル(test-doubles) | `attaches_screenshot_when_judge_reads_images` | #20、#33 | [x] | [ ] | — |
| T-042 | state | 同値分割(blackbox-partition)、回帰テスト(levels-operational) | `other_modes_send_text_only` | #30、#33 | [x] | [ ] | — |
| T-043 | ubiquitous | コントラクトテスト(levels-service)、API スキーマ検証(levels-service) | `image_field_shape_is_model_neutral`、`bridge_receives_image_field` | #19、#33 | [x] | [ ] | — |
| T-044 | event | コントラクトテスト(levels-service)、テストダブル(test-doubles) | `scripted_declares_image_support`、`bridge_declares_image_support`、`questions_follow_declared_capability` | #19、#20、#33 | [x] | [ ] | — |
| T-045 | event | コントラクトテスト(levels-service) | `capability_read_from_version_line` | #19、#33 | [x] | [ ] | — |
| T-046 | ubiquitous | セキュリティテスト(nonfunctional-attributes) | `screenshot_sent_unredacted_text_redacted` | #20、#33 | [x] | [ ] | 要 |
| T-047 | event | 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output) | `prompt_includes_persona_fields`、`interview_prompt_snapshot` | #20、#33 | [x] | [ ] | — |
| T-048 | event | 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output) | `first_request_asks_seven_items`、`interview_prompt_snapshot` | #20、#33、#79 | [x] | [ ] | — |
| T-049 | event | 同値分割(blackbox-partition) | `asks_purpose_guess_even_when_written` | #20、#33 | [x] | [ ] | — |
| T-050 | event | 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) | `free_text_kept_verbatim`、`expectation_alone_is_read_as_written` | #20、#33、#79 | [x] | [ ] | — |
| T-051 | unwanted | 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) | `missing_added_answer_is_anomaly`、`interview_passes_on_a_malformed_reply_when_an_added_answer_is_missing`、`second_call_is_not_made_when_no_operation_is_carried_out` | #20、#33、#79 | [x] | [ ] | 要 |
| T-052 | boundary | 境界値分析(blackbox-partition)、非決定的出力システムのテスト(ai-nondeterministic) | `added_answer_out_of_range_is_anomaly`、`hesitation_range_edges_accepted`、`interview_passes_on_a_malformed_reply_when_an_added_answer_is_out_of_range` | #20、#33 | [x] | [ ] | 要 |
| T-053 | state | 同値分割(blackbox-partition) | `target_options_are_all_viewport_elements` | #20、#33 | [x] | [ ] | — |
| T-054 | state | 同値分割(blackbox-partition) | `interview_offers_done_option` | #20、#33 | [x] | [ ] | — |
| T-055 | event | 状態遷移テスト(blackbox-state) | `goal_met_finishes_without_asking` | #13、#33 | [x] | [ ] | 要 |
| T-056 | unwanted | 禁止仕様からの導出(blackbox-state)、同値分割(blackbox-partition) | `forbidden_confirm_button_counts_as_reached` | #18、#33 | [x] | [ ] | 要 |
| T-057 | event | 状態遷移テスト(blackbox-state) | `inferred_done_is_claimed_reach` | #21、#33 | [x] | [ ] | 要 |
| T-058 | unwanted | 状態遷移テスト(blackbox-state) | `written_purpose_done_is_unverified_not_goal` | #21、#33 | [x] | [ ] | 要 |
| T-059 | event | 状態遷移テスト(blackbox-state) | `stuck_answer_records_gave_up` | #21、#33 | [x] | [ ] | 要 |
| T-060 | event | デシジョンテーブル(blackbox-partition) | `stuck_and_budget_count_as_unreached` | #21、#33 | [x] | [ ] | 要 |
| T-061 | unwanted | デシジョンテーブル(blackbox-partition) | `goal_and_claim_not_unreached` | #21、#33 | [x] | [ ] | 要 |
| T-062 | boundary | 境界値分析(blackbox-partition) | `hesitation_threshold_boundary` | #21、#33 | [x] | [ ] | 要 |
| T-063 | boundary | 境界値分析(blackbox-partition) | `low_confidence_threshold_boundary` | #21、#33 | [x] | [ ] | 要 |
| T-064 | event | 同値分割(blackbox-partition) | `executed_back_is_clue` | #21、#33 | [x] | [ ] | 要 |
| T-065 | event | 状態遷移テスト(blackbox-state) | `ping_pong_between_two_screens_is_clue` | #21、#33 | [x] | [ ] | 要 |
| T-066 | unwanted | 禁止仕様からの導出(blackbox-state) | `ping_pong_leaves_stuck_counter` | #21、#33 | [x] | [ ] | 要 |
| T-067 | event | 同値分割(blackbox-partition) | `listed_hard_to_see_element_is_visibility_issue` | #22、#33 | [x] | [ ] | — |
| T-068 | unwanted | 同値分割(blackbox-partition) | `no_listed_element_no_visibility_issue` | #22、#33 | [x] | [ ] | — |
| T-069 | event | 同値分割(blackbox-partition) | `unreached_trial_records_last_screen_and_reason` | #22、#33 | [x] | [ ] | — |
| T-070 | unwanted | デシジョンテーブル(blackbox-partition) | `no_visibility_issue_without_image_support`、`visibility_skipped_note_when_no_image_support` | #22、#24、#33 | [x] | [ ] | 要 |
| T-071 | unwanted | デシジョンテーブル(blackbox-partition)、コントラクトテスト(levels-service) | `no_screenshot_when_judge_cannot_read_images`、`no_image_sent_to_text_only_bridge` | #19、#20、#33 | [x] | [ ] | 要 |
| T-072 | unwanted | デシジョンテーブル(blackbox-partition) | `hard_to_see_optional_without_image_support` | #20、#33 | [x] | [ ] | 要 |
| T-073 | unwanted | デシジョンテーブル(blackbox-partition)、非決定的出力システムのテスト(ai-nondeterministic) | `hard_to_see_required_with_image_support` | #20、#33 | [x] | [ ] | 要 |
| T-074 | event | テストダブル(test-doubles) | `asks_once_per_issue_outside_budget` | #23、#33 | [x] | [ ] | 要 |
| T-075 | boundary | 境界値分析(blackbox-partition) | `cap_asks_in_ledger_order_rest_unchecked`、`unchecked_section_between_body_and_off_persona` | #23、#24、#33 | [x] | [ ] | 要 |
| T-076 | event | 同値分割(blackbox-partition) | `natural_answer_goes_to_body` | #23、#33 | [x] | [ ] | 要 |
| T-077 | unwanted | 同値分割(blackbox-partition) | `unnatural_answer_goes_to_off_persona`、`off_persona_section_is_last` | #23、#24、#33 | [x] | [ ] | 要 |
| T-078 | unwanted | 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) | `unusable_answer_goes_to_unchecked` | #23、#33 | [x] | [ ] | 要 |
| T-079 | ubiquitous | 同値分割(blackbox-partition) | `issue_cites_persona_trait` | #24、#33 | [x] | [ ] | — |
| T-080 | ubiquitous | ゴールデンテスト(oracle-past-output) | `first_line_is_fix_instruction`、`ledger_snapshot` | #24、#33 | [x] | [ ] | — |
| T-081 | ubiquitous | ゴールデンテスト(oracle-past-output)、同値分割(blackbox-partition) | `issue_has_all_fields`、`mismatch_issue_fields_without_element`、`ledger_snapshot` | #24、#33 | [x] | [ ] | — |
| T-082 | ubiquitous | 同値分割(blackbox-partition) | `no_source_location_fields` | #24、#33 | [x] | [ ] | — |
| T-083 | boundary | 境界値分析(blackbox-partition) | `single_occurrence_kept_with_count` | #22、#33 | [x] | [ ] | — |
| T-084 | ubiquitous | プロパティベーステスト(generative-property)、同値分割(blackbox-partition) | `body_order_is_rate_desc_then_number`、`mismatch_issue_sorted_by_pattern_reach_rate` | #24、#33 | [x] | [ ] | — |
| T-085 | boundary | 境界値分析(blackbox-partition)、プロパティベーステスト(generative-property) | `ties_ordered_by_number` | #24、#33 | [x] | [ ] | — |
| T-086 | ubiquitous | プロパティベーステスト(generative-property) | `same_screen_element_kind_same_number` | #22、#33 | [x] | [ ] | 要 |
| T-087 | unwanted | 同値分割(blackbox-partition) | `different_key_different_number` | #22、#33 | [x] | [ ] | 要 |
| T-088 | ubiquitous | 同値分割(blackbox-partition) | `elementless_number_from_screen_and_kind`、`mismatch_number_from_url_pattern_and_kind` | #22、#33 | [x] | [ ] | 要 |
| T-089 | unwanted | セキュリティテスト(nonfunctional-attributes)、エラー推測(experience-checklist) | `issue_screenshot_name_from_numbers_only`、`mismatch_screenshot_named_from_first_match` | #30、#33 | [x] | [ ] | 要 |
| T-090 | ubiquitous | プロパティベーステスト(generative-property) | `write_then_read_round_trips` | #24、#33 | [x] | [ ] | 要 |
| T-091 | unwanted | プロパティベーステスト(generative-property)、エラー推測(experience-checklist)、セキュリティテスト(nonfunctional-attributes) | `free_text_cannot_forge_structure`、`write_then_read_round_trips` | #24、#33 | [x] | [ ] | 要 |
| T-092 | unwanted | 同値分割(blackbox-partition) | `fix_instruction_stays_one_line` | #24、#33 | [x] | [ ] | 要 |
| T-093 | unwanted | セキュリティテスト(nonfunctional-attributes) | `ledger_text_passes_redaction` | #24、#33 | [x] | [ ] | 要 |
| T-094 | ubiquitous | 同値分割(blackbox-partition) | `header_has_persona_purpose_url_count` | #24、#33 | [x] | [ ] | — |
| T-095 | event | 同値分割(blackbox-partition) | `interview_issues_exit_zero` | #31、#33 | [x] | [ ] | 要 |
| T-096 | event | 同値分割(blackbox-partition) | `edited_checkbox_reads_as_selected` | #24、#33 | [x] | [ ] | 要 |
| T-097 | event | 状態遷移テスト(blackbox-state) | `toggle_selects_issue`、`toggle_writes_back_to_same_file` | #27、#33 | [x] | [ ] | 要 |
| T-098 | unwanted | 冪等性テスト(process-static) | `set_selection_changes_only_checkbox`、`toggle_twice_restores_bytes` | #24、#27、#33 | [x] | [ ] | 要 |
| T-099 | unwanted | 並行・競合の封じ込め(flakiness-concurrency)、禁止仕様からの導出(blackbox-state) | `shows_external_edit_notice`、`external_edit_blocks_overwrite` | #27、#33 | [x] | [ ] | 要 |
| T-100 | state | 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output) | `filter_by_kind`、`ledger_list_filtered_snapshot` | #27、#33 | [x] | [ ] | — |
| T-101 | event | ゴールデンテスト(oracle-past-output) | `ledger_issue_detail_snapshot` | #27、#33 | [x] | [ ] | — |
| T-102 | ubiquitous | 回帰テスト(levels-operational)、ゴールデンテスト(oracle-past-output) | `watch_view_snapshot` | #27、#33 | [x] | [ ] | — |
| T-103 | event | デシジョンテーブル(blackbox-partition) | `seen_again_is_remaining` | #25、#33 | [x] | [ ] | 要 |
| T-104 | event | デシジョンテーブル(blackbox-partition) | `only_now_is_new` | #25、#33 | [x] | [ ] | 要 |
| T-105 | boundary | ドメイン分析テスト(blackbox-partition)、境界値分析(blackbox-partition) | `resolved_when_reach_not_fewer`、`mismatch_reach_counts_pattern_trials` | #25、#33 | [x] | [ ] | 要 |
| T-106 | boundary | 境界値分析(blackbox-partition) | `one_fewer_reach_is_unverified` | #25、#33 | [x] | [ ] | 要 |
| T-107 | unwanted | 同値分割(blackbox-partition) | `unreached_screen_is_unverified` | #25、#33 | [x] | [ ] | 要 |
| T-108 | event | デシジョンテーブル(blackbox-partition) | `selection_carries_to_remaining` | #25、#33 | [x] | [ ] | 要 |
| T-109 | event | デシジョンテーブル(blackbox-partition) | `new_issue_unselected` | #25、#33 | [x] | [ ] | 要 |
| T-110 | boundary | 境界値分析(blackbox-partition) | `no_previous_all_new` | #25、#33 | [x] | [ ] | 要 |
| T-111 | unwanted | 構文テスト(experience-scenario)、ネガティブテスト(process-static) | `broken_ledger_read_fails`、`broken_previous_ledger_exits_two` | #24、#31、#33 | [x] | [ ] | 要 |
| T-112 | unwanted | デシジョンテーブル(blackbox-partition)、ネガティブテスト(process-static) | `persona_or_purpose_mismatch_rejected`、`previous_head_checked_before_trials`、`mismatched_previous_ledger_exits_two` | #25、#31、#33 | [x] | [ ] | 要 |
| T-113 | event | デシジョンテーブル(blackbox-partition) | `inferred_runs_compare_by_start_url` | #25、#33 | [x] | [ ] | 要 |
| T-114 | event | 同値分割(blackbox-partition) | `check_accepts_valid_documents` | #31、#33 | [x] | [ ] | 要 |
| T-115 | unwanted | 構文テスト(experience-scenario)、ネガティブテスト(process-static) | `done_when_without_declarations_rejected`、`check_reports_field_and_exits_two` | #18、#31、#33 | [x] | [ ] | 要 |
| T-116 | ubiquitous | チェックリストベーステスト(experience-checklist) | `guide_has_frontmatter_steps_and_examples` | #32、#33 | [x] | [ ] | — |
| T-117 | ubiquitous | チェックリストベーステスト(experience-checklist) | `guide_shows_done_when_at_confirm_screen` | #32、#33 | [x] | [ ] | — |
| T-118 | ubiquitous | チェックリストベーステスト(experience-checklist) | `guide_lives_outside_dev_skills` | #32、#33 | [x] | [ ] | — |
| T-119 | ubiquitous | 回帰テスト(levels-operational) | `web_exploratory_qa_feature` | #33 | [x] | [ ] | 要 |
| T-120 | unwanted | 同値分割(blackbox-partition)、回帰テスト(levels-operational) | `forbidden_by_keyword`、`back_confidence_boundary`、`missing_element_aborts` | #4、#33 | [x] | [ ] | 要 |
| T-121 | unwanted | 境界値分析(blackbox-partition)、回帰テスト(levels-operational) | `budget_exhausted_finishes` | #12、#33 | [x] | [ ] | 要 |
| T-122 | unwanted | セキュリティテスト(nonfunctional-attributes) | `screen_text_redacted_in_interview` | #20、#33 | [x] | [ ] | 要 |
| T-123 | ubiquitous | 禁止仕様からの導出(blackbox-state)、テストダブル(test-doubles) | `writes_only_ledger_events_report_screenshots`、`detected_problem_screenshots_do_not_overwrite_across_trials` | #30、#33 | [x] | [ ] | 要 |
| T-124 | unwanted | デシジョンテーブル(blackbox-partition)、テストダブル(test-doubles) | `no_mismatch_question_without_reaching_trial` | #20、#33 | [x] | [ ] | 要 |
| T-125 | boundary | 境界値分析(blackbox-partition)、禁止仕様からの導出(blackbox-state) | `cut_trial_reach_does_not_trigger_mismatch_question` | #20、#33 | [x] | [ ] | 要 |
| T-126 | unwanted | デシジョンテーブル(blackbox-partition) | `no_mismatch_question_without_usable_guess` | #20、#33 | [x] | [ ] | 要 |
| T-127 | unwanted | 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition)、デシジョンテーブル(blackbox-partition) | `mismatch_answer_missing_or_off_choice_is_unusable`、`unusable_mismatch_answer_goes_to_unchecked` | #20、#22、#33 | [x] | [ ] | 要 |
| T-128 | boundary | 境界値分析(blackbox-partition)、テストダブル(test-doubles) | `mismatch_questions_take_cap_before_natural`、`mismatch_questions_leave_trial_budget` | #23、#30、#33 | [x] | [ ] | 要 |
| T-129 | boundary | 境界値分析(blackbox-partition)、同値分割(blackbox-partition)、テストダブル(test-doubles) | `guesses_across_matching_screens_merged_trimmed` | #20、#33 | [x] | [ ] | 要 |
| T-130 | event | 同値分割(blackbox-partition)、テストダブル(test-doubles) | `mismatch_issue_also_gets_natural_question` | #23、#33 | [x] | [ ] | 要 |
| T-131 | event | 同値分割(blackbox-partition)、境界値分析(blackbox-partition) | `mismatch_issue_counts_trials_reaching_pattern`、`mismatch_screenshot_from_first_matching_observation` | #22、#33 | [x] | [ ] | 要 |
| T-132 | boundary | 境界値分析(blackbox-partition) | `hard_to_see_cut_to_the_first_three` |  | [x] | [ ] | — |
| T-133 | event | 同値分割(blackbox-partition)、非決定的出力システムのテスト(ai-nondeterministic) | `hard_to_see_required_with_image_support`、`interview_passes_on_a_malformed_reply_when_hard_to_see_is_not_an_option` |  | [x] | [ ] | — |
| T-135 | event | 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output)、テストダブル(test-doubles) | `expectation_question_names_the_operation_and_the_element`、`an_operation_on_an_element_names_it`、`the_element_passed_over_for_being_unmoved_is_the_one_named`、`expectation_prompt_snapshot`、`second_call_asks_only_the_expectation_of_the_chosen_operation`、`second_call_names_the_element_the_core_picked`、`each_call_is_logged_and_recorded_in_order` | #79 | [x] | [ ] | — |
| T-136 | unwanted | 同値分割(blackbox-partition)、テストダブル(test-doubles) | `nothing_is_chosen_when_the_core_carries_out_nothing`、`expectation_is_empty_when_it_was_not_asked`、`an_empty_expectation_leaves_no_parentheses`、`second_call_is_not_made_when_no_operation_is_carried_out` | #79 | [x] | [ ] | 要 |
| T-137 | unwanted | 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) | `missing_expectation_is_unreadable_after_two_calls`、`expectation_alone_is_read_as_written` | #79 | [x] | [ ] | 要 |

## 対応する unwanted 行

異常系は同じ行の中でなく、別の行が受け持つ。event/state/optional 行ごとの対応は次のとおり。

- T-001(ペルソナの読み込み): T-002(必須の項目の欠け)、T-003(書式の崩れ)
- T-004、T-055(書いた目的の done_when): T-056(確定のボタンが禁止要素)、T-058(done_when を満たさない done)、T-115(宣言が無い done_when)
- T-005(推測した目的を書く): T-006(目的のずれ)
- T-006、T-131(目的のずれの課題とその項目): T-008(書いていない画面)、T-127(答えが使えない)
- T-007、T-129(同じと答えたら出さない、複数の画面の推測をまとめて聞く): T-006、T-124(届いた試行が無い)、T-125(打ち切りの試行だけ)、T-126(使える推測が無い)、T-127(答えが使えない)、T-128(上限を超える)
- T-008(画面ごとの目的を書いていない画面): T-006、T-009(URL の型の書式)
- T-011(モード別の行き詰まりの閾値): T-010(範囲の外の設定)
- T-013、T-014、T-015(ブラウザの開き直し、初期化のコマンド): T-016(失敗)、T-017(シェルの注入)、T-018(API キー)、T-019(出力の伏せ字)
- T-020、T-021、T-030(試行ごとに初期状態から): T-123(ほかのファイルを書かない)
- T-022、T-026、T-028(実行全体の窓と次の試行の判定): T-023(初期化のコマンドのアクセスは数えない)、T-025(上限の半分を下回る)、T-029(上限を小さくした試行が上限に達する)
- T-031、T-032、T-034、T-037、T-040(待ちと期限と中断): T-035(期限の後の待ち時間切れ)、T-036(打ち切りの手がかり)、T-038(打ち切りを数えない)、T-039(終えた試行が0回)
- T-042(既存のモードはテキストだけ): T-119(既存の受け入れシナリオが通る)。T-042 は既存のモードの振る舞いを変えないことの確認で、その異常系は既存のモードの受け入れの回帰が受け持つ
- T-041、T-044、T-045(画像の受け渡しと申告): T-042(既存のモードはテキストだけ)、T-046(スクリーンショットを伏せない)、T-070〜T-073(画像を扱えない申告)
- T-047〜T-050、T-053、T-054(問いの中身): T-051(足した答えの欠け)、T-052(範囲の外)、T-122(伏せ字)
- T-135(期待の問い): T-136(操作を行わないステップでは問わない)、T-137(期待の答えの欠け)
- T-057、T-059、T-060(終わり方と数え方): T-058、T-061(数えない終わり方)
- T-064、T-065(迷いの手がかり): T-062、T-063(閾値の境界)、T-066(行き来で行き詰まりにしない)
- T-133(揺れた答えの読み): T-073(一覧に無い番号と答えそのものの欠けは応答の異常)
- T-067、T-069(視認性と目的に届かない課題): T-068(挙げなければ出さない)、T-070
- T-074、T-076、T-130(自然さの問い、ずれの課題も自然さの問いにかける): T-075(上限を超える)、T-077(自然でない)、T-078(応答が使えない)、T-128(ずれの問いと上限を分け合う)
- T-095(終了コード): T-016(初期化のコマンドの失敗で3)、T-035(期限の後の待ち時間切れは3でない)
- T-096、T-097(選択): T-098(ほかの中身を変えない)、T-099(外での書き換え)
- T-100、T-101(TUI の一覧と中身): T-102(実行中の画面を変えない)
- T-103、T-104、T-108、T-109、T-113(照らし合わせ): T-105、T-106(届いた数の境界)、T-107(届かない)、T-110(前回の台帳が無い)、T-111(読めない前回の台帳)、T-112(ペルソナか目的の食い違い)
- T-114(形式の検査): T-115(誤りのある文書)

## 受け入れ範囲の外にした要件

抽出台帳のトレーサビリティで受け入れテストの対象外とした要件は、この台帳でも行を立てない。

- S-033 と S-101 の利用者向けの説明に書くこと: 文書のレビューで確かめる。
- S-074(デモのページとペルソナ1つで回し、人が台帳を読んで確かめる): 人の確認で、自動のテストにしない。
- S-075(人の判定と自動で突き合わせる評価の仕組みを作らない): 作らないものの宣言である。
- S-080(品質ゲートの基準と設定を変えない): `just check` と品質ゲートの設定の差分で確かめる。
- S-081 のうち、判断モデルの抽象をほかに変えていないこと: コードレビューで確かめる。

## 割り当てで迷った行

- T-006、T-007、T-124〜T-131: 目的のずれの判定は、親の判断(抽出台帳の S-021、S-059、S-102、S-103、設計メモ6)で決まった。文字列の一致では決めず、試行をすべて終えたあと、画面ごとの目的の1項目(URL の型1つ)につき1回だけ、その型に合う画面で終えた試行に出た推測を全部(前後の空白を除いて重複をまとめる)と書いた目的を判断モデルに渡して聞く。デシジョンテーブルの条件は「画面ごとの目的を書いたか」×「ずれの問いの答え(同じ、同じでない、使えない)」にし、書いていない画面(T-008)、同じ(T-007)、同じでない(T-006)、使えない(T-127)を各1ルールにした。問いを聞かない条件(届いた試行が無い、打ち切りだけ、使える推測が無い)は T-124〜T-126、推測のまとめ方の境界(同じ型に合う複数の画面、空白だけ違う推測)は T-129 が受け持つ。判断モデルの答えは台本どおりに答えるスタブで与えるので、テストダブルを足した。問いを組み立てる側は `interview::ask`、答えを検証する側は `interview::answer`、課題にする側は `interview::issue`、上限と自然さの問いへのかけ方(T-128、T-130)は `interview::natural` に置いた。目的のずれの課題の番号(URL の型、種類)は T-088 に足した。目的のずれの課題の項目(画面は URL の型、N と届いた試行の数はその型に届いた終えた試行の数、k = N、スクリーンショットはその型に最初に合った観測)は T-131 が受け持ち、その決め方に合わせて T-006(画面が URL の型)、T-081(要素の無い項目)、T-084(k = N の割合での並び)、T-089(最初に合った観測の試行とステップからの名前)、T-105(届いた試行の数を型で数える照らし合わせ)にテストを1つずつ足した。最も多く出た推測を選ぶ規則と同数の規則は判断から無くなったので、そのテストは「推測をまとめて1回だけ聞く」`mismatch_question_passes_all_guesses_once` に置き換えた。【推測】に依存する行はもう無く、`[~]` の行は無い。T-007 は「同じと答えたら出さない」という正常な分岐なので、種別を event にした
- T-026: 判定が初期化のコマンドの前に来ることは単体で書けるが、課題の指示に沿って性質テストも割り当てた。抽出台帳の5つの性質には当たる性質が無いので、`no_reset_while_waiting_for_window` を足した。
- T-030: 窓を待ってから始めた試行の中核の窓が0から数えることは、中核の初期状態の話で、試行の外のループの性質ではないと判断し、性質テストを割り当てなかった。
- T-055、T-120、T-121: 既存のモードと共有する中核の道筋なので、既存の台帳と同じ単体のテスト名を書いた。新しい単体のテストは足さず、受け入れの層だけを足す。既存のテストは WebExploratoryQA 台帳の T-015(T-055)、T-007、T-005、T-008(T-120)、T-020(T-121)の行で、まだ実装していないので、この3行はそれらの実装に依存する。各行のテスト名の欄にも書いた。
- T-116〜T-118: 手引きは配布物の Markdown なので、Rust のテストで読むか、文書の検査(`just check`)に載せるかで迷った。受け入れの Scenario がすでにファイルを読むので、下の層も `webqa/tests/guide.rs` に置き、チェックリストベーステストにした。手引きの置き場所は `share/skills/` で確定した。
- T-017、T-018: シェルを通さないことと環境変数を渡さないことは、実際の子プロセスで確かめるしかないので、`webqa-judge/tests/command.rs` にならって `webqa/tests/reset.rs` を契約の層として足した。
- T-119: 既存の feature を回すこと自体が振る舞いなので、テスト名は `acceptance.rs` が既存の feature を走らせる単位(`web_exploratory_qa_feature`)にした。cucumber の harness の中の名前の付け方は実装で決める。

## 閉じる前のチェック(設計用: 0段を閉じる条件)

- [x] 用語定義表が埋まり、曖昧な語が残っていない(目的のずれは、URL の型ごとのずれの問いの答えで決めると用語定義に書いた。同数の境界は無くなり、同じ型に合う複数の画面と空白だけ違う推測を境界にした)
- [x] 全 T-ID に種別ラベルが振られている(証跡は下の「機械確認」)
- [x] 各 `event`/`state` 行に対応する `unwanted` 行が最低1つある(「対応する unwanted 行」の節)
- [x] 全 T-ID の「設計済」が `[x]`(133 行。証跡は下の「機械確認」)
- [x] T-001 から連番に欠番なし(証跡は下の「機械確認」)
- [x] 各 T-ID に手法が割り当たっている(台帳の各行の「手法」欄。割り当てた手法の技法スキルはすべて開いて参照した)
- [x] 正常系だけでなく、境界と異常系が各機能(Rule)に最低1つある(証跡は下の「機械確認」の Rule 別集計)。正常系は event/state/ubiquitous/optional、境界は boundary、異常系は unwanted と数える。非機能(セキュリティ、負荷)は拒否・抑止の振る舞いとして unwanted で立てた。4つの Rule は境界の行を持たない。試行の終わり方は終了理由ごとの分類で順序のある入力が無く、done_when と判断予算の境界は既存の台帳の行が受け持つ。課題を選ぶことと TUI は選択の真偽と画面の状態だけで境界が無い。手引きは文書の形の確認で境界が無い。既存のモードとの関係の境界は既存の台帳の行が受け持つ

### 機械確認

台帳と feature の突き合わせは、リポジトリの直下で次のコマンドを実行して確かめた。出力はそのまま貼る。

```console
$ python3 tasks/test-design/tools/check_ledger.py InterviewMode
feature の sha256: 1ae06b5206c10a33ca45fced4577c0e7d64e373013e2728f2b4697e03fa9e92f
台帳の sha256:     1ae06b5206c10a33ca45fced4577c0e7d64e373013e2728f2b4697e03fa9e92f
`- [ ]` の行: 0
台帳の T-ID 行: 133(期待 133)
欠番: なし
feature の Scenario: 133
台帳から指されない Scenario: なし
`[~]` の行: 0(なし)
冒頭の直し待ちの一覧: 0
仕様待ちの issue: なし
マトリクスの行: 133
Rule 別の種別の内訳:
  ペルソナと目的の入力: boundary 4、event 6、optional 1、state 1、unwanted 7
  試行: boundary 3、event 7、optional 1、state 6、ubiquitous 5、unwanted 7
  判断モデルへの問い: boundary 1、event 6、state 4、ubiquitous 2、unwanted 1
  試行の終わり方: event 4、unwanted 3
  課題の見つけ方: boundary 3、event 5、unwanted 6
  ペルソナから外れないこと: boundary 1、event 2、ubiquitous 1、unwanted 2
  台帳の書式: boundary 2、event 1、ubiquitous 8、unwanted 5
  課題を選ぶことと TUI: event 3、state 1、ubiquitous 1、unwanted 2
  再実行と照らし合わせ: boundary 3、event 5、unwanted 3
  手引きと形式の検査: event 1、ubiquitous 3、unwanted 1
  既存のモードとの関係: ubiquitous 2、unwanted 3
OK
```

照合スクリプトは `tasks/test-design/tools/check_ledger.py` を一般化したもので、台帳の名前を引数で選ぶ。issue の欄は、この台帳だけ空を許し、WebExploratoryQA では今までどおり空を NG にする。
比べた版は、この台帳を作り始めたときの作業ツリーの `check_ledger.py`(コミットされていない変更を含む版)である。その版をこのセッションの最初に実行した出力を保存しておき、一般化の差分を逆に当てて戻した版がその出力を再現することを確かめた。一般化の差分は次のとおり(`<` が比べた版、`>` が今の版)。

```console
$ diff <比べた版>/check_ledger.py tasks/test-design/tools/check_ledger.py
1c1
< """テスト項目台帳 WebExploratoryQA を、転記元の feature と突き合わせる。
---
> """テスト項目台帳を、転記元の feature と突き合わせる。
3c3,5
< 使い方(リポジトリの直下で): python3 tasks/test-design/tools/check_ledger.py
---
> 使い方(リポジトリの直下で): python3 tasks/test-design/tools/check_ledger.py [台帳の名前]
> 台帳の名前は LEDGERS のキー。省けば WebExploratoryQA を検査する。
> 台帳は tasks/test-design/<名前>.md、feature は tasks/loopeng/<名前>.feature に置く。
14,16c16,21
< LEDGER = Path("tasks/test-design/WebExploratoryQA.md")
< FEATURE = Path("tasks/loopeng/WebExploratoryQA.feature")
< EXPECTED = 111
---
> # 名前: (Scenario の数, issue の欄が空でもよいか)
> LEDGERS = {"WebExploratoryQA": (111, False), "InterviewMode": (131, True)}
> NAME = sys.argv[1] if len(sys.argv) > 1 else "WebExploratoryQA"
> EXPECTED, ISSUE_OPTIONAL = LEDGERS[NAME]
> LEDGER = Path(f"tasks/test-design/{NAME}.md")
> FEATURE = Path(f"tasks/loopeng/{NAME}.feature")
19c24
<     r"^- \[(.)\] (T-\d{3}) WebExploratoryQA\.feature:(\d+)\((.+?)\) \[(\w+)\] 「(.+)」(.*)$"
---
>     rf"^- \[(.)\] (T-\d{{3}}) {re.escape(NAME)}\.feature:(\d+)\((.+?)\) \[(\w+)\] 「(.+)」(.*)$"
21c26
< DETAIL = re.compile(r"^      → 手法: (.+?) / テスト名: (.+?) / issue: (.*)$")
---
> DETAIL = re.compile(r"^      → 手法: (.+?) / テスト名: (.+?) / issue: ?(.*)$")
97c102
<         if not ISSUE.match(d.group(3)):
---
>         if not (ISSUE.match(d.group(3)) or (ISSUE_OPTIONAL and not d.group(3))):
```

引数を省いたときの WebExploratoryQA の出力は、比べた版と今の版で同じだった。

```console
$ diff <(python3 <比べた版>/check_ledger.py) <(python3 tasks/test-design/tools/check_ledger.py); echo $?
0
```

わざと壊して NG になることも確かめた。T-046 の題名を変え、T-086 の手法からスキル名を消し、T-090 を `[ ]` にし、T-126 の行を消し、T-007 の種別を変えてから実行した出力(抜粋)は次のとおり。

```console
$ python3 tasks/test-design/tools/check_ledger.py InterviewMode
`- [ ]` の行: 1
台帳の T-ID 行: 130(期待 131)
欠番: [126]
台帳から指されない Scenario: [105]
NG `- [ ]` の行が残っている
NG `[x]` でも `[~]` でもない行: ['T-090']
NG 行数か欠番
NG T-046: 題名が違う: 'スクリーンショットは伏せずに渡す'
NG Scenario の数か、指されない Scenario
NG T-086: 手法に参照したスキル名が無い: プロパティベーステスト
NG マトリクスと本文の T-ID が違う: ['T-126']
NG T-007: マトリクスの種別が本文と違う: 'event' / 'unwanted'
NG T-086: マトリクスの手法が本文と違う: 'プロパティベーステスト(generative-property)' / 'プロパティベーステスト'
NG 9 件
```

転記元の sha256 の1文字を変え、T-001 の issue の欄に `14` と書いたときの出力(抜粋)は次のとおり。

```console
$ python3 tasks/test-design/tools/check_ledger.py InterviewMode
feature の sha256: 86b7c35c1bac7e3beb5c023ef3bd00a0caeb30d85d12521b307ecf0444e20a81
台帳の sha256:     86b7c35c1bac7e3beb5c023ef3bd00a0caeb30d85d12521b307ecf0444e20a81
NG feature の sha256 が台帳の値と違う
NG T-001: issue の欄が空か形が違う: '14'
NG T-001: マトリクスのissueが本文と違う: '' / '14'
NG 3 件
```

WebExploratoryQA の台帳で T-001 の issue の欄を空にしたときの出力(抜粋)は次のとおり。

```console
$ python3 tasks/test-design/tools/check_ledger.py
NG T-001: issue の欄が空か形が違う: ''
NG T-001: マトリクスのissueが本文と違う: '#6' / ''
NG 2 件
```

確かめたあと、どちらの台帳も壊す前の版に戻し(`cmp` で一致を確認)、上の OK の出力はこの戻した版で取った。
