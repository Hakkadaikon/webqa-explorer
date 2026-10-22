//! The cases of the settings tests.

use super::super::Interview;

/// Rejected while reading the YAML.
pub const UNREADABLE: [&str; 2] = [
    "  reset: { command: [x], timeout_secs: 0 }\n",
    "  trial: 3\n",
];

const HESITATION: &str = "interview.hesitation_threshold は 0 より大きく 1 以下の値にする";
const CONFIDENCE: &str = "interview.confidence_threshold は 0 より大きく 1 以下の値にする";
const TRIALS: &str = "interview.trials は 1 以上 100 以下にする";

/// Rejected while loading, with the message expected.
pub const OUT_OF_RANGE: [(&str, &str); 12] = [
    ("  trials: 0\n", TRIALS),
    (
        "  stuck_threshold: 0\n",
        "interview.stuck_threshold は 1 以上にする",
    ),
    (
        "  max_natural_questions: 0\n",
        "interview.max_natural_questions は 1 以上にする",
    ),
    ("  trials: 101\n", TRIALS),
    ("  hesitation_threshold: 0\n", HESITATION),
    ("  hesitation_threshold: 1.5\n", HESITATION),
    ("  hesitation_threshold: .nan\n", HESITATION),
    ("  hesitation_threshold: -0.1\n", HESITATION),
    ("  confidence_threshold: 0\n", CONFIDENCE),
    ("  confidence_threshold: 1.5\n", CONFIDENCE),
    (
        "  requests_per_hour: 1\n",
        "interview.requests_per_hour は 2 以上にする",
    ),
    ("  trials: 101\n  hesitation_threshold: 0\n", TRIALS),
];

/// A field of the interview as a number.
pub type Field = fn(&Interview) -> f64;

/// Edges accepted, with the field and the value expected.
pub const EDGES: [(&str, Field, f64); 8] = [
    ("  trials: 1\n", |i| f64::from(i.trials.get()), 1.0),
    ("  trials: 100\n", |i| f64::from(i.trials.get()), 100.0),
    (
        "  hesitation_threshold: 1\n",
        |i| i.hesitation_threshold,
        1.0,
    ),
    (
        "  hesitation_threshold: 0.01\n",
        |i| i.hesitation_threshold,
        0.01,
    ),
    (
        "  confidence_threshold: 1\n",
        |i| i.confidence_threshold,
        1.0,
    ),
    (
        "  stuck_threshold: 1\n",
        |i| f64::from(i.stuck_threshold.get()),
        1.0,
    ),
    (
        "  max_natural_questions: 1\n",
        |i| f64::from(i.max_natural_questions.get()),
        1.0,
    ),
    (
        "  requests_per_hour: 2\n",
        |i| f64::from(i.requests_per_hour.get()),
        2.0,
    ),
];

/// Reset commands with no program.
pub const NO_COMMAND: [&str; 3] = ["[]", "[\"\"]", "[\"  \", \"x\"]"];
