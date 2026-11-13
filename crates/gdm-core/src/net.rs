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
            return Ok(());
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
        .redirect(reqwest::redirect::Policy::custom(move |att| match redirect_ok(att.previous(), att.url(), &p2) {
            Ok(()) => att.follow(),
            Err(e) => att.error(e),
        }));
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
