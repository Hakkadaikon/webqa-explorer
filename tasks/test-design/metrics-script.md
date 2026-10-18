# テスト項目抽出台帳: metrics-script

対象: `scripts/metrics.py`(rust-code-analysis の JSON を読み、複雑度の閾値を超えた関数とファイルを報告する開発ツール)
走査アンカー: 判定の閾値(関数9種、ファイル2種)と、入出力(読み込み、一覧表示、引数による切り替え)
テスト対象: scripts/metrics_test.py

## 用語定義(抽出より先に埋める)

| 用語 | 定義(曖昧さを残さない) | この定義が生む境界と異常系 |
|------|--------------------------|----------------------------|
| 閾値 | 関数ごと: CCN ≤ 3、認知的複雑度 ≤ 4、Halstead 体積 ≤ 1000、難度 ≤ 30、バグ ≤ 0.5、MI ≥ 65、MI(VS)≥ 20、出口 ≤ 4、引数 ≤ 7。ファイルごと: 体積 ≤ 8000、バグ ≤ 2 | 閾値ちょうどは合格、超えたら不合格 |
| 値が無い | rust-code-analysis が null を返した指標(演算子の無い関数の Halstead や MI)。判定しない | null の指標 |
| 関数 | `kind` が `function` の空間。impl やモジュールなど、ほかの空間の中に入れ子で現れるものも含む | 入れ子 |
| 警告の形式 | `<ファイル>:<行>: warning: <関数名> has <指標名> <値(小数2桁)>`。ファイル単位は行を 1、関数名の代わりに `file` | 書式のずれ |

## 台帳

- [x] T-001 metrics.py の判定 [ubiquitous] 「すべての指標が閾値内なら、何も出力せず 0 を返す」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: CheckTest.test_passes_when_every_metric_is_within_the_limits
- [x] T-002 metrics.py の判定 [boundary] 「関数の各指標が閾値ちょうどなら合格」
      → 手法: 境界値分析 (blackbox-partition/SKILL.md) / テスト名: CheckTest.test_a_function_at_each_limit_passes
- [x] T-003 metrics.py の判定 [unwanted] 「関数の各指標が閾値を超えたら、警告の形式で報告し 1 を返す」
      → 手法: 境界値分析 (blackbox-partition/SKILL.md) / テスト名: CheckTest.test_a_function_over_each_limit_is_reported
- [x] T-004 metrics.py の判定 [boundary] 「値が無い指標は判定しない」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: CheckTest.test_a_missing_value_is_not_judged
- [x] T-005 metrics.py の判定 [event] 「ほかの空間の中に入れ子になった関数も判定する」
      → 手法: 分岐網羅 (whitebox-controlflow-basic/SKILL.md) / テスト名: CheckTest.test_functions_nested_in_other_spaces_are_checked
- [x] T-006 metrics.py の判定 [unwanted] 「ファイルの体積かバグが閾値を超えたら報告する」
      → 手法: 境界値分析 (blackbox-partition/SKILL.md) / テスト名: CheckTest.test_a_file_over_its_limits_is_reported
- [x] T-007 metrics.py の判定 [boundary] 「ファイルの指標が閾値ちょうどなら合格」
      → 手法: 境界値分析 (blackbox-partition/SKILL.md) / テスト名: CheckTest.test_a_file_at_its_limits_passes
- [x] T-008 metrics.py の閾値 [ubiquitous] 「閾値は合意した値(用語定義の表)と一致する」
      → 手法: 境界の定数の一致確認 (blackbox-partition/SKILL.md の落とし穴) / テスト名: LimitsTest.test_limits_are_the_agreed_ones
- [x] T-009 metrics.py の読み込み [event] 「指定ディレクトリ配下の JSON を入れ子のディレクトリまで読み、ファイル名の順に並べる」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: LoadTest.test_reads_every_json_below_the_directory_in_name_order
- [x] T-010 metrics.py の一覧 [event] 「見出しと関数ごとの行を出し、値が無い指標は `-` にする」
      → 手法: 同値分割 (blackbox-partition/SKILL.md) / テスト名: ReportTest.test_prints_a_row_per_function_with_dashes_for_missing_values
- [x] T-011 metrics.py の引数 [event] 「`--check` なら判定の終了コード、なければ一覧を出して 0 を返す」
      → 手法: 分岐網羅 (whitebox-controlflow-basic/SKILL.md) / テスト名: MainTest.test_check_flag_selects_the_check, MainTest.test_without_the_flag_it_reports
- [x] T-012 metrics.py の起動 [event] 「スクリプトとして実行すると、コマンドライン引数で main を呼び、その戻り値を終了コードにする」
      → 手法: 分岐網羅 (whitebox-controlflow-basic/SKILL.md) / テスト名: EntryTest.test_running_the_script_exits_with_the_result_of_main

## トレーサビリティマトリクス(消し込み台帳)

| 振る舞い(T番号) | 種別 | 手法(reference) | テスト名 | 設計済 | 緑確認済 | mutation(criticalのみ) |
|------------------|------|------------------|----------|--------|----------|------------------------|
| T-001 | ubiquitous | 同値分割 | CheckTest.test_passes_when_every_metric_is_within_the_limits | [x] | [x] | — |
| T-002 | boundary | 境界値分析 | CheckTest.test_a_function_at_each_limit_passes | [x] | [x] | — |
| T-003 | unwanted | 境界値分析 | CheckTest.test_a_function_over_each_limit_is_reported | [x] | [x] | — |
| T-004 | boundary | 同値分割 | CheckTest.test_a_missing_value_is_not_judged | [x] | [x] | — |
| T-005 | event | 分岐網羅 | CheckTest.test_functions_nested_in_other_spaces_are_checked | [x] | [x] | — |
| T-006 | unwanted | 境界値分析 | CheckTest.test_a_file_over_its_limits_is_reported | [x] | [x] | — |
| T-007 | boundary | 境界値分析 | CheckTest.test_a_file_at_its_limits_passes | [x] | [x] | — |
| T-008 | ubiquitous | 境界の定数の一致確認 | LimitsTest.test_limits_are_the_agreed_ones | [x] | [x] | — |
| T-009 | event | 同値分割 | LoadTest.test_reads_every_json_below_the_directory_in_name_order | [x] | [x] | — |
| T-010 | event | 同値分割 | ReportTest.test_prints_a_row_per_function_with_dashes_for_missing_values | [x] | [x] | — |
| T-011 | event | 分岐網羅 | MainTest.test_check_flag_selects_the_check、MainTest.test_without_the_flag_it_reports | [x] | [x] | — |
| T-012 | event | 分岐網羅 | EntryTest.test_running_the_script_exits_with_the_result_of_main | [x] | [x] | — |

mutation 列は全行「—」。開発用の検査ツールで、金額・セキュリティ・状態遷移に当たらない。

## 対応する unwanted 行

- T-001、T-002、T-004、T-005、T-007: T-003、T-006(閾値を超えたら報告する)
- T-009〜T-012(入出力): 開発者が手で回すツールで、壊れた JSON や存在しないディレクトリは Python の例外でそのまま止まれば足りる。独自の異常系を作らないので unwanted 行を立てない

## 閉じる前のチェック(設計用: 0段を閉じる条件)

- [x] 用語定義表が埋まり、曖昧な語が残っていない
- [x] 全 T-ID に種別ラベルが振られている
- [x] 各 `event`/`state` 行に対応する `unwanted` 行が最低1つある(無いものは上の節に理由)
- [x] 全 T-ID の「設計済」が `[x]`
- [x] T-001 から連番に欠番なし
- [x] 各 T-ID に手法が割り当たっている
- [x] 正常系だけでなく、境界、異常系が各機能に最低1つある(入出力は上の節の理由で異常系なし)
