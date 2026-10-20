//! Steps of this Rule of the feature, and the scenarios that have them.

#[path = "questions_asked.rs"]
mod asked;
#[path = "questions_chosen.rs"]
mod chosen;
#[path = "questions_images.rs"]
mod images;
#[path = "questions_modes.rs"]
mod modes;
#[path = "questions_replies.rs"]
pub mod replies;
#[path = "questions_unusable.rs"]
mod unusable;

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "インタビューのモードでは、画像を扱えると申告したアダプタに、アクセシビリティ情報のテキストとスクリーンショットを渡す",
    "探索モードと決め打ちシナリオでは、これまでどおりテキストだけを渡す",
    "外部コマンドへの問いの JSON に、特定のモデルに依らない形で画像を入れる",
    "判断モデルのアダプタは、どれも起動のときに画像を扱えるかを申告する",
    "橋渡しは起動のときに、JSON の版の項目で画像を扱えるかを申告する",
    "スクリーンショットは伏せずに渡す",
    "毎ステップの問いにペルソナの説明を添えて、この人ならどうするかを問う",
    "1ステップで今の4つの問いに加えて、迷い・期待・見えにくい要素・画面の目的を問う",
    "目的を書いたときも、画面の目的の推測を問う",
    "期待や理由は、自由な文の答えとしてそのまま受け取る",
    "足した問いの答えが欠けていたら、応答の異常として扱う",
    "足した問いの答えが範囲の外なら、応答の異常として扱う",
    "操作の対象は候補を絞らず、表示領域の操作可能要素から判断モデルに選ばせる",
    "インタビューのモードでは、操作種別の選択肢に done を入れる",
];
