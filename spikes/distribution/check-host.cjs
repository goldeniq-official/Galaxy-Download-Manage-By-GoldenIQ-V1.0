// SPIKE ONLY (G2): protocol self-check for gdm-bridge. Usage: node check-host.js <exe>
const { spawnSync } = require("child_process");
const assert = require("assert");
const exe = process.argv[2];
const ORIGIN = "chrome-extension://fccmejdfnnlgffbbcbakjpdodjjoeonh/";
const frame = (o) => { const b = Buffer.from(typeof o === "string" ? o : JSON.stringify(o)); const h = Buffer.alloc(4); h.writeUInt32LE(b.length); return Buffer.concat([h, b]); };
const run = (args, input) => {
  const r = spawnSync(exe, args, { input });
  const out = []; let i = 0;
  while (i + 4 <= r.stdout.length) { const n = r.stdout.readUInt32LE(i); out.push(JSON.parse(r.stdout.slice(i + 4, i + 4 + n))); i += 4 + n; }
  assert.strictEqual(i, r.stdout.length, "stdout has bytes outside complete frames");
  return { code: r.status, out };
};
let r = run([ORIGIN], Buffer.concat([frame({ type: "ping", nonce: "n1" }), frame({ type: "ack", nonce: "n1", ver: "0.1.0" })]));
assert.deepStrictEqual([r.code, r.out[0].type, r.out[0].nonce, r.out[1].ok], [0, "pong", "n1", true]);
r = run(["chrome-extension://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/"], frame({ type: "ping", nonce: "x" }));
assert.deepStrictEqual([r.code, r.out.length], [3, 0]); // B-01 foreign origin
r = run(["C:/x/firefox.json", "evil@x"], frame({ type: "ping", nonce: "x" }));
assert.strictEqual(r.code, 3); // B-01 foreign Firefox id
r = run(["C:/x/firefox.json", "gdm-spike@goldeniq.dev"], frame({ type: "ping", nonce: "f" }));
assert.strictEqual(r.out[0].nonce, "f");
r = run([ORIGIN], frame({ type: "handoff", url: "https://e.com/a.exe", dir: "C:/Startup" }));
assert.strictEqual(r.out[0].ok, false); // B-01 no dir field
const big = Buffer.alloc(4); big.writeUInt32LE(0xFFFFFFF0);
r = run([ORIGIN], big);
assert.deepStrictEqual([r.code, r.out.length], [2, 0]); // B-03 4 GB prefix refused before alloc
r = run([ORIGIN], frame("[".repeat(100000)));
assert.strictEqual(r.out[0].ok, false); // B-03 deep JSON rejected, no crash
r = run([ORIGIN], frame('{"type":"handoff","url":"https://e.com/","name":' + "[".repeat(50000) + "]".repeat(50000) + "}"));
assert.deepStrictEqual([r.code, r.out[0].ok], [0, false]); // deep nesting inside a tagged object
r = run([ORIGIN], frame({ type: "handoff", url: "https://e.com/" + "a".repeat(9000) }));
assert.strictEqual(r.out[0].ok, false); // URL > 8 KiB
r = run([ORIGIN], frame({ type: "handoff", url: "file:///C:/Windows/win.ini" }));
assert.strictEqual(r.out[0].ok, false); // non-http scheme
r = run([ORIGIN], Buffer.from([1, 0])); // short header
assert.deepStrictEqual([r.code, r.out.length], [0, 0]);
const lie = Buffer.alloc(4); lie.writeUInt32LE(1000); // length with no payload
r = run([ORIGIN], lie);
assert.deepStrictEqual([r.code, r.out.length], [2, 0]);
const bad = Buffer.from([0x7b, 0xff, 0xfe, 0x7d]); const bh = Buffer.alloc(4); bh.writeUInt32LE(4);
r = run([ORIGIN], Buffer.concat([bh, bad])); // broken UTF-8
assert.strictEqual(r.out[0].ok, false);
for (let i = 0; i < 100; i++) { // foreign origins 100/100
  const id = [...Array(32)].map(() => "abcdefghijklmnop"[Math.random() * 16 | 0]).join("");
  assert.strictEqual(run([`chrome-extension://${id}/`], frame({ type: "ping", nonce: "x" })).code, 3);
}
// every stdout byte is a frame: 50 pings in one connection give exactly 50 frames, nothing else
r = run([ORIGIN], Buffer.concat([...Array(50)].map((_, i) => frame({ type: "ping", nonce: "k" + i }))));
assert.deepStrictEqual([r.out.length, r.out[49].nonce], [50, "k49"]);
console.log("check-host: all 15 checks passed (incl. 100/100 foreign origins rejected)");
