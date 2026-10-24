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
- `bridge/`の橋渡しを動かすPython(`python3`)

gitのフック、`.config/npm`のNodeのツール、uvで入れるPythonは入れない。
コミットと文書の検査は、コンテナの外で行う。
`bridge/`の橋渡しはuvの3.14ではなく、Ubuntu nobleの`python3`(3.12)で動く。
手元で通るPythonのコードでも、3.13以降の機能を使うとコンテナの中では動かない。

判断モデルへの経路は、HTTPのゲートウェイだけが使える。
`bridge/jev.py`がVercel AI Gatewayへ送る経路である。
橋渡しの`--text-claude`(`bridge/claude_cli.py`)は、手元の`claude`のCLIとそのログイン情報を使う。
コンテナの中にはどちらも無いので、`--text-claude`は使えない。

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

### ファイルの持ち主

コンテナはrootで動く。
rootで動くDockerでは、コンテナが作業ディレクトリに作ったファイルの持ち主がrootになる。

`.aqua/`と`target/`が手元に無いまま回すと、Dockerがマウントの先としてこの2つをrootの持ち主で作る。
そのあと手元で`sh scripts/setup.sh`や`cargo`を回すと、この2つに書けずに失敗する。
一度もsetupしていない機械では、初めて回す前にリポジトリの直下で次を実行する。

```sh
mkdir -p .aqua target
```

実行の記録(`.webqa/`)の持ち主もrootになる。
持ち主は`sudo chown -R "$(id -u):$(id -g)" .webqa`で戻せる。

## APIキーの渡し方

キーはイメージに焼き込まず、回すときに渡す。
`webqa`は起動のときに作業ディレクトリの`.env`を読む([設定](arch/config.md)の「環境変数」)。
マウントした作業ディレクトリに`.env`があれば、コンテナの中でも手元と同じように読まれる。
`.env`を使わずに渡すなら、`docker run`に`-e <名前>`の形で書く。
名前は、設定の`judge.api_key`が指す変数の名前である。
[設定](arch/config.md)の例なら`-e JEV_API_KEY`になる。
値を書かずに名前だけを書くと、手元の環境変数の値がコンテナに渡り、コマンドの履歴に値が残らない。

## イメージを作り直すとき

次のファイルを変えたら、イメージを作り直す。

- `docker/Dockerfile`。Playwrightの版を上げたときも含む
- `rust-toolchain.toml`
- `.config/aqua/`

`rust-toolchain.toml`と`.config/aqua/`の中身はイメージを作るときに入れる。
作り直さなくても回るが、イメージに無いtoolchainとツールを回すたびに入れ直すので遅い。
入れたものは、`--rm`でコンテナごと消えるためである。

### Playwrightの版

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
