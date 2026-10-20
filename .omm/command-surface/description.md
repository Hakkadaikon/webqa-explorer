`webqa`コマンドには、`run`、`explore`、`replay`、`interview`、`check-docs`、`watch`、`ledger`の7つのサブコマンドがある。
`run`、`explore`、`replay`は同じ起動の手順を通り、`src/run/session.rs`で1回の実行を行う。
`interview`は、起動の手順のあとに試行を繰り返す。
`check-docs`、`watch`、`ledger`は中核の状態機械を回さない。
`ledger`は、画面を閉じれば0、台帳を開けなければ2、端末が無ければ3で終わる。
サブコマンドの読み分けは`crates/webqa/src/cli.rs`、起動の手順は`src/cli/app.rs`と`src/cli/start.rs`にある。
