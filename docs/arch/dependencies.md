# 使っている依存

webqaが使う外部のcrate、ブラウザ、判断モデル、開発ツールをまとめる。
版は`Cargo.toml`、`rust-toolchain.toml`、`.config/aqua/aqua.yaml`、`.config/npm/package.json`が持つ。
この文書は版の写しなので、食い違ったら設定のほうが正しい。

## Rust

ツールチェーンは`rust-toolchain.toml`で1.99.1に固定する。
edition 2024で書き、最小の版(`rust-version`)は1.99である。

### workspaceのcrate

| crate | 用途 |
|---|---|
| `webqa-core` | 状態機械、ガードレール、探索の記憶。同期の純粋なコードだけを置く |
| `webqa-browser` | Playwrightのアダプタ |
| `webqa-judge` | 判断モデルのアダプタ |
| `webqa-tui` | イベントログを表示する`watch`の画面と、インタビューの台帳の画面 |
| `webqa-eval` | インタビューのモードの課題を、仕込みの正解表と突き合わせて測る。仕込みのサイトを配る`serve`も持つ([課題を測る仕組み](eval.md)) |
| `webqa` | 実行ファイル。実行ループ、保存、CLIを持つ |

分け方の理由は[全体設計](overview.md)にある。

### 外部のcrate

版は`Cargo.toml`の`[workspace.dependencies]`にまとめる。
各crateは、使う外部のcrateに`<名前>.workspace = true`を書く。

| crate | 版 | 使う場所 | 用途 |
|---|---|---|---|
| playwright-rs | 0.20.1 | webqa-browser | ブラウザ操作 |
| tokio | 1.53.3 | webqa、webqa-judge。webqa-browserはテストだけ | 非同期ランタイム、シグナル、子プロセス |
| serde | 1.0.229 | webqa-tui以外 | 直列化 |
| serde_json | 1.0.154 | webqa-core、webqa-browser、webqa-judge、webqa | イベントログ、記憶、実行結果、橋渡し |
| serde-saphyr | 2.0.0 | webqa-core、webqa-eval | YAMLの設定と文書の読み込み |
| url | 2.5.8 | webqa-core、webqa-browser、webqa、webqa-eval。webqa-judgeはテストだけ | URLとオリジンの比較 |
| percent-encoding | 2.3.2 | webqa-core | URLの伏せ字 |
| regex | 1.13.1 | webqa-core | 伏せ字の規則 |
| thiserror | 2.0.21 | 全部 | エラーの型 |
| base64 | 0.23.1 | webqa-judge | 橋渡しで画像を送る |
| ratatui | 0.30.2 | webqa-tui | 描画 |
| crossterm | 0.29.0 | webqa-tui | 端末の消去と大きさ、台帳の画面のキーの読み取りとrawモード |
| clap | 4.6.7 | webqa、webqa-eval | 引数 |
| dotenvy | 0.15.7 | webqa | 作業ディレクトリの`.env`の読み込み |
| jiff | 0.2.38 | webqa | 本物の時計の時刻の表記 |
| tracing | 0.1.44 | webqa | 診断ログ |
| tracing-subscriber | 0.3.23 | webqa | 診断ログの出力 |
| cucumber | 0.23.0 | webqa(テストだけ) | 受け入れシナリオ |
| insta | 1.49.0 | webqa、webqa-tui(テストだけ) | ゴールデンテスト |
| proptest | 1.12.0 | webqa(テストだけ) | 試行の外のループの性質テスト |

テストでの使い方は[テストの層](testing.md)にある。

### 監査で許したライセンス

cargo-denyが許すライセンスは、`.config/deny.toml`の一覧に限る。
一覧の外のライセンスは、次の2つだけをcrate単位の例外で許す。
どちらもplaywright-rsの依存の下で入る。

| ライセンス | crate | 入る経路 |
|---|---|---|
| MPL-2.0 | `option-ext` | `dirs`の依存 |
| CDLA-Permissive-2.0 | `webpki-roots` | `ureq`の依存 |

### 版の重複で許したcrate

clippyの`multiple_crate_versions`は、同じcrateの版が2つ入ると止まる。
次のcrateは依存の下で2版が入り、依存の側では消せない。
そこで、`.config/clippy/clippy.toml`の`allowed-duplicate-crates`で許す。
一覧に無い重複は止まる。

| crate | 2版が入る理由 |
|---|---|
| `base64`、`bitflags`、`getrandom`、`nix`、`r-efi`、`syn`、`thiserror`、`thiserror-impl` | ratatui 0.30の依存の下で2版が入る |
| `windows-sys` | playwright-rs 0.20の依存の下で2版が入る |
| `rand_core` | playwright-rsから`tokio-tungstenite`、`tungstenite`、`rand` 0.10と辿った先で2版が入る |
| `block-buffer`、`cpufeatures`、`crypto-common`、`digest`、`rand` | 古い版はratatuiの側から、新しい版はplaywright-rsから`tungstenite`と辿った側から入る |

## ブラウザ

ブラウザはChromiumだけを使う。
playwright-rsは、ビルドのときにNodeで動くPlaywrightのdriverを取得する。
Chromiumは、そのdriverが求める版を`just install-browsers`で入れる。
入れ方とCIでの扱いは[テストの層](testing.md)にある。

## 判断モデル

判断モデルへは、外部コマンドの橋渡しを通してつなぐ。
同梱する橋渡しは`bridge/jev.py`で、Vercel AI Gatewayを通してJEVへ問いを送る。
既定のモデルは`typesafe-ai/jev-1.14.0`である。
JEVが答えない自由な文の問いは、AI Gatewayの言語モデルへ送る。
ただし、送るのは`--text-model`を渡したときだけである。
`--text-claude`を渡したときは、自由な文の問いをGatewayへ送らない。
Claude Codeの`claude -p`で答える。
JEVの問いは、そのときもGatewayへ送る。
`bridge/`はPythonの標準ライブラリだけで書き、外部のパッケージを使わない。
橋渡しの形式と引数は[判断モデル](judge.md)にある。

## 開発ツール

開発ツールは、aquaで`.aqua/`の下に入れる。
版は`.config/aqua/aqua.yaml`で固定し、チェックサムも検査する。
`just setup`が全部を入れる。
ra-jitだけは別の設定に置き、`just ra-jit`を初めて回したときに入れる(下に書く)。

| ツール | 版 | 用途 |
|---|---|---|
| just | 1.59.0 | タスクランナー |
| lefthook | v2.2.2 | gitのフック |
| gitleaks | v8.30.1 | 秘密情報と手元の絶対パスの検査 |
| mado | v0.3.2 | Markdownの検査 |
| typos | v1.51.1 | スペルの検査 |
| taplo | 0.10.0 | TOMLの整形 |
| actionlint | v1.7.12 | GitHub Actionsの検査 |
| shellcheck | v0.11.0 | シェルの検査 |
| shfmt(mvdan/sh) | v3.14.1 | シェルの整形 |
| ruff | 0.17.1 | Pythonの整形と検査 |
| uv | 0.13.1 | Pythonとその実行環境 |
| node | v26.12.0 | Nodeで動く開発ツールの実行 |
| yamlfmt | v0.21.0 | YAMLの整形 |
| cargo-llvm-cov | v0.9.1 | Rustのカバレッジ |
| cargo-deny | 0.20.2 | 依存の監査 |
| cargo-mutants | v27.1.0 | mutationテスト |
| rust-code-analysis | v0.0.25 | 複雑度の計測 |
| sccache | v0.18.0 | コンパイル結果の使い回し |
| ra-jit | 0.1.0 | 常駐して増分で組む(`just ra-jit`で入れる) |

cargo-mutantsとrust-code-analysisは、aquaの標準のregistryに無い。
そこで、`.config/aqua/registry.yaml`に定義を置く。
rust-code-analysisの配布物はLinuxのx86_64向けだけである。
ほかの環境では`cargo install rust-code-analysis-cli --version 0.0.25`で入れる。

ra-jitは、rust-analyzerの上に作った、常駐して増分で組むコンパイラである。
`just ra-jit`で、ワークスペースをra-jitで組む。
引数は`cargo ra-jit`にそのまま渡す。
`just ra-jit test`や`just ra-jit stop`のように使える。
版は0.1.0で、crates.ioからaquaの`cargo`の型で入れる。
GitHubのリリースには動く`nightly`のタグしか無く、版を固定できないためである。
ra-jitだけは`.config/aqua/ra-jit.yaml`に分け、`aqua.yaml`には入れない。
ソースからのビルドに時間がかかるので、CIと`just setup`では入れない。
`just ra-jit`を初めて回したときに入れる。
`just ra-jit`の外で`cargo ra-jit`を打つと、aquaが`aqua.yaml`に無いと言って止まる。

aquaは、`cargo`の型の道具のチェックサムを検査しない。
ra-jitの中身を固定しているのは次の3つである。
crates.ioでは、出した版の中身を書き換えられない。
cargoは、落としたクレートを索引のsha256と照らし合わせる。
`--locked`で、クレートに入ったCargo.lockの版の依存を使う。
依存はcargo-denyの検査の外にある。
crates.ioのra-jitの持ち主は`andrewgazelka`である。
GitHubのリポジトリの作者と同じであることを、2026-10-21に確かめた。

ra-jitが対応するRustは1.99.0だけである。
このリポジトリは1.99.1に固定しているので、`just ra-jit`の中だけ1.99.0で組む。
1.99.0とそのrust-srcは、足りなければ`just ra-jit`がrustupで入れる。
置き場は`target/ra-jit`に分け、普通のビルドの成果物と混ぜない。
そのぶん、worktreeごとのディスクの使い方が数GB増える。
ra-jitは借用検査をしないので、品質ゲートとCIは今までどおり普通の`cargo`で回す。

ra-jitは常駐のプロセスを立て、`~/.cache/ra-jit`にソケットと状態を置く。
worktreeごとに、rust-analyzerの中身を持つプロセスも立つ。
どれも、使われないまま30分たつと止まる(`RA_JIT_IDLE_SECS`で変えられる)。
並列にworktreeで回すとメモリを取り合う。
終わったら`just ra-jit stop`で止める。
`just ra-jit status`で、動いているかを見られる。

IntelのMac(x86_64)では、初回のビルドはrustcに任せて通るが、組み直しで失敗する。
そのため、`just ra-jit`はLinuxとAppleシリコンのMac以外では理由を書いて止まる。
LinuxのCIでは、手動のワークフロー`ra-jit`(`.github/workflows/ra-jit.yml`)で確かめる。
組んだあと1つのファイルを書き換えて組み直し、常駐のプロセスが組んだことを`status`で確かめる。
PRとmainへのpushでは回らず、マージの条件にも入らない。
AppleシリコンのMacで組めるかは、まだ確かめていない。

ほかに、次のものを使う。

| ツール | 版 | 入れ方 | 用途 |
|---|---|---|---|
| Python | 3.14 | `uv python install` | `scripts/`と`bridge/`の実行 |
| coverage | 7.16.2 | `uv run --with`で実行のたびに使う | Pythonのカバレッジ |
| oh-my-mermaid | 0.2.0 | npmで`.config/npm/`に入れる | `.omm/`の図と説明 |

各ツールが受け持つ検査は[品質ゲート](../quality-gates.md)にある。
