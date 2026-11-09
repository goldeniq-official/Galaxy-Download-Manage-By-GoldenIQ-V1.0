// SPIKE ONLY (G2): launch Chrome / Edge / Firefox with a fresh profile + the spike
// extension, wait, then report what the native host logged for that browser.
// Usage: node browsers.cjs <label> [chrome|edge|firefox ...]
const puppeteer = require("puppeteer-core");
const fs = require("fs"), os = require("os"), path = require("path");
const EXT = path.resolve(__dirname, "../extension");
const LOG = path.resolve(__dirname, "../out/host.log");
const L = process.env.LOCALAPPDATA;
const BIN = {
  chrome: `${L}\\Google\\Chrome\\Application\\chrome.exe`,
  edge: "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  firefox: `${L}\\Mozilla Firefox\\firefox.exe`,
};
const label = process.argv[2] || "run";
const which = process.argv.slice(3).length ? process.argv.slice(3) : Object.keys(BIN);
process.env.GDM_SPIKE_LOG = LOG; // inherited by the host the browser spawns

(async () => {
  const results = {}; let failed = false;
  for (const b of which) {
    const before = fs.existsSync(LOG) ? fs.readFileSync(LOG, "utf8").length : 0;
    const prof = fs.mkdtempSync(path.join(os.tmpdir(), `gdm-${b}-`));
    let browser;
    try {
      if (b === "firefox") {
        browser = await puppeteer.launch({ browser: "firefox", executablePath: BIN[b], userDataDir: prof, headless: false });
        await browser.installExtension(EXT);
      } else if (b === "edge") { // puppeteer exits code 0 with Edge 157 here; plain spawn works
        const cp = require("child_process").spawn(BIN[b], [`--user-data-dir=${prof}`, "--no-first-run", "--no-default-browser-check",
          `--load-extension=${EXT}`, `--disable-extensions-except=${EXT}`, "about:blank"], { stdio: "ignore" });
        browser = { close: async () => { require("child_process").spawnSync("taskkill", ["/T", "/F", "/PID", String(cp.pid)]); }, process: () => null };
      } else {
        browser = await puppeteer.launch({ executablePath: BIN[b], userDataDir: prof, headless: false, pipe: true, enableExtensions: [EXT],
          args: ["--no-first-run", "--no-default-browser-check"] });
      }
      await new Promise((r) => setTimeout(r, 25000));
    } catch (e) { results[b] = "FAIL LAUNCH ERROR: " + e.message; failed = true; }
    finally { // close can hang (seen with Firefox); kill the process after 5 s
      if (browser) await Promise.race([browser.close().catch(() => {}), new Promise((r) => setTimeout(r, 5000))])
        .then(() => { try { browser.process()?.kill(); } catch {} });
    }
    const lines = (fs.existsSync(LOG) ? fs.readFileSync(LOG, "utf8").slice(before) : "").trim().split("\n").filter(Boolean);
    // PASS only when the extension confirmed every check (ACK lines) and the host saw all calls.
    const acks = ["", "-all100", "-ports2", "-dirrej"].map((t) => lines.some((l) => new RegExp(` ACK \\w+${t} ver=`).test(l)));
    const pings = lines.filter((l) => / PING /.test(l)).length;
    const vers = [...new Set(lines.map((l) => l.split(" ")[0]))].join(",");
    const ok = acks.every(Boolean) && pings === 102;
    if (!ok) failed = true;
    results[b] ??= `${ok ? "PASS" : "FAIL"} first=${acks[0]} all100=${acks[1]} ports2=${acks[2]} dirrej=${acks[3]} pings=${pings}/102 hostver=${vers}`;
    fs.appendFileSync(path.resolve(__dirname, "../out/browsers.log"), `[${label}] ${b} ${BIN[b]}: ${results[b]}\n${lines.map((l) => "    " + l).join("\n")}\n`);
  }
  console.log(label, JSON.stringify(results, null, 1));
  process.exit(failed ? 1 : 0);
})();
