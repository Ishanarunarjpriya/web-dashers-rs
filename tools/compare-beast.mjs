// Differential test of the BeastAnim state machine (animations.js `createState`, `advance`, `command`, `commandMany`, `defaultBase`)
// versus beast.rs, on random operation sequences with identical random streams.
// usage: node tools/compare-beast.mjs [seed] [rounds]
import fs from "node:fs";
import vm from "node:vm";
import { load } from "./physics-harness.mjs";

const root = new URL("..", import.meta.url).pathname;
const H = await load();
const { ctx, w } = H;
let seed = Number(process.argv[2] || 1);
const rounds = Number(process.argv[3] || 4000);
const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296);
const stream = (s0) => { let s = s0 >>> 0; return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296); };
ctx.__rnd = stream(99); vm.runInContext("Math.random = __rnd", ctx);
globalThis.__random = stream(99);
const put = (str) => { const b = new TextEncoder().encode(str); const p = w.wd_alloc(Math.max(1, b.length)); new Uint8Array(w.memory.buffer, p, b.length).set(b); return [p, b.length]; };
const text = (n) => new TextDecoder().decode(new Uint8Array(w.memory.buffer, w.wd_result_ptr(), n));
for (const k of [1, 2, 3, 4, 5]) {
  const name = `GJBeast0${k}`;
  const src = fs.readFileSync(root + `ref/assets/sheets/${name}_AnimDesc.json`, "utf8");
  ctx[`${name}_AnimDesc`] = JSON.parse(src);
  const [np, nl] = put(name), [jp, jl] = put(src);
  if (!w.wd_beast_desc_add(np, nl, jp, jl)) throw new Error("desc rejected " + name);
}
vm.runInContext(fs.readFileSync(root + "ref/assets/scripts/core/animations.js", "utf8"), ctx);
const B = ctx.window.BeastAnim;
const ids = [918, 1327, 1328, 1584, 2012];
const names = { 918: ["bite", "attack01", "attack01_end", "idle01", "attack01_loop"], 1584: 10, 2012: 9 };
const jsJson = (s) => [s.base ?? "", s.animId, s.frameIdx, s.timer, s.rng, s.pendingCommand ? [s.pendingCommand.base, s.pendingCommand.animIdx, s.pendingCommand.delay] : null, s.total];
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const states = [];
let bad = 0, ops = 0;
for (let slot = 0; slot < 15; slot++) {
  const id = ids[slot % 5];
  const s = B.createState(id);
  states.push(s);
  w.wd_beast_test_new(slot, id);
}
const rj = (n) => JSON.parse(text(n));
const check = (slot, rust, what) => {
  const a = jsJson(states[slot]);
  if (!same(a, rust)) { console.log(`MISMATCH after ${what} slot ${slot} (id ${ids[slot % 5]}): js=${JSON.stringify(a)} rust=${JSON.stringify(rust)}`); bad++; return false; }
  return true;
};
const seen = new Set();
for (let r = 0; r < rounds && !bad; r++) {
  const slot = Math.floor(rnd() * 15), s = states[slot];
  const x = rnd();
  ops++;
  if (x < 0.62) {
    const dt = [1 / 60, 1 / 60, 1 / 30, 0.016, 0.0001, 0.05, 0.1, 0.3, 2, 0, -1, NaN][Math.floor(rnd() * 12)];
    B.advance(s, dt);
    if (!check(slot, rj(w.wd_beast_test_op(slot, 0, dt, 0)), `advance ${dt}`)) break;
  } else if (x < 0.8) {
    const idx = Math.floor(rnd() * 12) - 1;
    const ov = rnd() < 0.4 ? undefined : [0, 0.1, 0.4, 0.9, NaN, -2][Math.floor(rnd() * 6)];
    B.command(s, idx, ov);
    if (!check(slot, rj(w.wd_beast_test_op(slot, 1, idx, ov === undefined ? NaN : ov)), `command ${idx} ${ov}`)) break;
  } else if (x < 0.93) {
    const n = 1 + Math.floor(rnd() * 4);
    const pick = Array.from({ length: n }, () => Math.floor(rnd() * 15));
    const uniq = [...new Set(pick)];
    const idx = Math.floor(rnd() * 10);
    B.commandMany(uniq.map((i) => states[i]), idx);
    const p = w.wd_alloc(uniq.length * 8); new Float64Array(w.memory.buffer, p, uniq.length).set(uniq);
    w.wd_beast_test_many(p, uniq.length, idx);
    for (const i of uniq) if (!check(i, rj(w.wd_beast_test_op(i, 3, 0, 0)), `commandMany ${idx}`)) break;
  } else {
    B.defaultBase(s);
    if (!check(slot, rj(w.wd_beast_test_op(slot, 2, 0, 0)), "defaultBase")) break;
  }
  seen.add(`${ids[slot % 5]}:${s.base}`);
}
console.log(`${ops} operations, ${seen.size} (beast, animation) combinations seen: ${bad ? "MISMATCH" : "all identical"}`);
process.exit(bad ? 1 : 0);
