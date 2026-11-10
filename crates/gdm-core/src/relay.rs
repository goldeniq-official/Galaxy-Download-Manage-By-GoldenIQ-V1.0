// SPDX-License-Identifier: RPL-1.5
//! Policy relay (council G7, ADR-6, W-02/W-05): a loopback proxy INSIDE GDM. reqwest (and yt-dlp, ADR-6) talk
//! to it; it resolves every destination itself with the W-02 checks and reaches it only by the checked IP:
//! directly, through the user's HTTP proxy as `CONNECT <ip>:<port>`, or through SOCKS5 with an IP address, so
//! no upstream proxy ever resolves a name for GDM. TLS stays end to end (reqwest keeps the hostname for SNI and
//! the certificate check). Plain http is tunnelled the same way (CONNECT ip:80, then the request in origin
//! form). Proxy auth (Basic, NTLM/Negotiate) happens on the relay's own upstream socket.
//! Each client gets its own credentials on the relay, bound to its job's policy (allow_private hosts).
use crate::net::Policy;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// how one destination is reached
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Direct,
    /// an HTTP proxy (host:port) and how to log in to it
    Http(String, ProxyAuth),
    /// SOCKS5 proxy (host:port) and optional user/password (RFC 1929)
    Socks5(String, Option<(String, String)>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProxyAuth {
    None,
    /// `Proxy-Authorization: Basic ...` (user + password from Credential Manager)
    Basic(String),
    /// the logged-on user's Windows credentials over SSPI ("Negotiate" or "NTLM"), every leg on one socket
    Sspi(String),
}

/// destination URL ("https://host:port/" for a tunnel) -> routes to try in order
pub type Router = Arc<dyn Fn(&url::Url) -> Vec<Route> + Send + Sync>;

/// ports a download never needs and an open tunnel could abuse (mail, remote shells; RFC 9110 §9.3.6)
const BLOCKED_PORTS: &[u16] = &[0, 22, 23, 25, 110, 135, 139, 143, 445, 465, 587, 993, 995, 3389];
const MAX_HEAD: usize = 16 * 1024;
const MAX_TUNNELS: usize = 256;

pub struct Relay {
    pub addr: SocketAddr,
    secret: String,
    clients: Mutex<HashMap<String, Weak<Policy>>>,
    router: Mutex<Router>,
    active: std::sync::atomic::AtomicUsize,
    /// every CONNECT target the relay sent upstream (tests: never a name)
    pub log: Mutex<Vec<String>>,
    /// requests (plain) and tunnels (CONNECT) this relay passed on
    pub served: std::sync::atomic::AtomicUsize,
}

fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    getrandom::fill(&mut b).expect("system random");
    crate::finalize::hex(&b)
}

fn b64(raw: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::new();
    for c in raw.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            s.push(if i <= c.len() { T[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    s
}

pub fn basic(user: &str, pass: &str) -> String {
    format!("Basic {}", b64(format!("{user}:{pass}").as_bytes()))
}

impl Relay {
    pub async fn start(router: Router) -> std::io::Result<Arc<Relay>> {
        let l = TcpListener::bind("127.0.0.1:0").await?;
        let r = Arc::new(Relay {
            addr: l.local_addr()?,
            secret: random_hex(16),
            clients: Default::default(),
            router: Mutex::new(router),
            active: Default::default(),
            log: Default::default(),
            served: Default::default(),
        });
        let me = Arc::downgrade(&r);
        tokio::spawn(async move {
            loop {
                let Ok((s, peer)) = l.accept().await else { continue };
                let Some(r) = me.upgrade() else { return };
                if !peer.ip().is_loopback() || r.active.load(std::sync::atomic::Ordering::Relaxed) >= MAX_TUNNELS {
                    continue;
                }
                tokio::spawn(async move {
                    r.active.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let _ = r.serve(s).await;
                    r.active.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                });
            }
        });
        Ok(r)
    }

    pub fn set_router(&self, router: Router) {
        *self.router.lock().unwrap_or_else(|p| p.into_inner()) = router;
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// credentials for one client, bound to its policy (dropped with the policy)
    pub fn register(&self, policy: &Arc<Policy>) -> (String, String) {
        let user = random_hex(8);
        let mut c = self.clients.lock().unwrap_or_else(|p| p.into_inner());
        c.retain(|_, w| w.strong_count() > 0);
        c.insert(user.clone(), Arc::downgrade(policy));
        (user, self.secret.clone())
    }

    fn policy_for(&self, auth: Option<&str>) -> Option<Arc<Policy>> {
        let v = auth?.strip_prefix("Basic ")?.trim();
        let c = self.clients.lock().unwrap_or_else(|p| p.into_inner());
        c.iter().find(|(u, _)| basic(u, &self.secret).strip_prefix("Basic ") == Some(v)).and_then(|(_, w)| w.upgrade())
    }

    async fn serve(&self, mut s: TcpStream) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1024);
        let head_end = loop {
            if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break i + 4;
            }
            if buf.len() > MAX_HEAD {
                return Ok(());
            }
            let mut t = [0u8; 4096];
            let n = tokio::time::timeout(Duration::from_secs(15), s.read(&mut t)).await.map_err(|_| std::io::ErrorKind::TimedOut)??;
            if n == 0 {
                return Ok(());
            }
            buf.extend_from_slice(&t[..n]);
        };
        let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
        let rest = buf[head_end..].to_vec();
        let mut lines = head.split("\r\n");
        let first = lines.next().unwrap_or("");
        let hdrs: Vec<(String, String)> =
            lines.filter_map(|l| l.split_once(':')).map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string())).collect();
        let hv = |k: &str| hdrs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
        let Some(policy) = self.policy_for(hv("proxy-authorization")) else {
            return s.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await;
        };
        let mut parts = first.split(' ');
        let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        // CONNECT host:port | GET http://host[:port]/path (plain http through the tunnel too)
        let (dest, plain) = if method == "CONNECT" {
            (url::Url::parse(&format!("https://{target}/")).ok(), None)
        } else {
            match url::Url::parse(target) {
                Ok(u) if u.scheme() == "http" && matches!(method, "GET" | "HEAD") => {
                    let path = match u.query() {
                        Some(q) => format!("{}?{q}", u.path()),
                        None => u.path().to_string(),
                    };
                    (Some(u.clone()), Some(path))
                }
                _ => (None, None),
            }
        };
        let Some(dest) = dest else { return s.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n").await };
        let (host, port) = (dest.host_str().unwrap_or("").to_string(), dest.port_or_known_default().unwrap_or(0));
        let refuse = |why: &str| {
            format!("HTTP/1.1 403 Forbidden\r\nX-GDM-Refused: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", why.replace(['\r', '\n'], " "))
        };
        if BLOCKED_PORTS.contains(&port) {
            return s.write_all(refuse("port").as_bytes()).await;
        }
        // W-02: one lookup, every answer checked; the upstream only ever gets this address
        let host_bare = host.trim_start_matches('[').trim_end_matches(']').to_string();
        let addrs: Vec<SocketAddr> = match tokio::net::lookup_host((host_bare.as_str(), port)).await {
            Ok(a) => a.collect(),
            Err(_) => vec![],
        };
        if let Err(e) = policy.check(&host_bare, &addrs) {
            return s.write_all(refuse(&e.0).as_bytes()).await;
        }
        let routes = (self.router.lock().unwrap_or_else(|p| p.into_inner()))(&dest);
        let routes = if routes.is_empty() { vec![Route::Direct] } else { routes };
        let mut up = None;
        let mut last_err = String::from("no route");
        // every checked address, IPv4 first (many proxies have no IPv6 route), for each route in order
        let mut order = addrs.clone();
        order.sort_by_key(|a| a.is_ipv6());
        'outer: for r in &routes {
            for ip in order.iter().take(8) {
                match self.open(r, *ip).await {
                    Ok(t) => {
                        up = Some(t);
                        break 'outer;
                    }
                    Err(e) => last_err = e,
                }
            }
        }
        let Some(mut up) = up else { return s.write_all(refuse(&format!("upstream: {last_err}")).as_bytes()).await };
        match plain {
            None => s.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").await?,
            Some(path) => {
                // origin-form request line; the client's Proxy-* headers never go upstream
                // one request per connection (security review G7): with keep-alive the client's next request,
                // carrying its Proxy-Authorization for this relay, would go upstream unparsed
                let mut h = format!("{method} {path} HTTP/1.1\r\n");
                for (k, v) in &hdrs {
                    if !k.starts_with("proxy-") && k != "connection" && k != "keep-alive" {
                        h.push_str(&format!("{k}: {v}\r\n"));
                    }
                }
                h.push_str("Connection: close\r\n\r\n");
                up.write_all(h.as_bytes()).await?;
            }
        }
        self.served.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tunnel = method == "CONNECT";
        if tunnel {
            if !rest.is_empty() {
                up.write_all(&rest).await?;
            }
            let _ = tokio::io::copy_bidirectional(&mut s, &mut up).await;
        } else {
            // plain http: exactly one bodiless request went upstream; anything else the client sends on this
            // connection is never forwarded (it would carry this relay's Proxy-Authorization). The response
            // head tells the client `Connection: close` (so it opens a new connection for its next request),
            // the body is passed on (exactly Content-Length bytes when given, else until the server closes)
            let mut b = Vec::new();
            let end = loop {
                if let Some(i) = b.windows(4).position(|w| w == b"\r\n\r\n") {
                    break i + 4;
                }
                if b.len() > MAX_HEAD {
                    return Ok(());
                }
                let mut t = [0u8; 4096];
                let n = up.read(&mut t).await?;
                if n == 0 {
                    return Ok(());
                }
                b.extend_from_slice(&t[..n]);
            };
            let head = String::from_utf8_lossy(&b[..end]).to_string();
            let mut out = String::new();
            let mut len: Option<u64> = None;
            for (i, l) in head.split("\r\n").enumerate() {
                if l.is_empty() {
                    continue;
                }
                let k = l.split(':').next().unwrap_or("").trim().to_ascii_lowercase();
                if i > 0 && (k == "connection" || k == "keep-alive" || k == "proxy-connection") {
                    continue;
                }
                if k == "content-length" {
                    len = l.split_once(':').and_then(|(_, v)| v.trim().parse().ok());
                }
                out.push_str(l);
                out.push_str("\r\n");
            }
            out.push_str("Connection: close\r\n\r\n");
            s.write_all(out.as_bytes()).await?;
            let body = &b[end..];
            let no_body = method == "HEAD" || head.split(' ').nth(1).is_some_and(|c| c == "204" || c == "304" || c.starts_with('1'));
            match (len, no_body) {
                (_, true) => {}
                (Some(n), _) => {
                    let first = body.len().min(n as usize);
                    s.write_all(&body[..first]).await?;
                    let mut rest = (&mut up).take(n - first as u64);
                    let _ = tokio::io::copy(&mut rest, &mut s).await;
                }
                (None, _) => {
                    s.write_all(body).await?;
                    let _ = tokio::io::copy(&mut up, &mut s).await;
                }
            }
            let _ = s.shutdown().await;
        }
        Ok(())
    }

    /// a byte stream to `ip` over one route
    async fn open(&self, r: &Route, ip: SocketAddr) -> Result<TcpStream, String> {
        let conn = |a: String| async move {
            tokio::time::timeout(Duration::from_secs(15), TcpStream::connect(a)).await.map_err(|_| "timeout".to_string())?.map_err(|e| e.to_string())
        };
        match r {
            Route::Direct => conn(ip.to_string()).await,
            Route::Http(proxy, auth) => {
                let authority = ip.to_string(); // "1.2.3.4:443" or "[::1]:443"
                self.log.lock().unwrap_or_else(|p| p.into_inner()).push(authority.clone());
                let mut t = conn(proxy.clone()).await?;
                let first = match auth {
                    ProxyAuth::Basic(b) => Some(b.clone()),
                    _ => None,
                };
                let (mut status, mut hdrs) = connect_once(&mut t, &authority, first.as_deref()).await?;
                if status == 407 {
                    if let ProxyAuth::Sspi(pkg) = auth {
                        // the handshake stays on THIS socket (NTLM is connection-bound); a proxy that closes it
                        // after the 407 gets a new socket and the handshake starts again from the first leg
                        let spn = format!("HTTP/{}", proxy.rsplit_once(':').map(|(h, _)| h).unwrap_or(proxy));
                        for attempt in 0..2 {
                            if attempt == 1 || hdrs.iter().any(|(k, v)| k == "connection" && v.eq_ignore_ascii_case("close")) {
                                t = conn(proxy.clone()).await?;
                            }
                            let mut c = gdm_auth::SspiClient::new(pkg, &spn)?;
                            let mut tok = match c.step(None)? {
                                gdm_auth::Step::Continue(t) => t,
                                gdm_auth::Step::Done(t) => t.unwrap_or_default(),
                            };
                            let mut legs = 0;
                            loop {
                                legs += 1;
                                let v = gdm_auth::proxy_header_value(pkg, &tok);
                                match connect_once(&mut t, &authority, Some(&v)).await {
                                    Ok((st, h)) => {
                                        (status, hdrs) = (st, h);
                                    }
                                    Err(_) => break, // closed mid-handshake: start over on a new socket
                                }
                                if status != 407 || legs > 4 {
                                    break;
                                }
                                let chal =
                                    hdrs.iter().filter(|(k, _)| k == "proxy-authenticate").find_map(|(_, v)| gdm_auth::parse_proxy_challenge(v, pkg));
                                let Some(chal) = chal else { break };
                                tok = match c.step(Some(&chal))? {
                                    gdm_auth::Step::Continue(t) => t,
                                    gdm_auth::Step::Done(Some(t)) => t,
                                    gdm_auth::Step::Done(None) => break,
                                };
                            }
                            if status == 200 {
                                break;
                            }
                        }
                    }
                }
                if status == 200 {
                    Ok(t)
                } else {
                    Err(format!("proxy answered {status} to CONNECT {authority} (it may refuse IP addresses)"))
                }
            }
            Route::Socks5(proxy, auth) => {
                let mut t = conn(proxy.clone()).await?;
                let e = |e: std::io::Error| e.to_string();
                t.write_all(if auth.is_some() { &[5, 1, 2] } else { &[5, 1, 0] }).await.map_err(e)?;
                let mut m = [0u8; 2];
                t.read_exact(&mut m).await.map_err(e)?;
                if m[0] != 5 || m[1] == 0xFF {
                    return Err("socks: no acceptable method".into());
                }
                if m[1] == 2 {
                    let (u, p) = auth.clone().ok_or("socks: wants a login")?;
                    let mut a = vec![1, u.len() as u8];
                    a.extend(u.as_bytes());
                    a.push(p.len() as u8);
                    a.extend(p.as_bytes());
                    t.write_all(&a).await.map_err(e)?;
                    let mut r = [0u8; 2];
                    t.read_exact(&mut r).await.map_err(e)?;
                    if r[1] != 0 {
                        return Err("socks: login refused".into());
                    }
                }
                // CONNECT with an IP address type, never a domain (RFC 1928 §5)
                let mut q = vec![5, 1, 0];
                match ip {
                    SocketAddr::V4(v) => {
                        q.push(1);
                        q.extend(v.ip().octets());
                    }
                    SocketAddr::V6(v) => {
                        q.push(4);
                        q.extend(v.ip().octets());
                    }
                }
                q.extend(ip.port().to_be_bytes());
                self.log.lock().unwrap_or_else(|p| p.into_inner()).push(format!("socks5 {ip}"));
                t.write_all(&q).await.map_err(e)?;
                let mut h = [0u8; 4];
                t.read_exact(&mut h).await.map_err(e)?;
                if h[1] != 0 {
                    return Err(format!("socks: connect refused ({})", h[1]));
                }
                let skip = match h[3] {
                    1 => 4 + 2,
                    4 => 16 + 2,
                    3 => {
                        let mut l = [0u8; 1];
                        t.read_exact(&mut l).await.map_err(e)?;
                        l[0] as usize + 2
                    }
                    _ => return Err("socks: bad reply".into()),
                };
                let mut sk = vec![0u8; skip];
                t.read_exact(&mut sk).await.map_err(e)?;
                Ok(t)
            }
        }
    }
}

/// one `CONNECT` on an open proxy socket: status + headers; a 407 body is read off so the socket can be reused
async fn connect_once(t: &mut TcpStream, authority: &str, auth: Option<&str>) -> Result<(u16, Vec<(String, String)>), String> {
    let mut req = format!("CONNECT {authority} HTTP/1.1\r\nHost: {authority}\r\n");
    if let Some(a) = auth {
        req.push_str(&format!("Proxy-Authorization: {a}\r\n"));
    }
    req.push_str("\r\n");
    t.write_all(req.as_bytes()).await.map_err(|e| e.to_string())?;
    let mut b = Vec::new();
    loop {
        let mut c = [0u8; 1];
        if t.read(&mut c).await.map_err(|e| e.to_string())? == 0 || b.len() > MAX_HEAD {
            return Err("proxy closed".into());
        }
        b.push(c[0]);
        if b.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let head = String::from_utf8_lossy(&b).to_string();
    let mut lines = head.split("\r\n");
    let status = lines.next().and_then(|l| l.split(' ').nth(1)).and_then(|c| c.parse().ok()).unwrap_or(0);
    let hdrs: Vec<(String, String)> =
        lines.filter_map(|l| l.split_once(':')).map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string())).collect();
    if status != 200 {
        let n: usize = hdrs.iter().find(|(k, _)| k == "content-length").and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
        let mut body = vec![0u8; n.min(1 << 16)];
        t.read_exact(&mut body).await.map_err(|e| e.to_string())?;
    }
    Ok((status, hdrs))
}

/// the process-wide relay reqwest clients use when the user configured a proxy (None = direct, as before)
static RELAY: Mutex<Option<Arc<Relay>>> = Mutex::new(None);

pub fn set_global(r: Option<Arc<Relay>>) {
    *RELAY.lock().unwrap_or_else(|p| p.into_inner()) = r;
}

pub fn global() -> Option<Arc<Relay>> {
    RELAY.lock().unwrap_or_else(|p| p.into_inner()).clone()
}

/// "host:port" with a default port
pub fn hostport(s: &str, default_port: u16) -> String {
    let s = s.trim().trim_start_matches("http://").trim_start_matches("socks5://").trim_end_matches('/');
    if s.rsplit_once(':').is_some_and(|(_, p)| p.parse::<u16>().is_ok()) && !s.ends_with(']') {
        s.to_string()
    } else {
        format!("{s}:{default_port}")
    }
}

/// bypass list (`*.corp.local`, `<local>` = names without a dot), like Windows/IDM
pub fn bypassed(list: &[String], host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    list.iter().any(|p| {
        let p = p.trim().to_ascii_lowercase();
        if p == "<local>" {
            return !h.contains('.');
        }
        match p.strip_prefix("*.") {
            Some(rest) => h == rest || h.ends_with(&format!(".{rest}")),
            None => p == h || (p.starts_with('*') && h.ends_with(p.trim_start_matches('*'))),
        }
    })
}
