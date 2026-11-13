// SPDX-License-Identifier: RPL-1.5
// Content script: reports link clicks with the modifier held (C-04: Ctrl = force GDM, Alt = browser; MV3
// downloads events carry no modifier keys) and collects links for "download all links" (P1 #15).
// It never reads page content beyond <a href>, and sends nothing outside the extension.
const keyOf = (e: MouseEvent) => (e.altKey ? "alt" : e.ctrlKey ? "ctrl" : e.shiftKey ? "shift" : null);

document.addEventListener(
  "click",
  (e) => {
    if (!e.isTrusted) return; // a page script cannot fake a Ctrl/Alt click to steer the catch policy
    const a = (e.target as Element | null)?.closest?.("a[href]") as HTMLAnchorElement | null;
    if (!a || !/^https?:/i.test(a.href)) return;
    void chrome.runtime.sendMessage({ t: "click", url: a.href, key: keyOf(e) }).catch(() => {});
  },
  true,
);

chrome.runtime.onMessage.addListener((m, sender, reply) => {
  if (sender.id !== chrome.runtime.id || m?.t !== "collect") return;
  const sel = window.getSelection();
  const all = [...document.querySelectorAll<HTMLAnchorElement>("a[href]")];
  const pick = m.selection && sel && !sel.isCollapsed ? all.filter((a) => sel.containsNode(a, true)) : all;
  reply(pick.map((a) => a.href).filter((h) => /^https?:/i.test(h)));
});

// D5 page video button: only on sites the user turned it on for. Counts <video> elements that play something,
// including MSE players (blob: sources: most video sites), since only the page address goes to GDM (yt-dlp reads
// the page); a count of players, not proof that a download is possible. Tells the extension icon.
void (async () => {
  if (window.top !== window || !/^https?:/.test(location.href)) return;
  const v = await chrome.storage.local.get("videoSites").catch(() => ({}) as Record<string, unknown>);
  const sites = Array.isArray((v as { videoSites?: unknown }).videoSites) ? ((v as { videoSites: string[] }).videoSites) : [];
  if (!sites.includes(location.host)) return;
  const count = () =>
    [...document.querySelectorAll("video")].filter((e) => /^(https?|blob):/i.test(e.currentSrc || e.src || e.querySelector("source")?.src || "")).length;
  let last = -1;
  const report = () => {
    const n = count();
    if (n !== last) {
      last = n;
      void chrome.runtime.sendMessage({ t: "videos", n }).catch(() => {});
    }
  };
  setInterval(report, 2000);
  report();
})();
