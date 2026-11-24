// Differential test of the colour manager: the original ColorManager (triggers.js, run as-is) vs colors.rs under random operations.
// usage: node tools/compare-colors.mjs [seed] [sequences] [opsPerSequence]
import fs from "node:fs";
import vm from "node:vm";
import { load } from "./physics-harness.mjs";

const root = new URL("..", import.meta.url).pathname;
const H = await load();
const { ctx, w } = H;
vm.runInContext("var fs = 1000; var gs = 1001;", ctx);
vm.runInContext(fs.readFileSync(root + "ref/assets/scripts/core/triggers.js", "utf8") + "\nthis.ColorManager = ColorManager; this.parseChannelHsv = parseChannelHsv;", ctx);
let seed = Number(process.argv[2] || 1);
const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296);
const pick = (a) => a[Math.floor(rnd() * a.length)];
const SEQ = Number(process.argv[3] || 300), OPS = Number(process.argv[4] || 60);
const CH = [1, 2, 3, 4, 5, 1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007, 1009, 1010, 1011, 1012, 8, 20];
const SRC = CH.filter((c) => ![1000, 1001, 1007].includes(c)); // copying from those can recurse forever in the original (getColor(fs) restarts the depth count)
const ptr = w.wd_alloc(16 * 8), out = w.wd_alloc(5 * 8);
const mem = () => new Float64Array(w.memory.buffer, ptr, 16), om = () => new Float64Array(w.memory.buffer, out, 5);
const op = (code, args) => { const m = mem(); m.fill(0); args.forEach((v, i) => (m[i] = v)); w.wd_cm_op(code, ptr); };
const same = (a, b) => Object.is(a, b) || a === b || (Number.isNaN(a) && Number.isNaN(b));
const rgbv = () => ({ r: Math.floor(rnd() * 256), g: Math.floor(rnd() * 256), b: Math.floor(rnd() * 256) });
const hsvv = () => (rnd() < 0.5 ? null : { h: (rnd() - 0.5) * 720, s: rnd() * 2, v: rnd() * 2, sAdd: rnd() < 0.3, vAdd: rnd() < 0.3 });
const hsvArgs = (h) => (h ? [1, h.h, h.s, h.v, +h.sAdd, +h.vAdd] : [0, 0, 0, 0, 0, 0]);
let bad = 0, checks = 0;
for (let s = 0; s < SEQ && bad < 5; s++) {
  const main = Math.floor(rnd() * 0xffffff), sec = Math.floor(rnd() * 0xffffff);
  ctx.window.mainColor = main; ctx.window.secondaryColor = sec;
  const cm = new ctx.ColorManager();
  w.wd_cm_new(main, sec);
  // `new ColorManager()` already ran reset(); initial values are set like loadLevel does
  const log = [];
  for (let k = 0; k < OPS; k++) {
    const r = rnd();
    let desc;
    if (r < 0.06) { const ch = pick(CH), c = rgbv(); cm.setInitialColor(ch, c); op(1, [ch, c.r, c.g, c.b]); desc = "initialColor"; }
    else if (r < 0.1) { const ch = pick(CH), src = pick(SRC), h = hsvv(); cm.setInitialCopy(ch, { source: src, hsv: h }); op(2, [ch, src, ...hsvArgs(h)]); desc = "initialCopy"; }
    else if (r < 0.14) { const ch = pick(CH), v = rnd(); cm.setInitialOpacity(ch, v); op(3, [ch, v]); desc = "initialOpacity"; }
    else if (r < 0.2) { cm.reset(); op(0, []); desc = "reset"; }
    else if (r < 0.5) {
      const ch = pick(CH), c = rgbv(), dur = rnd() < 0.3 ? 0 : rnd() * 1200, copy = rnd() < 0.3 ? { source: pick(SRC), hsv: hsvv() } : null, opacity = rnd() < 0.3 ? rnd() * 1.4 - 0.2 : undefined;
      cm.triggerColor(ch, c, dur, copy, opacity);
      op(4, [ch, c.r, c.g, c.b, dur, copy ? copy.source : 0, ...(copy ? hsvArgs(copy.hsv) : [0, 0, 0, 0, 0, 0]), opacity === undefined ? NaN : opacity]);
      desc = "trigger";
    } else if (r < 0.62) {
      const ch = pick(CH), c = rgbv(), fi = rnd() < 0.2 ? 0 : rnd() * 800, ho = rnd() * 800, fo = rnd() < 0.2 ? 0 : rnd() * 800, h = rnd() < 0.3 ? hsvv() : null, cs = rnd() < 0.3 ? pick(SRC) : 0;
      cm.startPulse(ch, { color: c, hsv: h, fadeIn: fi, hold: ho, fadeOut: fo, copySource: cs });
      op(6, [ch, c.r, c.g, c.b, fi, ho, fo, ...hsvArgs(h), cs]);
      desc = "pulse";
    } else if (r < 0.64) { cm.clearPulses(); op(7, []); desc = "clearPulses"; }
    else { const ms = rnd() < 0.2 ? rnd() * 3000 : 16.67 * (1 + Math.floor(rnd() * 3)); cm.step(ms); op(5, [ms]); desc = "step " + ms; }
    log.push(desc);
    for (const ch of CH) {
      const c = cm.getColor(ch), o = cm.getOpacity(ch), hx = cm.getHex(ch);
      w.wd_cm_query(ch, out);
      const m = om();
      checks++;
      if (!same(c.r, m[0]) || !same(c.g, m[1]) || !same(c.b, m[2]) || !same(o, m[3]) || !same(hx, m[4])) {
        bad++;
        console.log(`MISMATCH seq ${s} op ${k} (${desc}) channel ${ch}: js rgb=${c.r},${c.g},${c.b} op=${o} hex=${hx} | rust rgb=${m[0]},${m[1]},${m[2]} op=${m[3]} hex=${m[4]}\n  ops: ${log.slice(-6).join(", ")}`);
        break;
      }
    }
    if (bad) break;
  }
}
console.log(`${SEQ} sequences, ${checks} colour reads: ${bad ? bad + " MISMATCHED" : "all identical"}`);
process.exit(bad ? 1 : 0);
