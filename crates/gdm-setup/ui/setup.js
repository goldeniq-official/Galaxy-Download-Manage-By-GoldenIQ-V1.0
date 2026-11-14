// SPDX-License-Identifier: RPL-1.5
// GDM Setup page. Talks to Setup only through window.ipc.postMessage({c: ...}); Setup answers with gdm({t: ...}).
// Every URL, path and browser stays on the Rust side: the page sends keys only.
"use strict";

const T = {
  km: {
    s1: "ដំឡើង", s2: "ភ្ជាប់ browser", s3: "ចាប់ផ្ដើម", by: "ឥតគិតថ្លៃ · GoldenIQ",
    welcomeT: "ដំឡើង Galaxy Download Manager", updateT: "ធ្វើបច្ចុប្បន្នភាព GDM ទៅ v{v}", againT: "GDM v{v} មានរួចហើយ", again: "ដំឡើងម្ដងទៀត",
    welcomeP: "កម្មវិធីទាញយកឯកសារ លឿន មានសុវត្ថិភាព និងឥតគិតថ្លៃសម្រាប់គ្រប់គ្នា។",
    f1: "ទាញលឿន ច្រើនខ្សែក្នុងពេលតែមួយ", f1s: "ផ្អាក បន្ត ហើយទាញបន្តពីកន្លែងដែលដាច់",
    f2: "ចាប់ download ពី browser", f2s: "Chrome, Edge, Firefox, Brave, Vivaldi, Opera",
    f3: "ឯកសារត្រូវបានផ្ទៀងផ្ទាត់", f3s: "ពិនិត្យដោយ Windows Security មុនពេលអ្នកបើក",
    where: "ដំឡើងសម្រាប់តែអ្នក នៅក្នុង", noPayload: "ច្បាប់នេះគ្មានកម្មវិធីដំឡើងនៅខាងក្នុងទេ (build សម្រាប់អ្នកអភិវឌ្ឍ)។",
    license: "អាជ្ញាប័ណ្ណ", privacy: "ឯកជនភាព", cancel: "បោះបង់", install: "ដំឡើង", update: "ធ្វើបច្ចុប្បន្នភាព",
    runT: "កំពុងដំឡើង…", runP: "ចំណាយពេលតែប៉ុន្មានវិនាទីប៉ុណ្ណោះ។",
    runNote: "បើ GDM កំពុងបើក វានឹងបិទខ្លួនឯង ហើយ download ដែលកំពុងដំណើរការនឹងបន្តពេលបើកម្ដងទៀត។",
    doneOk: "ដំឡើងរួចរាល់", doneT: "ជំហានចុងក្រោយ៖ ភ្ជាប់ browser របស់អ្នក",
    doneP: "បន្ថែម extension GDM ទៅ browser ដែលអ្នកប្រើ ដើម្បីឲ្យ GDM ចាប់ download ជំនួស browser។",
    add: "បន្ថែម", sent: "បានបើក", none: "រកមិនឃើញ browser ទេ។ ពេលក្រោយ អ្នកអាចបន្ថែមពី GDM › ការកំណត់ › Browser។",
    h1: "ចុច “បន្ថែម” ក្បែរ browser", h2: "នៅក្នុងហាង ចុច “Add to {b}” ឬ “Get”",
    h2u: "បិទភ្ជាប់ {p} ក្នុង address bar (បានចម្លងរួច) › បើក “Developer mode” › អូស folder “extension” ដែល GDM បើកឲ្យ ទម្លាក់លើទំព័រនោះ", h3: "GDM បង្ហាញ “បានភ្ជាប់” នោះជាការរួចរាល់",
    close: "បិទ", open: "បើក GDM",
    failOk: "មិនបានដំឡើង", failT: "ការដំឡើងមិនបានបញ្ចប់",
    failP: "គ្មានអ្វីផ្សេងត្រូវបានប្ដូរទេ។ សូមព្យាយាមម្ដងទៀត ឬប្រាប់យើងជាមួយសារខាងក្រោម។",
    help: "រាយការណ៍បញ្ហា", retry: "ព្យាយាមម្ដងទៀត", min: "បង្រួម", closeTip: "បិទ",
  },
  en: {
    s1: "Install", s2: "Connect browser", s3: "Start", by: "Free · GoldenIQ",
    welcomeT: "Install Galaxy Download Manager", updateT: "Update GDM to v{v}", againT: "GDM v{v} is already installed", again: "Reinstall",
    welcomeP: "A fast, safe download manager, free for everyone.",
    f1: "Fast, many connections at once", f1s: "Pause, resume, and pick up where a download broke",
    f2: "Catches downloads from your browser", f2s: "Chrome, Edge, Firefox, Brave, Vivaldi, Opera",
    f3: "Every file verified", f3s: "Checked by Windows Security before you open it",
    where: "Installs for you only, in", noPayload: "This copy has no installer inside (a developer build).",
    license: "License", privacy: "Privacy", cancel: "Cancel", install: "Install", update: "Update",
    runT: "Installing…", runP: "This takes only a few seconds.",
    runNote: "If GDM is open it closes itself; running downloads continue when it opens again.",
    doneOk: "Installed", doneT: "Last step: connect your browser",
    doneP: "Add the GDM extension to the browsers you use, so GDM catches their downloads.",
    add: "Add", sent: "Opened", none: "No browser found. You can add it later from GDM › Settings › Browsers.",
    h1: "Click “Add” next to a browser", h2: "In the store, click “Add to {b}” or “Get”",
    h2u: "Paste {p} in the address bar (copied) › turn on “Developer mode” › drag the “extension” folder GDM opened onto that page", h3: "GDM shows “Connected”: you're done",
    close: "Close", open: "Open GDM",
    failOk: "Not installed", failT: "Setup could not finish",
    failP: "Nothing else was changed. Try again, or tell us with the message below.",
    help: "Report a problem", retry: "Try again", min: "Minimize", closeTip: "Close",
  },
};

const ICONS = {
  minus: '<path d="M5 12h14"/>',
  x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  zap: '<path d="M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z"/>',
  puzzle: '<path d="M15.39 4.39a1 1 0 0 0 1.68-.474 2.5 2.5 0 1 1 3.014 3.015 1 1 0 0 0-.474 1.68l1.683 1.682a2.414 2.414 0 0 1 0 3.414L19.61 15.39a1 1 0 0 1-1.68-.474 2.5 2.5 0 1 0-3.014 3.015 1 1 0 0 1 .474 1.68l-1.683 1.682a2.414 2.414 0 0 1-3.414 0L8.61 19.61a1 1 0 0 0-1.68.474 2.5 2.5 0 1 1-3.014-3.015 1 1 0 0 0 .474-1.68l-1.683-1.682a2.414 2.414 0 0 1 0-3.414L4.39 8.61a1 1 0 0 1 1.68.474 2.5 2.5 0 1 0 3.014-3.015 1 1 0 0 1-.474-1.68l1.683-1.682a2.414 2.414 0 0 1 3.414 0z"/>',
  shield: '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m9 12 2 2 4-4"/>',
  folder: '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>',
  download: '<path d="M12 15V3"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m7 10 5 5 5-5"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  alert: '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
  globe: '<circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/>',
  link: '<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
};

const $ = (s) => document.querySelector(s);
const post = (c, x = {}) => window.ipc.postMessage(JSON.stringify({ c, ...x }));
const S = { lang: "km", page: "welcome", version: "", dir: "", upgrade: false, browsers: [], payload: true, sent: new Set(), busy: false, unpacked: "" };

function icon(el) {
  el.setAttribute("viewBox", "0 0 24 24");
  el.setAttribute("aria-hidden", "true");
  el.innerHTML = ICONS[el.dataset.i] ?? ""; // fixed strings above, never data
}
function tr(k, v = {}) {
  return (T[S.lang][k] ?? k).replace(/\{(\w+)\}/g, (_, n) => v[n] ?? "");
}
const storeName = (k) => (k === "firefox" ? "Firefox" : k === "edge" ? "Edge" : "Chrome");

function render() {
  document.documentElement.lang = S.lang;
  document.querySelectorAll("[data-t]").forEach((el) => (el.textContent = tr(el.dataset.t)));
  document.querySelectorAll("#lang [data-l]").forEach((b) => b.setAttribute("aria-selected", String(b.dataset.l === S.lang)));
  $("#ver").textContent = S.version ? `v${S.version}` : "";
  $("#dir").textContent = S.dir;
  if (S.installed) {
    const same = S.installed === S.version;
    $("#p-welcome h1").textContent = tr(same ? "againT" : "updateT", { v: S.version });
    $("#go span").textContent = tr(same ? "again" : "update");
  }
  $("#min").setAttribute("aria-label", tr("min"));
  $("#x").setAttribute("aria-label", tr("closeTip"));
  $("#x").disabled = S.page === "run";
  $("#nopayload").hidden = S.payload;
  $("#go").disabled = !S.payload;
  const first = S.browsers[0]?.key ?? "chrome";
  $("#p-done .how li:nth-child(2) span").textContent = tr(S.unpacked ? "h2u" : "h2", { b: storeName(first), p: S.unpacked });
  const step = { welcome: 1, run: 1, fail: 1, done: 2 }[S.page];
  document.querySelectorAll("#steps li").forEach((li) => {
    const n = Number(li.dataset.s);
    li.className = n === step ? "on" : n < step ? "past" : "";
    li.querySelector("i").textContent = n < step ? "✓" : String(n);
  });
  const list = $("#browsers");
  list.replaceChildren();
  if (!S.browsers.length) {
    const p = document.createElement("p");
    p.className = "none";
    p.textContent = tr("none");
    list.append(p);
  }
  for (const b of S.browsers) {
    const row = document.createElement("div");
    row.className = "brow";
    const tile = document.createElement("span");
    tile.className = "tile";
    const g = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    g.classList.add("ico");
    g.dataset.i = "globe";
    icon(g);
    tile.append(g);
    const name = document.createElement("b");
    name.textContent = b.name;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "fx btn btn-s" + (S.sent.has(b.key) ? " sent" : "");
    btn.textContent = S.sent.has(b.key) ? tr("sent") : tr("add");
    btn.onclick = () => {
      post("store", { b: b.key });
      S.sent.add(b.key);
      render();
    };
    row.append(tile, name, btn);
    list.append(row);
  }
}

function show(page) {
  S.page = page;
  for (const p of ["welcome", "run", "done", "fail"]) $(`#p-${p}`).hidden = p !== page;
  render();
  // keyboard users keep their place on the new page; a mouse user gets no focus fill they did not ask for
  const f = { welcome: "#go", done: "#open", fail: "#retry" }[page];
  if (f && S.keys) $(f).focus({ preventScroll: true });
}

function install() {
  if (S.busy || !S.payload) return;
  S.busy = true;
  show("run");
  post("install");
}

window.gdm = (m) => {
  if (m.t === "hello") {
    Object.assign(S, { version: m.version, lang: m.lang, dir: m.dir, installed: m.installed, browsers: m.browsers, payload: m.payload });
    show("welcome");
  } else if (m.t === "unpacked") {
    S.unpacked = m.page; // no store item yet: Setup started the browser and showed the folder; the user pastes the page
    navigator.clipboard?.writeText(m.page).catch(() => undefined);
    render();
  } else if (m.t === "done") {
    S.busy = false;
    show("done");
  } else if (m.t === "failed") {
    S.busy = false;
    $("#err").textContent = m.error;
    show("fail");
  }
};

document.querySelectorAll("svg[data-i]").forEach(icon);
document.querySelectorAll("#lang [data-l]").forEach((b) => (b.onclick = () => ((S.lang = b.dataset.l), render())));
document.querySelectorAll("[data-act=close]").forEach((b) => (b.onclick = () => post("close")));
document.querySelectorAll("[data-k]").forEach((b) => (b.onclick = () => post("link", { k: b.dataset.k })));
$("#x").onclick = () => post("close");
$("#min").onclick = () => post("min");
$("#go").onclick = install;
$("#retry").onclick = install;
$("#open").onclick = () => post("launch");
// the frameless window moves by its empty areas (never from a control)
document.addEventListener("mousedown", (e) => {
  if (e.button === 0 && e.target instanceof Element && e.target.closest("[data-drag]") && !e.target.closest("button")) post("drag");
});
document.addEventListener("mousedown", () => (S.keys = false), true);
document.addEventListener("keydown", (e) => {
  S.keys = true;
  if (e.key === "Escape" && S.page !== "run") post("close");
});
render();
post("ready");
