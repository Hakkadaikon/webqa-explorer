`Browser::observe`がページを読み、伏せ字の前の`Observation`を返す。
`Observation`は、操作できる要素の一覧、ビューポートの大きさ、要素の参照、禁止の要素と課金画面の判定の材料を持つ。
前回の観測からのコンソールのエラーと、`done_when`が読むページ全体の要素と本文も含む。
実装は`webqa-browser`の`src/page.rs`と`src/snapshot.rs`にある。
