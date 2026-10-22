# コンテナで回す

Linuxで、Chromiumが要るOSのライブラリを手元のOSに入れずに、webqaとテストを回す手順をまとめる。
手元の`just install-browsers-with-deps`はOSにライブラリを入れる。
Playwrightが`sudo apt-get install`を呼ぶためである。
コンテナを使えば、ライブラリはイメージの中に収まる。

macOSでは要らない。
macOSのChromiumはOSのライブラリを別に求めないので、`sh scripts/setup.sh`だけで回る。

## イメージの中身

イメージは`docker/Dockerfile`で作る。
元にするのは、Playwright公式のイメージ`mcr.microsoft.com/playwright:v<版>-noble`である。
このイメージには、その版のPlaywrightが使うChromiumと、Chromiumが要るOSのライブラリが入っている。
その上に、`scripts/setup.sh`の一部に当たる次のものを入れる。

- rustupと、`rust-toolchain.toml`のtoolchain
- aquaと、`.config/aqua/aqua.yaml`のツール
- ビルドに要るCのコンパイラとOpenSSL(`build-essential`、`libssl-dev`、`pkg-config`)

gitのフック、`.config/npm`のNodeのツール、uvで入れるPythonは入れない。
コミットと文書の検査は、コンテナの外で行う。
`bridge/`の橋渡しはイメージにあるOSのPython(`python3`)で動く。

## イメージを作る

リポジトリの直下で次を実行する。

```sh
docker build -f docker/Dockerfile -t webqa-dev .
```

ビルドに渡すのは、Dockerfileが写す`rust-toolchain.toml`、`scripts/install-aqua.sh`、`.config/aqua/`だけである。
ほかのファイルは`.dockerignore`で外す。
そのため、`target/`と`.env`(APIキー)はイメージに入らない。

## 回す

リポジトリの直下で、作業ディレクトリを`/work`にマウントして回す。

```sh
docker run --rm -it \
  -v "$PWD":/work -v /work/.aqua \
  -v webqa-target:/work/target -v webqa-cargo:/opt/cargo/registry \
  -e CARGO_BUILD_JOBS=2 \
  webqa-dev <コマンド>
```

| 引数 | 役目 |
|---|---|
| `-v "$PWD":/work` | 作業ディレクトリをマウントする。コードの変更はそのまま中に見える |
| `-v /work/.aqua` | イメージに入れたツールを、手元の`.aqua/`の上に重ねる。名前の無いボリュームなので、`--rm`で消える |
| `-v webqa-target:/work/target` | ビルドの置き場を名前つきのボリュームにして、手元の`target/`と混ぜない。Playwrightのdriverもここに置く |
| `-v webqa-cargo:/opt/cargo/registry` | 取得したcrateを、次に回すときのために残す |
| `-e CARGO_BUILD_JOBS=2` | ビルドの並列数を絞る。メモリが少ない機械向けで、要らなければ外す |

`<コマンド>`にはたとえば次を渡す。

| すること | コマンド |
|---|---|
| 受け入れのテスト | `cargo test -p webqa --test acceptance --test interview_acceptance` |
| ブラウザの契約テスト | `cargo test -p webqa-browser --test contract` |
| 探索 | `cargo run -p webqa -- explore --config <設定>` |
| シェルに入る | `bash` |

コンテナはrootで動く。
rootで動くDockerでは、作業ディレクトリに書いた実行の記録(`.webqa/`など)の持ち主がrootになる。
マウントの先として、空の`.aqua/`と`target/`のディレクトリも作業ディレクトリにできる。

## APIキーの渡し方

キーはイメージに焼き込まず、回すときに渡す。
`webqa`は起動のときに作業ディレクトリの`.env`を読む([設定](arch/config.md)の「環境変数」)。
マウントした作業ディレクトリに`.env`があれば、コンテナの中でも手元と同じように読まれる。
`.env`を使わずに渡すなら、`docker run`に`-e WEBQA_JUDGE_API_KEY`のように変数の名前だけを書く。
名前だけなら、手元の環境変数の値がコンテナに渡り、コマンドの履歴に値が残らない。

## Playwrightの版を上げるとき

イメージの版は、playwright-rsが使うPlaywrightの版に合わせる。
版がずれると、driverが求める版のChromiumがイメージに無く、ブラウザを起動できない。
playwright-rsの版を上げたら、`docker/Dockerfile`の`FROM`の版も同じ版に上げる。

版のずれは、`crates/webqa-browser/tests/container.rs`が検査する。
このテストは、`FROM`の行が`playwright_rs::PLAYWRIGHT_VERSION`の版のイメージを指していなければ失敗する。
失敗の文に、合わせるべき版が出る。

## 片づける

イメージとボリュームは、次で消す。

```sh
docker volume rm webqa-target webqa-cargo
docker image rm webqa-dev
```
