# webqa-explorer

Webサービス向けの探索的テストエージェント。
軽量な判断モデルに画面のアクセシビリティ情報を渡し、返ってきた操作をブラウザで実行する。
これを繰り返して画面を自律的に探索し、不具合や表示の違和感、アクセシビリティ上の問題を見つける。
実装はRustで、状況の表示にはtearsによるTUIを使う。

## 現状

実装の前の設計検討は完了した。
要件をEARS記法で構造化し、TLA+で設計を検査し、受け入れシナリオまで書いた。
検討の記録は`tasks/`にあり、読み方は[`docs/design-records.md`](docs/design-records.md)にまとめた。

開発環境と品質ゲートは用意済みで、実装コードはまだ無い。
アーキテクチャとディレクトリ構造は[`docs/architecture.md`](docs/architecture.md)で決めた。

## 開発環境

開発用のツールはaquaで`.aqua/`配下に固定している。
事前に要るのはrustupとcurlだけで、次の1行でそろう。

```sh
sh scripts/setup.sh
```

その後は`.aqua/bin`をPATHに通し、`just`で作業する。
`just`を引数なしで実行すると、レシピの一覧が出る。
コミット前にはlefthookが軽い検査を回す。
CIはそれに加えて、依存の監査とmutationテストを回す。
基準と直し方は[`docs/quality-gates.md`](docs/quality-gates.md)にまとめた。
開発の進め方と構成は[CONTRIBUTING.md](CONTRIBUTING.md)にまとめた。
