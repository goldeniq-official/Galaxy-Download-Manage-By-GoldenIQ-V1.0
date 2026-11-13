// SPDX-License-Identifier: RPL-1.5
// Popup (P1 #17): connection state, catch on/off, "don't catch this site" (C-01), recent downloads.
// Everything is set with textContent: names come from the web (X-01).
const $ = (id: string) => document.getElementById(id)!;
const msg = (k: string) => chrome.i18n.getMessage(k);
document.documentElement.lang = chrome.i18n.getUILanguage().startsWith("km") ? "km" : "en";
$("title").textContent = msg("name");
$("lCatch").textContent = msg("catching");
$("lRecent").textContent = msg("recent");
$("keys").textContent = msg("keys");
$("catch").setAttribute("aria-label", msg("catching"));

void (async () => {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  const host = tab?.url && /^https?:/.test(tab.url) ? new URL(tab.url).host : "";
  const s = await chrome.storage.local.get(["paused", "sites"]);
  const sites: string[] = Array.isArray(s.sites) ? s.sites : [];
  const tg = $("catch");
  tg.setAttribute("aria-checked", String(!s.paused));
  tg.onclick = async () => {
    const on = tg.getAttribute("aria-checked") !== "true";
    tg.setAttribute("aria-checked", String(on));
    await chrome.storage.local.set({ paused: !on });
  };
  const site = $("site") as HTMLButtonElement;
  site.hidden = !host;
  site.textContent = sites.includes(host) ? `${msg("siteSkipped")}: ${host}` : `${msg("thisSite")}: ${host}`;
  site.onclick = async () => {
    const cur: string[] = ((await chrome.storage.local.get("sites")).sites as string[]) ?? [];
    const next = cur.includes(host) ? cur.filter((x) => x !== host) : [...cur, host].slice(-500);
    await chrome.storage.local.set({ sites: next });
    site.textContent = next.includes(host) ? `${msg("siteSkipped")}: ${host}` : `${msg("thisSite")}: ${host}`;
  };
  const r = await chrome.runtime.sendMessage({ t: "popup", tab: tab?.id }).catch(() => null);
  // D5: the page video button is off everywhere until the user turns it on for this site (and GDM allows it)
  const vrow = $("vrow");
  const vt = $("video");
  const vget = $("vget") as HTMLButtonElement;
  $("lVideo").textContent = msg("videoSite");
  vt.setAttribute("aria-label", msg("videoSite"));
  vget.textContent = msg("videoGet");
  const vs = await chrome.storage.local.get("videoSites");
  const vsites: string[] = Array.isArray(vs.videoSites) ? vs.videoSites : [];
  vrow.hidden = !host || !r?.policy?.video;
  vt.setAttribute("aria-checked", String(vsites.includes(host)));
  vt.onclick = async () => {
    const cur: string[] = ((await chrome.storage.local.get("videoSites")).videoSites as string[]) ?? [];
    const on = !cur.includes(host);
    await chrome.storage.local.set({ videoSites: on ? [...cur, host].slice(-500) : cur.filter((x) => x !== host) });
    vt.setAttribute("aria-checked", String(on));
    if (tab?.id != null) await chrome.tabs.reload(tab.id);
  };
  const count = tab?.id != null ? await chrome.action.getBadgeText({ tabId: tab.id }).catch(() => "") : "";
  vget.hidden = vrow.hidden || !vsites.includes(host) || !count;
  vget.onclick = async () => {
    await chrome.runtime.sendMessage({ t: "video", page: tab?.url }).catch(() => null);
    window.close();
  };
  // streams seen on the network (HLS / DASH / direct media): each one can go to GDM by itself (IDM parity)
  const sl = $("streams");
  const found = vrow.hidden || !vsites.includes(host) ? [] : ((r?.streams as { url: string; kind: string }[] | undefined) ?? []);
  for (const s of found.slice(0, 8)) {
    const b = document.createElement("button");
    b.className = "btn";
    let leaf = s.url;
    try {
      leaf = decodeURIComponent(new URL(s.url).pathname.split("/").pop() || s.url);
    } catch {
      /* keep the address */
    }
    b.textContent = `${msg("videoStream")} · ${s.kind.toUpperCase()} · ${leaf}`;
    b.onclick = async () => {
      await chrome.runtime.sendMessage({ t: "video", page: s.url }).catch(() => null);
      window.close();
    };
    const li = document.createElement("li");
    li.append(b);
    sl.append(li);
  }
  sl.hidden = found.length === 0;
  const st = $("state");
  st.textContent = r?.connected ? msg("connected") : msg("offline");
  st.className = "st " + (r?.connected ? "ok" : "off");
  ($("help") as HTMLElement).hidden = !!r?.connected;
  $("help").textContent = msg("offlineHelp");
  const get = $("get") as HTMLButtonElement; // a store user may have the extension before GDM itself
  get.hidden = !!r?.connected;
  get.textContent = msg("getGdm");
  get.onclick = () => chrome.tabs.create({ url: "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/releases/latest" });
  const ul = $("recent");
  for (const it of (r?.status?.recent as { name: string; state: string }[] | undefined) ?? []) {
    const li = document.createElement("li");
    const a = document.createElement("span");
    a.textContent = it.name;
    a.title = it.name;
    const b = document.createElement("span");
    b.className = "meta";
    b.textContent = it.state;
    li.append(a, b);
    ul.append(li);
  }
})();
