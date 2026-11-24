// Differential test: JS original (ref/assets/scripts/core/level.js) vs the Rust/wasm port.
globalThis.__random ??= Math.random;
import fs from "node:fs";
import vm from "node:vm";
import assert from "node:assert";

const root = new URL("..", import.meta.url).pathname;
const src = fs.readFileSync(root + "ref/assets/scripts/core/level.js", "utf8");
const a = src.indexOf("function _decodeTextObjectString");
const b = src.indexOf("function getBackgroundTextureIndex");
const ctx = vm.createContext({ atob, btoa, TextDecoder, TextEncoder, Uint8Array, console });
vm.runInContext(fs.readFileSync(root + "ref/assets/scripts/libs/pako.min.js", "utf8"), ctx);
vm.runInContext(src.slice(a, b) + "\nthis.parseLevel = parseLevel; this.parseObject = parseObject;", ctx);

const { instance } = await WebAssembly.instantiate(fs.readFileSync(root + "target/wasm32-unknown-unknown/release/wd_core.wasm"), { env: { js_sin: Math.sin, js_cos: Math.cos, js_tan: Math.tan, js_acos: Math.acos, js_atan: Math.atan, js_atan2: Math.atan2, js_pow: Math.pow, js_random: () => globalThis.__random() } });
const w = instance.exports;
const rustParse = (str) => {
  const bytes = new TextEncoder().encode(str);
  const p = w.wd_alloc(bytes.length);
  new Uint8Array(w.memory.buffer, p, bytes.length).set(bytes);
  const n = w.wd_parse_level(p, bytes.length);
  return JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer, w.wd_result_ptr(), n)));
};
const jsParse = (str) => JSON.parse(JSON.stringify(ctx.parseLevel(str))); // NaN -> null, like the Rust writer

let files = fs.readdirSync(root + "ref/assets/levels").filter((f) => f.endsWith(".txt")).sort();
let objects = 0, bad = 0;
const check = (name, str) => {
  const js = jsParse(str), rs = rustParse(str);
  try { assert.deepStrictEqual(rs, js); objects += js.objects.length; }
  catch (e) {
    bad++;
    const i = js.objects.findIndex((o, k) => JSON.stringify(o) !== JSON.stringify(rs.objects[k]));
    console.log("MISMATCH", name, "settings equal:", js.settings === rs.settings, "objects", js.objects.length, rs.objects.length, "first diff at", i);
    if (i >= 0) console.log(" js:", JSON.stringify(js.objects[i]).slice(0, 400), "\n rs:", JSON.stringify(rs.objects[i]).slice(0, 400));
  }
};
for (const f of files) check(f, fs.readFileSync(root + "ref/assets/levels/" + f, "utf8"));

// hand-made edge cases for the number/base64 quirks
const edge = [
  "kA2,1;1,5,2,10abc,3,,4,1,6,45.5e1,32,0,33,1.2.2.3,57,3.0.-4,31,aGVsbG8,21,7,19,3;",
  "kS38,x;1,0,2,5;1,9,2,NaN,3,Infinity,128,-2,129,0;1,10,2,1e3,3,.5,5,1,131,10,132,200,31,a;1,11,31,_-8,31,aGk;",
  "plain,text;;;1,1,2,3,3,4;1,2,2",
  "   H4sIAAAAAAAA  ", "", "not base64 at all", ";1,1,2,3,3,4,31,aGVsbG8=;",
];
edge.forEach((s, i) => check("edge" + i, s));
console.log(`${files.length} level files + ${edge.length} edge cases: ${bad ? bad + " MISMATCHED" : "all identical"} (${objects} objects compared)`);
process.exit(bad ? 1 : 0);
