`crates/webqa/src/run/session.rs`の`conduct`は、`run`、`explore`、`replay`に共通の1回の実行である。
最初の行を書き、記憶を得て、実行ループを回し、最後に記憶、最後の行、レポートを書く。
記録を書く間も中断を見張り、2度目の中断では記録を書き終えずに止める。
