# §13 line-by-line review of reference code (v2, after council round 3) for the four most dangerous functions.
# The code is a specification (invariants), not shipped code; the real review gate runs on real commits.
import html as _h


def block(title, intro, src, notes):
    lines = src.strip("\n").split("\n")
    pre = "<pre>" + "\n".join("%2d  %s" % (i + 1, _h.escape(l)) for i, l in enumerate(lines)) + "</pre>"
    rows = "".join("<tr><td>%s</td><td>%s</td><td>%s</td></tr>" % (n, a, f) for n, a, f in notes)
    return ("<h3>%s</h3><p>%s</p>%s<div class=\"tw\"><table class=\"ln\"><thead><tr><th>បន្ទាត់</th><th>ការវាយប្រហារ / កំហុសដែលអាចកើត</th>"
            "<th>ការការពារក្នុងកូដគំរូ</th></tr></thead><tbody>%s</tbody></table></div>") % (title, intro, pre, rows)


SANITIZE = r"""
const FALLBACK: &str = "download";                                  // fixed, never caller-supplied
pub fn sanitize_filename(name: &str) -> String {                    // name already decoded by its parser
    let last = name.rsplit(['/', '\\']).next().unwrap_or("");
    let nfc: String = last.nfc().collect();
    let mut s: String = nfc.chars().filter(|c| !is_forbidden(*c)).collect();
    s = trim_dots_spaces(&s);
    let ext = split_ext(&s).map(str::to_owned);                       // owned: last ".xxx" <= 16 chars
    s = truncate_utf16_at_grapheme(&s, 199, ext.as_deref());          // 199: room for '_' below
    s = trim_dots_spaces(&s);
    if s.is_empty() { s = FALLBACK.to_owned(); }
    if is_reserved_device(&s) { s.insert(0, '_'); }                   // after truncation, on final name
    s
}
fn is_forbidden(c: char) -> bool {
    matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
        || (c as u32) < 0x20 || c == '\u{7f}'
        || matches!(c, '\u{200b}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{feff}')
}
fn is_reserved_device(name: &str) -> bool {
    let first = name.split('.').next().unwrap_or("").trim_end_matches([' ', '.']).to_ascii_uppercase();
    const R: [&str; 6] = ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"];
    R.contains(&first.as_str()) || ((first.starts_with("COM") || first.starts_with("LPT")) && first.chars().count() == 4
        && matches!(first.chars().nth(3), Some('0'..='9' | '¹' | '²' | '³')))
}
"""

SANITIZE_NOTES = [
    ("1, 10", "v1: fallback មកពីអ្នកហៅ (អាចមិនស្អាត) រំលង sanitize (ChatGPT r3 L7)", "Fallback ថេរ <code>download</code>"),
    ("2", "v1: decode RFC 8187 ក្នុងអនុគមន៍នេះ ធ្វើឱ្យឈ្មោះធម្មតា URL និង title ត្រូវ decode ខុស (r3 L2)", "Decode ជាការងាររបស់ parser នីមួយៗ (Content-Disposition <code>filename*</code> ⇒ RFC 8187 ម្ដង; URL ⇒ percent-decode ម្ដង)។ អនុគមន៍នេះទទួលអត្ថបទដែល decode រួច"),
    ("3", "Path ពេញ <code>C:\\x\\a.exe</code>, UNC, <code>\\\\?\\</code>, <code>C:a.exe</code>", "យក segment ចុងក្រោយ (RFC 6266 §4.3); <code>:</code> ត្រូវដកនៅបន្ទាត់ 5"),
    ("4", "NFD/NFC ពីរដែលមើលទៅដូចគ្នា", "NFC។ ការការពារការជាន់គ្នាពិតប្រាកដគឺការបង្កើតឯកសារដោយ no-replace លើ filesystem មិនមែនការប្រៀបធៀបអក្សរ (r3 note L4)"),
    ("5, 15–17", "ADS, wildcard, control char, bidi/RTLO", "ដក។ v2 <strong>រក្សា ZWNJ/ZWJ</strong> (U+200C/D) ព្រោះអាចមានន័យក្នុងអក្សរ ហើយមិនមែនជាការវាយប្រហារ path (r3 L16)។ ZWSP, LRM/RLM, bidi embedding/isolate និង BOM ត្រូវដក"),
    ("6, 9", "Dot/space ខាងចុង (Windows កាត់ចោល) ឬបង្កើតថ្មីក្រោយ truncate", "Trim មុន និងក្រោយ truncate"),
    ("7–8", "v1: <code>ext</code> ខ្ចីពី <code>s</code> រួចប្ដូរ <code>s</code> (borrow) (r3 L8–10); ext វែង 300", "Ext ជា <code>String</code> ផ្ទាល់; &gt; 16 តួ ⇒ គ្មាន ext"),
    ("8", "កាត់កណ្ដាល grapheme ខ្មែរ; v2 ដំបូង: បុព្វបទ <code>_</code> នៅបន្ទាត់ 11 អាចធ្វើឱ្យលើស 200 (ChatGPT r5)", "Truncate តាម grapheme ត្រឹម 199 UTF-16 unit (រួមទាំង ext, ext រាប់ជា UTF-16 ផងដែរ) ដូច្នេះ <code>_</code> នៅតែ ≤ 200; ទុកកន្លែងសម្រាប់ <code>\" (12)\"</code> + <code>.part</code> ក្នុង path budget"),
    ("10", "v1: ការកាត់អាចធ្វើឱ្យឈ្មោះទទេ (ការរកឃើញខ្លួនឯង)", "ពិនិត្យទទេក្រោយ trim ចុងក្រោយ"),
    ("11, 19–23", "v1: ពិនិត្យ device មុន truncate ⇒ truncate បង្ហាញ <code>CON</code> ថ្មី; <code>CON .foo.txt</code> រំលង (r3 L9–11)", "ពិនិត្យលើឈ្មោះចុងក្រោយ, component ទីមួយមុន <code>.</code>, កាត់ space/dot, អក្សរធំ, ¹²³"),
    ("—", "ឈ្មោះស្អាត តែថតគោលដៅជា junction", "មិនមែនការងារនៃអនុគមន៍នេះ: បង្កើតតាម handle ថត (N-03)"),
]

ACCEPT = r"""
fn accept_segment(resp: &Response, want: &Want, rep: &Representation) -> Result<u64, Reject> {
    match resp.status().as_u16() {
        206 => {}
        200 => return Err(if want.if_range.is_some() { Reject::Changed } else { Reject::NoRange }),
        416 => return Err(Reject::Changed),
        s => return Err(Reject::Status(s)),
    }
    if want.if_range.as_deref() != rep.validator.if_range_value() { return Err(Reject::Mismatch); } // release too
    if is_multipart(resp.headers()) || has_duplicates(resp.headers(),
        &[CONTENT_RANGE, CONTENT_LENGTH, ETAG, LAST_MODIFIED, CONTENT_ENCODING]) {
        return Err(Reject::Malformed);
    }
    let cr = resp.headers().get(CONTENT_RANGE).ok_or(Reject::NoContentRange)?;
    let (a, b, t) = parse_content_range_strict(cr.as_bytes())?;     // "bytes a-b/t", digits only, no overflow
    if !(a <= b && b < rep.total) || a != want.start || b != want.end_incl || t != Some(rep.total) {
        return Err(Reject::Mismatch);
    }
    if let Some(ce) = resp.headers().get(CONTENT_ENCODING) {
        if !ce.as_bytes().eq_ignore_ascii_case(b"identity") { return Err(Reject::Encoded); }
    }
    let len = b - a + 1;                                             // safe: a <= b < total <= u64::MAX
    if let Some(cl) = resp.content_length() { if cl != len { return Err(Reject::Mismatch); } }
    match &rep.validator {
        Validator::StrongEtag(e) => match resp.headers().get(ETAG) {
            Some(got) if got.as_bytes() != e.as_bytes() => return Err(Reject::Changed),
            _ => {}                    // missing ETag tolerated only because matching If-Range was sent
        },
        Validator::LastModified(lm) => if let Some(got) = resp.headers().get(LAST_MODIFIED) {
            if got.as_bytes() != lm.as_bytes() { return Err(Reject::Changed); }
        },
        Validator::None => {}                                       // job is flagged "unverified" (E-02)
    }
    Ok(len)    // caller: client built with no auto-decompression; stream exactly len bytes to a under the
}              // current lease generation; short body or extra byte => Reject; flush before commit (E-04)
"""

ACCEPT_NOTES = [
    ("1–8", "v2 ដំបូង: Boolean <code>if_range_sent</code> មិនបង្ហាញថា If-Range ពិតជាតម្លៃ validator ដែលរក្សា (r5 L1)។ Server ឆ្លើយ 200 ទៅ Range ⇒ ឯកសារទាំងមូលសរសេរចូល offset ខុស (E-01)", "Request ផ្ទុកតម្លៃ If-Range ពិតប្រាកដ (request builder ប្រើតម្លៃដដែលសម្រាប់ header ដែលផ្ញើចេញ) ហើយត្រូវស្មើ validator: មិនស្មើ ⇒ Mismatch ក្នុង release build ផងដែរ (បន្ទាត់ 8; v3 ដំបូងប្រើ <code>debug_assert</code> ដែលមិនដំណើរការក្នុង release, ChatGPT r6)។ 200 មិនដែលត្រូវសរសេរ: ក្រោម If-Range ⇒ ឯកសារប្ដូរ, គ្មាន If-Range ⇒ no-range ⇒ ចាប់ផ្ដើម stream តាមលំដាប់។ 416 ⇒ Changed (ការវិនិច្ឆ័យបែបប្រុងប្រយ័ត្ន)"),
    ("9–12", "multipart ឬ header ស្ទួន/ផ្ទុយគ្នា (r3 L21–22; r5: បន្ថែម Last-Modified, Content-Encoding)", "បដិសេធ"),
    ("14", "<code>bytes 0-499/*</code>, space ពីរ, sign, លេខ &gt; u64", "Parser តឹងរឹង ជាមួយ checked arithmetic"),
    ("15–17", "v1: មិនពិនិត្យ <code>a ≤ b &lt; total</code> (r3 L11)", "បន្ថែម ហើយត្រូវស្មើ range ដែលស្នើ"),
    ("18–20", "gzip លើ range (E-09)", "បដិសេធ; client បិទ auto-decompression (បន្ទាត់ 33)"),
    ("21", "v1: overflow សន្មត (r3 L20)", "បន្ទាប់ពីបន្ទាត់ 15, <code>b − a + 1 ≤ total</code> ដូច្នេះមិន overflow"),
    ("22", "Content-Length ខុស", "ត្រូវស្មើ len"),
    ("23–32", "v1: return មុនពិនិត្យប្រវែង ពេលគ្មាន ETag, និងមិនពិនិត្យ Last-Modified (r3 L15–19)", "ពិនិត្យប្រវែងមុន validator; ETag ខុស ⇒ Changed។ RFC 9110 §15.3.7 តម្រូវឱ្យ 206 មាន ETag ពេល 200 មាន ដូច្នេះការខ្វះ ETag ត្រូវបានអត់អោនតែជាការសម្របសម្រួលដើម្បីដំណើរការជាមួយ server ដែលខូច ហើយតែពេល If-Range ដែលត្រូវគ្នាត្រូវបានផ្ញើ (r5)។ Last-Modified ខុស ⇒ Changed។ គ្មាន validator ⇒ job 'មិនបានផ្ទៀងផ្ទាត់' ហើយការសម្រេច seam/ការយល់ព្រមត្រូវកើតឡើងមុនការសរសេរ"),
    ("33–34", "Body ខ្លី ឬលើស; worker ចាស់សរសេរក្រោយ split", "រាប់ byte ពិត, lease generation (E-06), flush មុន commit"),
]

NATIVE = r"""
fn main() -> ExitCode {
    let caller = match browser_caller(std::env::args().collect()) {    // Chrome: origin arg; Firefox:
        Some(c) if ALLOWED_CALLERS.contains(&c.as_str()) => c,         // manifest path + extension id
        _ => return ExitCode::from(2),
    };
    let mut stdin = io::stdin().lock();
    loop {
        let mut len = [0u8; 4];
        match read_exact_or_eof(&mut stdin, &mut len) {
            Ok(Read::Eof) => return ExitCode::SUCCESS,                   // clean close before any byte
            Ok(Read::Full) => {}
            Err(_) => return ExitCode::from(4),                         // truncated prefix or I/O error
        }
        let n = u32::from_ne_bytes(len) as usize;
        if n == 0 || n > 256 * 1024 { return ExitCode::from(3); }
        let mut buf = vec![0u8; n];
        if stdin.read_exact(&mut buf).is_err() { return ExitCode::from(4); }
        let reply = serde_json::from_slice::<Msg>(&buf).map_err(|_| Error::BadMessage)  // deny_unknown_fields
            .and_then(|m| validate(m, &caller)).and_then(forward_prepared);               // B-05 prepared
        if write_native(&reply.into_wire()).is_err() { return ExitCode::from(5); }       // reply <= 4 KiB
    }
}
fn validate(m: Msg, caller: &str) -> Result<Handoff, Error> {
    let Msg { handoff_id, url, referrer, cookies, name_hint, size, mime, browser_dl_id, incognito } = m;
    if url.len() > 8192 || name_hint.as_deref().map_or(0, str::len) > 1024 { return Err(Error::TooLong); }
    let url = Url::parse(&url).map_err(|_| Error::BadUrl)?;
    if !matches!(url.scheme(), "http" | "https" | "ftp" | "ftps") { return Err(Error::Scheme); }
    if !url.username().is_empty() || url.password().is_some() { return Err(Error::Userinfo); }
    let cookies = scope_cookies(cookies, &url, 32 * 1024)?;            // domain/path/secure/partition, total cap
    let referrer = referrer.as_deref().map(parse_referrer).transpose()?;  // http(s), <= 4 KiB, no userinfo
    Ok(Handoff { handoff_id, url, referrer, cookies, name_hint, size, mime, browser_dl_id, incognito,
                 caller: caller.to_owned() })                          // every field explicit, no raw passthrough
}
"""

NATIVE_NOTES = [
    ("2–5", "v1: Chrome និង Firefox ផ្ញើ argument ខុសគ្នា (r3 L2–3); extension ផ្សេង", "<code>browser_caller</code> parse តាម browser (Chrome: origin; Firefox: manifest path + extension id) ហើយផ្ទៀងនឹង allowlist។ Process ផ្សេងអាចក្លែង argv ⇒ engine មិនទុកចិត្ត host (residual)"),
    ("9–13", "v1: prefix ខ្លី ឬ I/O error ត្រូវចាត់ទុកជា EOF ស្អាត (r3 L7)", "បែងចែក EOF (0 byte) ពី prefix ខ្លី ឬកំហុស"),
    ("14–15", "Length 0xFFFFFFFF", "ពិនិត្យមុន allocate"),
    ("18–19", "Field មិនស្គាល់; timeout ធ្វើឱ្យលទ្ធផលមិនច្បាស់ (r3 L13)", "<code>deny_unknown_fields</code>; engine ធ្វើ prepared handoff + reconciliation (B-05, B-09) ដូច្នេះ timeout មិនបាត់ការទាញ"),
    ("20", "v1: មិនពិនិត្យកំហុស write/flush (r3 L16)", "កំហុស ⇒ exit; reply តូច"),
    ("24–26", "v1: ពិនិត្យប្រវែងក្រោយ parse (r3 L20–23); v2 ដំបូង: <code>Handoff::from(m)</code> ក្រោយ move <code>m.cookies</code> ជាកំហុស ownership (r5 L30)", "Destructure <code>Msg</code> ជាមុន; ពិនិត្យប្រវែងមុន parse; ប្រើ URL ដែល parse រួច; សង់ <code>Handoff</code> ដោយ field ច្បាស់ៗ (គ្មាន field ឆៅឆ្លងកាត់)"),
    ("27–28", "Scheme <code>javascript:</code>/<code>file:</code>, userinfo (wget CVE-2024-38428)", "Allowlist scheme; បដិសេធ userinfo"),
    ("29", "v1: 64 × 4096 &gt; 32 KiB ដែលសន្យា; cookie domain ផ្សេង (r3 L24)", "ចាប់យកតាម domain/path/Secure/partition និង public suffix; កម្រិតសរុប 32 KiB"),
    ("30", "v1: ពិនិត្យតែ prefix referrer (r3 L25)", "Parse ពេញ, http(s), ≤ 4 KiB, គ្មាន userinfo, HeaderValue បដិសេធ CRLF"),
]

UPDATE = r"""
async fn check_update(db: &Db, now: SystemTime) -> Result<Option<Staged>, UpdErr> {
    let body = get_capped(MANIFEST_URL, 64 * 1024, &UPDATE_HOSTS).await?;   // https; redirects only to UPDATE_HOSTS
    let sig  = get_capped(&format!("{MANIFEST_URL}.minisig"), 2048, &UPDATE_HOSTS).await?;
    verify_minisign(UPDATE_PUBKEY, &body, &sig)?;                         // before parsing anything
    let m: Manifest = serde_json::from_slice(&body)?;
    if m.channel != db.channel()? { return Ok(None); }
    let highest = db.highest_seen_version(&m.channel)?;                   // per channel (r3 L6-8)
    if m.version <= CURRENT_VERSION || m.version < highest { return Ok(None); }
    let grace = Duration::from_secs(86_400);
    let max = m.issued.checked_add(Duration::from_secs(45 * 86_400)).ok_or(UpdErr::Stale)?;
    let not_future = now.checked_add(grace).ok_or(UpdErr::Stale)?;
    if m.issued > m.expires || m.issued > not_future || m.expires > max
        || now > m.expires.checked_add(grace).ok_or(UpdErr::Stale)? {
        return Err(UpdErr::Stale);                                        // bounded horizon, 24 h clock grace
    }
    db.set_highest_seen_version(&m.channel, &m.version)?;
    let art = m.artifact_for(TARGET).ok_or(UpdErr::NoTarget)?;
    let staged = download_staged(&art.url, 250 << 20, &UPDATE_HOSTS).await?;  // GDM-owned dir, no reparse
    // staged = StagingGuard over the CREATING handle (deny write/delete); deletes on any drop before keep()
    if sha256_handle(staged.handle())? != art.sha256 { return Err(UpdErr::Hash); }
    verify_minisign_handle(UPDATE_PUBKEY, staged.handle(), &art.signature)?;  // Tauri updater signature
    if REQUIRE_AUTHENTICODE { authenticode_check(staged.handle(), SIGNER_PINS)?; } // compiled pins, build policy
    Ok(Some(Staged { held: staged.keep(), version: m.version, security: m.security }))  // idle install (§07)
}
"""

UPDATE_NOTES = [
    ("2–3", "MITM (IDM CVE-2025-56231), redirect ទៅ host ផ្សេង, manifest 1 GB", "rustls verify; redirect តែក្នុង <code>UPDATE_HOSTS</code> និង https; កម្រិត 64 KB"),
    ("4–5", "Parse JSON មុនពិនិត្យសញ្ញា", "ពិនិត្យសញ្ញាលើ byte ឆៅជាមុន"),
    ("6–8", "Beta ⇒ stable; replay manifest ចាស់ (rollback); v1 highest-seen មិនតាម channel (r3)", "Channel ក្នុងខ្លឹមសារដែលមានហត្ថលេខា; highest-seen តាម channel; ការប្ដូរ Beta→Stable ជាសកម្មភាពអ្នកប្រើ ហើយកំណត់ highest-seen ថ្មី"),
    ("9–15", "Freeze attack; v1 គណនាពេលដោយគ្មាន checked arithmetic និងគ្មាន horizon (r3 L9); v2 ដំបូង: manifest ដែលមាន issued នៅអនាគតអាចរំលង horizon (r5)", "<code>issued ≤ expires</code>, <code>issued ≤ now + grace</code>, <code>expires</code> ≤ issued + 45 ថ្ងៃ, grace 24 ម៉ោង, checked_add។ Stale ⇒ badge 'ពិនិត្យ update មិនបាន' (មិនរារាំង app)។ Expiry រកឃើញភាពចាស់ តែមិនអាចបង្ខំឱ្យ update ទៅដល់បានទេ"),
    ("16", "Highest-seen កើនមុនទាញរួច", "អាចព្យាយាមកំណែដដែលម្ដងទៀតបាន (ChatGPT បញ្ជាក់ថាត្រឹមត្រូវ)"),
    ("18", "Disk ពេញ; v1 <code>CREATE_NEW</code> មិនការពារថតមេ (r3 L12)", "កម្រិត 250 MB; ថតដែល GDM បង្កើត និងបដិសេធ reparse point; redirect policy ដូចបន្ទាត់ 2; ការការពារ reparse គ្របដណ្ដប់ថតមេទាំងអស់"),
    ("19–21", "v1: ផ្ទៀងតាម path រួចរត់ពេលក្រោយ ⇒ ប្ដូរឯកសារបាន (r3, U-06); v2 ដំបូង: <code>?</code> មិនលុបឯកសារដែលបរាជ័យ (r5)", "<code>download_staged</code> បង្កើតឯកសារ ហើយត្រឡប់ <code>StagingGuard</code> ដែលកាន់ handle បង្កើតដដែល (ហាម write/delete) ដូច្នេះគ្មានការបើកឡើងវិញដែលអាចបរាជ័យនៅចន្លោះទេ។ Downloader លុប partial file របស់ខ្លួនពេលបរាជ័យ, guard លុបឯកសារលើ return ណាមួយមុន <code>keep()</code>។ v3 ដំបូងបង្កើត guard ក្រោយការបើកឡើងវិញ ⇒ ការបើកបរាជ័យទុកឯកសារចោល (ChatGPT r6)។ Hash និង signature ធ្វើលើ handle"),
    ("22", "Signer text ក្លែងបាន; v2 ដំបូង: pin មកពី manifest ដែលចុះហត្ថលេខាដោយ key ដដែល (r5); key ពីរក្នុង CI តែមួយ (r3)", "<code>SIGNER_PINS</code> និង <code>REQUIRE_AUTHENTICODE</code> compile ចូល binary (build policy, មិនមែនជម្រើស manifest); key ទាំងពីរមិននៅក្នុង secret store តែមួយ; មុនពេលមាន cert ⇒ minisign តែមួយស្រទាប់ (ហានិភ័យដែលបានកត់ត្រា)"),
    ("23", "Install ពេលកំពុងទាញ", "State machine §07 (U-02); ថតតាមកំណែ + launcher ថេរ + counter ចាប់ផ្ដើមបរាជ័យ (U-04)។ ការត្រឡប់ទៅ DB មុន migration ធ្វើតែពេលស្ដារ upgrade ដែលបរាជ័យ"),
]

CODE_HTML = ("<p class=\"note\">កូដខាងក្រោមជា <strong>កូដគំរូ (reference specification) v2</strong> ដែលទីប្រឹក្សាសរសេរដើម្បីកំណត់ invariant សម្រាប់អនុគមន៍ ៤ ដែលគ្រោះថ្នាក់បំផុត។ វាមិនទាន់ compile និងមិនមែនកូដផលិតផលទេ។ "
             "v1 ត្រូវបានពិនិត្យម្ដងមួយបន្ទាត់ដោយខ្ញុំ (រកឃើញ 1 កំហុស) និងដោយ ChatGPT ជុំទី ៣ (រកឃើញ 20 ចំណុច)។ v2 កែគ្រប់ចំណុច ហើយតារាងបង្ហាញថា v1 ខុសត្រង់ណា។ "
             "ពេលមានកូដពិត ការពិនិត្យម្ដងមួយបន្ទាត់ត្រូវធ្វើឡើងវិញលើ commit ពិត ដោយយកតារាងទាំងនេះជា checklist។</p>"
             + block("13.1 · sanitize_filename (F04, N-01/N-02/N-06)", "ឈ្មោះឯកសារមកពី Content-Disposition, URL, extension, yt-dlp ឬ grabber សុទ្ធតែឆ្លងកាត់អនុគមន៍នេះ។", SANITIZE, SANITIZE_NOTES)
             + block("13.2 · accept_segment (F03, E-01/E-02/E-09)", "ហៅមុនពេលសរសេរ byte ណាមួយនៃ segment។", ACCEPT, ACCEPT_NOTES)
             + block("13.3 · gdm-bridge main/validate (F01, B-01..B-09)", "Native messaging host: ព្រំដែនរវាង browser និងម៉ាស៊ីន។", NATIVE, NATIVE_NOTES)
             + block("13.4 · check_update (F13, U-01/U-06)", "ច្រកតែមួយដែល code ថ្មីចូលម៉ាស៊ីនអ្នកប្រើ។", UPDATE, UPDATE_NOTES))
