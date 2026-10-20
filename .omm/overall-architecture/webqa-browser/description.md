`crates/webqa-browser`は、`Browser`ポートをPlaywrightで実装するアダプタである。
Chromiumを開始のURLで起動し、保存したログインの状態があれば読み込む。
ページはアクセシビリティのスナップショットから読み、操作できる要素とその参照を取り出す。
要素ごとにページのスクリプトで調べ、禁止の要素と課金画面の判定に要る材料を集める。
操作は`click`、`input`、`scroll_down`、`scroll_up`、`back`、`enter`、`wait`、`reload`、`return`の表から名前で引く。
スクリーンショットとコンソールのエラーも、このアダプタが取る。
