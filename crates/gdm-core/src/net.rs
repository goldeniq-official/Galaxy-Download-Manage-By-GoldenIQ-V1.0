// SPDX-License-Identifier: RPL-1.5
//! Network policy (F05): TLS through rustls + the Windows platform verifier only (W-03), SSRF checks on every
//! connection and redirect hop with no second DNS lookup (W-02), no https -> http, <= 10 redirects, and
//! credentials only for their own origin (W-01).
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// raised by the resolver / redirect policy; found again by walking the reqwest error's source chain
#[derive(Debug)]
pub struct Forbidden(pub String);
impl std::fmt::Display for Forbidden {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "forbidden by GDM network policy: {}", self.0)
    }
}
impl std::error::Error for Forbidden {}

/// scheme + host + port
pub type Origin = (String, String, u16);
pub fn origin(u: &Url) -> Origin {
    (u.scheme().to_string(), u.host_str().unwrap_or("").to_ascii_lowercase(), u.port_or_known_default().unwrap_or(0))
}

/// Private / special addresses need the user's intent: the host the user typed (or confirmed) is listed in
/// `allow_private`; a redirect or a DNS answer that points anywhere else private is refused.
#[derive(Clone, Debug, Default)]
pub struct Policy {
    pub allow_private: Vec<String>,
}

/// A host that names the local network by its form: a non-public IP literal, a single-label name, or a LAN
/// suffix. A download the user clicked in the browser on such a host may be private without the "Allow" step
/// (IDM never asks; NAS, router, intranet and dev-server downloads are common). A public-looking name that
/// resolves private still needs "Allow": the attacker's own domain could be re-pointed at the LAN between the
/// browser and GDM (DNS rebinding; council 2026-11-15). A public IP is not LAN (it may host an attacker's page).
pub fn lan_host(u: &Url) -> bool {
    match u.host() {
        Some(url::Host::Ipv4(v)) => lan_ip(IpAddr::V4(v)),
        Some(url::Host::Ipv6(v)) => lan_ip(IpAddr::V6(v)),
        Some(url::Host::Domain(d)) => {
            let d = d.trim_end_matches('.').to_ascii_lowercase();
            !d.is_empty() && (!d.contains('.') || [".local", ".lan", ".home.arpa", ".internal", ".localhost"].iter().any(|s| d.ends_with(s)))
        }
        None => false,
    }
}

/// private / loopback / ULA, but never link-local in any form: 169.254.169.254 is the cloud metadata service
fn lan_ip(ip: IpAddr) -> bool {
    !public(ip) && !link_local(ip)
}

/// link-local in any form, also hidden in an IPv6 address that carries an IPv4 one
fn link_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => v.is_link_local(),
        IpAddr::V6(v) => (v.segments()[0] & 0xffc0) == 0xfe80 || embedded_v4(v).is_some_and(|m| m.is_link_local()),
    }
}

/// the IPv4 address an IPv6 form carries: mapped ::ffff:a.b.c.d, compatible ::a.b.c.d, NAT64 64:ff9b::/96,
/// 6to4 2002:AABB:CCDD::/48
fn embedded_v4(v: std::net::Ipv6Addr) -> Option<std::net::Ipv4Addr> {
    let s = v.segments();
    let v4 = |hi: u16, lo: u16| std::net::Ipv4Addr::new((hi >> 8) as u8, hi as u8, (lo >> 8) as u8, lo as u8);
    if let Some(m) = v.to_ipv4_mapped() {
        return Some(m);
    }
    match s {
        [0, 0, 0, 0, 0, 0, hi, lo] if hi != 0 => Some(v4(hi, lo)),
        [0x64, 0xff9b, 0, 0, 0, 0, hi, lo] => Some(v4(hi, lo)),
        [0x2002, hi, lo, ..] => Some(v4(hi, lo)),
        _ => None,
    }
}

#[cfg(test)]
mod lan_tests {
    #[test]
    fn only_names_that_say_lan_count() {
        let l = |s: &str| super::lan_host(&url::Url::parse(s).unwrap());
        for n in [
            "http://169.254.169.254/latest/meta-data/",
            "http://[fe80::1]/x",
            "http://[::ffff:169.254.1.1]/x",
            "http://[64:ff9b::a9fe:a9fe]/x",
            "http://[::a9fe:a9fe]/x",
            "http://[2002:a9fe:a9fe::1]/x",
            "http://203.0.113.5/x",
            "http://8.8.8.8/x",
            "http://[2001:db8::1]/x",
        ] {
            assert!(!l(n), "{n}");
        }
        for y in [
            "http://192.168.1.10/a.zip",
            "http://127.0.0.1:8765/x",
            "http://[fd00::5]/x",
            "http://nas/f.iso",
            "http://localhost:3000/a",
            "http://router.lan/fw.bin",
            "http://pi.local/x",
            "http://box.home.arpa/x",
            "http://files.corp.internal/x",
            "http://NAS.LOCAL./x",
        ] {
            assert!(l(y), "{y}");
        }
        // an allowed name never resolves into link-local; a typed link-local IP stays allowed
        let p = super::Policy { allow_private: vec!["nas.local".into(), "169.254.1.5".into()] };
        let sa = |s: &str| vec![std::net::SocketAddr::new(s.parse().unwrap(), 80)];
        assert!(p.check("nas.local", &sa("192.168.1.9")).is_ok());
        assert!(p.check("nas.local", &sa("169.254.169.254")).is_err());
        assert!(p.check("169.254.1.5", &sa("169.254.1.5")).is_ok());
        for n in [
            "https://example.com/a.zip",
            "http://evil.example/x",
            "http://local.example.com/x",
            "http://lan.attacker.io/x",
            "http://xn--lan-9za.com/",
        ] {
            assert!(!l(n), "{n}");
        }
    }
}

pub fn public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            let o = v.octets();
            !(v.is_loopback() || v.is_private() || v.is_link_local() || v.is_unspecified() || v.is_multicast() || v.is_broadcast()
                || o[0] == 0
                || (o[0] == 100 && (o[1] & 0xC0) == 64)   // CGNAT 100.64/10
                || o[0] >= 240                            // reserved
                || (o[0] == 192 && o[1] == 0 && o[2] == 0)) // IETF protocol assignments
        }
        IpAddr::V6(v) => {
            if let Some(m) = v.to_ipv4_mapped() {
                return public(IpAddr::V4(m));
            }
            let s = v.segments();
            if s[0] == 0x64 && s[1] == 0xff9b {
                return public(IpAddr::V4(std::net::Ipv4Addr::new((s[6] >> 8) as u8, s[6] as u8, (s[7] >> 8) as u8, s[7] as u8)));
            }
            !(v.is_loopback() || v.is_unspecified() || v.is_multicast()
                || (s[0] & 0xfe00) == 0xfc00   // ULA
                || (s[0] & 0xffc0) == 0xfe80   // link-local
                || (s[0] == 0 && s[1] == 0 && s[2] == 0 && s[3] == 0 && s[4] == 0 && s[5] == 0))
            // v4-compatible
        }
    }
}

impl Policy {
    fn allowed_host(&self, host: &str) -> bool {
        let h = host.trim_start_matches('[').trim_end_matches(']').to_ascii_lowercase();
        self.allow_private.iter().any(|a| a.eq_ignore_ascii_case(&h))
    }
    /// every answer must pass: a mixed public/private answer is refused as a whole
    pub fn check(&self, host: &str, addrs: &[SocketAddr]) -> Result<(), Forbidden> {
        if addrs.is_empty() {
            return Err(Forbidden(format!("{host}: no address")));
        }
        if self.allowed_host(host) {
            // an allowed NAME never reaches link-local (169.254.169.254 = cloud metadata): a LAN name can be
            // answered by anyone on the LAN (mDNS); only an IP the user typed itself may be link-local
            let literal = host.trim_start_matches('[').trim_end_matches(']').parse::<IpAddr>().is_ok();
            return match addrs.iter().find(|a| !literal && link_local(a.ip())) {
                Some(a) => Err(Forbidden(format!("{host} -> {} (link-local address)", a.ip()))),
                None => Ok(()),
            };
        }
        match addrs.iter().find(|a| !public(a.ip())) {
            Some(a) => Err(Forbidden(format!("{host} -> {} (private or special address)", a.ip()))),
            None => Ok(()),
        }
    }
    /// scheme + IP-literal host check (literals never reach the resolver)
    pub fn check_url(&self, u: &Url) -> Result<(), Forbidden> {
        if !matches!(u.scheme(), "http" | "https") {
            return Err(Forbidden(format!("scheme {}", u.scheme())));
        }
        if !u.username().is_empty() || u.password().is_some() {
            return Err(Forbidden("credentials in URL".into()));
        }
        let port = u.port_or_known_default().unwrap_or(0);
        match u.host() {
            Some(url::Host::Ipv4(ip)) => self.check(&ip.to_string(), &[SocketAddr::new(ip.into(), port)]),
            Some(url::Host::Ipv6(ip)) => self.check(&ip.to_string(), &[SocketAddr::new(ip.into(), port)]),
            Some(url::Host::Domain(_)) => Ok(()),
            None => Err(Forbidden("no host".into())),
        }
    }
}

struct Resolver(Arc<Policy>);
impl reqwest::dns::Resolve for Resolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let p = self.0.clone();
        let host = name.as_str().to_string();
        Box::pin(async move {
            // one lookup; reqwest connects only to these checked addresses (DNS rebinding cannot swap them)
            let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0)).await?.collect();
            p.check(&host, &addrs)?;
            Ok(Box::new(addrs.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

/// rustls ClientConfig: ring provider owned by this config (no process-global provider), TLS 1.2/1.3 only,
/// certificates checked by the Windows platform verifier (corporate CAs in the Windows store work).
pub fn tls_config(alpn: &[&[u8]]) -> Result<rustls::ClientConfig, rustls::Error> {
    use rustls_platform_verifier::BuilderVerifierExt;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut c = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?
        .with_platform_verifier()?
        .with_no_client_auth();
    c.alpn_protocols = alpn.iter().map(|a| a.to_vec()).collect();
    Ok(c)
}

/// `h1_only`: the segmenter uses one TCP per segment like IDM (G3: over h2 all segments became streams on one
/// TCP, 9.9 MB/s). Otherwise h2 is offered by ALPN.
pub fn client(policy: Arc<Policy>, h1_only: bool, read_timeout: Duration) -> Result<reqwest::Client, String> {
    client_ua(policy, h1_only, read_timeout, None)
}

pub const DEFAULT_UA: &str = concat!("GalaxyDownloadManager/", env!("CARGO_PKG_VERSION"));

/// `client` with a User-Agent (None = GDM's own, never empty)
pub fn client_ua(policy: Arc<Policy>, h1_only: bool, read_timeout: Duration, ua: Option<&str>) -> Result<reqwest::Client, String> {
    build(policy, h1_only, read_timeout, ua, true)
}

/// the same client, but redirects are returned to the caller (site grabber F09: every hop is checked against
/// the project's scope, avoid list and robots.txt before it is requested; the caller applies `redirect_ok`)
pub fn client_manual(policy: Arc<Policy>, read_timeout: Duration, ua: Option<&str>) -> Result<reqwest::Client, String> {
    build(policy, false, read_timeout, ua, false)
}

fn build(policy: Arc<Policy>, h1_only: bool, read_timeout: Duration, ua: Option<&str>, follow: bool) -> Result<reqwest::Client, String> {
    let tls = tls_config(if h1_only { &[b"http/1.1"] } else { &[b"h2", b"http/1.1"] }).map_err(|e| e.to_string())?;
    let p2 = policy.clone();
    let relay = crate::relay::global();
    let ua = ua.filter(|u| !u.trim().is_empty() && u.len() <= 512 && !u.chars().any(|c| c.is_control())).unwrap_or(DEFAULT_UA);
    let mut b = reqwest::Client::builder().tls_backend_preconfigured(tls).no_proxy().user_agent(ua);
    if let Some(r) = &relay {
        // a user proxy (W-05): every request goes through GDM's own relay, which resolves and checks the
        // destination itself (W-02) and hands the upstream proxy only the checked IP (council G7)
        let (u, pw) = r.register(&policy);
        b = b.proxy(reqwest::Proxy::all(r.url()).map_err(|e| e.to_string())?.basic_auth(&u, &pw));
    }
    let mut b = b
        .dns_resolver(Arc::new(Resolver(policy)))
        .read_timeout(read_timeout)
        .connect_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(crate::seg::MAX_CONNS)
        .redirect(if follow {
            reqwest::redirect::Policy::custom(move |att| match redirect_ok(att.previous(), att.url(), &p2) {
                Ok(()) => att.follow(),
                Err(e) => att.error(e),
            })
        } else {
            reqwest::redirect::Policy::none()
        });
    if h1_only {
        b = b.http1_only();
    }
    b.build().map_err(|e| e.to_string())
}

/// one redirect hop: <= 10 hops, never https -> http, scheme + IP-literal policy on the new URL
pub fn redirect_ok(previous: &[Url], next: &Url, p: &Policy) -> Result<(), Forbidden> {
    if previous.len() >= 10 {
        return Err(Forbidden("more than 10 redirects".into()));
    }
    if previous.iter().any(|u| u.scheme() == "https") && next.scheme() != "https" {
        return Err(Forbidden("https -> http redirect".into()));
    }
    p.check_url(next)
}

/// credentials (Cookie / Authorization) captured for one origin; attached only to requests to that origin.
/// reqwest additionally strips them on a cross-host redirect, and we never follow https -> http.
#[derive(Clone, Debug, Default)]
pub struct Creds {
    pub origin: Option<Origin>,
    pub headers: Vec<(String, String)>,
    /// the browser's User-Agent for a download caught from it (IDM parity: a Cloudflare `cf_clearance` cookie
    /// is only valid with the UA it was issued to); used for every request of the job, memory only
    pub ua: Option<String>,
    /// a site login answered with Digest (RFC 7616): a fresh Authorization per request (nonce count + 1)
    pub digest: Option<Arc<Digest>>,
}

#[derive(Debug)]
pub struct Digest {
    pub challenge: gdm_auth::Challenge,
    pub user: String,
    pub pass: String,
    pub nc: std::sync::atomic::AtomicU32,
    pub cnonce: String,
}

impl Creds {
    /// split a "User-Agent" pair out of credential headers (the browser handoff carries it with the cookie)
    pub fn new(origin: Origin, headers: Vec<(String, String)>) -> Creds {
        let (ua, headers): (Vec<_>, Vec<_>) = headers.into_iter().partition(|(k, _)| k.eq_ignore_ascii_case("user-agent"));
        Creds { origin: Some(origin), headers, ua: ua.into_iter().next().map(|(_, v)| v), digest: None }
    }

    /// every header for a GET of `u` (nothing for another origin)
    pub fn headers_for(&self, u: &Url) -> Vec<(String, String)> {
        let mut v = self.for_url(u).to_vec();
        if let (Some(d), true) = (&self.digest, self.origin.as_ref() == Some(&origin(u))) {
            let n = d.nc.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            let uri = match u.query() {
                Some(q) => format!("{}?{q}", u.path()),
                None => u.path().to_string(),
            };
            v.retain(|(k, _)| !k.eq_ignore_ascii_case("authorization"));
            v.push(("Authorization".into(), gdm_auth::authorization(&d.challenge, &d.user, &d.pass, "GET", &uri, n, &d.cnonce)));
        }
        v
    }

    pub fn for_url(&self, u: &Url) -> &[(String, String)] {
        match &self.origin {
            Some(o) if *o == origin(u) => &self.headers,
            _ => &[],
        }
    }
}

/// every error in the chain, including errors wrapped inside io::Error (whose source() skips them)
fn chain<'a>(e: &'a (dyn std::error::Error + 'static)) -> Vec<&'a (dyn std::error::Error + 'static)> {
    let mut v = vec![];
    let mut s: Option<&(dyn std::error::Error + 'static)> = Some(e);
    while let Some(x) = s {
        v.push(x);
        s = match x.downcast_ref::<std::io::Error>().and_then(|io| io.get_ref()) {
            Some(inner) => Some(inner as &(dyn std::error::Error + 'static)),
            None => x.source(),
        };
        if v.len() > 32 {
            break;
        }
    }
    v
}

/// a policy refusal anywhere in the chain
pub fn forbidden(e: &(dyn std::error::Error + 'static)) -> Option<String> {
    chain(e).into_iter().find_map(|x| x.downcast_ref::<Forbidden>().map(|f| f.0.clone()))
}

/// TLS certificate errors (W-03) are reported as such, never retried into success
pub fn tls_error(e: &(dyn std::error::Error + 'static)) -> Option<String> {
    chain(e).into_iter().find_map(|x| x.downcast_ref::<rustls::Error>().map(|r| r.to_string()))
}
