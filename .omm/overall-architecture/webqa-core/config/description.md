`src/config.rs`と`src/config/`は、YAMLの設定を読み、検証する。
`raw::parse`がYAMLを生の型に読み、`validate::validate`が検証済みの`Config`に変える。
ペルソナと目的の文書の形も、ここで確かめる。
