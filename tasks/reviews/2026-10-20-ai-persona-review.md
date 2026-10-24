# 仕様のレビュー: AIペルソナの記事と論文に照らして(2026-10-20)

Zenn の記事「AIペルソナは実ユーザーの代わりになるか」(PKSHA、https://zenn.dev/pksha/articles/1f2e8032a5324c)と、その参考論文に照らして、origin/main 23d4caa の仕様を見たレビューの記録。
「厳しいところ」を、(A) 弱いところ・抜けているところと、(B) きつすぎて目的を損ねるところ、の2つの意味で洗い出した。
裏づけには、同じ日の it-trend.jp のインタビューのモードの実行(試行2回、判断の上限20回、課題126件)を使った。
TLA+ の仕様を変える必要があるものは無い。どれも feature、設計書、問いの作り方の変更で済む。

## 指摘(重要度の順)

1. (A) 1ステップの問いが2つのモデルに分かれ、自由文の答えは画像も選んだ操作も知らない(`docs/arch/judge.md`、`interview-mode.md` の問いの表、`bridge/jev.py` の `ask_text`)。期待の文が実際の操作と食い違った(操作は `scroll_down` なのに「クリックした」など)。直し方: 期待は JEV が操作を決めたあとに、選んだ操作と要素を入れて問う。自由文の問いにも画像を添える(見えにくい要素は fix-sight で直し中)。根拠: Duan et al. CHI 2024 の 3.3、5.4.7、5.4.8。
2. (A) 観測が、表示領域の操作可能要素の役割と名前だけ(`explore-mode.md` の要素の範囲、`screen.rs` の `screen_text`)。URL、タイトル、見出し、alert、本文、位置が届かない。直し方: 画面のテキストにそれらを足す。根拠: Duan et al. 3.3、Kuric et al.。
3. (A) 位置づけが「インタビューの代わり」「AIがそのまま直せる指示」(`interview-mode.md` の冒頭と台帳)。AIの指摘の適合率は人の0.83に対して0.60(Duan et al.)。指示どおりに直すとペルソナの気持ちと逆になる課題があった(I-1C9D6578)。直し方: 課題を「確かめる仮説」と位置づけ、指示を「確かめてから直す」形にする。
4. (A) 課題が本物かを測る評価の仕組みが無い。直し方: 既知の不具合を仕込んだ fixture のサイトで再現率と適合率を測る。実物は一部を抜き出して人がラベルを付ける。根拠: Duan et al.、Hwang と Kang。
5. (A)(B) 自然さの問いが同じ系統のモデルの自己確認で、上限50件。今日は50件中49件を「自然」と答え、126件中76件が上限で確かめられなかった。直し方: 門ではなく並べ替えの材料にする。別のモデルに証拠と画像を添えて問う。まとめてから問う。根拠: Kuric et al. の abstract と 4.2.1、Duan et al. 5.4.9。
6. (A) 試行の繰り返しの k/N を「起きた割合」として扱う。AIの選択は実ユーザーより偏る(Kuric et al. 4.1、4.6)。今日は126件中67件が「2回のうち2回」。直し方: 「同じモデルで再現した回数」と呼ぶ。多様さは複数のペルソナやモデルで得る。
7. (A) 「迷い」を操作の種類の確率で読む(`judge.md` の確信度)。どの要素を選ぶかの迷いを見ていない。直し方: `target` の1位と2位の差かエントロピーで読み、台帳では「モデルの不確かさ」と呼ぶ。根拠: Kuric et al. 4.1、5.2。
8. (A) 判断モデルにそれまでの操作の履歴を渡さない(`judge.md`)。今日はスクロールを14回続け、入力と検索を繰り返した。直し方: 直近 N 回の操作と画面が変わったかを問いに添える。
9. (A)(B) 判断予算切れを「目的に届かない」に数える。スクロールで新しい要素が見えると行き詰まりを数え直す。今日は2試行とも `budget_spent` で、ツールの設定をペルソナの失敗として書いた(I-5E142923)。直し方: 予算切れは打ち切りか別の結末にする。インタビューではスクロールで見えた要素を進展に数えない。
10. (A) 操作しても何も起きないことを課題にしていない。直し方: インタビューで「操作に反応が無い」を手がかりに足す。根拠: Nielsen の1番目と9番目。
11. (B) 確定の前で止まり入力の値も固定なので、フォームの誤りと回復を見られない。直し方: テスト環境で利用者がはっきり許したときだけ送信先を `guard.allowed` に入れる。秘密でない欄にはペルソナが作った値を打てるようにする。
12. (B) 確信度の低い `back` を止める(既定0.7)。迷いの手がかり「引き返した」がほとんど出ない(今日は0回)。直し方: インタビューのモードだけ `back_confidence` を別にできるようにする。
13. (B) 視認性の課題を操作可能要素の番号に縛る。画面全体の課題(本文の小ささ、コントラスト、情報の優先順位)を出せない。直し方: 要素の無い「画面全体」の視認性の課題を足す。根拠: Duan et al. 5.5.3。
14. (A) インタビューの実行は再生できず、台帳にモデルの名前も残らない。直し方: 台帳の頭にモデルの ID を書き、照らし合わせで比べる。インタビューも再生できるようにする。
15. (A) ペルソナが5つの項目の要約だけ(優先度は低い)。直し方: インタビューのメモの抜粋を任意の項目として渡せるようにする。根拠: Hwang と Kang の Coverage(属性46%、ペルソナ57%、書き起こし76%)。

## 満たしている点

- 課題をチェックボックスで人が選んでから AI に渡す。
- 探索モードで、意味の判断が要る検出を参考情報に分ける。
- インタビューのモードは、アクセシビリティのテキストとスクリーンショットを JEV に渡す。
- ペルソナ、目的、画面ごとの目的の構成が、Persona-Scenario-Goal に近い。
- 利用者のニーズや動機を調べる対象から外している。
- 画面を渡り歩いて課題を見る。
- temperature などの調整に頼っていない。

## 進め方(利用者と決めた順)

直している3件(inputs の `secret: false`、見えにくい要素の問いと読み、DOM の静まりの待ち)を先に入れ、そのあと次の順に直す。

1. 8・9・1(履歴、上限の扱い、期待の問い方): ツールの動きを正しくする。
2. 3・6・7(位置づけ、k/N の呼び名、迷いの読み方): 台帳の言葉を事実に合わせる。
3. 2・4・5(画面の材料、評価の仕組み、自然さの問い): 設計を広げる。1件ずつ利用者と相談する。

## 参照した記事と論文

1. Zenn「AIペルソナは実ユーザーの代わりになるか」(PKSHA) https://zenn.dev/pksha/articles/1f2e8032a5324c
2. Duan, Warner, Li, Hartmann. "Generating Automatic Feedback on UI Mockups with Large Language Models", CHI 2024. https://arxiv.org/abs/2403.13139
3. Hwang and Kang. "Vibe Design: Human-in-the-loop Agentic Framework for UI Design with Large Language Models"(SSRN). https://papers.ssrn.com/sol3/papers.cfm?abstract_id=6297816 (本文は取得できず、記事の要約による)
4. Hwang and Kang. "Vibe Design: Human-in-the-loop AI Agents for UI Design with Large Language Models", HICSS-59. https://doi.org/10.24251/HICSS.2026.530 (同上)
5. Kuric, Demcak, Krajcovic. "What Would GPT Click: Practical Effects of Human-AI Behavioral Misalignment and the Cost of Synthetic Participants in User Experience". https://arxiv.org/abs/2605.18302
6. Aoyama. "Persona-Scenario-Goal Methodology for User-Centered Requirements Engineering", RE 2007. https://doi.org/10.1109/RE.2007.50
7. Nielsen. "10 Usability Heuristics for User Interface Design". https://www.nngroup.com/articles/ten-usability-heuristics/
