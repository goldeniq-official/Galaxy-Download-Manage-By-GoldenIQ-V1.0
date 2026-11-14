// SPDX-License-Identifier: RPL-1.5
//! F09 site grabber (G9): cards G-01 (traps, limits, robots), G-02 (pages to avoid, GET only), G-03 (mirror
//! paths), G-04 (W-02 on every link, other domains by filter only), G-05 (static JS scan, nothing executed).
mod common;
use common::*;
#[cfg(windows)]
use gdm_core::fsx::Dir;
use gdm_core::grab::{self, Config, Report, Robots};
use gdm_fixture::{pct_encode, Fixture};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};
use url::Url;

async fn fx() -> Fixture {
    Fixture::start("127.0.0.1:0").await
}

/// typed loopback start, no spacing (the spacing test sets its own), small budgets
fn cfg(start: String) -> Config {
    Config { start, typed: true, delay_ms: 0, types: "*.zip, *.jpg".into(), ..Config::default() }
}

async fn run(c: &Config) -> Report {
    let stop = AtomicBool::new(false);
    tokio::time::timeout(Duration::from_secs(60), grab::crawl(c, &stop, &|_| {})).await.expect("crawl ends by itself").expect("crawl")
}

fn names(r: &Report) -> Vec<String> {
    let mut v: Vec<String> = r.files.iter().map(|f| f.name.clone()).collect();
    v.sort();
    v
}

fn paths(f: &Fixture) -> Vec<String> {
    f.log().iter().map(|q| q.target.clone()).collect()
}

const INDEX: &str = r#"<!doctype html><html><head><title>t</title></head><body>
<a href="page2.html">page 2</a> <img src='a.jpg' alt=x>
<a href="/logout">out</a> <a href="/LogOut?next=/">out2</a> <a href="/x/%6Cogout">out3</a>
<a href="/account/delete?confirm=1">delete account</a> <a href="/static/site/unsubscribe.html">bye</a>
<a href="/redirect?to=%2Flogout">sneaky redirect</a>
<a href="http://192.168.1.1/router.zip">router</a> <a href="http://[fd00::1]/ula.zip">ula</a>
<a href="http://10.0.0.1/ten.zip">ten</a>
<!-- <a href="/commented.zip">x</a> -->
<form action="/static/site/form-target.zip" method="post"><input type=submit></form>
<script>var u = "/static/site/js.zip"; fetch('/delete-everything'); document.cookie = "x";</script>
</body></html>"#;
const PAGE2: &str = r#"<a href="b.zip">b</a> <a href="b.zip#frag">b again</a> <a href="./b.zip">b 3</a>
<a href="../site/index.html">back</a> <a href="sub/c.zip">c</a>"#;

async fn site() -> Fixture {
    let f = fx().await;
    f.add_blob("site/index.html", INDEX.as_bytes().to_vec());
    f.add_blob("site/page2.html", PAGE2.as_bytes().to_vec());
    f.add_blob("site/unsubscribe.html", b"<a href='u.zip'>u</a>".to_vec());
    for n in ["a.jpg", "b.zip", "sub/c.zip", "js.zip", "u.zip"] {
        f.add_blob(&format!("site/{n}"), vec![7; 10]);
    }
    f
}

// ---------------- G-01 ----------------

#[tokio::test(flavor = "multi_thread")]
async fn g01_calendar_query_trap_stops_at_query_variant_cap() {
    let f = fx().await;
    let mut c = cfg(f.url("/cal?d=0"));
    c.depth = 20; // depth (max 20 => 41 pages) is a guard too; here the query-variant guard must stop it first
    c.limits.query_variants = 10;
    let r = run(&c).await;
    let hits = f.log().iter().filter(|q| q.target.starts_with("/cal")).count();
    assert!(hits <= 11, "{hits} calendar pages fetched (start + 10 variants)");
    assert!(r.skipped.iter().any(|s| s.reason == "query-variants"), "the trap guard fired: {:?}", r.skipped.first());
}

#[tokio::test(flavor = "multi_thread")]
async fn g01_page_budget_stops_the_crawl() {
    let f = fx().await;
    let mut c = cfg(f.url("/cal?d=0"));
    c.limits.pages = 7;
    c.depth = 20;
    c.limits.query_variants = 10_000;
    let r = run(&c).await;
    assert_eq!(r.stop, "pages");
    assert_eq!(r.pages, 7);
    assert_eq!(f.log().iter().filter(|q| q.target.starts_with("/cal")).count(), 7);
}

#[tokio::test(flavor = "multi_thread")]
async fn g01_path_loop_guard() {
    let f = fx().await;
    let mut c = cfg(f.url("/loop/"));
    c.depth = 20;
    let r = run(&c).await;
    let deepest = paths(&f).iter().map(|p| p.matches("a/").count()).max().unwrap_or(0);
    assert!(deepest < 3, "segment 'a' never repeated 3 times: {:?}", paths(&f));
    assert!(r.skipped.iter().any(|s| s.reason == "loop"));
}

#[tokio::test(flavor = "multi_thread")]
async fn g01_depth_time_and_byte_budgets() {
    let f = fx().await;
    // depth 2: d=0 (depth 0) -> d=1/-1 (1) -> d=2/-2... (2); nothing deeper
    let mut c = cfg(f.url("/cal?d=0"));
    c.depth = 2;
    run(&c).await;
    assert_eq!(f.log().iter().filter(|q| q.target.starts_with("/cal")).count(), 5, "{:?}", paths(&f));

    f.clear_log();
    let mut c = cfg(f.url("/cal?d=0"));
    c.limits.query_variants = 10_000;
    c.limits.bytes = 300;
    let r = run(&c).await;
    assert_eq!(r.stop, "bytes");
    assert!(r.bytes <= 300 + c.limits.page_bytes, "{}", r.bytes);

    let mut c = cfg(f.url("/cal?d=0"));
    c.limits.query_variants = 10_000;
    c.limits.secs = 1;
    c.delay_ms = 400;
    let t = Instant::now();
    let r = run(&c).await;
    assert_eq!(r.stop, "time");
    assert!(t.elapsed() < Duration::from_secs(4), "{:?}", t.elapsed());
}

#[tokio::test(flavor = "multi_thread")]
async fn g01_per_host_spacing_and_cancel() {
    let f = fx().await;
    let mut c = cfg(f.url("/cal?d=0"));
    c.delay_ms = 250;
    c.limits.pages = 4;
    run(&c).await;
    let ms: Vec<u128> = f.log().iter().map(|q| q.ms).collect();
    assert!(ms.len() >= 4, "{ms:?}");
    for w in ms.windows(2) {
        assert!(w[1] - w[0] >= 240, "requests to one host are spaced: {ms:?}");
    }
    // cancel during a rate wait
    let mut c = cfg(f.url("/cal?d=0"));
    c.delay_ms = 10_000;
    let stop = AtomicBool::new(false);
    let t = Instant::now();
    let (r, _) = tokio::join!(grab::crawl(&c, &stop, &|_| {}), async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        stop.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    assert_eq!(r.unwrap().stop, "cancel");
    assert!(t.elapsed() < Duration::from_secs(3));
}

#[test]
fn g01_hard_ceilings() {
    let mut l = grab::Limits { pages: 1_000_000, requests: u32::MAX, bytes: u64::MAX, secs: u64::MAX, ..Default::default() };
    l.clamp();
    assert_eq!(l.pages, 5_000, "blueprint G-01: pages <= 5,000");
    assert!(l.requests <= 20_000 && l.bytes <= 1 << 30 && l.secs <= 2 * 3600);
    let d = grab::Limits::default();
    assert_eq!((d.pages, d.query_variants), (1_000, 50));
    assert_eq!(Config::default().delay_ms, 1_000, "1 request/s per host by default");
    assert!(Config::default().robots, "robots.txt on by default");
}

#[tokio::test(flavor = "multi_thread")]
async fn g01_robots_disallow_is_never_requested() {
    let f = site().await;
    f.add_blob("robots.txt", b"User-agent: *\nDisallow: /static/site/page2\n".to_vec());
    let r = run(&cfg(f.url("/static/site/index.html"))).await;
    let p = paths(&f);
    assert_eq!(p[0], "/robots.txt", "robots first");
    assert!(!p.iter().any(|x| x.contains("page2")), "{p:?}");
    assert!(!names(&r).contains(&"b.zip".to_string()));
    assert!(r.skipped.iter().any(|s| s.reason == "robots"));
    // robots off: page2 is explored
    f.clear_log();
    let mut c = cfg(f.url("/static/site/index.html"));
    c.robots = false;
    let r = run(&c).await;
    assert!(!paths(&f).contains(&"/robots.txt".to_string()));
    assert!(names(&r).contains(&"b.zip".to_string()));
}

#[test]
fn g01_robots_rfc9309_matching() {
    let r = Robots::parse("User-agent: *\nDisallow: /private\nAllow: /private/ok\nDisallow: /*.php$\nDisallow: /a%3cd\n");
    let ok = |p: &str| r.allowed(&Url::parse(&format!("http://h{p}")).unwrap());
    assert!(!ok("/private/x"));
    assert!(ok("/private/ok/y"), "longest match wins");
    assert!(!ok("/index.php") && ok("/index.php?x=1") && ok("/index.phps"), "$ anchors, * wildcard");
    assert!(!ok("/a%3Cd"), "percent-encoding compared case-insensitively");
    assert!(ok("/robots.txt"));
    // equal length: Allow wins
    let r = Robots::parse("User-agent: *\nDisallow: /p\nAllow: /p\n");
    assert!(r.allowed(&Url::parse("http://h/p").unwrap()));
    // our own group replaces *, and groups with our token are merged
    let r = Robots::parse(
        "User-agent: *\nDisallow: /\n\nUser-agent: GalaxyDownloadManager\nDisallow: /x\n\nUser-agent: other\nUser-agent: galaxydownloadmanager\nDisallow: /y\n",
    );
    let ok = |p: &str| r.allowed(&Url::parse(&format!("http://h{p}")).unwrap());
    assert!(ok("/z") && !ok("/x") && !ok("/y"));
    // empty Disallow allows all; unknown lines ignored; garbage does not panic
    assert!(Robots::parse("User-agent: *\nDisallow:\nSitemap: x\n\u{0}\u{ffff}::").allowed(&Url::parse("http://h/a").unwrap()));
    assert!(!Robots::unreachable().allowed(&Url::parse("http://h/a").unwrap()));
}

// ---------------- G-02 ----------------

#[tokio::test(flavor = "multi_thread")]
async fn g02_robots_redirect_into_an_avoided_page_is_not_followed() {
    let f = site().await;
    f.add_blob("robots.redirect", b"/logout?robots".to_vec());
    let r = run(&cfg(f.url("/static/site/index.html"))).await;
    assert!(!paths(&f).iter().any(|p| p.contains("logout")), "{:?}", paths(&f));
    assert!(names(&r).contains(&"a.jpg".to_string()), "a refused robots hop = no robots.txt, the crawl goes on");
}

#[tokio::test(flavor = "multi_thread")]
async fn g02_logged_in_crawl_never_requests_avoided_pages() {
    let f = site().await;
    let mut c = cfg(f.url("/static/site/index.html"));
    c.js = true;
    c.creds = vec![("Authorization".into(), "Basic U0VDUkVU".into())];
    let r = run(&c).await;
    let p = paths(&f);
    for bad in ["logout", "LogOut", "%6Cogout", "delete", "unsubscribe", "/redirect"] {
        assert!(!p.iter().any(|x| x.contains(bad)), "{bad} was requested: {p:?}");
    }
    assert!(f.log().iter().all(|q| q.method == "GET"), "GET only");
    assert!(f.log().iter().filter(|q| q.target != "/robots.txt").all(|q| q.headers.iter().any(|(k, _)| k == "authorization")));
    assert!(r.skipped.iter().filter(|s| s.reason == "avoid").count() >= 5, "{:?}", r.skipped);
    assert!(!names(&r).contains(&"form-target.zip".to_string()), "forms are never submitted or followed");
}

#[tokio::test(flavor = "multi_thread")]
async fn g02_redirect_into_an_avoided_page_is_not_followed() {
    let f = fx().await;
    f.add_blob("r.html", format!("<a href=\"{}\">go</a>", "/static/hop.html").into_bytes());
    let mut c = cfg(f.url("/redirect?to=%2Fstatic%2Fr.html"));
    c.avoid = "hop.html".into(); // a user avoid pattern; the redirect start itself is allowed
    let r = run(&c).await;
    assert!(!paths(&f).iter().any(|p| p.contains("hop.html")), "{:?}", paths(&f));
    assert!(r.skipped.iter().any(|s| s.reason == "avoid"));
    // and a redirect whose Location is avoided is never requested
    f.clear_log();
    let r = run(&cfg(f.url("/redirect?to=%2Flogout"))).await;
    assert!(!paths(&f).iter().any(|p| p == "/logout"), "{:?}", paths(&f));
    assert!(r.skipped.iter().any(|s| s.reason == "avoid"));
}

#[test]
fn g02_avoid_matching() {
    let u = |s: &str| Url::parse(s).unwrap();
    for bad in
        ["http://h/logout", "http://h/Sign-Out", "http://h/a?action=delete", "http://h/%6C%6Fgout", "http://h/user/remove/1", "http://h/log-off"]
    {
        assert!(grab::avoided(&u(bad), &[]), "{bad}");
    }
    assert!(!grab::avoided(&u("http://h/files/report.zip"), &[]));
    assert!(grab::avoided(&u("http://h/cart/empty"), &["*cart/empty*".to_string()]));
}

// council G9 final review: robots.txt itself avoided, robots.txt cut short, redirects meet every page rule

#[tokio::test(flavor = "multi_thread")]
async fn g02_avoided_robots_txt_is_never_requested() {
    let f = site().await;
    let mut c = cfg(f.url("/static/site/index.html"));
    c.avoid = "robots.txt".into();
    let r = run(&c).await;
    assert!(!paths(&f).iter().any(|p| p.contains("robots")), "{:?}", paths(&f));
    assert!(names(&r).contains(&"a.jpg".to_string()));
}

#[tokio::test(flavor = "multi_thread")]
async fn g01_robots_cut_short_means_disallowed() {
    let f = site().await;
    f.add_blob("robots.cut", b"User-agent: *\nAllow: /\n".to_vec());
    let r = run(&cfg(f.url("/static/site/index.html"))).await;
    assert_eq!(paths(&f), ["/robots.txt"], "a broken-off robots.txt is not a permission");
    assert!(r.skipped.iter().any(|s| s.reason == "robots"));
}

#[tokio::test(flavor = "multi_thread")]
async fn g01_redirects_meet_path_filters_and_other_site_depth() {
    let f = site().await;
    let other = fx().await;
    other.add_blob("p.html", b"<a href='q.zip'>q</a>".to_vec());
    f.add_blob("secret.html", b"<a href='/static/s.zip'>s</a>".to_vec());
    let page = format!(
        "<a href=\"/redirect?to=%2Fstatic%2Fsecret.html\">x</a> <a href=\"/redirect?to={}\">y</a>",
        pct_encode(&format!("http://localhost:{}/static/p.html", other.addr.port()))
    );
    f.add_blob("site/index.html", page.into_bytes());
    let mut c = cfg(f.url("/static/site/index.html"));
    c.no_parent = false;
    c.exclude = "/static/secret*".into();
    c.domains = "localhost".into(); // allowed for files, but other-site depth stays 0
    let r = run(&c).await;
    assert!(!paths(&f).iter().any(|p| p.starts_with("/static/secret")), "{:?}", paths(&f));
    assert!(other.log().is_empty(), "other-site depth 0: a redirect does not explore it");
    assert!(r.skipped.iter().any(|s| s.reason == "filter"), "{:?}", r.skipped);
    assert!(r.skipped.iter().any(|s| s.reason == "depth"), "{:?}", r.skipped);
}

// ---------------- G-03 ----------------

#[test]
fn g03_mirror_segments_table() {
    let d = |s: &str| grab::mirror_dirs(&Url::parse(s).unwrap());
    // url::Url already resolves literal dot segments; percent-encoded ones survive parsing
    // WHATWG URL parsing already resolves %2e%2e / .%2e / %2E segments: nothing climbs above the host folder
    assert_eq!(d("http://h/a/%2e%2e/%2e%2e/x.zip"), ["h"]);
    assert_eq!(d("http://h/a/.%2E/b/%2E/y.zip"), ["h", "b"]);
    assert_eq!(d("http://h/a%2fb%5cc/z.zip"), ["h", "a_b_c"], "encoded separators never split");
    assert_eq!(d("http://h/%00evil/z.zip"), ["h", "_evil"]);
    assert_eq!(d("http://h/CON/z.zip")[1], "_CON", "reserved device name");
    assert_eq!(d("http://h/c%3A%5Cwin/z.zip")[1], "c__win");
    assert_eq!(d("http://h/dots.../z.zip")[1], "dots");
    assert_eq!(d("http://h/%252e%252e/z.zip")[1], "%2e%2e", "decoded once only");
    assert_eq!(d("http://h:8080/z.zip"), ["h_8080"], "port kept apart from the default-port host");
    assert_eq!(d("http://h/ខ្មែរ/z.zip")[1], "ខ្មែរ");
    assert!(d(&format!("http://h/{}/z.zip", "a".repeat(400)))[1].len() <= 255);
    assert_eq!(d("http://h/z.zip"), ["h"]);
    assert_eq!(d("http://h/a//b/z.zip"), ["h", "a", "b"], "empty segments dropped");
}

/// Windows only (junction, share mode); a symlinked mirror folder on Unix is refused by Dir::subdir (fsx_unix.rs)
#[cfg(windows)]
#[test]
fn g03_mirror_folders_are_held_and_junctions_refused() {
    let root = tmpdir("g03");
    let outside = tmpdir("g03-outside");
    let dl = root.join("dl");
    std::fs::create_dir_all(dl.join("h")).unwrap();
    let st = std::process::Command::new("cmd").args(["/c", "mklink", "/J"]).arg(dl.join("h").join("a")).arg(&outside).output().unwrap();
    assert!(st.status.success());
    let top = Dir::open(&dl).unwrap();
    let r = grab::mirror(&top, &Url::parse("http://h/a/b/z.zip").unwrap());
    assert!(r.is_err(), "junction in the tree refused");
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0, "nothing created outside the root");
    let held = grab::mirror(&top, &Url::parse("http://h/ok/b/z.zip").unwrap()).unwrap();
    assert_eq!(held.last().unwrap().path, dl.join("h").join("ok").join("b"));
    // held without FILE_SHARE_DELETE: the folder cannot be swapped while held
    assert!(std::fs::rename(dl.join("h").join("ok"), dl.join("h").join("ok2")).is_err());
    drop(held);
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&outside);
}

#[tokio::test(flavor = "multi_thread")]
async fn g03_results_become_paused_jobs_in_mirror_folders() {
    let f = site().await;
    let r = run(&cfg(f.url("/static/site/index.html"))).await;
    let t = engine("g03jobs", quick());
    let start = Url::parse(&f.url("/")).unwrap();
    let (jobs, held) = grab::jobs(&r.files, &t.dl, true, Some(start.host_str().unwrap())).unwrap();
    let ids = t.engine.add_many(jobs, true).unwrap();
    drop(held);
    assert_eq!(ids.len(), r.files.len());
    let c = ids.iter().map(|i| t.engine.job(i).unwrap()).find(|j| j.name == "c.zip").unwrap();
    assert!(c.dir.ends_with(std::path::Path::new(&format!("127.0.0.1_{}", f.addr.port())).join("static").join("site").join("sub")));
    assert_eq!(c.state, "paused");
    assert!(c.referrer.as_deref().unwrap_or("").ends_with("page2.html"), "the page that linked it");
    // one folder: everything flat in the chosen folder
    let (jobs, _h) = grab::jobs(&r.files, &t.dl, false, None).unwrap();
    let dl = std::fs::canonicalize(&t.dl).unwrap(); // macOS: /var/folders/... is /private/var/folders/...
    assert!(jobs.iter().all(|j| std::fs::canonicalize(&j.dir).unwrap() == dl && !j.typed));
}

// ---------------- G-04 ----------------

#[tokio::test(flavor = "multi_thread")]
async fn g04_private_and_foreign_links_are_skipped() {
    let f = site().await;
    let other = fx().await;
    other.add_blob("o.zip", vec![1]);
    let page = format!(
        "{INDEX}<a href=\"http://localhost:{}/static/o.zip\">other host</a> <a href=\"/redirect?to={}\">to private</a>",
        other.addr.port(),
        pct_encode("http://10.0.0.2/x.html")
    );
    f.add_blob("site/index.html", page.into_bytes());
    let r = run(&cfg(f.url("/static/site/index.html"))).await;
    let n = names(&r);
    for bad in ["router.zip", "ula.zip", "ten.zip", "o.zip"] {
        assert!(!n.contains(&bad.to_string()), "{bad} in {n:?}");
    }
    assert!(r.skipped.iter().any(|s| s.reason == "private"), "{:?}", r.skipped);
    assert!(r.skipped.iter().any(|s| s.reason == "other-domain"));
    assert!(other.log().is_empty(), "the other host was never contacted");
    assert_eq!(n, ["a.jpg", "b.zip", "c.zip"], "the real files are found (fragment/./ duplicates once)");
    // the domain filter lets the other host in (still W-02: it is loopback but not the typed host => refused)
    let mut c = cfg(f.url("/static/site/index.html"));
    c.domains = "localhost".into();
    let r = run(&c).await;
    assert!(names(&r).contains(&"o.zip".to_string()), "file links on a filtered domain are listed");
    assert!(other.log().is_empty(), "listing a file does not fetch it");
}

#[tokio::test(flavor = "multi_thread")]
async fn g04_untyped_private_start_is_refused() {
    let f = site().await;
    let mut c = cfg(f.url("/static/site/index.html"));
    c.typed = false;
    let r = grab::crawl(&c, &AtomicBool::new(false), &|_| {}).await;
    assert!(r.is_err() || r.as_ref().unwrap().files.is_empty(), "{r:?}");
    assert!(f.log().is_empty(), "never connected");
}

#[tokio::test(flavor = "multi_thread")]
async fn g04_credentials_stay_on_their_origin() {
    let a = site().await;
    let b = fx().await;
    b.add_blob("p.html", b"<a href='q.zip'>q</a>".to_vec());
    // same host (127.0.0.1), other port: inside the site scope, but another origin for credentials (W-01)
    a.add_blob("site/index.html", format!("<a href=\"{}\">b</a>", b.url("/static/p.html")).into_bytes());
    let mut c = cfg(a.url("/static/site/index.html"));
    c.creds = vec![("Authorization".into(), "Basic U0VDUkVU".into())];
    let r = run(&c).await;
    assert!(names(&r).contains(&"q.zip".to_string()));
    assert!(!b.log().is_empty());
    assert!(b.log().iter().all(|q| !q.headers.iter().any(|(k, _)| k == "authorization")));
    assert!(a.log().iter().any(|q| q.headers.iter().any(|(k, _)| k == "authorization")));
}

// ---------------- G-05 ----------------

#[test]
fn g05_static_js_scan_never_executes() {
    let base = Url::parse("http://h/dir/page.html").unwrap();
    let html = r#"<script>
      var a = "https://cdn.example/v/clip.mp4";
      var b = '/files/doc.pdf';
      var c = "/x/" + name + ".zip";          // computed: not followed
      var d = `/t/${id}.zip`;                  // template: not followed
      fetch("/api/delete.php");                // a literal: listed as a candidate, the avoid list keeps it out
      eval("location='/evil'");
      var e = "not a url"; var f = "/has space.zip"; var g = "\"/esc.zip\"";
    </script><style>body{background:url('bg.png')}</style><div style="background-image:url(&quot;s.gif&quot;)"></div>
    <img srcset="a-1x.jpg 1x, a,2x.jpg 2x, https://cdn.example/a-3x.jpg 3x" src=plain.jpg>
    <base href="http://h/other/"><a href=rel.zip>r</a> <a HREF = "x.zip?a=1&amp;b=2">x</a>
    <video poster="p.jpg"><source src="v.webm"></video> <a href="javascript:alert(1)">j</a> <a href="mailto:x@y">m</a>
    <!-- <a href="hidden.zip"> --><a href="data:text/plain,hi">d</a>"#;
    let got: Vec<String> = grab::links(html, &base, true).into_iter().map(|u| u.to_string()).collect();
    for want in [
        "https://cdn.example/v/clip.mp4",
        "http://h/files/doc.pdf",
        "http://h/api/delete.php",
        "http://h/dir/bg.png",
        "http://h/dir/s.gif",
        "http://h/dir/a-1x.jpg",
        "http://h/dir/a,2x.jpg",
        "https://cdn.example/a-3x.jpg",
        "http://h/dir/plain.jpg",
        "http://h/other/rel.zip",
        "http://h/other/x.zip?a=1&b=2",
        "http://h/other/p.jpg",
        "http://h/other/v.webm",
    ] {
        assert!(got.contains(&want.to_string()), "missing {want} in {got:?}");
    }
    for never in ["/x/", "/t/", "evil", "has space", "hidden.zip", "javascript", "mailto", "data:", "esc.zip"] {
        assert!(!got.iter().any(|g| g.contains(never)), "{never} in {got:?}");
    }
    // JS parsing off (default): script text is not scanned at all
    let off: Vec<String> = grab::links(html, &base, false).into_iter().map(|u| u.to_string()).collect();
    assert!(!off.iter().any(|g| g.contains("clip.mp4") || g.contains("doc.pdf")));
    assert!(off.contains(&"http://h/dir/bg.png".to_string()));
}

#[test]
fn g05_malformed_markup_does_not_panic() {
    let base = Url::parse("http://h/").unwrap();
    for s in ["<", "<a", "<a href=", "<a href=\"", "<script>", "<!--", "<a href='x.zip'", "&#xFFFFFFFF;<a href=&#x;>", "<a\u{0}href=y>", "<<<>>>"] {
        let _ = grab::links(s, &base, true);
    }
    // multi-byte text right after '&' (security review: no slicing inside a char)
    let _ = grab::links("<a href='/&ខ្មែរខ្មែរ.zip'>&ក្ខ</a>&", &base, true);
    let big = "<a href=\"/x.zip\">".repeat(100_000);
    assert_eq!(grab::links(&big, &base, false).len(), 1, "deduplicated");
}

#[test]
fn filters() {
    assert!(grab::glob("*.zip", "a.ZIP"));
    assert!(grab::glob("*", "anything"));
    assert!(!grab::glob("*.zip", "a.zip.exe"));
    assert!(grab::glob("/static/*/b*", "/static/site/b.zip"));
    let c = Config { types: "*.jpg ,*.png".into(), ..Config::default() };
    assert!(c.wants("x.PNG") && !c.wants("x.zip"));
}
