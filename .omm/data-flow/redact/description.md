`webqa`の`src/run/perform.rs`が、観測を`Redactor`に通して`Seen`を作る。
画面、ページ全体の要素と本文、コンソールのエラーを伏せ字にする。
判定の材料は伏せ字にせず、`Seen`の`material`として中核に渡す。
伏せ字の規則は`webqa-core`の`src/redact.rs`にある。
