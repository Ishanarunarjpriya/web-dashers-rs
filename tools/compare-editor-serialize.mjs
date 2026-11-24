// Differential test: LevelEditor._serializeLevel/_serializeObject (JS original) vs editor.rs (wasm).
// Both parse every shipped level (and hand-made edge cases), re-serialize it, and the decompressed texts must be identical.
globalThis.__random ??= Math.random;
import fs from "node:fs";
import vm from "node:vm";
import assert from "node:assert";

const root = new URL("..", import.meta.url).pathname;
const lvl = fs.readFileSync(root + "ref/assets/scripts/core/level.js", "utf8");
const ed = fs.readFileSync(root + "ref/assets/scripts/core/level-editor.js", "utf8");
const a = lvl.indexOf("function _decodeTextObjectString");
const b = lvl.indexOf("function getBackgroundTextureIndex");
const ctx = vm.createContext({ atob, btoa, TextDecoder, TextEncoder, Uint8Array, console, window: {} });
vm.runInContext(fs.readFileSync(root + "ref/assets/scripts/libs/pako.min.js", "utf8"), ctx);
vm.runInContext(fs.readFileSync(root + "ref/assets/scripts/game/allObjects.js", "utf8") + "\nconst allObjects = window.allobjects();", ctx);
vm.runInContext(lvl.slice(a, b) + "\nthis.parseLevel = parseLevel;", ctx);
vm.runInContext(lvl.slice(lvl.indexOf("function getObjectFromId"), lvl.indexOf("const EMPTY_TRIGGER_LIST")) + "\nthis.getObjectFromId = getObjectFromId;", ctx);
// the editor's helpers, sliced verbatim
const helpers = ed.slice(0, ed.indexOf("class LevelEditor"));
const method = (name, next) => { const i = ed.indexOf("\n  " + name + "(") + 1; const j = ed.indexOf("\n  " + next + "(", i); return ed.slice(i, j); };
const grp = method("_getEditorObjectGroupIds", "_syncEditorObjectGroupCaches");
const serLevel = ed.slice(ed.indexOf("_serializeLevel(levelData)"), ed.indexOf("  _saveEditorLevel()"));
vm.runInContext(helpers + "\nconst scene = {};\nclass Fake {\n" + grp + "\n" + serLevel + "\n}\nconst f = new Fake();\nf._serializeObject = f._serializeObject.bind(f);\nthis.serializeLevel = (d) => f._serializeLevel(d);\nthis.pakoInflate = (s) => pako.inflate(Uint8Array.from(atob(s.replace(/-/g,'+').replace(/_/g,'/')), c => c.charCodeAt(0)), { to: 'string' });", ctx);

const { instance } = await WebAssembly.instantiate(fs.readFileSync(process.env.WASM || root + "target/wasm32-unknown-unknown/release/wd_core.wasm"), { env: { js_sin: Math.sin, js_cos: Math.cos, js_tan: Math.tan, js_acos: Math.acos, js_atan: Math.atan, js_atan2: Math.atan2, js_pow: Math.pow, js_random: () => globalThis.__random() } });
const w = instance.exports;
const rs = (str, plain) => {
  const bytes = new TextEncoder().encode(str);
  const p = w.wd_alloc(Math.max(1, bytes.length));
  new Uint8Array(w.memory.buffer, p, bytes.length).set(bytes);
  const n = w.wd_ed_roundtrip(p, bytes.length, plain);
  return new TextDecoder().decode(new Uint8Array(w.memory.buffer, w.wd_result_ptr(), n));
};
let bad = 0, objects = 0;
const check = (name, str) => {
  const parsed = ctx.parseLevel(str);
  const jsOut = ctx.serializeLevel({ objects: parsed.objects, settings: parsed.settings });
  const jsPlain = ctx.pakoInflate(jsOut);
  const rsPlain = rs(str, 1);
  // the Rust compressed output must decode (with the JS decoder) to the same text
  const rsPlain2 = ctx.pakoInflate(rs(str, 0));
  try { assert.strictEqual(rsPlain, jsPlain); assert.strictEqual(rsPlain2, jsPlain); objects += parsed.objects.length; }
  catch (e) {
    bad++;
    const A = jsPlain.split(";"), B = rsPlain.split(";");
    const i = A.findIndex((x, k) => x !== B[k]);
    console.log("MISMATCH", name, "objects", A.length, B.length, "first diff at", i, "\n js:", (A[i] || "").slice(0, 300), "\n rs:", (B[i] || "").slice(0, 300));
  }
};
const files = fs.readdirSync(root + "ref/assets/levels").filter((f) => f.endsWith(".txt")).sort();
for (const f of files) check(f, fs.readFileSync(root + "ref/assets/levels/" + f, "utf8"));
const edge = [
  "kA2,1;1,5,2,10abc,3,,4,1,6,45.5e1,32,0,33,1.2.2.3,57,3.0.-4,31,aGVsbG8,21,7,19,3;",
  "kS38,x;1,0,2,5;1,9,2,NaN,3,Infinity,128,-2,129,0;1,10,2,1e3,3,.5,5,1,131,10,132,200,31,a;1,11,31,_-8,31,aGk;",
  "plain,text;;;1,1,2,3,3,4;1,2,2",
  ";1,1,2,3,3,4,31,aGVsbG8=;", ";749,1,2,3;1,1,2,1e21,3,1e-7,6,-0;1,2,2,0.1,3,1e300,32,2.5;1,3,2,4,3,5,128,1,129,2,01,x,007,y,kZ,1,10,q,4294967295,z,4294967294,w;",
  ";1346,5,51,4.5.6,71,0.9,2,0,3,0;1346,5,51,x,71,3,2,0,3,0;914,1,2,5,3,5,31,SGVsbG8gV29ybGQ,2,1,3,1;914,2,31,SGk,2,1,3,1;1,1,2,5,3,5,31,xyz,31,abc;",
  ";1,1,2,5,3,5,6,10,131,20,132,50;1,1,2,5,3,5,131,350,132,10;1,1,2,5,3,5,kA2,3,kA3,1,kA4,2,kA8,1,kA28,1,kA11,1,kA3,0;",
];
edge.forEach((s, i) => check("edge" + i, s));
// random fuzz of numeric fields
let seed = 12345; const rnd = () => (seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296;
const nums = ["0", "1", "-1", "0.5", "1e5", "1e21", "1.5e-7", "123456789.123456789", "-0", "NaN", "abc", "30", "15.000000000000002", "0.1", ".7", "7.", "1e400"];
for (let n = 0; n < 300; n++) {
  const objs = [];
  for (let k = 0; k < 8; k++) {
    const keys = ["2", "3", "6", "32", "128", "129", "131", "132", "20", "24", "25", "61", "21", "22", "57", "33", "kA2", "kA3", "kA4", "kA8", "kA28", "kA11", "4", "5", "19"];
    const parts = ["1", String([1, 8, 35, 211, 914, 1346, 749, 1, 100][Math.floor(rnd() * 9)])];
    for (const key of keys) if (rnd() < 0.4) parts.push(key, rnd() < 0.8 ? nums[Math.floor(rnd() * nums.length)] : String(Math.floor(rnd() * 200)));
    if (rnd() < 0.3) parts.push("zz" + Math.floor(rnd() * 5), "v");
    objs.push(parts.join(","));
  }
  check("fuzz" + n, "kA9,0;" + objs.join(";"));
}
console.log(`${files.length} level files + ${edge.length} edge cases + 300 fuzz: ${bad ? bad + " MISMATCHED" : "all identical"} (${objects} objects compared)`);
process.exit(bad ? 1 : 0);
