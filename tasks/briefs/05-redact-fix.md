# brief: issue #5 の直し(レビューの指摘)

元の brief は同じディレクトリの`05-redact.md`。触ってよいファイルも同じ。
レビューで見つかった次の点を直す。テストを先に足し、期待どおりの理由で落ちることを確かめてから直す。

## must

1. 秘密の値が`Debug`で出ない。
   - `Rule`、`Redactor`(と内部の型)の`Debug`は手で書き、規則の名前だけを出す。パターンと値は出さない。
   - テスト`debug_hides_values`: literal`hunter2`と pattern`[\w.+-]+@[\w-]+\.[\w.]+`の規則で作った`Rule`と`Redactor`の`format!("{:?}")`に、`hunter2`もパターンの文字列も含まれず、規則の名前は含まれる。

## should

2. 1回の走査で置き換える。
   - 全規則を1本の正規表現(規則ごとの捕獲グループの選択)にまとめ、`replace_all`で1回だけ置き換える。どのグループが当たったかで名前を選ぶ。
   - 並び順: literal を長い順に先に置き、その後に pattern を設定の順に置く。
   - テスト(`redacts_configured_values`に足す): literal「短」=`abc`と「長」=`abcdef`を短い順で渡しても、`abcdef`は`[[長]]`になる。規則名「メール」と literal`メール`の組で、置き換えた`[[メール]]`がさらに置き換わらない。
3. 空の文字列に一致する規則を拒む。
   - `Redactor::new`は、literal が空のときと、pattern が空の文字列に一致する(`regex.is_match("")`)ときにエラーを返す。
   - エラーは自前の型にし、どの規則かの名前を持たせる(正規表現の誤りも同じ型にまとめる。`regex::Error`は`source()`で返す)。値やパターンはエラーの文に入れない。
   - テスト(`redacts_configured_values`に足す): literal`""`、pattern`a*`はエラーになり、エラーの`Display`に規則の名前が含まれる。不正な正規表現`(`のエラーにも名前が含まれる。
4. literal は正規表現として解釈しない。
   - テスト: literal`a.b`の規則で`axb a.b`が`axb [[名前]]`になる。
5. JSON のキーは伏せない。
   - テスト(`redacts_names_in_event_payloads`に足す): キー`taro@example.test`はそのまま残る。`notes[0]`は`"見た [[メールアドレス]]"`と完全に一致する。

## 受け入れ条件

元の brief と同じ(`just check`が終了コード0、mutation の生き残り0)。
worktree では最初に`just worktree-tools`を実行する(本体の`justfile`にあるレシピ。worktree が古ければ、本体の`justfile`を見て同じリンクを張る)。
