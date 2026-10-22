//! Why the input of an interview run was rejected.

/// Why the input of an interview run was rejected. No variant carries a
/// file's contents, a path, a URL pattern, or a command argument.
#[derive(Debug, thiserror::Error)]
pub enum InterviewError {
    /// The run is an interview but the configuration has no `interview`.
    #[error("インタビューのモードでは設定に interview を書く")]
    Missing,
    /// A file named by the configuration could not be read.
    #[error("{0} のファイルを読めない")]
    Unreadable(&'static str),
    /// A document is not YAML of the expected shape.
    #[error("{document} を決まった形式で読めない: {source}")]
    Malformed {
        /// Which document: `persona` or `purpose`.
        document: &'static str,
        /// The error from reading the YAML, without a snippet of it. Boxed
        /// to keep the error small.
        source: Box<serde_saphyr::Error>,
    },
    /// A required field is absent, blank, or an empty list.
    #[error("{0} を書く")]
    MissingField(&'static str),
    /// A written `done_when` has no declaration.
    #[error("{0} に url、element、text のどれかを1つ以上書く")]
    NoDeclarations(&'static str),
    /// A purpose document with neither `done_when` nor `screens`.
    #[error("purpose に done_when か screens を書く")]
    EmptyPurpose,
    /// The URL pattern of the screen purpose at this 1-based position is
    /// malformed.
    #[error("purpose.screens の {0} 番目の url の型が書式に合わない")]
    UrlPattern(usize),
    /// The URL pattern at this 1-based position repeats an earlier one.
    #[error("purpose.screens の {0} 番目の url の型が、前の項目と同じ")]
    DuplicateUrlPattern(usize),
    /// The purpose text at this 1-based position is blank.
    #[error("purpose.screens の {0} 番目の purpose を書く")]
    BlankScreenPurpose(usize),
    /// `interview.trials` is 0 or over 100.
    #[error("interview.trials は 1 以上 100 以下にする")]
    Trials,
    /// A count that must be at least 1 is 0.
    #[error("{0} は 1 以上にする")]
    Zero(&'static str),
    /// The requests per hour in an interview is 1.
    #[error("{0} は 2 以上にする")]
    TooFewRequests(&'static str),
}
