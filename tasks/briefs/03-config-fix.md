# brief: issue #3 の直し(レビューの指摘)

元の brief は同じディレクトリの`03-config.md`。触ってよいファイルも同じ。
まず main に rebase し、worktree の中で`just worktree-tools`を実行する。
テストを先に足し、期待どおりの理由で落ちることを確かめてから直す。

## 直すこと

1. `rejects_non_positive_and_empty`は、各ケースで期待するエラーの種類まで確かめる。
   0と負の値は YAML の読み込みのエラー、確信度は`Confidence(<項目名>)`、`ops: []`は`NoOps`、URL は`InvalidUrl(<項目名>)`。`is_err()`だけの確かめ方はやめる。
2. `allowed_origins`の`replacen("://*.", "://", 1)`は効いていないので消す。
   代わりに、`*.`はホストの先頭だけに許す。`https://a.*.test`と`https://*`はエラー(`InvalidUrl`)、`https://*.app.example.test`は通る、をテストに足す(`rejects_non_positive_and_empty`のケースに足してよい)。
3. `reads_target_inputs_and_stages`の2段階の YAML で、2つ目の段階の done_when の`text`も確かめる。
4. `malformed_storage_state_rejected`で、`read`に渡るパスが設定の`auth.storage_state`と同じであることを確かめる。`auth`を書かなければ`read`が呼ばれないことも確かめる。
5. 次の値はエラーにする(テストは`rejects_non_positive_and_empty`のケースに足す)。
   - `pricing`の負の値と NaN(`Pricing(<項目名>)`のようなエラー)
   - `scenario.stages: []`(`NoStages`のようなエラー)

## 決めたこと(直さない)

- 直書きの API キーを、`api_key`だけを先に読んで見分ける今のやり方は、そのまま採る。設計の文書は親が合わせる。
- `ops`の重複、`leave_paywall`の値の形、`done_when: {}`は、このissueでは扱わない(ガードレールのissueと、仕様待ちのissueで決める)。

## 受け入れ条件

元の brief と同じ(`just check`が終了コード0、config 配下の mutation の生き残り0)。
