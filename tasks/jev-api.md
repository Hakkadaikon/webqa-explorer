# 判断モデル JEV の API の形(Vercel AI Gateway 経由)

issue #7 の橋渡しのコマンドが叩く API の形を確かめた記録。
対象は `typesafe-ai/jev`、`typesafe-ai/jev-1.13.0`、`typesafe-ai/jev-1.14.0`(`/v1/models` の `type` は `evaluation`)。

## 確かめた日と方法

2026-10-18 に、次の3つで確かめた。

1. 文書を読んだ。
   - Vercel: `https://vercel.com/docs/ai-gateway/getting-started`、
     そこから辿った `.../getting-started/decision`、`.../modalities/decision`、`.../sdks-and-apis/typesafe`
   - TypeSafe(JEV の提供元): `https://docs.typesafe.ai/api.md`、`images.md`、`models.md`
     (目次は `https://docs.typesafe.ai/llms.txt`)
   - AI SDK: `https://ai-sdk.dev/providers/ai-sdk-providers/typesafe-ai`、`https://ai-sdk.dev/docs/ai-sdk-core/decisions`
2. `GET https://ai-gateway.vercel.sh/v1/models` の JEV の項目を読んだ。
3. 小さいリクエストを5回送った(成功2回、わざと失敗させたもの3回)。
   認証はセッションの中継(Network secrets)がヘッダーに足した。
   下の例の要求には `Authorization` を書いていない。キーの値はどこにも残していない。

## エンドポイントと要求の形

AI Gateway には、JEV を呼ぶ HTTP の入口が2つある。
どちらも `POST`、`Content-Type: application/json`、`Authorization: Bearer <AI Gateway のキー>`。
Chat Completions や Responses の入口からは呼べない(文書に明記)。

| 入口 | 問いの種類の名前 | 使用量の名前 | エラーの形 |
|---|---|---|---|
| `https://ai-gateway.vercel.sh/v1/evaluate`(Vercel の Decision API) | `boolean`、`choice`、`score` | `inputTokens`、`outputTokens` | `{"error":{"message","param","type"}}` |
| `https://ai-gateway.vercel.sh/typesafe/v1/systemone`(TypeSafe 互換) | `noul`、`choice`、`score` | `input_tokens`、`output_tokens` | `{"message","error_type"}` |

TypeSafe 互換の入口は、TypeSafe 本家(`https://api.typesafe.ai/v1/systemone`)と同じ形で、
`Noul` という名前がこのリポジトリの `Question::Noul` とそのまま重なる。
橋渡しには TypeSafe 互換の入口を勧める。
本家の文書(API 参照、画像、上限)がそのまま当てはまるからである。

要求の本体はどちらも同じ骨組み。

```json
{
  "model": "typesafe-ai/jev-1.14.0",
  "state": "文字列、またはオブジェクトか配列",
  "questions": {
    "<問いの名前>": { "type": "noul", "instructions": "…", "criteria": { "true": "…", "false": "…" } },
    "<問いの名前>": { "type": "choice", "instructions": "…", "criteria": { "<選択肢>": "説明 か null", "…": null } },
    "<問いの名前>": { "type": "score", "instructions": "…", "criteria": ["最も低い段", "…", "最も高い段"] }
  }
}
```

- `questions` は問いの名前から問いへの表。名前は自由に付けられ、答えは同じ名前で返る。名前はモデルには送られない。
- `instructions` と `criteria` の中身は、文字列のほか、オブジェクトや配列でもよい。
- `noul` の `criteria` は省ける。`choice` の `criteria` は必須で、選択肢の名前から説明への表。説明は `null` でよい(実際に送って確かめた)。選択肢は 1〜255 個。
- `score` の `criteria` は必須で、低い順に並べた段の説明の配列。2〜10 段。省くと 400(下の例3)。
- `/v1/evaluate` だけは、`providerOptions.gateway`(`zeroDataRetention`、`only`、`models` による低確信度のときの代わりのモデル)を受ける。TypeSafe 互換の入口も代わりのモデルの指定は受ける。
- 本体の上限(TypeSafe 本家): 32 MiB、UTF-8、入れ子は 64 段まで、数は有限のものだけ。

### 実際の要求1(`/v1/evaluate`、`typesafe-ai/jev`)

```json
{"model":"typesafe-ai/jev","state":"1: button \"保存\"\n2: link \"ヘルプ\"","questions":{"operation":{"type":"choice","instructions":"次に行う操作の種別を選ぶ","criteria":{"click":null,"scroll_down":null,"back":null}},"target":{"type":"choice","instructions":"操作する要素の番号を選ぶ","criteria":{"1":null,"2":null}},"error_shown":{"type":"boolean","instructions":"画面にエラーの表示があるか"}}}
```

### 実際の要求2(`/typesafe/v1/systemone`、`typesafe-ai/jev-1.14.0`)

要求1と同じ問いに、`noul` の名前と、`score` を1問足した。

```json
{"model":"typesafe-ai/jev-1.14.0","state":"1: button \"保存\"\n2: link \"ヘルプ\"","questions":{"operation":{"type":"choice","instructions":"次に行う操作の種別を選ぶ","criteria":{"click":null,"scroll_down":null,"back":null}},"target":{"type":"choice","instructions":"操作する要素の番号を選ぶ","criteria":{"1":null,"2":null}},"error_shown":{"type":"noul","instructions":"画面にエラーの表示があるか"},"clarity":{"type":"score","instructions":"画面の文言はわかりやすいか","criteria":["わかりにくい","ふつう","わかりやすい"]}}}
```

## 応答の形

### 応答1(200、約0.9秒)

```json
{
  "answers": {
    "operation": {
      "type": "choice",
      "choice": "click",
      "probabilities": { "click": 0.99, "scroll_down": 0, "back": 0.01 },
      "confidence": 0.98
    },
    "target": {
      "type": "choice",
      "choice": "1",
      "probabilities": { "1": 0.89, "2": 0.11 },
      "confidence": 0.78
    },
    "error_shown": { "type": "boolean", "probability": 0.06 }
  },
  "model": "typesafe-ai/jev",
  "providerMetadata": {
    "typesafe": { "confidence": { "operation": 0.98, "target": 0.78 } },
    "gateway": {
      "routing": {
        "originalModelId": "typesafe-ai/jev",
        "resolvedProvider": "typesafe-ai",
        "fallbacksAvailable": ["digitalocean"],
        "planningReasoning": "System credentials planned for: typesafe-ai, digitalocean. Total execution order: typesafe-ai(system) → digitalocean(system)",
        "canonicalSlug": "typesafe-ai/jev",
        "finalProvider": "typesafe-ai",
        "modelAttemptCount": 1,
        "modelAttempts": [
          {
            "canonicalSlug": "typesafe-ai/jev",
            "success": true,
            "providerAttemptCount": 1,
            "providerAttempts": [
              { "provider": "typesafe-ai", "credentialType": "system", "success": true,
                "startTime": 1792328194887, "endTime": 1792328194970, "statusCode": 200 }
            ]
          }
        ],
        "totalProviderAttemptCount": 1
      },
      "cost": "0.00001554",
      "marketCost": "0.00001554",
      "surchargeCost": "0",
      "gatewayCost": "0.00001554",
      "inferenceCost": "0.00001554",
      "inputInferenceCost": "0.00001554",
      "outputInferenceCost": "0",
      "generationId": "gen_01M57HBPCGEHVS4QGNNYBYVRRZ"
    }
  },
  "usage": { "inputTokens": 370, "outputTokens": 86 }
}
```

### 応答2(200、約0.3秒)

```json
{
  "model": "typesafe-ai/jev-1.14.0",
  "answers": {
    "operation": {
      "type": "choice", "choice": "click", "confidence": 0.98,
      "probabilities": { "click": 0.99, "scroll_down": 0, "back": 0.01 }
    },
    "target": {
      "type": "choice", "choice": "1", "confidence": 0.8,
      "probabilities": { "1": 0.9, "2": 0.1 }
    },
    "error_shown": { "type": "noul", "noul": 0.08 },
    "clarity": {
      "type": "score", "score": 1.86, "confidence": 0.79,
      "legend": { "0": "わかりにくい", "1": "ふつう", "2": "わかりやすい" },
      "probabilities": { "0": 0.01, "1": 0.12, "2": 0.87 }
    }
  },
  "usage": { "input_tokens": 426, "output_tokens": 100 },
  "provider_metadata": {
    "typesafe": { "confidence": { "operation": 0.98, "target": 0.8, "clarity": 0.79 } },
    "gateway": {
      "routing": {
        "originalModelId": "typesafe-ai/jev-1.14.0",
        "resolvedProvider": "typesafe-ai",
        "fallbacksAvailable": [],
        "planningReasoning": "System credentials planned for: typesafe-ai. Total execution order: typesafe-ai(system)",
        "canonicalSlug": "typesafe-ai/jev-1.14.0",
        "finalProvider": "typesafe-ai",
        "modelAttemptCount": 1,
        "modelAttempts": [ "…応答1と同じ形なので略…" ],
        "totalProviderAttemptCount": 1
      },
      "cost": "0.000017892",
      "marketCost": "0.000017892",
      "surchargeCost": "0",
      "gatewayCost": "0.000017892",
      "inferenceCost": "0.000017892",
      "inputInferenceCost": "0.000017892",
      "outputInferenceCost": "0",
      "generationId": "gen_01M57HBXPZB3VGF4D1S4CRS98K"
    }
  }
}
```

### 答えの形のまとめ

| 問い | 答えのフィールド | 意味 |
|---|---|---|
| Noul | `noul`(`/v1/evaluate` では `probability`) | 「はい」の確率 P(true)。0〜1 の数1つ。`false` の確率は返らない |
| Choice | `choice`、`probabilities`、`confidence` | `probabilities` はすべての選択肢の確率の表。`choice` は最大の選択肢 |
| Score | `score`、`legend`、`probabilities`、`confidence` | `score` は段の番号(0 から段数-1)の確率の重み付き平均。`probabilities` の鍵は段の番号の文字列 |

- 値は小数第2位に丸めてある。合計がちょうど 1 にならないことがある(応答1の `0.99 + 0 + 0.01` は 1 だが、保証は無い)。0 ちょうども返る。
- `confidence` は選ばれた選択肢の確率とは別の値(応答1の `target` は確率 0.89、確信度 0.78)。Noul には `confidence` が無い。
- `/v1/evaluate` では `confidence` が答えの中と `providerMetadata.typesafe.confidence` の両方に入る。
- 欠けうるもの:
  - 代わりのモデル(`providerOptions.gateway.models`)に言語モデルを指定し、それが答えたとき、Choice と Score は `confidence: 0`、`probabilities: {}` で返る。値が無いという意味で、確信度0ではない(Vercel の文書)。
  - 代わりのモデルを指定しなければ、上の欠けは起きない。本家の API 参照では、`probabilities` と `confidence` は必須のフィールド。
  - 問いの一部だけ答えが返ることは、文書には書かれていない(画像の文書には「部分的な答えは無い」とある)。
- `model` は、Gateway 経由では要求した名前をそのまま返した(`typesafe-ai/jev` は版に解決されなかった)。
  本家の文書は「答えた版の名前を返す」と書いているが、Gateway では当てにできない。
- `typesafe-ai/jev` には別の提供元(`digitalocean`)への切り替えが用意されている。版を固定した `jev-1.14.0` には無かった。
- 使用量は `usage` に入る。Gateway の費用は `providerMetadata.gateway.cost`(文字列の10進数、米ドル)。
- 代わりのモデルが働いたときは、`x-ai-gateway-decision-fallback-*` のヘッダーが付き、使用量は2回分の合計になる。

### エラーの形

例3: `score` の `criteria` を省いた(`/v1/evaluate`、400)。

```json
{"model":"typesafe-ai/jev-1.13.0","state":"1: button \"保存\"","questions":{"clarity":{"type":"score","instructions":"画面の文言はわかりやすいか"}}}
```

```json
{"error":{"message":"questions.clarity.criteria: Invalid input: expected array, received undefined","param":null,"type":"invalid_request_error"}}
```

例4: `/v1/evaluate` に `noul` を送った(400)。

```json
{"model":"typesafe-ai/jev-1.13.0","state":"1: button \"保存\"","questions":{"error_shown":{"type":"noul","instructions":"画面にエラーの表示があるか"}}}
```

```json
{"error":{"message":"questions.error_shown.type: Invalid discriminator value. Expected 'boolean' | 'choice' | 'score'","param":null,"type":"invalid_request_error"}}
```

例5: TypeSafe 互換の入口に `boolean` を送った(400)。

```json
{"model":"typesafe-ai/jev-1.13.0","state":"1: button \"保存\"","questions":{"error_shown":{"type":"boolean","instructions":"画面にエラーの表示があるか"}}}
```

```json
{"message":"questions.error_shown.type: expected one of 'noul', 'choice', 'score'","error_type":"invalid_request"}
```

本家の文書にある状態コード: 400(規則違反、`max_tokens_exceeded` は文脈長超え)、401(キー)、413(32 MiB 超え)、415(圧縮)、422(問いの形の誤り)、429(流量制限)、529(混雑)。
429 と 529 は待って再試行する。提供元が返したエラーは Gateway がそのまま通す。

## 画像

実際には送っていない。TypeSafe 本家の文書(`images.md`)による。

- JEV 1.13 と 1.14 の両方が画像を受ける。Box(画像の中の位置を返す問い)は 1.14 から。
  `/v1/models` でも 1.13.0 と 1.14.0 は `modalities.input` が `["text","image"]`、`typesafe-ai/jev` は `["text"]`。
- 試験的な機能。画像を含む要求には毎回 `TypeSafe-Danger-Zone-Images: true` のヘッダーが要る。無ければ 400。
  Gateway の CORS の許可ヘッダーにもこの名前が入っていた。
- 画像は最上位の `assets` に1回だけ載せ、参照の文字列を `state`、`instructions`、`criteria` の中に書く。参照の位置に画像が入る。

```json
{
  "model": "typesafe-ai/jev-1.14.0",
  "state": { "screen": "1: button \"保存\"\n2: link \"ヘルプ\"", "shot": "3f0c…-shot" },
  "assets": { "3f0c…-shot": { "type": "image", "format": "png", "data_base64": "iVBORw0KGgo…" } },
  "questions": {
    "error_shown": { "type": "noul", "instructions": "`shot` にエラーの表示があるか" }
  }
}
```

- 参照は文字列の中を素の文字列として探される。偶然一致しないよう、画像ごとに UUID を使うのがよい。1つの参照は 1,024 バイトまで。
- 参照されない `assets` の項目は 400。参照が出てくるたびに画像がもう1枚分読まれる(トークンも増える)。複数の問いで使う画像は `state` に1回置き、問いからは `` `shot` `` のように名前で指す。
- 送り方は2通り。`multipart/form-data`(最初の部分が `request` という名前の JSON、画像は別の部分にして `assets` の `part` で名前を指す。推奨)か、JSON の中の `data_base64`(標準の base64、`=` 付き、改行も `data:` も無し。`format` は `png`、`jpg`、`jpeg`)。
- 画像の上限: PNG か JPEG の静止画、1枚 16 MiB、縦横とも 280〜1568 ピクセル、画素数 78,400〜2,458,624。外れたら縮めずに拒む。1枚でも拒まれたら要求全体が失敗する。EXIF の向きは適用されない。
- 画像の分も入力トークンとして同じ単価で課金され、`usage.input_tokens` に含まれる。1枚が何トークンかの目安は、まだ公開されていない。
- Gateway の TypeSafe 互換の入口に multipart を送れるかは確かめていない。AI SDK の文書は、TypeSafe の提供元が画像を multipart で送ると書く一方、別の頁で「TypeSafe は画像を受けない」とも書いており、食い違っている。
- 本家の `models.md` の表は、入力を「Text」とだけ書いている(画像の頁と食い違う)。

## 料金と上限

`/v1/models` の項目(抜粋)。

| 項目 | `typesafe-ai/jev` | `typesafe-ai/jev-1.13.0` | `typesafe-ai/jev-1.14.0` |
|---|---|---|---|
| `type` | `evaluation` | `evaluation` | `evaluation` |
| `pricing.input`(1トークンあたり米ドル) | `0.000000042` | `0.000000042` | `0.000000042` |
| `pricing.output` | `0` | `0` | `0` |
| `context_window` | 32000 | 32000 | 32000 |
| `max_tokens` | 0 | 0 | 0 |
| `modalities.input` | `text` | `text`、`image` | `text`、`image` |
| `zdr` | `some` | `all` | `all` |
| `no_training` | `all` | `all` | `all` |
| `released`(UNIX 時刻) | 1789430400 | 1789430400 | 1792022400 |

`typesafe-ai/jev` の説明文: "Limits: 64,000 tokens total per request; 32,000 tokens for the state plus the longest question. No streaming or output-token limit."

- 料金は入力トークンだけ(100万トークンで 0.042 米ドル)。出力トークンは数として返るが課金されない。
  応答2では 426 × 0.000000042 = 0.000017892 で、`cost` と一致した。
- 上限は2つ。1要求で状態と全部の問いを合わせて 64k トークン、状態と問い1つで 32k トークン。問いの数そのものの上限は無い。
  `TypeSafe-Danger-Zone-Long-Context: true` で両方 262k に上がる(遅く、精度も下がる)。
- 流量制限は組織ごと(トークン毎秒と要求毎秒)。超えると 429。
- 言語: 英語が主で、日本語を含むほかの言語は精度が劣る(本家の文書)。画面の文字列は日本語になりうるので、`confidence` を見て扱う。
- 今回の5回の費用は、成功2回で計 0.000033432 米ドル。失敗3回は Gateway で弾かれ、提供元に届いていない。

## 橋渡しの形式との対応

`tasks/briefs/07-judge.md` の「橋渡しの形式(版 1)」と、`docs/architecture.md` の `Question` の案を、JEV の TypeSafe 互換の形に照らした。

### そのまま対応するもの

| 橋渡し | JEV |
|---|---|
| `screen`(文字列) | `state`(文字列) |
| `questions[].id` | `questions` の鍵 |
| `questions[].prompt` | `instructions` |
| `kind: "noul"` | `type: "noul"` |
| `kind: "choice"`、`options` | `type: "choice"`、`criteria`(選択肢の名前から `null` への表) |
| `answers.<id>` の Choice の列 | `probabilities` を `[{option, p}]` に並べ直す |
| `answers.<id>` の Noul | `[{"option":"true","p": noul}]` |
| `usage.input_tokens`、`output_tokens` | `usage.input_tokens`、`output_tokens`(TypeSafe 互換の入口なら名前も同じ) |
| `error.kind: "api"` | HTTP の 4xx と 5xx |
| `error.kind: "timeout"` | 橋渡しのコマンドの側の待ち時間切れ |

### 対応しないところ

1. **Score の段が無い。** `Question::Score { id, prompt }` には段の説明が無いが、JEV の `score` は `criteria`(2〜10 段の配列)が必須で、省くと 400(例3)。`Score` に `levels: Vec<String>` を足すか、`Score` を使わない(Choice か Noul で代える)かを決める必要がある。
2. **Score の値の範囲。** brief は「選択肢 `"score"` に 0 以上 1 以下の点数」とするが、JEV の `score` は 0〜(段数-1)。橋渡しで `score / (段数 - 1)` に直すか、brief を段ごとの確率(`probabilities` の `"0"`、`"1"`…)を列で返す形に変える。後者なら Choice と同じ扱いになり、`legend` も要らない。
3. **確信度の定義。** `docs/architecture.md` は「確信度は答えの確率の最大値」とするが、JEV は別に `confidence` を返す(応答1の `target` は最大の確率 0.89、`confidence` 0.78)。版 1 の答えの形には `confidence` を入れる場所が無い。中核が最大の確率を使うならこのままでよい。JEV の `confidence` を使うなら、答えの形に `confidence` を足す。
4. **Noul の `false`。** brief は「`"true"`(と、あれば `"false"`)」とするが、JEV は P(true) だけを返す。`"false"` は来ないと決めてよい(必要なら `1 - p` を橋渡しで足す)。
5. **確率の欠け(`p` が `null`)。** 代わりのモデルを指定しない限り JEV では起きない。brief の `NaN` の扱いは、言語モデルへの切り替えを使ったときと、ほかのモデルへつなぐときのために残してよい。
6. **選択肢の順序と重複。** JEV の `criteria` と `probabilities` は JSON のオブジェクトなので、順序は保証されず、同じ名前の選択肢は1つにまとまる。橋渡しは要求の `options` の順に並べ直して返すこと、中核は `options` に重複を入れないことを決めておく。
7. **選択肢の数。** Choice は 255 個まで。`step_questions` の要素の番号が 255 を超えると 400(または 422)になる。中核の側で、送る要素の数を 255 以下に抑えるか、問いを分ける。
8. **問いの名前の制約。** JEV の側には無い(鍵は自由)。橋渡しの `id` をそのまま鍵にしてよい。
9. **使用量と費用。** 版 1 の `usage` は2つの数だけ。JEV の出力トークンは課金されないので、設定の単価で費用を出すなら出力の単価を 0 にする。Gateway が返す `cost` を使うなら、答えの形に `cost`(文字列の10進数)を足す。
10. **画像。** 版 1 の要求には画像を入れる場所が無い。入れるときは版を上げ、要求に `screenshot`(PNG の base64 など)を足す。橋渡しはそれを `assets` に入れ、`state` をオブジェクト(`{"screen": …, "shot": "<参照>"}`)にし、`TypeSafe-Danger-Zone-Images: true` を付ける。スクリーンショットは 280〜1568 ピクセルに収める必要がある。
11. **`model` の名前。** Gateway では `typesafe-ai/` を付ける。本家の `jev-latest` などの別名は Gateway には無く、`typesafe-ai/jev` が別名に当たる。再現のために版を固定した名前(`typesafe-ai/jev-1.14.0`)を使うのがよい。応答の `model` は要求の名前のままなので、版の記録には要求した名前を使う。
12. **認証。** Network secrets の中継がキーを差し込むので、`judge.api_key` を省いたとき(`WEBQA_JUDGE_API_KEY` が無いとき)は、橋渡しは `Authorization` を付けずに送る。キーがあるときは `Authorization: Bearer <キー>` を付ける。

### 橋渡しのコマンドが変換すること

1. 要求の1行を読み、`version` と `seq` を覚える。
2. `{"model": <設定のモデル名>, "state": screen, "questions": {…}}` を組み立てる。`kind` を `type` に、`prompt` を `instructions` に、`options` を `criteria`(値は `null`)に移す。Score は段の説明を `criteria` に入れる(上の1を決めたあと)。
3. `POST https://ai-gateway.vercel.sh/typesafe/v1/systemone` に送る。キーがあれば `Authorization` を付ける。429 と 529 は、待ち時間の内で `retry-after` を見て再試行する。
4. 200 なら、問いごとに次の形へ直す。
   - Choice: `options` の順に `{"option": o, "p": probabilities[o]}`。`probabilities` に無い選択肢は `p` を省く。
   - Noul: `[{"option":"true","p": noul}]`。
   - Score: 上の2で決めた形。
   - `usage` の `input_tokens` と `output_tokens` をそのまま写す。
5. 200 以外、JSON として読めない、答えに問いが欠けている、のどれかなら `{"version":1,"seq":…,"error":{"kind":"api","message":…}}` を返す。`message` にはキーや画面の文字列を入れない(状態コードと `error_type` 程度にする)。
6. 標準エラーには何も書かない前提で作る(中核が捨てるが、念のため)。

### #7 の brief を直すべき点

1. 「橋渡しの形式」の Score の行: 「0 以上 1 以下の点数を `"score"` に」を、JEV の 0〜(段数-1)の値をどう写すかに合わせて書き直す(段ごとの確率の列にするのが素直)。
2. `Question::Score` に段の説明が無いと JEV に送れないことを書く。#15 の型を変えるか、Score を使わないと決めるか、#15 と合わせて決める。
3. Noul の答えは `"true"` の1つだけと書く(`"false"` は JEV からは来ない)。
4. 答えの列の順序は要求の `options` の順で返す、と橋渡しの決まりに足す。選択肢の重複を中核が作らないことも書く。
5. 確信度を「確率の最大値」で通すか、JEV の `confidence` を答えの形に足すかを決める。足すなら版 1 の答えに `confidence`(省ける)を置く。
6. Choice の選択肢は 255 個までという上限を、`step_questions` の要素の番号に関わる制約として書く。
7. `p` の欠け(`null`)は JEV の通常の応答では起きないこと、それでも扱いは残すことを書く(試験の根拠になる)。
8. 範囲の外に「画像を渡す形は版 2 で決める」と書き、版 1 の要求に画像のフィールドが無いことを明記する。
9. `judge.api_key` を省いたとき、橋渡しは `Authorization` を付けずに送れば中継がキーを足す、と書く(橋渡しのコマンドを書く人のため)。

## 確かめていないこと

- 画像を実際に送ること。Gateway の TypeSafe 互換の入口と `/v1/evaluate` が、`assets` と multipart をそのまま本家へ通すか。
- 中継がキーを差し込むとき、要求に `Authorization` が既にあれば上書きするか、拒むか。
- 429 と 529 の応答の本体と `retry-after` の形。
- `TypeSafe-Danger-Zone-Long-Context` が Gateway 経由で効くか(CORS の許可ヘッダーには名前があった)。
- 問いの数を増やしたときの待ち時間(今回は 0.3〜0.9 秒)。
- 日本語の画面での精度。今回の問いは易しく、答えはもっともらしかったが、評価はしていない。
- `GET https://ai-gateway.vercel.sh/typesafe/v1/models` の形。
- 選択肢の名前に記号や長い文字列を使ったときの扱い。

## 自由な文の問い(2026-10-18 の利用者の決定)

JEV が答えられる問いは真偽(noul)、選択(choice)、点数(score)の3つで、自由な文で答える問いは無い。
インタビューのモードの自由な文の問い(`Question::Text`、issue #19)は、橋渡しのコマンドが AI Gateway の言語モデルへ送る。
キーは JEV と同じ AI Gateway のキーを使い、モデルの名前は設定で選べるようにする。
Rust の側の形(橋渡しの版2)は、どのモデルが答えるかに依らない。
