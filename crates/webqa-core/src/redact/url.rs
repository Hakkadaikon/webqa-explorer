//! Masks the secrets in a URL, matched on its decoded form.
//!
//! Each part is decoded from `%` escapes before matching, and written back
//! only when masking changed it, so a URL with no secret stays as it was.

use percent_encoding::percent_decode_str;
use url::Url;

use super::{Redacted, Redactor};

impl Redactor {
    /// The URL as it may leave the process: its path, query and fragment
    /// masked as in [`Redactor::screen`], written with `Url::as_str`.
    #[must_use]
    pub fn url(&self, url: &Url) -> Redacted<String> {
        let mut url = url.clone();
        self.mask_url(&mut url);
        Redacted(url.as_str().to_owned())
    }

    /// Masks the path, the query and the fragment.
    pub(super) fn mask_url(&self, url: &mut Url) {
        self.mask_path(url);
        self.mask_query(url);
        self.mask_fragment(url);
    }

    /// The whole path at once, so a secret holding `/` is masked too.
    fn mask_path(&self, url: &mut Url) {
        if let Some(path) = self.masked(&decoded(url.path())) {
            url.set_path(&path);
        }
    }

    /// The query as written first, then with `+` read as a space.
    fn mask_query(&self, url: &mut Url) {
        let query = url.query().and_then(|q| {
            self.masked(&decoded(q))
                .or_else(|| self.masked(&decoded(&q.replace('+', " "))))
        });
        if query.is_some() {
            url.set_query(query.as_deref());
        }
    }

    fn mask_fragment(&self, url: &mut Url) {
        let fragment = url.fragment().and_then(|f| self.masked(&decoded(f)));
        if fragment.is_some() {
            url.set_fragment(fragment.as_deref());
        }
    }

    /// The masked string, only when masking changed it.
    fn masked(&self, decoded: &str) -> Option<String> {
        let masked = self.mask(decoded);
        (masked != decoded).then_some(masked)
    }
}

/// `%` escapes decoded, with bytes that are not UTF-8 as replacement characters.
fn decoded(s: &str) -> String {
    percent_decode_str(s).decode_utf8_lossy().into_owned()
}
