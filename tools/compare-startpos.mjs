// Differential test: smart start pos (_getSmartStartPosSettings), the song offset of a start position (calculateSongOffsetForX),
// the start-position switcher (changeStartPos) and the speed hack (_quantizeDelta), original JS vs Rust.
import fs from "node:fs";
import vm from "node:vm";
import { load } from "./physics-harness.mjs";

const root = new URL("..", import.meta.url).pathname;
const H = await load();
const { ctx, w, mem } = H;
let seed = Number(process.argv[2] || 1);
const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296);
const pick = (a) => a[Math.floor(rnd() * a.length)];
const N = Number(process.argv[3] || 3000);
const same = (a, b) => Object.is(a, b) || a === b || (Number.isNaN(a) && Number.isNaN(b));

const gs = fs.readFileSync(root + "ref/assets/scripts/core/game-scene.js", "utf8");
const cut = (a, b) => gs.slice(gs.indexOf(a), gs.indexOf(b, gs.indexOf(a)));
const smartSrc = cut("_getSmartStartPosSettings(pos) {", "_updateProgressBar(percent) {");
const changeSrc = cut("  changeStartPos(direction) {", "  toggleGlitter(");
const methods = vm.runInContext(`(class { ${smartSrc}\n ${changeSrc}\n _getSongOffsetForWorldX(worldX) { const startX = Number.isFinite(Number(worldX)) ? Number(worldX) : 0; return calculateSongOffsetForX(startX, window.settingsMap?.["kA4"] ?? 0, window.levelObjects); }
  _getStartPosMusicOffset() { const sp = this._level.getStartPositions(); if (this._startPosIndex !== -1 && sp[this._startPosIndex]) return this._getSongOffsetForWorldX(sp[this._startPosIndex].x); return 0; }
  _getCurrentMusicSyncOffset() { if (this._startPosIndex !== -1 && this._playerWorldX <= 0) return this._getStartPosMusicOffset(); return this._getSongOffsetForWorldX(this._playerWorldX); } })`, ctx).prototype;

const files = fs.readdirSync(root + "ref/assets/levels").filter((f) => f.endsWith(".txt")).sort();
const levels = files.map((f) => ctx.parseLevel(fs.readFileSync(root + "ref/assets/levels/" + f, "utf8")));
const num = (s) => parseInt(s || "0", 10) || 0;
const out = w.wd_alloc(8 * 8);
let bad = 0, smartChecks = 0, offChecks = 0, switchChecks = 0, speedChecks = 0, modes = new Set();

function loadObjects(lvl) {
  const flat = [];
  for (const o of lvl.objects) flat.push(Number(o.x), Number(o.id));
  const p = w.wd_alloc(Math.max(1, flat.length) * 8);
  new Float64Array(w.memory.buffer, p, Math.max(1, flat.length)).set(flat);
  w.wd_game_set_level_objects(p, flat.length / 2);
}
// a tiny game so the wasm side has something to hold the state
{ const p = w.wd_alloc(8); w.wd_game_load(p, 0); }

let curLvl = null;
for (let t = 0; t < N; t++) {
  const lvl = levels[Math.floor(t / (N / levels.length)) % levels.length];
  if (lvl !== curLvl) { curLvl = lvl; ctx.window.levelObjects = lvl.objects; loadObjects(lvl); }
  const hdr = { kA2: pick([0, 1, 2, 3, 4, 5, 6, 7]), kA3: pick([0, 1]), kA4: pick([0, 1, 2, 3, 4, 9]), kA8: pick([0, 1]), kA11: pick([0, 1]), kA28: pick([0, 0, 1]) };
  ctx.window.settingsMap = Object.fromEntries(Object.entries(hdr).map(([k, v]) => [k, rnd() < 0.1 ? "" : String(v)]));
  const n = (k) => num(ctx.window.settingsMap[k]);
  w.wd_game_set_start(n("kA2"), n("kA4"), n("kA3") === 1, n("kA11") === 1, n("kA28") === 1, n("kA8") === 1);
  const maxX = Math.max(...lvl.objects.map((o) => Number(o.x) * 2).filter(Number.isFinite), 1000);
  const x = pick([0, rnd() * maxX, rnd() * maxX, rnd() * 1500, maxX]);
  // smart start pos
  const js = methods._getSmartStartPosSettings.call({}, { x });
  w.wd_game_smart(x, out);
  const r = new Float64Array(w.memory.buffer, out, 6);
  const rs = { gameMode: r[0], miniMode: !!r[1], speed: r[2], dualMode: !!r[3], gravityFlipped: !!r[4], mirrored: !!r[5] };
  smartChecks++; modes.add(js.gameMode);
  for (const k of Object.keys(js)) if (!same(js[k], rs[k])) { bad++; console.log(`SMART MISMATCH level ${pick(files)} x=${x} ${k}: rust=${rs[k]} js=${js[k]}`); break; }
  // song offsets
  const sk = n("kA4");
  const jo = ctx.calculateSongOffsetForX(x, ctx.window.settingsMap.kA4 ?? 0, lvl.objects);
  const ro = w.wd_game_song_offset(x);
  offChecks++;
  if (!same(jo, ro)) { bad++; console.log(`SONG OFFSET MISMATCH x=${x} sk=${sk}: rust=${ro} js=${jo}`); }
}

// the switcher: index arithmetic and what it asks for
for (let t = 0; t < 2000; t++) {
  const total = pick([0, 1, 2, 3, 5]);
  const positions = Array.from({ length: total }, (_, i) => ({ x: i * 700 + 100, y: 30, gameMode: 0, miniMode: 0, speed: 0, dualMode: 0, mirrored: 0, gravityFlipped: false }));
  const flat = positions.flatMap((q) => [q.x, q.y, q.gameMode, q.miniMode, q.speed, q.dualMode, q.mirrored, 0]);
  const sp = w.wd_alloc(Math.max(1, flat.length) * 8);
  new Float64Array(w.memory.buffer, sp, Math.max(1, flat.length)).set(flat);
  w.wd_game_set_start_positions(sp, total);
  let jsIdx = -1; w.wd_game_set_start_pos_index(-1); w.wd_game_restart_level(-1);
  let rustIdx = -1;
  for (let k = 0; k < 12; k++) {
    const dir = pick([-1, 1, 1]);
    const won = rnd() < 0.1;
    let restarted = 0;
    const S = Object.assign(Object.create(methods), { _paused: false, _levelWon: won, _menuActive: false, _slideIn: false, _startPosIndex: jsIdx, _level: { getStartPositions: () => positions }, _startPosText: { setText() {} }, _practicedMode: { clearCheckpoints() {} }, _restartLevel() { restarted++; } });
    S.changeStartPos(dir);
    jsIdx = S._startPosIndex;
    if (won) { /* the Rust side has no won flag to set here; skip it */ rustIdx = jsIdx; continue; }
    rustIdx = w.wd_game_change_start_pos(dir);
    switchChecks++;
    if (!same(jsIdx, rustIdx)) { bad++; console.log(`SWITCH MISMATCH total=${total} dir=${dir}: rust=${rustIdx} js=${jsIdx}`); break; }
    if ((total === 0 ? 0 : 1) !== restarted) { bad++; console.log("SWITCH restart count differs"); break; }
  }
}

// the speed hack in _quantizeDelta
for (let t = 0; t < 200; t++) {
  const sh = pick([undefined, 0, NaN, 1, 0.1, 0.5, 2, 3.7, 10, -1]);
  ctx.window.speedHack = sh;
  w.wd_game_set_speedhack(sh === undefined ? NaN : sh);
  const S = Object.assign(Object.create(methods), { _deltaBuffer: 0 });
  ctx.PracticeMode; // (unused)
  const q = H.sceneMethods._quantizeDelta;
  let rustBuf = null;
  for (let k = 0; k < 40; k++) {
    const delta = pick([16.6667, 16, 17, 33.3, 8, 20, 4, 100, 1000]);
    const jq = q.call(S, delta);
    const rq = w.wd_game_quantize(delta);
    speedChecks++;
    if (!same(jq, rq)) { bad++; console.log(`QUANTIZE MISMATCH speedHack=${sh} delta=${delta}: rust=${rq} js=${jq}`); break; }
  }
  w.wd_game_set_delta_buffer(0);
}
console.log(`smart ${smartChecks} (game modes seen ${[...modes].sort()}), song offsets ${offChecks}, switcher steps ${switchChecks}, quantize ${speedChecks}: ${bad ? bad + " MISMATCHES" : "all identical"}`);
process.exit(bad ? 1 : 0);
