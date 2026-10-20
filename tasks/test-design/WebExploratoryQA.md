# テスト項目抽出台帳: WebExploratoryQA

対象: Web サービスの探索的テストエージェント(判断モデルの問い合わせ、ガードレール、ブラウザ操作、終了判定、段階、探索の記憶、負荷制御、問題の記録、設定、中断、終了コード)
走査アンカー: 受け入れ条件(`tasks/loopeng/WebExploratoryQA.feature` の Scenario 1件 = T-ID 1行で転記)
テスト対象: crates/*
転記元の版: コミット f830894 の `tasks/loopeng/WebExploratoryQA.feature`(確定)に、issue #4 の決定(許可リストは禁止より優先する)と issue #62〜#64 の決定(探索モードで対象に合わない操作を外し、記憶にも数えない)と issue #70 の決定(キーを押す操作 enter を足す)を反映した版。出典欄の行番号はこの版の行を指す。
転記元の sha256: 8994b2aac0b29a0913a57ee8975f560259223155b968a4b994b15b42384538f2
内ループで feature が変わったら、台帳の行を追従させてから、この sha256 を新しい版の値に書き換える。`tasks/test-design/tools/check_ledger.py` は、この値と実物の feature の sha256 が違えば失敗する。

各 ID は「テストすべき振る舞い」1つ。test-catalog で全行に手法とテスト名を割り当てた。
チェックボックスの意味は、ここで1回だけ定義する。台帳の行、マトリクスの「設計済」列、閉じる前のチェックで同じ意味に使う。
- `[ ]` 未着手、または未確認。
- `[~]` 部分完了。何が残るかを同じ項目に「残り:」で書く。T-ID の行では、手法とテスト名は割り当て済みで、仕様の直しで何が変わりうるかを `→ 残り:` に書く。
- `[x]` 完了。T-ID の行では設計済、すなわち手法とテスト名が決まり、仕様の直しを待たないことを表す。テストが緑になったことは表さない。緑の確認は、トレーサビリティマトリクスの「緑確認済」列で test-verify が消し込む。

手法の欄には、手法名と、割り当てるときに開いた技法スキルの名前を括弧で書いた。
テストの置き場所は `docs/architecture.md` の「テストの層」と「ディレクトリ構成」に従う。
どの行も受け入れの層を持つ。`webqa/tests/acceptance.rs` が cucumber で、題名が同じ Scenario を走らせる。
テスト名の欄には、受け入れのほかに、その振る舞いを受け持つ下の層のテストを書いた。単体は `crate::モジュール` の形で、ほかの層はテストのファイルで場所を示す。
テスト関数の名前には T-ID などの管理番号を入れない。
issue の欄は、その行を実装する GitHub の issue の番号である。受け入れの配線は全行を #14 が受け持つので、欄には書かない。T-050 の受け入れだけは、部品の噛み合わせを先に確かめるため #15 で先に配線する。
仕様の直しで変わりうる行は `[~]` にする。今は該当する行が無い。
直し待ちの行: なし
仕様待ちの issue: なし
T-015、T-021、T-055、T-079、T-081〜T-092 の16行は、TLA+ の仕様の直しを待つ `[~]` として issue #13 にまとめていた。v5 の仕様(抽出台帳の設計メモ11)で直しが済んだので `[x]` に戻した。issue は #13 のままで、#13 はもう仕様を待たない。
v5 で決まったのは2点である。決め打ちシナリオでは、実行の開始時と段階を進めた直後にも done_when を確かめ、すでに満たしていれば操作せずに進む。探索モードでは、判断モデルに1回以上問い合わせてからゴール達成を判定する。16行への影響は次のとおりで、手法とテスト名はどの行も変えていない。
- T-081、T-082: 探索モードのゴールに「1回以上問い合わせた後」が加わったので、Scenario の前提に「判断モデルに1回以上問い合わせている」を足した。単体 `webqa-core::explore::graph` は探索し終えたかだけを判定し、問い合わせの回数と組み合わせる判定は T-108、T-109 の `webqa-core::plan` が受け持つ。
- T-086: 段階を進めた直後に次の段階の done_when を確かめるので、Scenario の前提に「段階 "投稿" の done_when 条件は満たされていない」を足した。満たしている場合は T-111 が受け持つ。T-089〜T-092 は、feature 冒頭の「done_when が満たされていると書いていない段階は、入った時点で満たしていない」の決まりで前提を補う。
- T-015、T-021、T-079、T-087: done_when を確かめる時点は操作の後、実行の開始時、段階を進めた直後の3つになった。どの時点でも、確かめた結果を問い合わせより先に採る優先順位は変わらない。開始時と進めた直後の振る舞いは T-110、T-111 が受け持つ。
- T-055: 扱いは下の v6 の段落にまとめた。
- T-083〜T-085、T-088: v5 の2点はこれらの判定を変えない。

v6 の仕様では、操作可能な要素が1つも無い画面を表せるように、判断モデルが返しうる対象を直した。対象を取らない操作(scroll、back、wait など)の対象は「対象なし」にする。click と input は、要素が1つも無い画面では対象が存在しない要素番号になり、応答の異常として扱う。これで要素0件の画面でも問い合わせが起きる。影響する行は次のとおりで、手法とテスト名はどの行も変えていない。
- T-055: Scenario の前提に「判断モデルにまだ問い合わせていない」を足した。要素0件の画面で問い合わせ、対象を取らない scroll_down を実行することが、v6 の問い合わせの形と一致する。
- T-056: Examples に「操作可能な要素が1つも無い画面で返った click」を足した。v6 では、存在しない要素番号への click と同じ応答の異常になる。
- T-108、T-109: 要素0件の探索モードでも問い合わせが起きるので、1回問い合わせてからゴール達成になる。T-109 の Examples の「形式の崩れた応答による応答の異常」の行は、1回目の問い合わせが応答の異常でも問い合わせ1回に数えることを確かめる。
Scenario Outline は1行にまとめ、Examples の各行をそのテストのケースにする。
T-ID は一度振ったら変えない。T-052 以降は、レビューで足したシナリオを番号を保つために末尾へ追加したもの。T-060 は題名を言い直した(応答しないステップは固まりに合流させる)。
T-072 以降は、2026-10-18 の要件変更(行き詰まりの数え直し、時間切れ、利用者による中断、探索モードのゴール、段階の切り替え)と、要素の範囲・終了コードの決定に合わせて足したもの。
T-018 と T-058 は、行き詰まりの条件に「未操作の要素も新しく見えない」を足して題名を言い直した。
T-108 以降は、v5 の仕様(設計メモ11)で決まった、探索してよい要素が無い探索モードのゴールと、実行の開始時・段階を進めた直後の done_when に合わせて足したもの。
T-110、T-111 の単体は `webqa-core::machine::actions` に置く。段階を進める効果は actions の表の行で、どの条件を先に採るかの優先順位は `webqa-core::plan` が持つからである。
T-108 の全状態の探索は、候補の画面に「操作可能な要素が0件」と「操作可能な要素が禁止要素だけ」の2つを含める。どちらも探索してよい要素が空になる画面で、これが無いと問い合わせの前にゴール達成にしないことを確かめられない。
T-112 は、探索モードで使う操作種別に click も input も無い(done だけの場合も含む)設定を、設定エラーにする行である。要素を操作できないと探索が進まないためで、issue #3(設定の読み込みと検証)の追加の直しとして足した。実行のモードは設定でなく CLI のサブコマンドで決まるので、単体の `explore_mode_needs_click_or_input` は、検証済みの設定とモードを引数に取る中核の関数を確かめる。CLI が起動時にこの関数を呼ぶことは、T-044 の実行ファイルのテスト `explore_without_click_or_input_exits_two`(終了コード2)が確かめる。
T-113 は、issue #4 で利用者が「許可リストは禁止より優先する」と決めたので足した。それまで T-007 の Examples にあった「許可リストにも一致するが名前が "購入" を含むボタン」の行を、操作する側の Scenario に移したもので、feature の行番号がずれた出典をあわせて直した。
シナリオモードでも、何も起きなかった input は記憶に数えず、値の無い入力欄への input は skipped(no_input_value)で記録する。T-114 は、設定の操作種別が done を除くと input だけなら input を残す例外を、単体テストで確かめる。T-114、T-115 は、issue #62〜#64 で利用者が「探索モードでは、対象に合わない操作を選択肢から外し、記憶にも数えない」と決めたので足した。T-084 の Scenario は、入力欄が無い開始の画面で、選択肢が input を除く6種になるように直した。issue #70 で足した enter も、表示中に値のある入力欄が無ければ外れるので、この Scenario の6種は変わらない。値の設定された入力欄があるかは、設定の inputs の名前と、入力欄の役割で決める。 T-116 は、打てた input の Scenario を T-115 の Outline から分けたもので、打てた input は skipped を記録せず探索済みに数えることを確かめる。

## 用語定義(抽出より先に埋める)

| 用語 | 定義(曖昧さを残さない) | この定義が生む境界と異常系 |
|------|--------------------------|----------------------------|
| 1ステップ | 終了条件の確認 → 判断モデルへの問い合わせ → ガードレール → 実行 → 画面の観測の一巡。終了条件の確認は問い合わせより先 | 終了・固まり・ドメイン外のステップでは問い合わせない |
| 終了条件の優先順位 | done_when の成立(最後の段階ならゴール達成、そうでなければ次の段階へ進む。探索モードでは探索モードのゴール)> 行き詰まり > 判断予算切れ・時間切れ > 固まり復旧・ドメイン復帰・時間窓の満了 > 問い合わせ。3段目の2つは同じ段で、両方が成り立てばどちらを採ってもよい。4段目の3つは順不同で、成り立つものを1つずつ処理し、どの順でもよい。テストは3段目と4段目の中の順序を期待値にしない。利用者による中断と時刻の経過はこの順序の外で、いつでも起こりうる | 条件が重なったときに採る1つ、判断予算切れと時間切れの同時成立 |
| 画面が変化する | ブラウザ操作の前後で、判断モデルに送るアクセシビリティ情報のテキスト表現(操作可能要素の一覧と各要素の role・名前・値)か URL の少なくとも一方が1文字でも違うこと。操作可能要素の一覧は表示領域内の要素だけを含むので、スクロールで一覧が変われば変化になる。スクリーンショットの差分は使わない。ブラウザ操作が起きなかったステップ(ガードレールで止めた、応答の異常)では比べず、固まりにも行き詰まりにも数えない | 値だけの変化、URL だけの変化、スクロールで一覧が変わる |
| 固まり | 画面が変化しないステップが閾値回続いた状態。画面が変化すれば数え直す | 閾値ちょうど、閾値-1、変化による数え直し |
| ページが応答しない | 操作またはページの読み込みが設定の待ち時間内に終わらないこと。「画面が変化しなかった」ステップ1回として固まりに数え、閾値回続いたら開き直す。単発では開き直さない。click/input の対象は、応答しなくても操作したので探索済みに加える | 待ち時間の設定値 0、閾値回の連続 |
| 行き詰まり | 探索済みの要素の集合が増えず、かつ未操作の要素も新しく見えない操作が閾値回続いた状態。どちらかが起きれば数え直す。判断モデルの stuck 応答でも成立 | 閾値ちょうど、閾値-1、新しく操作した要素による数え直し、スクロールで新しく見えた要素による数え直し、離脱操作の繰り返し |
| 新しく見えた要素 | 今の画面でまだ表示領域に現れていなかった未操作の要素が、操作の結果として表示領域に現れたもの。表示領域の中かどうかは要素の位置と大きさで判定する。すでに一度表示領域に現れた要素、探索済みの要素、許可ドメインの外のページの要素は数えない | スクロールで見えた要素がすでに見たものだけ、ドメイン外のページ |
| 要素の範囲 | 画面の識別(探索の記憶のグラフの節)はページ全体の操作可能要素で行う。判断モデルに送る一覧、画面が変化したかの判定、新しく見えた要素の判定は、表示領域の中の操作可能要素で行う。done_when の要素とテキストはページ全体で判定する。スクロールしても同じ画面のままだが、表示領域の一覧が変わればスクロールは画面の変化と数える | 表示領域の外の要素、ページの一番下でのスクロール、表示領域の外にある done_when の見出し |
| 操作済みの要素 / 探索済みの要素 | 同じもの。click/input で操作した要素の集合で、探索の記憶として持つ。scroll/back/wait では増えない。固まり復旧でも消えない | 復旧の前後、実行またぎ |
| 未操作の要素 | 現在の画面の操作可能要素のうち、探索済みでないもの。禁止要素は未操作のまま残るが、探索の候補には入れない | 0件、未操作が禁止要素だけ |
| 未探索の分岐が残る / 探索し終えた | 探索済みの要素のうち、遷移先から辿れる画面(探索の記憶のグラフで到達できる画面すべて)に、禁止要素以外の未操作の要素が残っているものを「未探索の分岐が残る」という。残っていないものが「探索し終えた」要素。遷移の無い要素は操作した時点で探索し終えたとみなす | 途中のボタン、遷移先の画面がさらに分岐を持つ |
| 禁止要素 | 名前のキーワード、フォームの送信先、遷移先 URL、設定の禁止リストのいずれかに当たる要素(OR)。ただし設定の許可リストに当たる要素は、どれに当たっても禁止要素にしない | 表記ゆれ、名前の無い画像ボタン、名前に手がかりが無い送信ボタン、許可リストにも当たる禁止のキーワード |
| 決済導線 | 禁止要素のうち、購入・決済・課金に当たるもの。探索の候補からは禁止要素全体を除くので、決済導線も除かれる | 未操作の要素が決済導線だけ |
| 課金画面 | 課金を促す語、`/pricing`・`/upgrade`・`/checkout` 系 URL、主要操作を塞ぐモーダルのいずれか | 軸ごとの単独陽性、料金という語だけの通常ページ |
| 既定の離脱操作 | 設定で指定する、課金画面から抜ける操作。指定が無ければブラウザの「戻る」 | 離脱しても課金画面が残る |
| 低確信度 | 判断モデルの確率の最大値が閾値(既定0.7、設定可)未満、または確率が欠けている | 0.69 / 0.7 / 0.71、閾値の変更 |
| 応答の異常 | 判断モデルの応答が使えないこと。存在しない要素番号、問い合わせた選択肢に無い操作種別(9種以外、探索モードでの done)、形式の崩れ、API のエラー、タイムアウト(判断モデルの応答の待ち時間を過ぎること。ページの待ち時間とは別の設定値)。操作せずに記録し、呼び出しは1回に数える | 各種の異常、連続したときは判断予算切れで止まる(T-069) |
| エラー表示の判定閾値 | 判断モデルのエラー表示ありの確信度がこの値以上なら問題として記録する(既定0.8、設定可) | 閾値ちょうど、閾値未満 |
| 許可ドメイン | 最終的な遷移先(リダイレクト後)のオリジン(スキーム・ホスト・ポート)が許可リストと完全一致。サブドメインはワイルドカード指定時だけ | 似た名前のホスト、スキーム・ポート違い、`javascript:`/`data:`、リダイレクト |
| 要素の識別子 | role + 名前 + 位置。実行直前に取り直し、一意に見つからないか、問い合わせ時と別のDOMノードに入れ替わっていれば中止 | 消える、差し替わる、同名の位置入れ替え |
| 時間窓 | 実行開始の時刻から1時間ごとに区切る窓。その窓で対象サーバへ行ったアクセスの回数を上限で抑える。数えるのはブラウザ操作、課金画面の離脱、固まり復旧の再読み込み、許可ドメインへの戻り。ガードレールで止めた提案と判断モデルへの問い合わせは数えない。上限に達している間は、再読み込み・戻り・離脱も窓が明けるまで待つ。上限に達していなくても時刻で満了する | 上限ちょうど、窓の切り替わり、再読み込み・戻りの計上、却下した提案の非計上 |
| 判断予算 | 1回の実行で判断モデルを呼べる回数の上限。応答の異常も1回に数える | 上限ちょうど、上限-1、他の終了条件との重なり |
| 未検証の終了 | 決め打ちシナリオで、判断モデルが done を返したが今の段階の done_when を満たしていない終了。ゴール達成とは別に記録する。探索モードでは done を選択肢に入れないので起きない | 自己申告だけの done、探索モードで返った done |
| 段階 | 決め打ちシナリオを順に進める区切り。各段階は done_when を持つ。done_when の宣言(URL のパス、要素、テキスト)が複数あれば、すべて成り立ったときに満たしたとみなす。最後でない段階で満たしたら、判断モデルに問い合わせずに次の段階へ進み、固まりと行き詰まりの回数を0に戻す。最後の段階で満たしたらゴール達成。判断予算、時間窓、探索の記憶は実行全体で共有し、段階を進めても戻さない | 最後の段階と最後でない段階、宣言の一部だけが成り立つ、後の段階の done_when だけが成り立つ |
| 探索モードのゴール | 探索モードで、開始の画面から探索の記憶のグラフで辿れるすべての画面について、禁止要素を除くすべての要素を探索し終えたこと。成り立てばゴール達成で終了する。探索モードでは判断モデルに done を選ばせない | 未操作の要素が禁止要素だけ、ほかの画面に未操作の要素が残る |
| 時間切れ | 制限時間に達したときの終了理由。制限時間は探索モードの指定時間、または決め打ちシナリオの実行時間の上限で、実行の開始からの経過時間で測る。達したら(経過時間が制限時間以上なら)判断モデルに問い合わせずに終了する。判断予算切れと同じ段に置く。時間窓を待つのは、窓の始まりと制限時間の早いほうまで | 制限時間の手前、ちょうど、判断予算切れとの同時成立、時間窓の待ちの途中 |
| 利用者による中断 | 利用者がいつでも実行を中断できること。中断したら、今の待ちを打ち切り、探索の記憶・終了のイベント・実行結果を書いてから、終了理由を利用者による中断にして終わる。終了後は状態を変えない。書いている間に2回目の中断を受けたら、書き終わりを待たずに終わる | 待機中、操作の途中、終了後の中断、2回目の中断 |
| 終了コード | 0 は終了理由を記録して終わり確定の問題を見つけなかった、1 は確定の問題を見つけた、2 は設定エラーで起動しなかった、3 は実行時エラー(記録の書き込み失敗、2回目の中断で記録を書き終えずに終わった)で止まった。確定の問題はエラー表示、読み上げ名、コンソールエラー、許可ドメインの外への遷移で、参考情報は数えない。中断で終わったときも問題の有無で0か1 | 参考情報だけ、中断、書き込みの失敗 |
| 参考情報 | 判断モデルだけが根拠の検出。確定した不具合とは別の区分で記録する | わかりにくい表示などの意味判断 |
| 探索の記憶 | 対象(設定の対象 URL のオリジン)ごとに、画面と操作による遷移を実行をまたいで保存したもの | 初回、壊れたファイル、別の対象 |
| 機密情報を伏せた出力の再実行 | 実行結果の出力では、伏せた値の代わりに設定の伏せる対象の名前を書く。再実行するときは同じ設定から値を補う | —(再実行は同じ設定で起動するので、設定の誤りは起動時の設定エラーの行 T-049、T-066〜T-068 が受け持つ) |
| 当たり前要求 | 機密情報を判断モデル・出力・ログへ出さない、API キーを設定ファイルに書かない、TLS 検証を無効にしない、設定値の 0 以下と空を拒否する。スクリーンショットは画面をそのまま残すので伏せない(テスト環境での利用が前提。抽出台帳の STRIDE (2) の I 行と同じ判断) | 各セキュリティ・設定エラーの行 |

## 台帳

- [x] T-001 WebExploratoryQA.feature:19(1ステップの問い合わせと実行) [event] 「判断モデルに送る画面状態はアクセシビリティ情報のテキストで、操作可能要素の一覧を含む」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::screen` の `screen_text_lists_role_name_value` / issue: #6
- [x] T-002 WebExploratoryQA.feature:25(1ステップの問い合わせと実行) [event] 「1ステップで操作種別・対象要素・エラー表示の有無・ゴール達成の有無を問い合わせる」
      → 手法: テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::judge` の `step_asks_op_target_error_goal` / issue: #7
- [x] T-003 WebExploratoryQA.feature:35(1ステップの問い合わせと実行) [event] 「ガードレールを通った操作はそのままブラウザで実行する」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition)、テストダブル(test-doubles)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `passed_proposal_becomes_touch`、契約 `webqa-browser/tests/contract.rs` の `performs_each_operation` / issue: #9、#12
- [x] T-004 WebExploratoryQA.feature:51(1ステップの問い合わせと実行) [unwanted] 「画面が変わらないのに同じ要素を選んだら、次に確信度の高い候補へ切り替える」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::judge` の `same_unchanged_target_switches_to_next` / issue: #7
- [x] T-005 WebExploratoryQA.feature:91(ガードレール) [boundary] 「確信度の低い「戻る」は実行しない」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::confidence` の `back_confidence_boundary` / issue: #4
- [x] T-006 WebExploratoryQA.feature:105(ガードレール) [unwanted] 「確信度が欠けた応答の「戻る」は低確信度として扱い、実行しない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::confidence` の `back_without_probability_is_low` / issue: #4
- [x] T-007 WebExploratoryQA.feature:110(ガードレール) [unwanted] 「購入・削除・退会・サインインに当たる要素は操作しない」(Examples の各行を1ケースとする)
      → 手法: ゴールデンテスト(oracle-past-output)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::forbidden` の `forbidden_by_keyword`、ゴールデン `webqa-core/tests/guard_golden.rs` の `forbidden_fixtures` / issue: #4
- [x] T-008 WebExploratoryQA.feature:146(ガードレール) [unwanted] 「実行の直前に対象要素が無くなっていたら操作を中止する」
      → 手法: 状態遷移テスト(blackbox-state)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::identity` の `missing_element_aborts`、契約 `webqa-browser/tests/contract.rs` の `removed_element_is_missing_on_refetch` / issue: #4、#9
- [x] T-009 WebExploratoryQA.feature:153(ガードレール) [unwanted] 「名前が同じでも役割や位置が変わった要素は操作しない」
      → 手法: 同値分割(blackbox-partition)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::identity` の `role_or_position_change_aborts`、契約 `webqa-browser/tests/contract.rs` の `replaced_node_gets_new_ref` / issue: #4、#9
- [x] T-010 WebExploratoryQA.feature:160(ガードレール) [unwanted] 「同じ名前のボタンの位置が入れ替わったら操作を中止する」
      → 手法: 同値分割(blackbox-partition)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::identity` の `swapped_same_name_buttons_abort`、契約 `webqa-browser/tests/contract.rs` の `swapped_same_name_buttons_detected` / issue: #4、#9
- [x] T-011 WebExploratoryQA.feature:167(ガードレール) [event] 「位置で一意に決まる同名のボタンは操作する」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::identity` の `same_name_unique_by_position_passes` / issue: #4
- [x] T-012 WebExploratoryQA.feature:173(ガードレール) [unwanted] 「課金画面が出たら判断モデルに問い合わせず、既定の離脱操作をする」(Examples の各行を1ケースとする)
      → 手法: ゴールデンテスト(oracle-past-output)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::paywall` の `paywall_by_each_axis`、ゴールデン `webqa-core/tests/guard_golden.rs` の `paywall_fixtures` / issue: #4
- [x] T-013 WebExploratoryQA.feature:189(ガードレール) [unwanted] 「通常のページを課金画面と取り違えない」
      → 手法: 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::paywall` の `pricing_word_alone_is_not_paywall`、ゴールデン `webqa-core/tests/guard_golden.rs` の `paywall_fixtures` / issue: #4
- [x] T-014 WebExploratoryQA.feature:194(ガードレール) [unwanted] 「課金画面から離脱しても消えないまま3回続いたら、行き詰まりで終了する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `leaving_paywall_counts_toward_stuck` / issue: #12
- [x] T-015 WebExploratoryQA.feature:208(終了の判定) [event] 「done_when を満たしたら、判断モデルに問い合わせずにゴール達成で終了する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `goal_met_finishes_without_asking` / issue: #13
- [x] T-016 WebExploratoryQA.feature:214(終了の判定) [unwanted] 「done_when を満たさないまま判断モデルが done を返したら、未検証の終了として記録する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `self_reported_done_is_unverified` / issue: #12
- [x] T-017 WebExploratoryQA.feature:221(終了の判定) [event] 「判断モデルが stuck を返したら行き詰まりで終了する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `stuck_answer_finishes_stuck` / issue: #12
- [x] T-018 WebExploratoryQA.feature:226(終了の判定) [state] 「探索済みの要素が増えず、未操作の要素も新しく見えない操作が3回続けば、次のステップの前に行き詰まりで終了する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `stuck_after_threshold_without_progress` / issue: #12
- [x] T-019 WebExploratoryQA.feature:234(終了の判定) [state] 「新しい要素を操作すると行き詰まりの数え直しになる」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `operating_untouched_resets_stuck` / issue: #12
- [x] T-020 WebExploratoryQA.feature:256(終了の判定) [unwanted] 「判断モデルの呼び出しが上限に達したら実行を止める」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `budget_exhausted_finishes` / issue: #12
- [x] T-021 WebExploratoryQA.feature:263(終了の判定) [state] 「終了の条件が重なったら、ゴール達成、行き詰まり、判断予算切れの順に1つだけ採る」
      → 手法: デシジョンテーブル(blackbox-partition)、モデルベーステスト(oracle-model-based) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `goal_beats_stuck_beats_budget`、全状態の探索 `webqa-core/tests/exhaustive.rs` の `priority_holds_in_every_reachable_state` / issue: #13
- [x] T-022 WebExploratoryQA.feature:270(終了の判定) [state] 「行き詰まりと判断予算切れが重なったら行き詰まりで終了する」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `stuck_beats_budget` / issue: #12
- [x] T-023 WebExploratoryQA.feature:277(終了の判定) [state] 「固まりと判断予算切れが重なったら、ページを開き直さずに判断予算切れで終了する」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `budget_beats_freeze_recovery` / issue: #12
- [x] T-024 WebExploratoryQA.feature:310(終了の判定) [unwanted] 「終了した実行はそれ以上ステップを進めない」
      → 手法: 禁止仕様からの導出(blackbox-state)、モデルベーステスト(oracle-model-based) / テスト名: 受け入れ(同題の Scenario)、全状態の探索 `webqa-core/tests/exhaustive.rs` の `finished_is_absorbing` / issue: #12
- [x] T-025 WebExploratoryQA.feature:517(固まりとドメイン外からの復帰) [state] 「画面が3回続けて変化しなければ、ページを開き直して探索を続け、探索の記憶は残す」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `reload_after_freeze_keeps_memory` / issue: #12
- [x] T-026 WebExploratoryQA.feature:543(固まりとドメイン外からの復帰) [state] 「画面が変化すれば固まりの数え直しになる」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `screen_change_resets_freeze` / issue: #12
- [x] T-027 WebExploratoryQA.feature:562(固まりとドメイン外からの復帰) [unwanted] 「操作の結果ドメイン外に出たら記録して許可ドメインへ戻り、ドメイン外では問い合わせない」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `outside_domain_returns_without_asking` / issue: #12
- [x] T-028 WebExploratoryQA.feature:571(固まりとドメイン外からの復帰) [boundary] 「許可ドメインかどうかは、最終的な遷移先のオリジンの完全一致で判定する」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition)、境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::domain` の `origin_must_match_exactly` / issue: #4
- [x] T-029 WebExploratoryQA.feature:595(探索の記憶と優先順位) [ubiquitous] 「探索の記憶を次の実行に引き継ぐ」
      → 手法: CRUD/ライフサイクルテスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、保存 `webqa/tests/store.rs` の `memory_round_trips_across_runs` / issue: #14
- [x] T-030 WebExploratoryQA.feature:616(探索の記憶と優先順位) [state] 「未操作の要素を、未探索の分岐が残る操作済み要素より先に選ぶ」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::select` の `untouched_before_unfinished_branch` / issue: #10
- [x] T-031 WebExploratoryQA.feature:622(探索の記憶と優先順位) [state] 「未操作の要素が無ければ、未探索の分岐が残る操作済み要素を戻る・スクロールより先に選ぶ」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::select` の `unfinished_branch_before_back_and_scroll` / issue: #10
- [x] T-032 WebExploratoryQA.feature:628(探索の記憶と優先順位) [state] 「どの要素にも未探索の分岐が残っていなければ、戻るかスクロールを選ぶ」
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::select` の `back_or_scroll_when_all_finished` / issue: #10
- [x] T-033 WebExploratoryQA.feature:634(探索の記憶と優先順位) [unwanted] 「遷移先を探索し終えるまで、途中のボタンを探索し終えた要素として扱わない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::graph` の `finished_follows_reachable_screens` / issue: #10
- [x] T-034 WebExploratoryQA.feature:639(探索の記憶と優先順位) [unwanted] 「探索モードでも決済導線の要素は選ばない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::select` の `payment_elements_never_candidates` / issue: #10
- [x] T-035 WebExploratoryQA.feature:702(対象サーバへの負荷) [unwanted] 「1時間窓の操作回数が上限に達したら、窓が明けるまで問い合わせも操作もしない」
      → 手法: 境界値分析(blackbox-partition)、時刻依存の封じ込め(flakiness-value) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `window_full_waits_for_next_window` / issue: #12
- [x] T-036 WebExploratoryQA.feature:709(対象サーバへの負荷) [state] 「時間窓は時刻で区切られ、上限に達していなくても満了する」
      → 手法: 境界値分析(blackbox-partition)、時刻依存の封じ込め(flakiness-value) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::window` の `window_index_follows_elapsed_time` / issue: #12
- [x] T-037 WebExploratoryQA.feature:750(問題の記録) [event] 「問題を見つけたら、種類とステップ番号を含む名前でスクリーンショットを残す」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::store::screenshots` の `screenshot_name_has_kind_and_step` / issue: #14
- [x] T-038 WebExploratoryQA.feature:763(問題の記録) [unwanted] 「エラー表示の確信度が閾値に届かなければ問題として記録しない」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::issue` の `error_display_below_threshold_not_recorded` / issue: #8
- [x] T-039 WebExploratoryQA.feature:769(問題の記録) [event] 「読み上げ名が画像ファイル名や仮の値の要素はアクセシビリティの問題として記録する」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::issue` の `placeholder_accessible_names_are_issues` / issue: #8
- [x] T-040 WebExploratoryQA.feature:780(問題の記録) [event] 「ブラウザのコンソールにエラーが出たら問題として記録する」
      → 手法: コントラクトテスト(levels-service)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::issue` の `console_error_is_issue`、契約 `webqa-browser/tests/contract.rs` の `collects_console_errors` / issue: #8、#9
- [x] T-041 WebExploratoryQA.feature:785(問題の記録) [unwanted] 「意味の判断が要る検出は、判断モデルだけの判定なら参考情報として記録する」
      → 手法: 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::issue` の `judge_only_detection_is_reference` / issue: #8
- [x] T-042 WebExploratoryQA.feature:790(問題の記録) [ubiquitous] 「実行結果は再実行できる形で出力する」
      → 手法: ゴールデンテスト(oracle-past-output) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::store::report` の `report_matches_snapshot`、実行ファイル `webqa/tests/cli.rs` の `replay_reruns_report`、単体 `webqa::run::session` の `replay_starts_from_given_memory_and_keeps_stored`、単体 `webqa::run::session` の `stored_start_is_written_back`、単体 `webqa::store::report` の `report_keeps_start_memory`、単体 `webqa::store::report` の `start_memory_is_redacted`、単体 `webqa::replay` の `from_report_reads_start_memory`、単体 `webqa::cli::start` の `broken_start_memory_is_config_error` / issue: #14、#61
- [x] T-043 WebExploratoryQA.feature:798(設定と外部境界) [event] 「テスト対象・入力値・段階を YAML の設定で定義して実行する」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::raw` の `reads_target_inputs_and_stages` / issue: #3
- [x] T-044 WebExploratoryQA.feature:803(設定と外部境界) [ubiquitous] 「決め打ちシナリオと探索モードを同じツールで実行する」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、実行ファイル `webqa/tests/cli.rs` の `runs_scenario_and_explore_modes`、実行ファイル `webqa/tests/cli.rs` の `explore_without_click_or_input_exits_two` / issue: #14
- [x] T-045 WebExploratoryQA.feature:807(設定と外部境界) [optional] 「指定した認証状態を注入してから探索を始める」
      → 手法: コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、契約 `webqa-browser/tests/contract.rs` の `storage_state_opens_logged_in_page` / issue: #9
- [x] T-046 WebExploratoryQA.feature:812(設定と外部境界) [unwanted] 「設定で指定した機密情報は判断モデルに送る前に伏せる」
      → 手法: セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::redact` の `redacts_configured_values` / issue: #5
- [x] T-047 WebExploratoryQA.feature:818(設定と外部境界) [ubiquitous] 「API キーは設定ファイルに書かず、環境変数から読む」
      → 手法: セキュリティテスト(nonfunctional-attributes)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、契約 `webqa-judge/tests/command.rs` の `api_key_env_reaches_bridge` / issue: #7
- [x] T-048 WebExploratoryQA.feature:845(設定と外部境界) [unwanted] 「TLS 証明書の検証を無効にする設定は受け付けない」
      → 手法: セキュリティテスト(nonfunctional-attributes)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::raw` の `rejects_tls_verification_key` / issue: #3
- [x] T-049 WebExploratoryQA.feature:850(設定と外部境界) [boundary] 「0 以下の閾値・上限や空の操作種別は設定エラーにする」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::validate::check` の `rejects_non_positive_and_empty` / issue: #3
- [x] T-050 WebExploratoryQA.feature:878(設定と外部境界) [ubiquitous] 「判断モデルを差し替えても同じ探索ができる」
      → 手法: テストダブル(test-doubles)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-judge::table` の `selects_judge_by_kind`、契約 `webqa-judge/tests/command.rs` の `command_bridge_round_trips` / issue: #7、#15
- [x] T-051 WebExploratoryQA.feature:883(設定と外部境界) [nonfunctional] 「判断・操作・画面変化待ちの時間と、判断モデルのコストを記録する」
      → 手法: 時刻依存の封じ込め(flakiness-value)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::run` の `records_step_timings_with_fake_clock`、単体 `webqa::store::report` の `cost_from_usage_and_pricing` / issue: #14
- [x] T-052 WebExploratoryQA.feature:588(固まりとドメイン外からの復帰) [boundary] 「リダイレクトされたら、最終的な遷移先で許可ドメインかどうかを判定する」
      → 手法: コントラクトテスト(levels-service)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard::domain` の `judges_final_url`、契約 `webqa-browser/tests/contract.rs` の `reports_final_url_after_redirect` / issue: #4、#9
- [x] T-053 WebExploratoryQA.feature:715(対象サーバへの負荷) [unwanted] 「ページの再読み込みや許可ドメインへの戻りも、時間窓の操作回数に数える」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::window` の `counts_each_access_kind` / issue: #12
- [x] T-054 WebExploratoryQA.feature:742(対象サーバへの負荷) [boundary] 「ガードレールで止めて実行しなかった提案は、時間窓の操作回数に数えない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `rejected_proposal_uses_no_access` / issue: #12
- [x] T-055 WebExploratoryQA.feature:58(1ステップの問い合わせと実行) [boundary] 「操作可能な要素が無いページでも問い合わせ、要素を対象にしない操作だけを実行する」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::judge` の `empty_screen_allows_only_untargeted_ops` / issue: #13
- [x] T-056 WebExploratoryQA.feature:66(1ステップの問い合わせと実行) [unwanted] 「判断モデルの応答が使えないときは操作せず、応答の異常として記録して次のステップへ進む」(Examples の各行を1ケースとする)
      → 手法: 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::judge` の `rejects_unusable_answers`、契約 `webqa-judge/tests/command.rs` の `timeout_becomes_judge_error` / issue: #7
- [x] T-057 WebExploratoryQA.feature:285(終了の判定) [boundary] 「判断モデルの呼び出しが上限の1回手前なら、まだ問い合わせる」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `budget_one_left_still_asks` / issue: #12
- [x] T-058 WebExploratoryQA.feature:292(終了の判定) [boundary] 「探索済みの要素が増えず、未操作の要素も新しく見えない操作が閾値の1回手前までなら終了しない」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `stuck_below_threshold_continues` / issue: #12
- [x] T-059 WebExploratoryQA.feature:527(固まりとドメイン外からの復帰) [unwanted] 「固まりを検出しているステップでは判断モデルに問い合わせない」
      → 手法: 禁止仕様からの導出(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `no_question_while_frozen` / issue: #12
- [x] T-060 WebExploratoryQA.feature:533(固まりとドメイン外からの復帰) [unwanted] 「ページが応答しないステップが3回続けば、画面が変化しないステップと同じく開き直す」
      → 手法: コントラクトテスト(levels-service)、状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、契約 `webqa-browser/tests/contract.rs` の `page_timeout_becomes_unchanged_observation`、単体 `webqa-core::machine::counters` の `unresponsive_steps_count_as_freeze` / issue: #9、#12
- [x] T-061 WebExploratoryQA.feature:600(探索の記憶と優先順位) [boundary] 「初めての実行では空の記憶から始める」
      → 手法: CRUD/ライフサイクルテスト(blackbox-state)、境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、保存 `webqa/tests/store.rs` の `missing_memory_starts_empty` / issue: #14
- [x] T-062 WebExploratoryQA.feature:605(探索の記憶と優先順位) [unwanted] 「探索の記憶が壊れていたら、空の記憶で始めてそのことを記録する」
      → 手法: ネガティブテスト(process-static)、CRUD/ライフサイクルテスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、保存 `webqa/tests/store.rs` の `broken_memory_moved_aside_and_starts_empty` / issue: #14
- [x] T-063 WebExploratoryQA.feature:611(探索の記憶と優先順位) [unwanted] 「別の対象の探索の記憶は引き継がない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、保存 `webqa/tests/store.rs` の `memory_is_kept_per_origin` / issue: #14
- [x] T-064 WebExploratoryQA.feature:757(問題の記録) [boundary] 「エラー表示の確信度が閾値ちょうどなら問題として記録する」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::issue` の `error_display_at_threshold_recorded` / issue: #8
- [x] T-065 WebExploratoryQA.feature:823(設定と外部境界) [unwanted] 「実行結果の出力とログでも、指定した機密情報を伏せる」
      → 手法: セキュリティテスト(nonfunctional-attributes) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::redact` の `redacts_names_in_event_payloads` / issue: #5
- [x] T-066 WebExploratoryQA.feature:829(設定と外部境界) [unwanted] 「設定ファイルに API キーが直接書かれていたら設定エラーにする」
      → 手法: セキュリティテスト(nonfunctional-attributes)、ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::raw` の `literal_api_key_rejected_without_value` / issue: #3
- [x] T-067 WebExploratoryQA.feature:835(設定と外部境界) [unwanted] 「参照先の環境変数に API キーが無ければ起動しない」
      → 手法: ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::validate::secrets` の `missing_env_is_config_error`、実行ファイル `webqa/tests/cli.rs` の `missing_api_key_env_exits_two` / issue: #3
- [x] T-068 WebExploratoryQA.feature:840(設定と外部境界) [unwanted] 「注入する認証状態の形式が不正なら設定エラーにする」
      → 手法: ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::validate::storage_state` の `malformed_storage_state_rejected` / issue: #3
- [x] T-069 WebExploratoryQA.feature:297(終了の判定) [boundary] 「ブラウザ操作が起きなかったステップは固まりにも行き詰まりにも数えず、判断予算切れで止まる」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition)、状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `steps_without_browser_op_leave_counters` / issue: #12
- [x] T-070 WebExploratoryQA.feature:644(探索の記憶と優先順位) [unwanted] 「禁止要素は未操作のまま残っていても、探索の候補に入れない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::select` の `forbidden_untouched_excluded` / issue: #10
- [x] T-071 WebExploratoryQA.feature:728(対象サーバへの負荷) [unwanted] 「時間窓の操作回数が上限に達していたら、再読み込み・戻り・離脱も窓が明けるまで待つ」(Examples の各行を1ケースとする)
      → 手法: デシジョンテーブル(blackbox-partition)、時刻依存の封じ込め(flakiness-value) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `window_full_defers_reload_return_and_leave` / issue: #12
- [x] T-072 WebExploratoryQA.feature:83(1ステップの問い合わせと実行) [event] 「判断モデルに送る操作可能要素の一覧は、表示領域の中の要素だけにする」
      → 手法: ドメイン分析テスト(blackbox-partition)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::screen` の `viewport_filter_by_box`、契約 `webqa-browser/tests/contract.rs` の `aria_snapshot_boxes_mark_viewport` / issue: #6、#9
- [x] T-073 WebExploratoryQA.feature:241(終了の判定) [state] 「スクロールで未操作の要素が新しく見えたら、行き詰まりの数え直しになる」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `scroll_revealing_untouched_resets_stuck` / issue: #12
- [x] T-074 WebExploratoryQA.feature:249(終了の判定) [unwanted] 「スクロールしても未操作の要素が新しく見えなければ、行き詰まりを数え続ける」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `scroll_without_new_untouched_counts_stuck` / issue: #12
- [x] T-075 WebExploratoryQA.feature:316(終了の判定) [event] 「探索モードで制限時間に達したら、判断モデルに問い合わせずに時間切れで終了する」
      → 手法: 状態遷移テスト(blackbox-state)、時刻依存の封じ込め(flakiness-value) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `time_limit_finishes_without_asking` / issue: #12
- [x] T-076 WebExploratoryQA.feature:324(終了の判定) [boundary] 「制限時間の手前なら、時間切れにせず問い合わせる」
      → 手法: 境界値分析(blackbox-partition)、時刻依存の封じ込め(flakiness-value) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `time_limit_boundary` / issue: #12
- [x] T-077 WebExploratoryQA.feature:332(終了の判定) [event] 「決め打ちシナリオでも、実行時間の上限に達したら時間切れで終了する」
      → 手法: 同値分割(blackbox-partition)、時刻依存の封じ込め(flakiness-value) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `time_limit_applies_to_scenario_mode` / issue: #12
- [x] T-078 WebExploratoryQA.feature:339(終了の判定) [state] 「判断予算切れと時間切れが重なったら、どちらか1つの終了理由で終了する」
      → 手法: デシジョンテーブル(blackbox-partition)、モデルベーステスト(oracle-model-based) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `budget_and_timeout_yield_one_of_them`、全状態の探索 `webqa-core/tests/exhaustive.rs` の `budget_and_timeout_tie_yields_one_reason` / issue: #12
- [x] T-079 WebExploratoryQA.feature:347(終了の判定) [state] 「時間切れとゴール達成や行き詰まりが重なったら、ゴール達成や行き詰まりを採る」(Examples の各行を1ケースとする)
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `goal_and_stuck_beat_timeout` / issue: #13
- [x] T-080 WebExploratoryQA.feature:358(終了の判定) [unwanted] 「時間窓が明けるのを待っている間に制限時間に達したら、窓を待たずに時間切れで終了する」
      → 手法: 時刻依存の封じ込め(flakiness-value)、境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `timeout_during_window_wait`、単体 `webqa::run` の `window_wait_sleeps_until_earlier_deadline` / issue: #12、#14
- [x] T-081 WebExploratoryQA.feature:366(終了の判定) [event] 「探索モードで禁止要素を除くすべての要素を探索し終えたら、ゴール達成で終了する」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::graph` の `all_explored_except_forbidden_is_finished` / issue: #13
- [x] T-082 WebExploratoryQA.feature:374(終了の判定) [boundary] 「探索モードで未操作の要素が禁止要素だけになったら、ゴール達成で終了する」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::graph` の `only_forbidden_left_is_finished` / issue: #13
- [x] T-083 WebExploratoryQA.feature:382(終了の判定) [unwanted] 「探索モードで禁止要素以外の未操作の要素が残っていれば、ゴール達成にしない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::explore::graph` の `untouched_on_other_screen_blocks_finished` / issue: #13
- [x] T-084 WebExploratoryQA.feature:415(終了の判定) [state] 「探索モードでは、判断モデルに問い合わせる操作種別の選択肢に done を入れない」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::judge` の `explore_mode_omits_done_option` / issue: #13
- [x] T-085 WebExploratoryQA.feature:421(終了の判定) [unwanted] 「探索モードで判断モデルが done を返しても、未検証の終了にせず応答の異常として扱う」
      → 手法: 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::judge` の `done_in_explore_mode_is_anomaly` / issue: #13
- [x] T-086 WebExploratoryQA.feature:430(決め打ちシナリオの段階) [event] 「最後でない段階の done_when を満たしたら、判断モデルに問い合わせずに次の段階へ進む」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `non_last_stage_met_advances` / issue: #13
- [x] T-087 WebExploratoryQA.feature:439(決め打ちシナリオの段階) [event] 「最後の段階の done_when を満たしたら、ゴール達成で終了する」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `last_stage_met_achieves_goal` / issue: #13
- [x] T-088 WebExploratoryQA.feature:445(決め打ちシナリオの段階) [unwanted] 「後の段階の done_when だけを満たしても、段階を飛ばさずゴール達成にもしない」
      → 手法: 禁止仕様からの導出(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::stage` の `later_stage_done_when_does_not_skip` / issue: #13
- [x] T-089 WebExploratoryQA.feature:474(決め打ちシナリオの段階) [state] 「段階を進めたら、固まりと行き詰まりの回数を0に戻す」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::counters` の `stage_advance_resets_freeze_and_stuck` / issue: #13
- [x] T-090 WebExploratoryQA.feature:482(決め打ちシナリオの段階) [state] 「段階を進めても、判断予算・時間窓の操作回数・探索の記憶は引き継ぐ」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `stage_advance_keeps_budget_window_memory` / issue: #13
- [x] T-091 WebExploratoryQA.feature:494(決め打ちシナリオの段階) [boundary] 「done_when に宣言が複数あれば、すべて成り立ったときだけ満たしたとみなす」(Examples の各行を1ケースとする)
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::stage` の `done_when_requires_all_declarations` / issue: #13
- [x] T-092 WebExploratoryQA.feature:508(決め打ちシナリオの段階) [boundary] 「done_when の要素とテキストは、表示領域の外にあっても成り立つ」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::stage` の `done_when_uses_whole_page` / issue: #13
- [x] T-093 WebExploratoryQA.feature:548(固まりとドメイン外からの復帰) [state] 「スクロールで表示領域の要素が変われば、画面が変化したと数えて固まりの数え直しになる」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::screen` の `scroll_changing_viewport_is_change` / issue: #6
- [x] T-094 WebExploratoryQA.feature:555(固まりとドメイン外からの復帰) [boundary] 「ページの一番下でスクロールして表示領域の要素が変わらなければ、画面が変化しないステップと数える」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::screen` の `scroll_at_bottom_is_unchanged` / issue: #6
- [x] T-095 WebExploratoryQA.feature:650(探索の記憶と優先順位) [unwanted] 「スクロールしても、探索の記憶では同じ画面として扱う」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::screen` の `scroll_keeps_screen_identity` / issue: #6
- [x] T-096 WebExploratoryQA.feature:656(探索の記憶と優先順位) [event] 「ページ全体の操作可能要素が変われば、探索の記憶では別の画面として扱う」
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::screen` の `page_element_change_is_new_screen` / issue: #6
- [x] T-097 WebExploratoryQA.feature:890(利用者による中断) [event] 「待っている間に中断したら、記憶・終了のイベント・実行結果を書いてから中断で終了する」(Examples の各行を1ケースとする)
      → 手法: 並行・競合の封じ込め(flakiness-concurrency)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::run` の `interrupt_cancels_wait_and_writes_records`、実行ファイル `webqa/tests/cli.rs` の `interrupt_writes_records_then_exits` / issue: #14
- [x] T-098 WebExploratoryQA.feature:905(利用者による中断) [boundary] 「ブラウザ操作の途中で中断したら、ページの応答を待たずに記録を書いて中断で終了する」
      → 手法: 並行・競合の封じ込め(flakiness-concurrency)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::run` の `interrupt_during_browser_op_skips_page_wait` / issue: #14
- [x] T-099 WebExploratoryQA.feature:912(利用者による中断) [unwanted] 「中断で終了した実行は、それ以上状態を変えない」
      → 手法: 禁止仕様からの導出(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::run` の `interrupted_run_changes_nothing` / issue: #14
- [x] T-100 WebExploratoryQA.feature:919(利用者による中断) [unwanted] 「ほかの理由で終了したあとに中断しても、終了理由は変わらない」
      → 手法: 禁止仕様からの導出(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interrupt` の `interrupt_after_finish_keeps_reason` / issue: #14
- [x] T-101 WebExploratoryQA.feature:924(利用者による中断) [event] 「中断で終わったときの終了コードは、確定の問題の有無で0か1になる」(Examples の各行を1ケースとする)
      → 手法: デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::exit` の `interrupted_exit_code_by_confirmed_issues` / issue: #14
- [x] T-102 WebExploratoryQA.feature:935(利用者による中断) [unwanted] 「記録を書いている間に2回目の中断を受けたら、書き終わりを待たずに終了コード3で終わる」
      → 手法: 並行・競合の封じ込め(flakiness-concurrency) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::interrupt` の `second_interrupt_abandons_writing` / issue: #14
- [x] T-103 WebExploratoryQA.feature:943(終了コード) [event] 「確定の問題を見つけずに終わったら、終了コードは0になる」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::exit` の `no_confirmed_issue_exits_zero` / issue: #14
- [x] T-104 WebExploratoryQA.feature:956(終了コード) [event] 「確定の問題を1件でも見つけて終わったら、終了コードは1になる」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::exit` の `confirmed_issue_exits_one` / issue: #14
- [x] T-105 WebExploratoryQA.feature:968(終了コード) [boundary] 「参考情報だけを記録して終わったら、終了コードは0になる」
      → 手法: 境界値分析(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::exit` の `reference_only_exits_zero` / issue: #14
- [x] T-106 WebExploratoryQA.feature:973(終了コード) [unwanted] 「設定エラーで起動しなかったら、終了コードは2になる」(Examples の各行を1ケースとする)
      → 手法: ネガティブテスト(process-static) / テスト名: 受け入れ(同題の Scenario)、実行ファイル `webqa/tests/cli.rs` の `config_error_exits_two` / issue: #14
- [x] T-107 WebExploratoryQA.feature:987(終了コード) [unwanted] 「記録の書き込みに失敗したら、実行を止めて終了コード3で終わる」(Examples の各行を1ケースとする)
      → 手法: ネガティブテスト(process-static)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa::exit` の `runtime_error_exits_three`、単体 `webqa::run` の `store_failure_stops_run` / issue: #14
- [x] T-108 WebExploratoryQA.feature:389(終了の判定) [unwanted] 「探索モードで探索してよい要素が無くても、最初の問い合わせの前にはゴール達成にしない」(Examples の各行を1ケースとする)
      → 手法: 禁止仕様からの導出(blackbox-state)、境界値分析(blackbox-partition)、モデルベーステスト(oracle-model-based) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `explore_goal_never_before_first_question`、全状態の探索 `webqa-core/tests/exhaustive.rs` の `explore_goal_needs_a_question` / issue: #13
- [x] T-109 WebExploratoryQA.feature:401(終了の判定) [boundary] 「探索モードで探索してよい要素が無ければ、1回問い合わせた後にゴール達成で終了する」(Examples の各行を1ケースとする)
      → 手法: 境界値分析(blackbox-partition)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::plan` の `empty_explorable_goal_after_first_question` / issue: #13
- [x] T-110 WebExploratoryQA.feature:453(決め打ちシナリオの段階) [event] 「実行を開始した時点で最初の段階の done_when を満たしていれば、操作せずに段階を進める」(Examples の各行を1ケースとする)
      → 手法: 状態遷移テスト(blackbox-state)、デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `start_with_done_when_met_moves_on_without_ops` / issue: #13
- [x] T-111 WebExploratoryQA.feature:466(決め打ちシナリオの段階) [state] 「段階を進めた時点で次の段階の done_when も満たしていれば、操作せずにさらに進む」
      → 手法: 状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `advance_into_met_stage_moves_on_without_ops` / issue: #13
- [x] T-112 WebExploratoryQA.feature:867(設定と外部境界) [unwanted] 「探索モードで使う操作種別に click も input も無ければ設定エラーにする」(Examples の各行を1ケースとする)
      → 手法: ネガティブテスト(process-static)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::config::mode` の `explore_mode_needs_click_or_input`(設定とモードを引数に取る関数)、実行ファイル `webqa/tests/cli.rs` の `explore_without_click_or_input_exits_two` / issue: #17、#14
- [x] T-113 WebExploratoryQA.feature:140(ガードレール) [event] 「許可リストに一致する要素は、禁止のキーワードを含んでも操作する」
      → 手法: ゴールデンテスト(oracle-past-output)、同値分割(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::guard` の `allowed_list_wins_over_every_axis` / issue: #4
- [x] T-114 WebExploratoryQA.feature:661(探索の記憶と優先順位) [state] 「探索モードでは、値の設定された入力欄が候補にあるときだけ、操作種別の選択肢に input を含める」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition)、デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `input_is_offered_only_for_a_field_with_a_value`、単体 `webqa-core::machine::rules` の `input_does_nothing_without_a_field_or_a_value`、単体 `webqa-core::machine::actions` の `input_stays_when_the_rules_offer_nothing_else`、単体 `webqa-core::machine::actions` の `input_is_dropped_when_its_field_is_past_the_options` / issue: #62、#63
- [x] T-115 WebExploratoryQA.feature:679(探索の記憶と優先順位) [unwanted] 「何も打ち込まなかった input は、skipped として記録し、探索済みの要素に数えない」(Examples の各行を1ケースとする)
      → 手法: 同値分割(blackbox-partition)、テストダブル(test-doubles) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `input_is_remembered_only_when_it_typed`、単体 `webqa::run` の `input_that_does_nothing_is_recorded_and_run_continues` / issue: #62、#64
- [x] T-116 WebExploratoryQA.feature:692(探索の記憶と優先順位) [event] 「値の設定された入力欄に打てた input は、skipped を記録せず、探索済みの要素に数える」
      → 手法: 同値分割(blackbox-partition)、状態遷移テスト(blackbox-state) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `input_is_remembered_after_the_page_loses_the_field` / issue: #62、#64
- [x] T-117 WebExploratoryQA.feature:673(探索の記憶と優先順位) [event] 「入力欄に打ったあと Enter を押すと、画面が変わり、変わった先の画面が探索の記憶に加わる」
      → 手法: 状態遷移テスト(blackbox-state)、同値分割(blackbox-partition)、コントラクトテスト(levels-service) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `enter_is_offered_only_with_a_field_that_takes_a_value`、単体 `webqa-core::machine::actions` の `enter_stays_after_the_field_is_typed`、単体 `webqa-core::machine::actions` の `enter_alone_does_not_narrow_to_the_moves`、単体 `webqa-core::machine::actions` の `enter_needs_no_target`、契約 `webqa-browser/tests/contract.rs` の `enter_presses_only_into_a_focused_input_field` / issue: #70
- [x] T-118 WebExploratoryQA.feature:200(ガードレール) [unwanted] 「ページに禁止要素があれば、操作種別の選択肢に enter を含めない」
      → 手法: 同値分割(blackbox-partition)、デシジョンテーブル(blackbox-partition) / テスト名: 受け入れ(同題の Scenario)、単体 `webqa-core::machine::actions` の `enter_is_dropped_when_a_forbidden_element_is_on_the_page`、単体 `webqa-core::machine::actions` の `enter_is_dropped_even_when_it_is_the_only_kind`、単体 `webqa-core::config::mode` の `scenario_mode_needs_a_kind_besides_enter` / issue: #70

## トレーサビリティマトリクス(消し込み台帳)

mutation 列は、ガードレール・終了・固まりとドメイン外・記憶と優先順位・負荷の各 Rule と、機密情報・API キー・TLS・認証・応答の異常の行を「要」にした。終了コードの行(T-101、T-103〜T-107)も「要」にした。
「—」は、問い合わせの形と送る一覧(T-001〜T-004、T-055、T-072)、問題の記録と実行結果の形(T-037〜T-042、T-064)、設定の読み込みとモード(T-043、T-044)、判断モデルの差し替え(T-050)、計測(T-051)の行で、セキュリティ・状態遷移・終了コードに当たらない。
`webqa-core/tests/tlc_traces.rs` はどの行にも対応させていない。TLC の遷移の列と中核を突き合わせる仕様との回帰で、受け入れ条件の1つを受け持つテストではないためである。

| 振る舞い(T番号) | 種別 | 手法(reference) | テスト名 | issue | 設計済 | 緑確認済 | mutation(criticalのみ) |
|------------------|------|------------------|----------|-------|--------|----------|------------------------|
| T-001 | event | 同値分割(blackbox-partition) | `screen_text_lists_role_name_value` | #6 | [x] | [ ] | — |
| T-002 | event | テストダブル(test-doubles) | `step_asks_op_target_error_goal` | #7 | [x] | [ ] | — |
| T-003 | event | 同値分割(blackbox-partition)、テストダブル(test-doubles)、コントラクトテスト(levels-service) | `passed_proposal_becomes_touch`、`performs_each_operation` | #9、#12 | [x] | [ ] | — |
| T-004 | unwanted | デシジョンテーブル(blackbox-partition) | `same_unchanged_target_switches_to_next` | #7 | [x] | [ ] | — |
| T-005 | boundary | 境界値分析(blackbox-partition) | `back_confidence_boundary` | #4 | [x] | [ ] | 要 |
| T-006 | unwanted | 同値分割(blackbox-partition) | `back_without_probability_is_low` | #4 | [x] | [ ] | 要 |
| T-007 | unwanted | ゴールデンテスト(oracle-past-output)、同値分割(blackbox-partition) | `forbidden_by_keyword`、`forbidden_fixtures` | #4 | [x] | [ ] | 要 |
| T-008 | unwanted | 状態遷移テスト(blackbox-state)、コントラクトテスト(levels-service) | `missing_element_aborts`、`removed_element_is_missing_on_refetch` | #4、#9 | [x] | [ ] | 要 |
| T-009 | unwanted | 同値分割(blackbox-partition)、コントラクトテスト(levels-service) | `role_or_position_change_aborts`、`replaced_node_gets_new_ref` | #4、#9 | [x] | [ ] | 要 |
| T-010 | unwanted | 同値分割(blackbox-partition)、コントラクトテスト(levels-service) | `swapped_same_name_buttons_abort`、`swapped_same_name_buttons_detected` | #4、#9 | [x] | [ ] | 要 |
| T-011 | event | 同値分割(blackbox-partition) | `same_name_unique_by_position_passes` | #4 | [x] | [ ] | 要 |
| T-012 | unwanted | ゴールデンテスト(oracle-past-output)、同値分割(blackbox-partition) | `paywall_by_each_axis`、`paywall_fixtures` | #4 | [x] | [ ] | 要 |
| T-013 | unwanted | 同値分割(blackbox-partition)、ゴールデンテスト(oracle-past-output) | `pricing_word_alone_is_not_paywall`、`paywall_fixtures` | #4 | [x] | [ ] | 要 |
| T-014 | unwanted | 状態遷移テスト(blackbox-state) | `leaving_paywall_counts_toward_stuck` | #12 | [x] | [ ] | 要 |
| T-015 | event | 状態遷移テスト(blackbox-state) | `goal_met_finishes_without_asking` | #13 | [x] | [ ] | 要 |
| T-016 | unwanted | 状態遷移テスト(blackbox-state) | `self_reported_done_is_unverified` | #12 | [x] | [ ] | 要 |
| T-017 | event | 状態遷移テスト(blackbox-state) | `stuck_answer_finishes_stuck` | #12 | [x] | [ ] | 要 |
| T-018 | state | 状態遷移テスト(blackbox-state) | `stuck_after_threshold_without_progress` | #12 | [x] | [ ] | 要 |
| T-019 | state | 状態遷移テスト(blackbox-state) | `operating_untouched_resets_stuck` | #12 | [x] | [ ] | 要 |
| T-020 | unwanted | 状態遷移テスト(blackbox-state) | `budget_exhausted_finishes` | #12 | [x] | [ ] | 要 |
| T-021 | state | デシジョンテーブル(blackbox-partition)、モデルベーステスト(oracle-model-based) | `goal_beats_stuck_beats_budget`、`priority_holds_in_every_reachable_state` | #13 | [x] | [ ] | 要 |
| T-022 | state | デシジョンテーブル(blackbox-partition) | `stuck_beats_budget` | #12 | [x] | [ ] | 要 |
| T-023 | state | デシジョンテーブル(blackbox-partition) | `budget_beats_freeze_recovery` | #12 | [x] | [ ] | 要 |
| T-024 | unwanted | 禁止仕様からの導出(blackbox-state)、モデルベーステスト(oracle-model-based) | `finished_is_absorbing` | #12 | [x] | [ ] | 要 |
| T-025 | state | 状態遷移テスト(blackbox-state) | `reload_after_freeze_keeps_memory` | #12 | [x] | [ ] | 要 |
| T-026 | state | 状態遷移テスト(blackbox-state) | `screen_change_resets_freeze` | #12 | [x] | [ ] | 要 |
| T-027 | unwanted | 状態遷移テスト(blackbox-state) | `outside_domain_returns_without_asking` | #12 | [x] | [ ] | 要 |
| T-028 | boundary | 同値分割(blackbox-partition)、境界値分析(blackbox-partition) | `origin_must_match_exactly` | #4 | [x] | [ ] | 要 |
| T-029 | ubiquitous | CRUD/ライフサイクルテスト(blackbox-state) | `memory_round_trips_across_runs` | #14 | [x] | [ ] | 要 |
| T-030 | state | デシジョンテーブル(blackbox-partition) | `untouched_before_unfinished_branch` | #10 | [x] | [ ] | 要 |
| T-031 | state | デシジョンテーブル(blackbox-partition) | `unfinished_branch_before_back_and_scroll` | #10 | [x] | [ ] | 要 |
| T-032 | state | デシジョンテーブル(blackbox-partition) | `back_or_scroll_when_all_finished` | #10 | [x] | [ ] | 要 |
| T-033 | unwanted | 同値分割(blackbox-partition) | `finished_follows_reachable_screens` | #10 | [x] | [ ] | 要 |
| T-034 | unwanted | 同値分割(blackbox-partition) | `payment_elements_never_candidates` | #10 | [x] | [ ] | 要 |
| T-035 | unwanted | 境界値分析(blackbox-partition)、時刻依存の封じ込め(flakiness-value) | `window_full_waits_for_next_window` | #12 | [x] | [ ] | 要 |
| T-036 | state | 境界値分析(blackbox-partition)、時刻依存の封じ込め(flakiness-value) | `window_index_follows_elapsed_time` | #12 | [x] | [ ] | 要 |
| T-037 | event | 同値分割(blackbox-partition) | `screenshot_name_has_kind_and_step` | #14 | [x] | [ ] | — |
| T-038 | unwanted | 境界値分析(blackbox-partition) | `error_display_below_threshold_not_recorded` | #8 | [x] | [ ] | — |
| T-039 | event | 同値分割(blackbox-partition) | `placeholder_accessible_names_are_issues` | #8 | [x] | [ ] | — |
| T-040 | event | コントラクトテスト(levels-service)、同値分割(blackbox-partition) | `console_error_is_issue`、`collects_console_errors` | #8、#9 | [x] | [ ] | — |
| T-041 | unwanted | 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) | `judge_only_detection_is_reference` | #8 | [x] | [ ] | — |
| T-042 | ubiquitous | ゴールデンテスト(oracle-past-output) | `report_matches_snapshot`、`replay_reruns_report`、`replay_starts_from_given_memory_and_keeps_stored`、`stored_start_is_written_back`、`report_keeps_start_memory`、`start_memory_is_redacted`、`from_report_reads_start_memory`、`broken_start_memory_is_config_error` | #14、#61 | [x] | [ ] | — |
| T-043 | event | 同値分割(blackbox-partition) | `reads_target_inputs_and_stages` | #3 | [x] | [ ] | — |
| T-044 | ubiquitous | 同値分割(blackbox-partition) | `runs_scenario_and_explore_modes`、`explore_without_click_or_input_exits_two` | #14 | [x] | [ ] | — |
| T-045 | optional | コントラクトテスト(levels-service) | `storage_state_opens_logged_in_page` | #9 | [x] | [ ] | 要 |
| T-046 | unwanted | セキュリティテスト(nonfunctional-attributes) | `redacts_configured_values` | #5 | [x] | [ ] | 要 |
| T-047 | ubiquitous | セキュリティテスト(nonfunctional-attributes)、コントラクトテスト(levels-service) | `api_key_env_reaches_bridge` | #7 | [x] | [ ] | 要 |
| T-048 | unwanted | セキュリティテスト(nonfunctional-attributes)、ネガティブテスト(process-static) | `rejects_tls_verification_key` | #3 | [x] | [ ] | 要 |
| T-049 | boundary | 境界値分析(blackbox-partition)、ネガティブテスト(process-static) | `rejects_non_positive_and_empty` | #3 | [x] | [ ] | 要 |
| T-050 | ubiquitous | テストダブル(test-doubles)、コントラクトテスト(levels-service) | `selects_judge_by_kind`、`command_bridge_round_trips` | #7、#15 | [x] | [ ] | — |
| T-051 | nonfunctional | 時刻依存の封じ込め(flakiness-value)、同値分割(blackbox-partition) | `records_step_timings_with_fake_clock`、`cost_from_usage_and_pricing` | #14 | [x] | [ ] | — |
| T-052 | boundary | コントラクトテスト(levels-service)、同値分割(blackbox-partition) | `judges_final_url`、`reports_final_url_after_redirect` | #4、#9 | [x] | [ ] | 要 |
| T-053 | unwanted | 同値分割(blackbox-partition) | `counts_each_access_kind` | #12 | [x] | [ ] | 要 |
| T-054 | boundary | 同値分割(blackbox-partition) | `rejected_proposal_uses_no_access` | #12 | [x] | [ ] | 要 |
| T-055 | boundary | 境界値分析(blackbox-partition) | `empty_screen_allows_only_untargeted_ops` | #13 | [x] | [ ] | — |
| T-056 | unwanted | 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition)、コントラクトテスト(levels-service) | `rejects_unusable_answers`、`timeout_becomes_judge_error` | #7 | [x] | [ ] | 要 |
| T-057 | boundary | 境界値分析(blackbox-partition) | `budget_one_left_still_asks` | #12 | [x] | [ ] | 要 |
| T-058 | boundary | 境界値分析(blackbox-partition) | `stuck_below_threshold_continues` | #12 | [x] | [ ] | 要 |
| T-059 | unwanted | 禁止仕様からの導出(blackbox-state) | `no_question_while_frozen` | #12 | [x] | [ ] | 要 |
| T-060 | unwanted | コントラクトテスト(levels-service)、状態遷移テスト(blackbox-state) | `page_timeout_becomes_unchanged_observation`、`unresponsive_steps_count_as_freeze` | #9、#12 | [x] | [ ] | 要 |
| T-061 | boundary | CRUD/ライフサイクルテスト(blackbox-state)、境界値分析(blackbox-partition) | `missing_memory_starts_empty` | #14 | [x] | [ ] | 要 |
| T-062 | unwanted | ネガティブテスト(process-static)、CRUD/ライフサイクルテスト(blackbox-state) | `broken_memory_moved_aside_and_starts_empty` | #14 | [x] | [ ] | 要 |
| T-063 | unwanted | 同値分割(blackbox-partition) | `memory_is_kept_per_origin` | #14 | [x] | [ ] | 要 |
| T-064 | boundary | 境界値分析(blackbox-partition) | `error_display_at_threshold_recorded` | #8 | [x] | [ ] | — |
| T-065 | unwanted | セキュリティテスト(nonfunctional-attributes) | `redacts_names_in_event_payloads` | #5 | [x] | [ ] | 要 |
| T-066 | unwanted | セキュリティテスト(nonfunctional-attributes)、ネガティブテスト(process-static) | `literal_api_key_rejected_without_value` | #3 | [x] | [ ] | 要 |
| T-067 | unwanted | ネガティブテスト(process-static) | `missing_env_is_config_error`、`missing_api_key_env_exits_two` | #3 | [x] | [ ] | 要 |
| T-068 | unwanted | ネガティブテスト(process-static) | `malformed_storage_state_rejected` | #3 | [x] | [ ] | 要 |
| T-069 | boundary | 境界値分析(blackbox-partition)、状態遷移テスト(blackbox-state) | `steps_without_browser_op_leave_counters` | #12 | [x] | [ ] | 要 |
| T-070 | unwanted | 同値分割(blackbox-partition) | `forbidden_untouched_excluded` | #10 | [x] | [ ] | 要 |
| T-071 | unwanted | デシジョンテーブル(blackbox-partition)、時刻依存の封じ込め(flakiness-value) | `window_full_defers_reload_return_and_leave` | #12 | [x] | [ ] | 要 |
| T-072 | event | ドメイン分析テスト(blackbox-partition)、コントラクトテスト(levels-service) | `viewport_filter_by_box`、`aria_snapshot_boxes_mark_viewport` | #6、#9 | [x] | [ ] | — |
| T-073 | state | 状態遷移テスト(blackbox-state) | `scroll_revealing_untouched_resets_stuck` | #12 | [x] | [ ] | 要 |
| T-074 | unwanted | 状態遷移テスト(blackbox-state) | `scroll_without_new_untouched_counts_stuck` | #12 | [x] | [ ] | 要 |
| T-075 | event | 状態遷移テスト(blackbox-state)、時刻依存の封じ込め(flakiness-value) | `time_limit_finishes_without_asking` | #12 | [x] | [ ] | 要 |
| T-076 | boundary | 境界値分析(blackbox-partition)、時刻依存の封じ込め(flakiness-value) | `time_limit_boundary` | #12 | [x] | [ ] | 要 |
| T-077 | event | 同値分割(blackbox-partition)、時刻依存の封じ込め(flakiness-value) | `time_limit_applies_to_scenario_mode` | #12 | [x] | [ ] | 要 |
| T-078 | state | デシジョンテーブル(blackbox-partition)、モデルベーステスト(oracle-model-based) | `budget_and_timeout_yield_one_of_them`、`budget_and_timeout_tie_yields_one_reason` | #12 | [x] | [ ] | 要 |
| T-079 | state | デシジョンテーブル(blackbox-partition) | `goal_and_stuck_beat_timeout` | #13 | [x] | [ ] | 要 |
| T-080 | unwanted | 時刻依存の封じ込め(flakiness-value)、境界値分析(blackbox-partition) | `timeout_during_window_wait`、`window_wait_sleeps_until_earlier_deadline` | #12、#14 | [x] | [ ] | 要 |
| T-081 | event | 同値分割(blackbox-partition) | `all_explored_except_forbidden_is_finished` | #13 | [x] | [ ] | 要 |
| T-082 | boundary | 境界値分析(blackbox-partition) | `only_forbidden_left_is_finished` | #13 | [x] | [ ] | 要 |
| T-083 | unwanted | 同値分割(blackbox-partition) | `untouched_on_other_screen_blocks_finished` | #13 | [x] | [ ] | 要 |
| T-084 | state | 同値分割(blackbox-partition) | `explore_mode_omits_done_option` | #13 | [x] | [ ] | 要 |
| T-085 | unwanted | 非決定的出力システムのテスト(ai-nondeterministic)、同値分割(blackbox-partition) | `done_in_explore_mode_is_anomaly` | #13 | [x] | [ ] | 要 |
| T-086 | event | 状態遷移テスト(blackbox-state) | `non_last_stage_met_advances` | #13 | [x] | [ ] | 要 |
| T-087 | event | 状態遷移テスト(blackbox-state) | `last_stage_met_achieves_goal` | #13 | [x] | [ ] | 要 |
| T-088 | unwanted | 禁止仕様からの導出(blackbox-state) | `later_stage_done_when_does_not_skip` | #13 | [x] | [ ] | 要 |
| T-089 | state | 状態遷移テスト(blackbox-state) | `stage_advance_resets_freeze_and_stuck` | #13 | [x] | [ ] | 要 |
| T-090 | state | 状態遷移テスト(blackbox-state) | `stage_advance_keeps_budget_window_memory` | #13 | [x] | [ ] | 要 |
| T-091 | boundary | デシジョンテーブル(blackbox-partition) | `done_when_requires_all_declarations` | #13 | [x] | [ ] | 要 |
| T-092 | boundary | 同値分割(blackbox-partition) | `done_when_uses_whole_page` | #13 | [x] | [ ] | 要 |
| T-093 | state | 同値分割(blackbox-partition) | `scroll_changing_viewport_is_change` | #6 | [x] | [ ] | 要 |
| T-094 | boundary | 境界値分析(blackbox-partition) | `scroll_at_bottom_is_unchanged` | #6 | [x] | [ ] | 要 |
| T-095 | unwanted | 同値分割(blackbox-partition) | `scroll_keeps_screen_identity` | #6 | [x] | [ ] | 要 |
| T-096 | event | 同値分割(blackbox-partition) | `page_element_change_is_new_screen` | #6 | [x] | [ ] | 要 |
| T-097 | event | 並行・競合の封じ込め(flakiness-concurrency)、テストダブル(test-doubles) | `interrupt_cancels_wait_and_writes_records`、`interrupt_writes_records_then_exits` | #14 | [x] | [ ] | 要 |
| T-098 | boundary | 並行・競合の封じ込め(flakiness-concurrency)、テストダブル(test-doubles) | `interrupt_during_browser_op_skips_page_wait` | #14 | [x] | [ ] | 要 |
| T-099 | unwanted | 禁止仕様からの導出(blackbox-state) | `interrupted_run_changes_nothing` | #14 | [x] | [ ] | 要 |
| T-100 | unwanted | 禁止仕様からの導出(blackbox-state) | `interrupt_after_finish_keeps_reason` | #14 | [x] | [ ] | 要 |
| T-101 | event | デシジョンテーブル(blackbox-partition) | `interrupted_exit_code_by_confirmed_issues` | #14 | [x] | [ ] | 要 |
| T-102 | unwanted | 並行・競合の封じ込め(flakiness-concurrency) | `second_interrupt_abandons_writing` | #14 | [x] | [ ] | 要 |
| T-103 | event | 同値分割(blackbox-partition) | `no_confirmed_issue_exits_zero` | #14 | [x] | [ ] | 要 |
| T-104 | event | 同値分割(blackbox-partition) | `confirmed_issue_exits_one` | #14 | [x] | [ ] | 要 |
| T-105 | boundary | 境界値分析(blackbox-partition) | `reference_only_exits_zero` | #14 | [x] | [ ] | 要 |
| T-106 | unwanted | ネガティブテスト(process-static) | `config_error_exits_two` | #14 | [x] | [ ] | 要 |
| T-107 | unwanted | ネガティブテスト(process-static)、テストダブル(test-doubles) | `runtime_error_exits_three`、`store_failure_stops_run` | #14 | [x] | [ ] | 要 |
| T-108 | unwanted | 禁止仕様からの導出(blackbox-state)、境界値分析(blackbox-partition)、モデルベーステスト(oracle-model-based) | `explore_goal_never_before_first_question`、`explore_goal_needs_a_question` | #13 | [x] | [ ] | 要 |
| T-109 | boundary | 境界値分析(blackbox-partition)、同値分割(blackbox-partition) | `empty_explorable_goal_after_first_question` | #13 | [x] | [ ] | 要 |
| T-110 | event | 状態遷移テスト(blackbox-state)、デシジョンテーブル(blackbox-partition) | `start_with_done_when_met_moves_on_without_ops` | #13 | [x] | [ ] | 要 |
| T-111 | state | 状態遷移テスト(blackbox-state) | `advance_into_met_stage_moves_on_without_ops` | #13 | [x] | [ ] | 要 |
| T-112 | unwanted | ネガティブテスト(process-static)、同値分割(blackbox-partition) | `explore_mode_needs_click_or_input`、`explore_without_click_or_input_exits_two` | #17、#14 | [x] | [ ] | 要 |
| T-113 | event | ゴールデンテスト(oracle-past-output)、同値分割(blackbox-partition) | `allowed_list_wins_over_every_axis` | #4 | [x] | [ ] | 要 |
| T-114 | state | 同値分割(blackbox-partition)、デシジョンテーブル(blackbox-partition) | `input_is_offered_only_for_a_field_with_a_value`、`input_does_nothing_without_a_field_or_a_value`、`input_stays_when_the_rules_offer_nothing_else`、`input_is_dropped_when_its_field_is_past_the_options` | #62、#63 | [x] | [ ] | 要 |
| T-115 | unwanted | 同値分割(blackbox-partition)、テストダブル(test-doubles) | `input_is_remembered_only_when_it_typed`、`input_that_does_nothing_is_recorded_and_run_continues` | #62、#64 | [x] | [ ] | 要 |
| T-116 | event | 同値分割(blackbox-partition)、状態遷移テスト(blackbox-state) | `input_is_remembered_after_the_page_loses_the_field` | #62、#64 | [x] | [ ] | 要 |
| T-117 | event | 状態遷移テスト(blackbox-state)、同値分割(blackbox-partition)、コントラクトテスト(levels-service) | `enter_is_offered_only_with_a_field_that_takes_a_value`、`enter_stays_after_the_field_is_typed`、`enter_alone_does_not_narrow_to_the_moves`、`enter_needs_no_target`、`enter_presses_only_into_a_focused_input_field` | #70 | [x] | [ ] | 要 |
| T-118 | unwanted | 同値分割(blackbox-partition)、デシジョンテーブル(blackbox-partition) | `enter_is_dropped_when_a_forbidden_element_is_on_the_page`、`enter_is_dropped_even_when_it_is_the_only_kind`、`scenario_mode_needs_a_kind_besides_enter` | #70 | [x] | [ ] | 要 |

## 対応する unwanted 行

異常系は同じ行の中でなく、別の行が受け持つ。event/state 行ごとの対応は次のとおり。

- T-001〜T-003、T-072(問い合わせと実行、送る一覧の範囲): T-056(応答の異常)、T-004〜T-010(ガードレール)、T-046、T-065(機密情報を伏せる)
- T-011(位置で一意に決まる同名ボタン): T-009、T-010
- T-113(許可リストに一致する要素の操作): T-007(禁止要素)
- T-117(Enter を押す操作): T-118(禁止要素があれば enter を選択肢から外す)
- T-015、T-017(終了の記録): T-016(自己申告だけの done)、T-024(終了後に進めない)
- T-018、T-019、T-073、T-021〜T-023(行き詰まり・数え直し・重なり): T-074(新しく見えた要素が無いスクロール)、T-014(離脱の繰り返し)、T-020(判断予算切れ)、T-024
- T-075、T-077〜T-079(時間切れ・重なり): T-076(制限時間の手前)、T-080(時間窓の待ちの途中)、T-024
- T-081、T-084(探索モードのゴール・done を選ばせない): T-082(禁止要素だけが残る)、T-083(ほかの画面に未操作の要素が残る)、T-085(探索モードで返った done)、T-108(探索してよい要素が無くても最初の問い合わせの前はゴールにしない)、T-109(探索してよい要素が無いときのゴール)
- T-086、T-087、T-089、T-090、T-110、T-111(段階の切り替え、開始時と進めた直後の done_when): T-088(後の段階の done_when だけが成り立つ)、T-091(宣言の一部だけが成り立つ)、T-092(表示領域の外の宣言)
- T-097、T-101(中断): T-098(操作の途中)、T-099(終了後に状態を変えない)、T-100(終了後の中断)、T-102(2回目の中断)
- T-103、T-104(終了コード): T-105(参考情報だけ)、T-106(設定エラー)、T-107(記録の書き込み失敗)
- T-025、T-026、T-093(固まり、スクロールによる変化): T-059(固まり中に問い合わせない)、T-060(ページが応答しない)、T-069(操作が起きないステップは数えない)、T-094(ページの一番下でのスクロール)
- T-096(ページ全体で画面を識別): T-095(スクロールで別の画面にしない)
- T-030〜T-032(優先順位): T-033(途中のボタンを探索し終えたとしない)、T-034(決済導線を選ばない)、T-070(禁止要素を候補に入れない)
- T-036(時間窓の満了): T-035(上限到達で止める)、T-053(再読み込み・戻りの計上)、T-071(上限到達中の再読み込み・戻り・離脱)
- T-037、T-039、T-040(問題の記録): T-038(閾値未満は記録しない)、T-041(参考情報の区別)
- T-043(設定で定義): T-049(0 以下と空の設定値)、T-066〜T-068(設定エラー)、T-112(探索モードで click も input も無い操作種別)

## 受け入れ範囲の外にした要件

- 任意の Web アプリに使える OSS として配布すること: 配布形態の要件で、振る舞いのテストにならない。リポジトリのライセンスと公開設定で確認する。
- 現行のアクセシビリティ情報取得 API を使うこと: 実装で使う API の選択。依存ライブラリの版とコードレビューで確認する。

## 設計検証で保証できなかった範囲の受け持ち

設計段階の検証で partial や抽象化の限界とした範囲は、次の行が実装で確かめる。

| 範囲 | 受け持つ T-ID |
|------|---------------|
| 許可された提案をブラウザで実行すること | T-003、T-011 |
| 操作種別と対象要素の問い合わせ | T-002、T-055、T-056 |
| 画面が変わらないときの次の候補への切り替え | T-004 |
| 課金画面で問い合わせないこと | T-012、T-014 |
| 固まり・応答しないページからの復旧と記憶の保持 | T-025、T-059、T-060、T-069 |
| 探索の記憶の実行またぎの保存 | T-029、T-061〜T-063 |
| 探索の優先順位 | T-030〜T-034、T-070 |
| 探索モードのゴールの中身(辿れる画面と禁止要素の除外) | T-033、T-081〜T-083 |
| 段階の定義と done_when の判定(宣言の組、ページ全体での判定) | T-043、T-086〜T-092 |
| 新しく見えた要素の判定(表示領域と位置・大きさ) | T-073、T-074 |
| 要素の範囲(送る一覧、画面の変化、画面の識別) | T-072、T-093〜T-096 |
| 制限時間の実時間と時間窓の待ちとの関係 | T-075〜T-080 |
| 中断時の記録の書き出しと終了コード | T-097〜T-102 |
| 終了コード | T-101〜T-107 |
| 時間窓の実時間と数える単位 | T-035、T-036、T-053、T-054、T-071 |
| 判断モデルの応答の多様さ | T-003、T-005〜T-007、T-056 |
| 実行の結果の各分岐(変化なし・変化・ドメイン外) | T-003、T-025、T-026、T-027 |
| 禁止要素の判定 | T-007、T-034、T-113 |
| 課金画面の判定 | T-012、T-013 |
| 低確信度の判定 | T-005、T-006 |
| 許可ドメインの判定 | T-028、T-052 |
| 要素の存在の判定 | T-008〜T-011 |

## 閉じる前のチェック(設計用: 0段を閉じる条件)

- [x] 用語定義表が埋まり、曖昧な語が残っていない(test-design のレビュー3回で指摘された語をすべて足した)
- [x] 全 T-ID に種別ラベルが振られている(証跡は下の「機械確認」)
- [x] 各 `event`/`state` 行に対応する `unwanted` 行が最低1つある(「対応する unwanted 行」の節)
- [x] 全 T-ID の「設計済」が `[x]`(113 行。v5 の仕様で直し待ちの16行を `[x]` に戻した。証跡は下の「機械確認」)
- [x] T-001 から連番に欠番なし(証跡は下の「機械確認」)
- [x] 各 T-ID に手法が割り当たっている(台帳の各行の「手法」欄。割り当てた手法の技法スキルはすべて開いて参照した)
- [x] 正常系だけでなく、境界と異常系が各機能(Rule)に最低1つあり、非機能は必要な箇所にある(証跡は下の「機械確認」の Rule 別集計)。正常系は event/state/ubiquitous/optional、境界は boundary、異常系は unwanted、非機能は nonfunctional と数える。非機能は計測の行(T-051)だけで、セキュリティ(機密情報、API キー、TLS)と負荷の要求は、拒否・抑止の振る舞いとして unwanted で立てた

### 機械確認

台帳と feature の突き合わせは、リポジトリの直下で次のコマンドを実行して確かめた。出力はそのまま貼る。

```console
$ python3 tasks/test-design/tools/check_ledger.py
feature の sha256: 8994b2aac0b29a0913a57ee8975f560259223155b968a4b994b15b42384538f2
台帳の sha256:     8994b2aac0b29a0913a57ee8975f560259223155b968a4b994b15b42384538f2
`- [ ]` の行: 0
台帳の T-ID 行: 118(期待 118)
欠番: なし
feature の Scenario: 118
台帳から指されない Scenario: なし
`[~]` の行: 0(なし)
冒頭の直し待ちの一覧: 0
仕様待ちの issue: なし
マトリクスの行: 118
Rule 別の種別の内訳:
  1ステップの問い合わせと実行: boundary 1、event 4、unwanted 2
  ガードレール: boundary 1、event 2、unwanted 9
  終了の判定: boundary 6、event 5、state 9、unwanted 8
  固まりとドメイン外からの復帰: boundary 3、state 3、unwanted 3
  探索の記憶と優先順位: boundary 1、event 3、state 4、ubiquitous 1、unwanted 7
  対象サーバへの負荷: boundary 1、state 1、unwanted 3
  問題の記録: boundary 1、event 3、ubiquitous 1、unwanted 2
  設定と外部境界: boundary 1、event 1、nonfunctional 1、optional 1、ubiquitous 3、unwanted 7
  決め打ちシナリオの段階: boundary 2、event 3、state 3、unwanted 1
  利用者による中断: boundary 1、event 2、unwanted 3
  終了コード: boundary 1、event 2、unwanted 2
OK
```
