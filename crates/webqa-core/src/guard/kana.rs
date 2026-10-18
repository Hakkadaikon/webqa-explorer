//! Half-width katakana (U+FF61 to U+FF9F) to full-width, joining the voiced
//! and semi-voiced sound marks with the kana before them.

/// The full-width forms of U+FF61 to U+FF9F, in order.
const FULLWIDTH: &str = "。「」、・ヲァィゥェォャュョッーアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワン゛゜";

const FIRST: u32 = 0xFF61;

/// A sound mark, the kana it joins with, and how far the joined kana is.
const MARKS: [(char, &str, u32); 3] = [
    ('゛', "カキクケコサシスセソタチツテトハヒフヘホ", 1),
    ('゛', "ウ", 0x4E),
    ('゜', "ハヒフヘホ", 2),
];

/// The full-width form of a half-width katakana, or `c` itself.
pub(super) fn widen(c: char) -> char {
    u32::from(c)
        .checked_sub(FIRST)
        .and_then(|offset| usize::try_from(offset).ok())
        .and_then(|offset| FULLWIDTH.chars().nth(offset))
        .unwrap_or(c)
}

/// Pushes `c`, joining a sound mark with the kana before it.
pub(super) fn push(out: &mut String, c: char) {
    let joined = out.chars().next_back().and_then(|prev| join(prev, c));
    if let Some(kana) = joined {
        out.pop();
        out.push(kana);
    } else {
        out.push(c);
    }
}

fn join(prev: char, mark: char) -> Option<char> {
    MARKS
        .iter()
        .find(|(m, bases, _)| *m == mark && bases.contains(prev))
        .and_then(|(_, _, step)| char::from_u32(u32::from(prev) + step))
}
