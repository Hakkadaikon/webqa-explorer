//! Allowed origins: exploring stays inside them, judged on the final URL.

#[cfg(test)]
mod tests;

use url::Url;

/// An origin of `target.allowed_origins`: scheme, host and port only.
///
/// A host starting with `*.` allows its subdomains at any depth, but not
/// itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedOrigin {
    scheme: String,
    host: String,
    port: u16,
    subdomains: bool,
}

/// The value is not an origin of scheme, host and port only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("許可ドメインはスキーム、ホスト、ポートだけのオリジンで書く")]
pub struct InvalidOrigin;

type Part = fn(&Url) -> bool;

/// All of these hold for a URL that is an origin and nothing more.
const ORIGIN_ONLY: [Part; 5] = [
    |u| ["", "/"].contains(&u.path()),
    |u| u.query().is_none(),
    |u| u.fragment().is_none(),
    |u| u.username().is_empty(),
    |u| u.password().is_none(),
];

impl AllowedOrigin {
    /// Parses `https://app.example.test` or `https://*.app.example.test`.
    ///
    /// # Errors
    ///
    /// [`InvalidOrigin`] when the value is not a URL, has no host or known
    /// port, has a path, a query, a fragment or credentials, or has `*`
    /// anywhere but a leading `*.`.
    pub fn parse(s: &str) -> Result<Self, InvalidOrigin> {
        let url = Url::parse(s).map_err(|_| InvalidOrigin)?;
        Self::from_url(&url).ok_or(InvalidOrigin)
    }

    fn from_url(url: &Url) -> Option<Self> {
        let port = url.port_or_known_default()?;
        let (subdomains, host) = origin_host(url)?;
        Some(Self {
            scheme: url.scheme().to_owned(),
            host: host.to_owned(),
            port,
            subdomains,
        })
    }

    /// Whether `url` has this scheme and port and a matching host.
    #[must_use]
    pub fn allows(&self, url: &Url) -> bool {
        MATCHES.iter().all(|m| m(self, url))
    }

    /// The exact origin of `url`, without subdomains. `None` without a host
    /// or a known port.
    fn of_url(url: &Url) -> Option<Self> {
        Some(Self {
            scheme: url.scheme().to_owned(),
            host: url.host_str()?.to_owned(),
            port: url.port_or_known_default()?,
            subdomains: false,
        })
    }

    fn host_matches(&self, host: &str) -> bool {
        if self.subdomains {
            is_subdomain(host, &self.host)
        } else {
            host == self.host
        }
    }
}

type Match = fn(&AllowedOrigin, &Url) -> bool;

/// All of these hold for a URL the origin allows.
const MATCHES: [Match; 3] = [same_scheme, same_port, same_host];

fn same_scheme(o: &AllowedOrigin, url: &Url) -> bool {
    url.scheme() == o.scheme
}

fn same_port(o: &AllowedOrigin, url: &Url) -> bool {
    url.port_or_known_default() == Some(o.port)
}

/// Hosts without a host (`javascript:`, `data:`) never match.
fn same_host(o: &AllowedOrigin, url: &Url) -> bool {
    url.host_str().is_some_and(|h| o.host_matches(h))
}

/// The host split by [`wildcard`], when the URL is an origin and nothing more.
fn origin_host(url: &Url) -> Option<(bool, &str)> {
    url.host_str()
        .filter(|_| origin_only(url))
        .and_then(wildcard)
}

fn origin_only(url: &Url) -> bool {
    ORIGIN_ONLY.iter().all(|part| part(url))
}

/// Splits a leading `*.` off: (whether it was there, the rest). `None` when
/// the rest is empty or still has a `*`.
fn wildcard(host: &str) -> Option<(bool, &str)> {
    let (subdomains, rest) = host.strip_prefix("*.").map_or((false, host), |r| (true, r));
    (!rest.is_empty() && !rest.contains('*')).then_some((subdomains, rest))
}

/// `host` is `<one or more characters>.<base>`.
fn is_subdomain(host: &str, base: &str) -> bool {
    host.strip_suffix(base)
        .and_then(|p| p.strip_suffix('.'))
        .is_some_and(|p| !p.is_empty())
}

/// The origins to judge with: `listed` (`target.allowed_origins`), or only
/// the origin of `start` (`target.url`) when the list is empty. A start
/// without a host or a known port allows nothing.
#[must_use]
pub fn allowed_or_start(listed: Vec<AllowedOrigin>, start: &Url) -> Vec<AllowedOrigin> {
    if listed.is_empty() {
        AllowedOrigin::of_url(start).into_iter().collect()
    } else {
        listed
    }
}

/// Whether any of `allowed` allows `url`. False when `allowed` is empty.
#[must_use]
pub fn is_allowed(url: &Url, allowed: &[AllowedOrigin]) -> bool {
    allowed.iter().any(|origin| origin.allows(url))
}

/// A navigation an operation caused.
#[derive(Clone, PartialEq, Eq)]
pub struct Navigation {
    /// The URL the operation asked for. Kept for the record, not judged.
    pub requested: Url,
    /// The final URL after redirects.
    pub landed: Url,
}

/// Whether the navigation ended outside the allowed origins.
#[must_use]
pub fn left_allowed(nav: &Navigation, allowed: &[AllowedOrigin]) -> bool {
    !is_allowed(&nav.landed, allowed)
}
