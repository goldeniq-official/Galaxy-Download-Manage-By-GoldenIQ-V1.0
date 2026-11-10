// SPDX-License-Identifier: RPL-1.5
// GDM browser extension background (MV3 service worker on Chrome/Edge, background script on Firefox).
// It may only PROPOSE a download to GDM: url, referrer, suggested name, size, mime, the cookie of that url.
// No path, no hook, no auto-start (B-01, B-04). GDM decides and shows its Download Info dialog.
// B-05/B-06: the browser download is cancelled only after GDM acknowledged (3 s budget); otherwise the
// browser simply keeps it. If the browser had already finished, GDM is told to drop its copy ("dup").
declare const BUILD: "chrome" | "firefox";
const HOST = "com.goldeniq.gdm";
const VER = chrome.runtime.getManifest().version;

interface Catch { enabled: boolean; exts: string[]; min_kb: number; skip: string[]; force: string; bypass: string }
let policy: Catch | null = null; // null = GDM not reachable: catch nothing
let lastHello = 0;

const send = (msg: object, ms = 3000): Promise<Record<string, unknown> | null> =>
  Promise.race([
    chrome.runtime.sendNativeMessage(HOST, msg).catch(() => null) as Promise<Record<string, unknown> | null>,
    new Promise<null>((r) => setTimeout(() => r(null), ms)),
  ]);

async function hello(force = false): Promise<Catch | null> {
  if (!force && Date.now() - lastHello < 60_000) return policy;
  lastHello = Date.now();
  const browser = BUILD === "firefox" ? "firefox" : navigator.userAgent.includes("Edg/") ? "edge" : navigator.userAgent.includes("OPR/") ? "opera" : "chrome";
  const r = await send({ t: "hello", browser, ver: VER });
  policy = r && r.ok ? (r.catch as Catch) : null;
  await chrome.storage.local.set({ connected: !!policy });
  return policy;
}

// ---- evidence that a URL is a plain GET (C-02) and which modifier the user held (C-04) ----
const seen = new Map<string, { method?: string; key?: string | null; at: number }>();
function note(url: string, v: { method?: string; key?: string | null }) {
  const now = Date.now();
  for (const [k, x] of seen) if (now - x.at > 30_000) seen.delete(k);
  if (seen.size > 2000) seen.clear();
  seen.set(url, { ...seen.get(url), ...v, at: now });
}
chrome.webRequest.onBeforeRequest.addListener(
  (d) => {
    if (d.tabId >= 0) note(d.url, { method: d.method });
    return undefined;
  },
  { urls: ["http://*/*", "https://*/*"], types: ["main_frame", "sub_frame", "other"] },
);
chrome.runtime.onMessage.addListener((m, sender) => {
  if (sender.id !== chrome.runtime.id || !m || typeof m.url !== "string") return;
  if (m.t === "click") note(m.url, { key: typeof m.key === "string" ? m.key : null, method: "GET" });
});

function glob(pat: string, s: string) {
  const re = new RegExp("^" + pat.toLowerCase().replace(/[.+?^${}()|[\]\\]/g, "\\$&").replace(/\*/g, ".*") + "$");
  return re.test(s.toLowerCase());
}
const extOf = (name: string) => (name.match(/\.([a-z0-9]{1,8})$/i)?.[1] ?? "").toLowerCase();
const nameOf = (item: chrome.downloads.DownloadItem) => (item.filename ? item.filename.split(/[\\/]/).pop()! : decodeURIComponent(new URL(item.finalUrl || item.url).pathname.split("/").pop() || ""));

async function localSkip(): Promise<{ paused: boolean; sites: string[] }> {
  const s = await chrome.storage.local.get(["paused", "sites"]);
  return { paused: !!s.paused, sites: Array.isArray(s.sites) ? s.sites : [] };
}

/** C-01/C-02/C-04/C-05: catch only what the user clicked, with a GET, of a listed type and size */
async function decide(item: chrome.downloads.DownloadItem): Promise<boolean> {
  const p = await hello();
  if (!p || !p.enabled || item.incognito || item.byExtensionId) return false;
  const url = item.finalUrl || item.url;
  if (!/^https?:/i.test(url)) return false; // blob:, data:, file:
  const { paused, sites } = await localSkip();
  if (paused) return false;
  const ev = seen.get(item.url) ?? seen.get(url);
  if (ev?.key && ev.key === p.bypass) return false;
  if (ev?.method && ev.method !== "GET") return false; // POST/PUT: never replayed by GDM
  const forced = !!ev?.key && ev.key === p.force;
  if (!ev?.method && !forced) return false; // unknown method: leave it to the browser
  const host = new URL(url).host;
  const hp = url.replace(/^https?:\/\//i, "");
  if (sites.includes(host) || p.skip.some((s) => glob(s, hp))) return false;
  if (forced) return true;
  if (!p.exts.includes(extOf(nameOf(item)))) return false;
  const size = item.fileSize > 0 ? item.fileSize : item.totalBytes;
  if (size > 0) return size >= p.min_kb * 1024;
  return !/^(text\/|application\/(json|xml|javascript))/i.test(item.mime || "");
}

async function cookieFor(url: string): Promise<string | undefined> {
  try {
    const cs = await chrome.cookies.getAll({ url });
    const s = cs.map((c) => `${c.name}=${c.value}`).join("; ");
    return s && s.length <= 32 * 1024 ? s : undefined;
  } catch {
    return undefined;
  }
}

/** Chromium fills `filename` a moment after onCreated; the final URL of a CDN redirect often has no extension,
 * so the real name (Content-Disposition / save name) is needed to apply the type rule (found in the G6 Edge test) */
async function withName(item: chrome.downloads.DownloadItem): Promise<chrome.downloads.DownloadItem> {
  for (let i = 0; i < 20 && !item.filename && item.state === "in_progress"; i++) {
    await new Promise((r) => setTimeout(r, 100));
    item = (await chrome.downloads.search({ id: item.id }))[0] ?? item;
  }
  return item;
}

const handled = new Set<number>();
chrome.downloads.onCreated.addListener(async (created) => {
  if (handled.has(created.id)) return; // B-05: a restarted worker never sends the same download twice
  handled.add(created.id);
  const item = await withName(created);
  if (!(await decide(item))) return;
  const hid = crypto.randomUUID();
  const url = item.finalUrl || item.url;
  const r = await send({ t: "handoff", hid, url, referrer: /^https?:/.test(item.referrer) ? item.referrer : null, name: nameOf(item) || null, size: item.totalBytes > 0 ? item.totalBytes : null, mime: item.mime || null, cookie: (await cookieFor(url)) ?? null });
  if (!r || !r.ok) return; // no ack in 3 s / GDM refused: the browser keeps downloading (B-06)
  try {
    await chrome.downloads.cancel(item.id);
    const [now] = await chrome.downloads.search({ id: item.id });
    if (now && now.state === "complete") await send({ t: "dup", hid });
    else await chrome.downloads.erase({ id: item.id });
  } catch {
    await send({ t: "dup", hid });
  }
});

// ---- context menu (P1 #15) ----
function menus() {
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({ id: "gdm-link", title: chrome.i18n.getMessage("menuLink"), contexts: ["link"], targetUrlPatterns: ["http://*/*", "https://*/*"] });
    chrome.contextMenus.create({ id: "gdm-all", title: chrome.i18n.getMessage("menuAll"), contexts: ["page", "selection"] });
  });
}
chrome.runtime.onInstalled.addListener(menus);
chrome.runtime.onStartup.addListener(menus);
const onYouTube = (u?: string) => BUILD === "chrome" && !!u && /^https?:\/\/([^/]+\.)?youtube\.com\//i.test(u); // M-07

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (onYouTube(tab?.url)) return;
  const referrer = tab?.url && /^https?:/.test(tab.url) ? tab.url : null;
  if (info.menuItemId === "gdm-link" && info.linkUrl && /^https?:/i.test(info.linkUrl)) {
    await send({ t: "links", urls: [info.linkUrl], referrer });
  } else if (info.menuItemId === "gdm-all" && tab?.id != null) {
    const urls: string[] = await chrome.tabs.sendMessage(tab.id, { t: "collect", selection: info.menuItemId === "gdm-all" && !!info.selectionText }).catch(() => []);
    const ok = [...new Set(urls.filter((u) => typeof u === "string" && /^https?:/i.test(u) && u.length <= 8192))].slice(0, 500);
    if (ok.length) await send({ t: "links", urls: ok, referrer });
  }
});

// ---- popup ----
chrome.runtime.onMessage.addListener((m, sender, reply) => {
  if (sender.id !== chrome.runtime.id || m?.t !== "popup") return;
  void (async () => {
    const p = await hello(true);
    const st = p ? await send({ t: "status" }) : null;
    reply({ connected: !!p, policy: p, status: st });
  })();
  return true;
});
