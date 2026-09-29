//! Resolving the *client's* address behind a reverse proxy, without letting the
//! client choose it.
//!
//! # Why this exists
//!
//! `X-Forwarded-For` is a list the client can seed: a caller who sends
//! `X-Forwarded-For: 1.2.3.4` has that value at the **left** of the list, and
//! each proxy only appends the address it saw to the **right**. Reading the
//! leftmost entry — what both authenc's audit middleware and perlengkapan's
//! `ClientIp` did — therefore records whatever address the caller typed. In an
//! audit trail that is worse than recording nothing, because the forged value
//! looks plausible.
//!
//! The rule (MDN `X-Forwarded-For`; OWASP Logging; RFC 7239 §8):
//!
//! 1. Trust the header **only when the direct peer is a proxy you operate**.
//!    From any other peer the header is attacker-controlled and is ignored.
//! 2. Walk the list **from the right**, skipping every entry that is itself a
//!    trusted proxy. The first entry that is *not* trusted is the client — the
//!    closest address to the client that a trusted party observed.
//!
//! The set of trusted proxies is configuration, not code:
//! `TRUSTED_PROXY_CIDRS` (comma-separated CIDRs / addresses). **Unset means
//! "trust nobody"**, so the peer address is used and the header is ignored — the
//! safe default. In production it must name the ingress gateway's pod CIDR, or
//! every audit row will record the ingress as the user.
//!
//! Anything that does not parse as an address is discarded rather than stored:
//! `audit_logs.ip_address` is a Postgres `inet`, and a value that does not parse
//! as one makes the whole `INSERT` fail (verified against PostgreSQL 16), which
//! an unauthenticated caller could use to suppress its own audit row.

use ipnetwork::IpNetwork;
use std::net::{IpAddr, SocketAddr};
use std::sync::OnceLock;

/// Environment variable naming the proxies whose forwarding headers are trusted.
pub const TRUSTED_PROXY_ENV: &str = "TRUSTED_PROXY_CIDRS";

/// The set of reverse proxies whose `X-Forwarded-For` / `X-Real-IP` we believe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustedProxies(Vec<IpNetwork>);

impl TrustedProxies {
    /// Trust nobody. The default: forwarding headers are ignored.
    pub fn none() -> Self {
        Self::default()
    }

    /// Parse a comma/whitespace separated list of CIDRs or bare addresses.
    ///
    /// Entries that do not parse are **skipped**, never widened: a typo must
    /// shrink the trusted set, not grow it. (The number skipped is returned by
    /// [`parse_lossy`](Self::parse_lossy) for callers that want to warn.)
    pub fn parse(list: &str) -> Self {
        Self::parse_lossy(list).0
    }

    /// Like [`parse`](Self::parse), also returning how many entries were skipped.
    pub fn parse_lossy(list: &str) -> (Self, usize) {
        let mut nets = Vec::new();
        let mut skipped = 0;
        for token in list.split([',', ' ', '\n', '\t']).map(str::trim) {
            if token.is_empty() {
                continue;
            }
            match token.parse::<IpNetwork>() {
                Ok(net) => nets.push(net),
                Err(_) => skipped += 1,
            }
        }
        (Self(nets), skipped)
    }

    /// Load from [`TRUSTED_PROXY_ENV`]; empty/unset ⇒ trust nobody.
    pub fn from_env() -> Self {
        match std::env::var(TRUSTED_PROXY_ENV) {
            Ok(v) => {
                let (set, skipped) = Self::parse_lossy(&v);
                if skipped > 0 {
                    tracing::warn!(
                        skipped,
                        "{TRUSTED_PROXY_ENV}: ignored entries that are not CIDRs/addresses"
                    );
                }
                set
            }
            Err(_) => Self::none(),
        }
    }

    /// Process-wide set, loaded once from the environment.
    pub fn global() -> &'static TrustedProxies {
        static GLOBAL: OnceLock<TrustedProxies> = OnceLock::new();
        GLOBAL.get_or_init(TrustedProxies::from_env)
    }

    /// True when `ip` is one of the trusted proxies.
    pub fn is_trusted(&self, ip: IpAddr) -> bool {
        self.0.iter().any(|net| net.contains(ip))
    }

    /// True when no proxy is trusted (forwarding headers will be ignored).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Parse one header token as an address, tolerating the shapes proxies emit:
/// `1.2.3.4`, `1.2.3.4:5678`, `::1`, `[::1]`, `[2001:db8::1]:443`.
///
/// Returns `None` for anything else (`unknown`, `_hidden`, obfuscated
/// identifiers, garbage) — see the module docs for why that must never be
/// stored.
pub fn parse_ip_token(token: &str) -> Option<IpAddr> {
    let t = token.trim().trim_matches('"');
    if t.is_empty() {
        return None;
    }
    if let Ok(ip) = t.parse::<IpAddr>() {
        return Some(ip);
    }
    // `ip:port` / `[v6]:port` / `[v6]`
    if let Ok(sock) = t.parse::<SocketAddr>() {
        return Some(sock.ip());
    }
    if let Some(inner) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        return inner.parse::<IpAddr>().ok();
    }
    None
}

/// The address to attribute a request to.
///
/// * `peer` — the TCP peer (`ConnectInfo`). Always known, never spoofable.
/// * `xff` — the raw `X-Forwarded-For` header, if any.
/// * `x_real_ip` — the raw `X-Real-IP` header, if any (single-value proxies).
///
/// If `peer` is not a trusted proxy, the headers are ignored and `peer` is
/// returned. Otherwise the client is the first non-trusted entry counted from
/// the right of `xff`; an entry that does not parse ends the walk (we cannot
/// tell how much of the list to believe past it) and yields the last address
/// that *was* verified, i.e. the proxy hop nearest to it.
pub fn resolve_client_ip(
    peer: IpAddr,
    xff: Option<&str>,
    x_real_ip: Option<&str>,
    trusted: &TrustedProxies,
) -> IpAddr {
    if !trusted.is_trusted(peer) {
        return peer;
    }

    if let Some(list) = xff {
        let mut nearest = peer;
        for token in list.rsplit(',') {
            let Some(ip) = parse_ip_token(token) else {
                // Unparseable entry: stop, do not skip past it.
                return nearest;
            };
            if trusted.is_trusted(ip) {
                nearest = ip;
                continue;
            }
            return ip;
        }
        // Every hop was one of ours; the leftmost is the best we can say.
        return nearest;
    }

    // Single-value proxies. Only consulted when the peer is trusted.
    x_real_ip
        .and_then(parse_ip_token)
        .filter(|ip| !trusted.is_trusted(*ip))
        .unwrap_or(peer)
}

/// `resolve_client_ip` over the process-wide [`TrustedProxies`], returning the
/// text form stored in audit rows.
pub fn client_ip_string(peer: IpAddr, xff: Option<&str>, x_real_ip: Option<&str>) -> String {
    resolve_client_ip(peer, xff, x_real_ip, TrustedProxies::global()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn proxies(list: &str) -> TrustedProxies {
        TrustedProxies::parse(list)
    }

    #[test]
    fn untrusted_peer_cannot_choose_its_address() {
        // The whole point: a client talking directly (or via an untrusted hop)
        // sets X-Forwarded-For, and it must be ignored.
        let trusted = proxies("10.0.0.0/8");
        let got = resolve_client_ip(
            ip("203.0.113.9"),
            Some("1.2.3.4, 5.6.7.8"),
            Some("9.9.9.9"),
            &trusted,
        );
        assert_eq!(got, ip("203.0.113.9"));
    }

    #[test]
    fn empty_trust_set_ignores_headers_entirely() {
        let got = resolve_client_ip(
            ip("10.1.2.3"),
            Some("1.2.3.4"),
            None,
            &TrustedProxies::none(),
        );
        assert_eq!(
            got,
            ip("10.1.2.3"),
            "unset TRUSTED_PROXY_CIDRS must trust nobody"
        );
    }

    #[test]
    fn leftmost_entry_is_not_believed() {
        // Ingress 10.0.0.5 appended the real client (198.51.100.7). The client
        // had pre-seeded 1.2.3.4. Reading the LEFT would record the forgery.
        let trusted = proxies("10.0.0.0/8");
        let got = resolve_client_ip(
            ip("10.0.0.5"),
            Some("1.2.3.4, 198.51.100.7"),
            None,
            &trusted,
        );
        assert_eq!(got, ip("198.51.100.7"));
    }

    #[test]
    fn walks_past_multiple_trusted_hops_from_the_right() {
        let trusted = proxies("10.0.0.0/8, 172.16.0.0/12");
        let got = resolve_client_ip(
            ip("10.0.0.5"),
            Some("198.51.100.7, 172.16.4.4, 10.9.9.9"),
            None,
            &trusted,
        );
        assert_eq!(got, ip("198.51.100.7"));
    }

    #[test]
    fn all_hops_trusted_falls_back_to_the_nearest_proxy() {
        let trusted = proxies("10.0.0.0/8");
        let got = resolve_client_ip(ip("10.0.0.5"), Some("10.1.1.1, 10.2.2.2"), None, &trusted);
        assert_eq!(got, ip("10.1.1.1"));
    }

    #[test]
    fn garbage_entry_stops_the_walk_instead_of_being_stored() {
        let trusted = proxies("10.0.0.0/8");
        // The rightmost hop is ours; the entry beyond it is not an address.
        let got = resolve_client_ip(ip("10.0.0.5"), Some("x, 10.3.3.3"), None, &trusted);
        assert_eq!(
            got,
            ip("10.3.3.3"),
            "must return the last VERIFIED hop, not 'x'"
        );
        // And a lone garbage header yields the peer, never garbage.
        let got = resolve_client_ip(ip("10.0.0.5"), Some("unknown"), None, &trusted);
        assert_eq!(got, ip("10.0.0.5"));
    }

    #[test]
    fn x_real_ip_only_counts_from_a_trusted_peer() {
        let trusted = proxies("10.0.0.0/8");
        assert_eq!(
            resolve_client_ip(ip("10.0.0.5"), None, Some("198.51.100.7"), &trusted),
            ip("198.51.100.7")
        );
        assert_eq!(
            resolve_client_ip(ip("203.0.113.9"), None, Some("198.51.100.7"), &trusted),
            ip("203.0.113.9")
        );
    }

    #[test]
    fn tokens_are_parsed_in_the_shapes_proxies_emit() {
        assert_eq!(parse_ip_token("1.2.3.4"), Some(ip("1.2.3.4")));
        assert_eq!(parse_ip_token(" 1.2.3.4:5678 "), Some(ip("1.2.3.4")));
        assert_eq!(parse_ip_token("::1"), Some(ip("::1")));
        assert_eq!(parse_ip_token("[::1]"), Some(ip("::1")));
        assert_eq!(parse_ip_token("[2001:db8::1]:443"), Some(ip("2001:db8::1")));
        assert_eq!(parse_ip_token("\"1.2.3.4\""), Some(ip("1.2.3.4")));
        for bad in [
            "",
            "unknown",
            "garbage",
            "1.2.3",
            "_hidden",
            "1.2.3.4, 5.6.7.8",
            "999.1.1.1",
        ] {
            assert_eq!(parse_ip_token(bad), None, "{bad:?} must not parse");
        }
    }

    #[test]
    fn a_typo_in_the_trust_list_shrinks_it() {
        let (set, skipped) = TrustedProxies::parse_lossy("10.0.0.0/8, not-a-cidr, 300.1.1.1/8");
        assert_eq!(skipped, 2);
        assert!(set.is_trusted(ip("10.4.4.4")));
        assert!(!set.is_trusted(ip("192.168.1.1")));
        assert!(TrustedProxies::parse("").is_empty());
    }

    #[test]
    fn every_string_we_can_return_is_a_valid_inet() {
        // The audit column is `inet`; whatever `client_ip_string` yields must
        // round-trip through IpAddr, or the INSERT fails.
        let trusted = proxies("10.0.0.0/8");
        for xff in [
            "1.2.3.4",
            "1.2.3.4:80",
            "garbage",
            "[::1]:1",
            "x, y, z",
            "",
            "a,b,,,c",
        ] {
            let got = resolve_client_ip(ip("10.0.0.5"), Some(xff), None, &trusted);
            assert!(got.to_string().parse::<IpAddr>().is_ok());
        }
    }
}
