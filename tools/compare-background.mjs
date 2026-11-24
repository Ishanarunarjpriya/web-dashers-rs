// Differential test of the scrolling background and art-id helpers: original JS (verbatim slices) vs background.rs.
import fs from "node:fs";
import vm from "node:vm";
import { load } from "./physics-harness.mjs";

const root = new URL("..", import.meta.url).pathname;
const { ctx, w } = await load();
const lv = fs.readFileSync(root + "ref/assets/scripts/core/level.js", "utf8");
const gs = fs.readFileSync(root + "ref/assets/scripts/core/game-scene.js", "utf8");
const i0 = lv.indexOf("function getBackgroundTextureIndex"), i1 = lv.indexOf("const PERSP_LINE_FRAME_OFFSET_Y");
const a0 = gs.indexOf("  _applyMirroredBackgroundTexture(textureKey) {"), a1 = gs.indexOf("  _updateCameraY(");
const sh = gs.indexOf("  _mirrorProgress() {");
vm.runInContext(`${lv.slice(i0, i1)}
this.getBackgroundTextureIndex = getBackgroundTextureIndex; this.getBackgroundDisplayId = getBackgroundDisplayId; this.getGroundTextureId = getGroundTextureId;
const screenHeight = 640;
this.BG = class BG {
  _createMirroredBackgroundTexture(k) { return k; }
  ${gs.slice(a0, a1)}
  ${gs.slice(sh, gs.indexOf("  toggleMirror(", sh))}
};`, ctx);
let seed = 7;
const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296);
const same = (a, b) => a === b || (Number.isNaN(a) && Number.isNaN(b));
const enc = (s) => { const b = new TextEncoder().encode(s); const p = w.wd_alloc(Math.max(1, b.length)); new Uint8Array(w.memory.buffer, p, b.length).set(b); return [p, b.length]; };
const str = (n) => new TextDecoder().decode(new Uint8Array(w.memory.buffer, w.wd_result_ptr(), n));
let bad = 0, checks = 0;
const samples = ["1", "0", "-3", "7", "20", "abc", "12abc", " 5", "3.9", "", "007", "1e3", "99999999999", "-0"];
for (const sv of [...samples, undefined, null]) {
  const [p, n] = sv == null ? [0, -1] : enc(sv);
  const js = [ctx.getBackgroundTextureIndex(sv), ctx.getBackgroundDisplayId(sv), ctx.getGroundTextureId(sv)].map(String);
  const rs = [0, 1, 2].map((k) => str(w.wd_art_id(k, p, n)));
  checks++;
  if (js.join() !== rs.join()) { bad++; console.log("ids", JSON.stringify(sv), js, rs); }
}
const ptr = w.wd_alloc(8 * 8);
const m = () => new Float64Array(w.memory.buffer, ptr, 8);
for (let s = 0; s < 500; s++) {
  const bg = new ctx.BG();
  bg._bgSpeedX = 0.1; bg._bgSpeedY = 0.1; bg._bg = { tilePositionX: 0, tilePositionY: 0, setTexture() {} };
  bg._cameraX = 0; bg._prevCameraX = 0; bg._cameraY = 0; bg._mirrorAnimating = false; bg._state = { mirrored: false };
  let shifted = 0; bg._level = { shiftGroundTiles(a) { shifted += a; } };
  const h = rnd() < 0.2 ? 0 : Math.round(500 + rnd() * 1500);
  bg.textures = { get: () => ({ source: [{ height: h }] }) };
  bg._applyMirroredBackgroundTexture("k");
  w.wd_bg_op(0, ptr); m()[0] = h; w.wd_bg_op(1, ptr);
  let cam = rnd() * 1000;
  for (let k = 0; k < 200; k++) {
    cam += rnd() < 0.2 ? 0 : (rnd() - 0.2) * 40;
    const cy = rnd() * 600, mp = rnd() < 0.5 ? 0 : (rnd() < 0.5 ? 1 : rnd());
    bg._cameraX = cam; bg._cameraY = cy;
    if (mp === 1) { bg._state.mirrored = true; bg._mirrorAnimating = false; } else if (mp === 0) { bg._state.mirrored = false; bg._mirrorAnimating = false; } else { bg._mirrorAnimating = true; bg._mirrorP = mp; }
    shifted = 0;
    bg._updateBackground();
    m().set([cam, cy, mp]); w.wd_bg_op(2, ptr);
    const o = m();
    checks++;
    if (!same(bg._bg.tilePositionX, o[3]) || !same(bg._bg.tilePositionY, o[4]) || !same(shifted, o[5])) { bad++; console.log(`MISMATCH seq ${s} step ${k}`, bg._bg.tilePositionX, o[3], bg._bg.tilePositionY, o[4], shifted, o[5]); if (bad > 5) process.exit(1); }
  }
}
console.log(`${checks} checks: ${bad ? bad + " mismatches" : "all identical"}`);
process.exit(bad ? 1 : 0);
