// SPIKE ONLY (G2). On start, call the native host and validate every reply, then report the
// results to the host as ACK nonces, so the host log proves the browser side checked them:
//   <n>        first ping round trip      <n>-all100  100/100 replies valid
//   <n>-ports2 connectNative x2 replied   <n>-dirrej  forbidden `dir` field answered ok:false
const HOST = "com.goldeniq.gdm";
const api = globalThis.browser ?? chrome;
const send = globalThis.browser // Firefox: promise API, no callback argument
  ? (msg) => browser.runtime.sendNativeMessage(HOST, msg)
  : (msg) => new Promise((ok, bad) => chrome.runtime.sendNativeMessage(HOST, msg, (r) => {
      const e = chrome.runtime.lastError; e ? bad(new Error(e.message)) : ok(r);
    }));
const valid = (r, nonce) => r && r.ok === true && r.type === "pong" && r.nonce === nonce && typeof r.ver === "string";
const portPing = (nonce) => new Promise((ok, bad) => {
  const port = api.runtime.connectNative(HOST);
  const t = setTimeout(() => { port.disconnect(); bad(new Error("port timeout")); }, 5000);
  port.onDisconnect.addListener(() => { clearTimeout(t); bad(new Error("port closed early")); });
  port.onMessage.addListener((r) => { clearTimeout(t); port.disconnect(); ok(valid(r, nonce)); });
  port.postMessage({ type: "ping", nonce });
});
(async () => {
  const n = Math.random().toString(36).slice(2, 12);
  try {
    const r = await send({ type: "ping", nonce: n });
    if (!valid(r, n)) throw new Error("bad first reply");
    const ack = (tag) => send({ type: "ack", nonce: tag ? `${n}-${tag}` : n, ver: r.ver });
    await ack("");
    let good = 1;
    for (let i = 0; i < 99; i++) if (valid(await send({ type: "ping", nonce: `${n}-${i}` }), `${n}-${i}`)) good++;
    if (good === 100) await ack("all100");
    let ports = 0;
    for (const tag of ["port1", "port2"]) if (await portPing(`${n}-${tag}`).catch(() => false)) ports++;
    if (ports === 2) await ack("ports2");
    // B-01: the protocol has no dir/path field; the host must answer ok:false.
    const d = await send({ type: "handoff", url: "https://example.com/a.exe", dir: "C:/evil" }).catch(() => null);
    if (d && d.ok === false) await ack("dirrej");
  } catch (e) { console.error("native host failed:", e.message); }
})();
