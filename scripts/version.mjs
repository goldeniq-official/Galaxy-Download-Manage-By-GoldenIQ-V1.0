// SPDX-License-Identifier: RPL-1.5
// GDM has ONE version: package.json (UI, extension) == [workspace.package] in Cargo.toml (app, bridge, core UA;
// tauri.conf.json takes it from Cargo). This is the only thing that changes it; the release workflow runs it.
//   node scripts/version.mjs            -> prints the current version
//   node scripts/version.mjs minor      -> 0.9.0 -> 0.10.0   (also: major, patch, or an exact X.Y.Z)
import fs from "node:fs";

const read = (f) => fs.readFileSync(f, "utf8");
const pkg = JSON.parse(read("package.json"));
const cur = pkg.version;
const arg = process.argv[2];
if (!arg) {
  console.log(cur);
  process.exit(0);
}
const [ma, mi, pa] = cur.split(".").map(Number);
const next = { major: `${ma + 1}.0.0`, minor: `${ma}.${mi + 1}.0`, patch: `${ma}.${mi}.${pa + 1}` }[arg] ?? arg;
if (!/^\d+\.\d+\.\d+$/.test(next)) throw new Error(`not major|minor|patch|X.Y.Z: ${arg}`);

const swap = (f, re, to) => {
  const s = read(f);
  if (!re.test(s)) throw new Error(`${f}: version not found`);
  fs.writeFileSync(f, s.replace(re, to));
};
swap("package.json", /("version":\s*)"[^"]+"/, `$1"${next}"`);
swap("Cargo.toml", /(\[workspace\.package\]\r?\nversion = )"[^"]+"/, `$1"${next}"`);
// the lock entries of the workspace's own crates (the release builds with --locked)
const own = ["galaxy-download-manager", "gdm-core", "gdm-bridge", "gdm-media", "gdm-auth", "gdm-fixture"];
swap("Cargo.lock", new RegExp(`(name = "(?:${own.join("|")})"\\r?\\nversion = )"[^"]+"`, "g"), `$1"${next}"`);
console.log(next);
