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
  const r = await chrome.runtime.sendMessage({ t: "popup" }).catch(() => null);
  const st = $("state");
  st.textContent = r?.connected ? msg("connected") : msg("offline");
  st.className = "st " + (r?.connected ? "ok" : "off");
  ($("help") as HTMLElement).hidden = !!r?.connected;
  $("help").textContent = msg("offlineHelp");
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
