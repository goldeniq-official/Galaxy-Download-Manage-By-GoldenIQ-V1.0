// SPDX-License-Identifier: RPL-1.5
// Content script: reports link clicks with the modifier held (C-04: Ctrl = force GDM, Alt = browser; MV3
// downloads events carry no modifier keys) and collects links for "download all links" (P1 #15).
// It never reads page content beyond <a href>, and sends nothing outside the extension.
const keyOf = (e: MouseEvent) => (e.altKey ? "alt" : e.ctrlKey ? "ctrl" : e.shiftKey ? "shift" : null);

document.addEventListener(
  "click",
  (e) => {
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
