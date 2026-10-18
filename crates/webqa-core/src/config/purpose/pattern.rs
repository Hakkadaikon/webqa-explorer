//! URL patterns of screen purposes.

use url::Url;

/// A pattern of URL paths, where a `*` segment matches any non-empty
/// segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlPattern {
    /// The pattern as written.
    written: String,
    /// The segments, percent-encoded as in [`Url::path`].
    segments: Vec<String>,
}

/// Characters a pattern never has, besides whitespace and control
/// characters.
const FORBIDDEN: [char; 7] = ['?', '#', '[', ']', '{', '}', '\\'];

/// The base the pattern is joined to, to encode it as a URL path.
const BASE: &str = "https://pattern.invalid/";

impl UrlPattern {
    /// The pattern, or `None` when `pattern` is malformed.
    #[must_use]
    pub fn parse(pattern: &str) -> Option<Self> {
        let rest = pattern.strip_prefix('/')?;
        well_formed(rest).then(|| encode(pattern)).flatten()
    }

    /// The pattern as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.written
    }

    /// Whether the path of `url` matches. The query and fragment are ignored.
    #[must_use]
    pub fn matches(&self, url: &Url) -> bool {
        url.path_segments()
            .is_some_and(|path| self.fits(&path.collect::<Vec<_>>()))
    }

    /// Whether both patterns match the same paths.
    pub(super) fn same_paths(&self, other: &Self) -> bool {
        self.segments == other.segments
    }

    fn fits(&self, path: &[&str]) -> bool {
        path.len() == self.segments.len()
            && self
                .segments
                .iter()
                .zip(path)
                .all(|(pattern, segment)| segment_matches(pattern, segment))
    }
}

fn segment_matches(pattern: &str, segment: &str) -> bool {
    if pattern == "*" {
        !segment.is_empty()
    } else {
        pattern == segment
    }
}

/// `rest`, the pattern after its leading `/`, is well formed.
fn well_formed(rest: &str) -> bool {
    !rest.contains(forbidden) && segments_well_formed(rest)
}

fn forbidden(c: char) -> bool {
    FORBIDDEN.contains(&c) || c.is_whitespace() || c.is_control()
}

/// Every segment is well formed, and only the last may be empty.
fn segments_well_formed(rest: &str) -> bool {
    rest.split('/').all(segment_well_formed) && !rest.split('/').rev().skip(1).any(str::is_empty)
}

/// A wildcard is a whole segment, and no segment is `.` or `..`.
fn segment_well_formed(segment: &str) -> bool {
    wildcard_is_whole(segment) && !matches!(segment, "." | "..")
}

fn wildcard_is_whole(segment: &str) -> bool {
    segment == "*" || !segment.contains('*')
}

/// The pattern with its segments encoded as a URL path.
fn encode(pattern: &str) -> Option<UrlPattern> {
    let url = Url::parse(BASE).and_then(|base| base.join(pattern)).ok();
    url.map(|url| UrlPattern {
        written: pattern.to_owned(),
        segments: url.path().split('/').skip(1).map(str::to_owned).collect(),
    })
}
