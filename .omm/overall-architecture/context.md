`webqa-core`は、ワークスペースのほかのcrateに依存しない。
アダプタのcrateどうしは互いに依存せず、`webqa-core`の型だけを共有する。
判断モデルは、別のプロセスで動く橋渡しのコマンドを通して呼ぶ。
橋渡しのコマンドは設定の`judge.command`で選び、`bridge/`にPythonの実装がある。
