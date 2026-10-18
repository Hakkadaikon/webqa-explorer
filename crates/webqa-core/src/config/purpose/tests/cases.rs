//! The cases of the purpose tests.

/// Purpose documents with the message expected.
pub const WITHOUT_DECLARATIONS: [(&str, &str); 11] = [
    (
        "goal: 注文する\ndone_when: {}\n",
        "purpose.done_when に url、element、text のどれかを1つ以上書く",
    ),
    ("goal: 注文する\ndone_when:\n", "purpose.done_when を書く"),
    ("goal: 注文する\n", "purpose.done_when を書く"),
    ("done_when: { text: 確認 }\n", "purpose.goal を書く"),
    (
        "goal: \" \"\ndone_when: { text: 確認 }\n",
        "purpose.goal を書く",
    ),
    (
        "goal: 注文する\ndone_when: { url: \"\", text: 確認 }\n",
        "purpose.done_when.url を書く",
    ),
    (
        "goal: 注文する\ndone_when: { element: { role: \" \", name: 注文 } }\n",
        "purpose.done_when.element.role を書く",
    ),
    (
        "goal: 注文する\ndone_when: { element: { role: button, name: \"\" } }\n",
        "purpose.done_when.element.name を書く",
    ),
    (
        "goal: 注文する\ndone_when: { text: \"  \" }\n",
        "purpose.done_when.text を書く",
    ),
    ("", "purpose に done_when か screens を書く"),
    ("# メモだけ\n", "purpose に done_when か screens を書く"),
];

/// URL patterns that are malformed, as written between double quotes in
/// YAML.
pub const BAD_PATTERNS: [&str; 22] = [
    "/products/[",
    "/products/]",
    "",
    "products/*",
    "//host/x",
    "/a//b",
    "/a/*b",
    "/a/b*",
    "/**",
    "/a?q=1",
    "/a#top",
    "/a/./b",
    "/a/../b",
    "/a b",
    "/a{b}",
    "/a\\\\b",
    "/a\\u0001b",
    "/a{b",
    "/a}b",
    "/a/%2e/b",
    "/a/%2E%2E/b",
    "/a/.%2e/b",
];

/// Screen purposes with a null value, with the message expected.
pub const NULL_ITEMS: [(&str, &str); 2] = [
    (
        "screens:\n  - url: /a\n    purpose:\n",
        "purpose.screens の 1 番目の purpose を書く",
    ),
    (
        "screens:\n  - url:\n    purpose: 見る\n",
        "purpose.screens の 1 番目の url の型が書式に合わない",
    ),
];

/// URL patterns that are well formed.
pub const GOOD_PATTERNS: [&str; 6] = [
    "/",
    "/*",
    "/products/*",
    "/products/*/reviews",
    "/products/",
    "/検索",
];

/// Patterns with paths they match and paths they do not.
pub const MATCHES: [(&str, &[&str], &[&str]); 5] = [
    (
        "/products/*",
        &["/products/42", "/products/42?x=1#f"],
        &[
            "/products",
            "/products/",
            "/products/42/reviews",
            "/Products/42",
            "/items/42",
        ],
    ),
    ("/products/", &["/products/"], &["/products"]),
    ("/", &["/"], &["/a"]),
    ("/*", &["/a"], &["/"]),
    ("/検索", &["/検索"], &["/a"]),
];
