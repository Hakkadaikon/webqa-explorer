Feature: WebExploratoryQA
  # Outer-loop source of truth: the EARS requirements behind this feature.
  # Each EARS clause maps to one scenario; the inner loop fills these in from
  # TLC counterexample traces via trace_to_gherkin.py.
  #
  # STATUS: formalize工程の骨格のみ(未着手)。loopeng-modelcheck(中ループ)でTLCの
  # mutation検査を回したのち、loopeng-gherkin(内ループ)でtrace_to_gherkin.pyによる
  # 反例の機械変換と正常系シナリオの追加を行う。このFeatureファイルは次工程で
  # 書き直される前提の雛形であり、現時点で具体的なシナリオが無いことは
  # formalize工程の未達ではない。

  # EARS ubiquitous:  "The <system> SHALL <response>."
  # EARS event:       "WHEN <trigger> the <system> SHALL <response>."
  # EARS state:       "WHILE <state> the <system> SHALL <response>."
  # EARS unwanted:    "IF <condition> THEN the <system> SHALL <response>."
  # EARS optional:    "WHERE <feature> the <system> SHALL <response>."

  Scenario: WebExploratoryQA happy path
    Given the initial domain state
    When the triggering event occurs
    Then the system responds as the EARS clause requires
