// Browser front end for the Rust port: the game (simulation, triggers, sprite state) runs in wasm, this file only draws what it hands over.
import { AudioFx } from "./audio_fx.js";
import { createEndFx } from "./endfx.js";
const params = new URLSearchParams(location.search);
const LEVEL = params.get("level") || "1";
const AUTO = params.get("auto");          // "N": play N frames without a keyboard (headless screenshots), jumping every ~JUMP frames
const JUMP = Number(params.get("jump") || 0);
const WASM = params.get("wasm") || "wd_core.wasm";
const SHEETS = "../ref/assets/sheets/";
const canvas = document.getElementById("c");
const ctx = canvas.getContext("2d");
const hud = document.getElementById("hud");
const W = 1138, H = 640;

const fail = (e) => { document.body.insertAdjacentHTML("beforeend", `<pre style="position:fixed;left:8px;bottom:8px;color:#f66;font:12px monospace">${String(e && e.stack || e)}</pre>`); console.error(e); window.__error = String(e); };
window.addEventListener("error", (e) => fail(e.error || e.message));

const loadImage = (url) => new Promise((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = () => rej(new Error("image " + url)); i.src = url; });
const loadJson = async (url) => (await fetch(url)).json();
const loadText = async (url) => { const r = await fetch(url); if (!r.ok) throw new Error(url + " " + r.status); return r.text(); };

async function main() {
  // ---- wasm ----
  const { instance } = await WebAssembly.instantiateStreaming(fetch(WASM), { env: { js_sin: Math.sin, js_cos: Math.cos, js_tan: Math.tan, js_acos: Math.acos, js_atan: Math.atan, js_atan2: Math.atan2, js_pow: Math.pow, js_random: Math.random } });
  const w = instance.exports;
  const enc = new TextEncoder(), dec = new TextDecoder();
  const put = (s) => { const b = enc.encode(s); const p = w.wd_alloc(Math.max(1, b.length)); new Uint8Array(w.memory.buffer, p, b.length).set(b); return [p, b.length]; };
  const text = (n) => dec.decode(new Uint8Array(w.memory.buffer, w.wd_result_ptr(), n));

  const afx = new AudioFx();
  window.__WD = { w, put, text };

  // ---- atlases ----
  const sheets = ["GJ_GameSheet", "GJ_GameSheet02", "GJ_GameSheet03", "GJ_GameSheet04", "GJ_GameSheetGlow", "GJ_GameSheetIcons", "GJ_WebSheet", "GJ_EditorSheet01", "GJ_GameSheetEditor", "PixelSheet_01", "GJ_ParticleSheet", "GJ_LaunchSheet", "FireSheet_01-hd"];
  const atlas = {};
  await Promise.all(sheets.map(async (name) => {
    const [json, img] = await Promise.all([loadText(SHEETS + name + ".json"), loadImage(SHEETS + name + ".png")]);
    const [np, nl] = put(name), [jp, jl] = put(json);
    w.wd_atlas_add(np, nl, jp, jl);
    const j = JSON.parse(json);
    let frames = j.frames;
    if (!frames && j.textures) frames = j.textures.flatMap((t) => t.frames);
    if (Array.isArray(frames)) frames = Object.fromEntries(frames.map((f) => [f.filename, f]));
    atlas[name] = { frames, img };
  }));

  // ---- BeastAnim descriptions, mirror portals ----
  await Promise.all([1, 2, 3, 4, 5].map(async (k) => {
    const name = `GJBeast0${k}`;
    try { const [np, nl] = put(name), [jp, jl] = put(await loadText(SHEETS + name + "_AnimDesc.json")); w.wd_beast_desc_add(np, nl, jp, jl); } catch (e) {}
  }));
  // `localStorage` writes the game logic asked for (saved coins), as `[[key, value], ...]`
  const flushStorage = () => { try { for (const [k, v] of JSON.parse(text(w.wd_coins_outbox()))) localStorage.setItem(k, v); } catch (e) {} };

  // ---- level (can be switched from the level select) ----
  let frameNames = [], LEVEL_ID = LEVEL, info = {}, bgImg = null, groundImg = null, ground2Img = null, frames = [];
  const standalone = {}; window.__standalone = standalone;
  const loadLevel = async (id) => loadLevelText(id, await loadText(`../ref/assets/levels/${id}.txt`));
  const loadLevelText = async (id, levelText) => {
    LEVEL_ID = id;
    w.wd_set_official(/^\d{5,}$/.test(id) ? 0 : 1);
    w.wd_visual_seed((Math.random() * 4294967295) >>> 0);
    {
      const official = !/^\d{5,}$/.test(id), key = official ? "level_" + id : id;
      const ls = (k, d) => { try { return localStorage.getItem(k) ?? d; } catch (e) { return d; } };
      const ids = (k) => { try { const a = JSON.parse(ls(k, "[]")); return Array.isArray(a) ? a.map(String).join(",") : ""; } catch (e) { return ""; } };
      const [kp, kl] = put(key), [sp, sl] = put(ids("gd_secretCoins_" + key)), [up, ul] = put(ids("gd_userCoins_" + key));
      w.wd_coins_config(official ? 1 : 0, key.startsWith("local_") ? 1 : 0, 0, kp, kl, sp, sl, up, ul, parseInt(ls("gd_totalsecretcoins", "0"), 10) || 0, parseInt(ls("gd_totalusercoins", "0"), 10) || 0);
    }
    { const [p, l] = put(levelText); w.wd_game_load_level(p, l); }
    w.wd_mirror_enable(1);
    w.wd_game_after_load();
    w.wd_game_restart_level(-1);
    info = JSON.parse(text(w.wd_render_info()));
    bgImg = await loadImage(`../ref/assets/game-bg/game_bg_${info.background || "01"}_001-hd.png`).catch(() => null);
    const groundIdx = String((parseInt(info.ground || "00", 10) || 0) + 1).padStart(2, "0");
    groundImg = await loadImage(`../ref/assets/game-ground/groundSquare_${groundIdx}_001.png`).catch(() => null);
    ground2Img = await loadImage(`../ref/assets/game-ground/groundSquare_${groundIdx}_2_001.png`).catch(() => null);
    standalone["groundSquare_main"] = groundImg; standalone["groundSquare_second"] = ground2Img;
    w.wd_render_setup(groundImg ? groundImg.width : 1012, ground2Img ? 1 : 0, bgImg ? bgImg.height : 0);
    if (params.get("mode")) w.wd_game_set_start(Number(params.get("mode")), Number(params.get("speed") || 0), params.get("mini") ? 1 : 0, params.get("flip") ? 1 : 0, params.get("mirror") ? 1 : 0, params.get("dual") ? 1 : 0); // test hook
    w.wd_game_restart_level(-1);
    const frameTable = JSON.parse(text(w.wd_render_frames()));
    frameNames = frameTable;
    frames = frameTable.map(([a, f]) => { const at = atlas[a]; const d = at && at.frames[f]; return d ? { img: at.img, d } : null; });
  };
  const levels = await (async () => { try { const src = await loadText("../ref/assets/scripts/game/allLevels.js"); const win = {}; new Function("window", src)(win); return win.allLevels || []; } catch (e) { return []; } })();
  let loadingLevel = false;
  const loadSong = (id) => {
    const entry = levels.find((l) => l[2] === "level_" + id);
    window.__songData = null; songBuf = null;
    if (AUTO || !entry) return Promise.resolve();
    const file = entry[4] || String(entry[1]).replaceAll(" ", "");
    return fetch(`../ref/assets/music/${file}.mp3`).then((r) => r.arrayBuffer()).then(async (ab) => { window.__songData = ab; if (ac) songBuf = await ac.decodeAudioData(ab.slice(0)); }).catch(() => {});
  };
  let ac = null, analyser = null, gain = null, source = null, songBuf = null;
  await loadLevel(LEVEL);
  loadSong(LEVEL);

  // ---- UI (menus) ----
  w.wd_ui_new(W);
  for (const [name, file] of [["GJ_MenuBeta", "GJ_MenuBeta"], ["square04_001", "square04_001"], ["GJ_button01", "GJ_button_01"], ["sliderBar", "sliderBar"], ["sliderBar2", "sliderBar2"], ["slidergroove2", "slidergroove2"], ["GJ_square67", "GJ_square67"], ["GJ_button05", "GJ_button_05"], ["GJ_button02", "GJ_button_02"], ["square01_001", "square01_001"], ["GJ_square01", "GJ_square01"]].map(([n, f]) => [n, `../ref/assets/sprites/${f}.png`]).concat(window.WD_ASSETS || [])) { const img = await loadImage(file).catch(() => null); if (img) { standalone[name] = img; const [p, l] = put("/" + name); w.wd_ui_size(p, l, img.width, img.height); } }
  const fonts = {};
  for (const name of ["bigFont", "goldFont"]) {
    const [fnt, img] = await Promise.all([loadText(`../ref/assets/fonts/${name}.fnt`), loadImage(`../ref/assets/fonts/${name}.png`)]);
    const chars = {}, kern = {};
    let size = 0;
    for (const line of fnt.split("\n")) {
      const kv = Object.fromEntries([...line.matchAll(/(\w+)=("[^"]*"|\S+)/g)].map((m) => [m[1], m[2].replace(/"/g, "")]));
      if (line.startsWith("info ")) size = Number(kv.size);
      else if (line.startsWith("char ")) chars[kv.id] = { x: +kv.x, y: +kv.y, w: +kv.width, h: +kv.height, xo: +kv.xoffset, yo: +kv.yoffset, adv: +kv.xadvance };
      else if (line.startsWith("kerning ")) kern[kv.first + "," + kv.second] = +kv.amount;
    }
    fonts[name] = { img, chars, kern, size };
    const [np, nl] = put(name), [fp, fl] = put(fnt); w.wd_ui_font(np, nl, fp, fl);
  }
  const hidden = ["level_22", "level_67"];
  const visibleLevels = levels.filter((l) => !hidden.includes(l[2]));
  const pushLevels = (page) => { const [p, l] = put(JSON.stringify(visibleLevels.map((l) => [l[0], l[1], l[2], l[4] || null]))); w.wd_ui_levels(p, l, Math.max(0, page)); };
  pushLevels(visibleLevels.findIndex((l) => l[2] === "level_" + LEVEL));
  const refGround = await loadImage("../ref/assets/game-ground/groundSquare_01_001.png").catch(() => null);
  const pushProgress = () => {
    let total = 0;
    for (const l of visibleLevels) {
      const id = l[2];
      let slots = []; try { slots = JSON.parse(localStorage.getItem(`gd_secretCoins_${id}_slots`) || "[]") || []; } catch (e) {}
      const mask = slots.reduce((m, sl) => (sl >= 0 && sl < 3 ? m | (1 << sl) : m), 0);
      total += slots.length;
      const [p, n] = put(id);
      w.wd_ui_progress(p, n, parseFloat(localStorage.getItem("bestPercent_" + id) || "0"), parseFloat(localStorage.getItem("practiceBestPercent_" + id) || "0"), mask);
    }
    w.wd_ui_ground(groundImg ? groundImg.width : 1012, refGround ? refGround.height : 270, ground2Img ? 1 : 0, total);
  };
  const bump = (k) => { try { localStorage.setItem(k, String((parseInt(localStorage.getItem(k) || "0", 10) || 0) + 1)); } catch (e) {} };
  const bumpLevel = (kind) => bump("level" + kind + "_level_" + LEVEL_ID);
  pushProgress();
  const START_PLAYING = !!AUTO || params.get("play") === "1";
  if (!START_PLAYING) { w.wd_menu_mode(1); w.wd_ui_main_menu(); }
  else { w.wd_ui_start_game(); w.wd_ui_hud(); bump("gd_totalAttempts"); bumpLevel("Attempts"); w.wd_ui_attempt(1, w.wd_game_scene(0)); }
  let menuMode = !START_PLAYING;
  let paused = false, attempts = 1, practiceOn = false, won = false, winClock = 0, winStarted = false, playTime = 0, jumps = 0, wasHeld = false;
  const MESSAGES = ["Awesome!", "Good\nJob!", "Well\nDone!", "Impressive!", "Amazing!", "Incredible!", "Skillful!", "Brilliant!", "Not\nbad!", "Warp\nSpeed!", "Challenge\nBreaker!", "Reflex\nMaster!", "I am\nspeechless...", "You are...\nThe One!", "How is this\npossible!?", "You beat\nme..."];
  if (params.get("ui") === "select" && menuMode) { w.wd_ui_open_level_select(); const pg = Number(params.get("page") || -1); if (pg >= 0) for (let k = 0; k < pg; k++) w.wd_ui_switch(1); }
  if (params.get("ui") === "settings" && menuMode) { w.wd_ui_set_options(0.6, 1, 1); w.wd_ui_open_settings(); }

  // ---- drawing helpers ----
  let tintCache = new WeakMap(), tintPixels = 0;
  let bgTile = null;
  const tinted = (fr, tint) => {
    // keyed by image object, then by frame rectangle, then by tint: nothing is built per call (the callers make a fresh `fr` object every time)
    let byFrame = tintCache.get(fr.img); if (!byFrame) tintCache.set(fr.img, byFrame = new Map());
    const f0 = fr.d.frame, fk = ((f0.x * 8192 + f0.y) * 8192 + f0.w) * 8192 + f0.h;
    let per = byFrame.get(fk); if (!per) byFrame.set(fk, per = new Map());
    let c = per.get(tint);
    if (c) return c;
    const { x, y, w: fw, h: fh } = fr.d.frame;
    c = document.createElement("canvas"); c.width = fw; c.height = fh;
    const g = c.getContext("2d");
    g.drawImage(fr.img, x, y, fw, fh, 0, 0, fw, fh);
    g.globalCompositeOperation = "multiply";
    g.fillStyle = "#" + (tint & 0xffffff).toString(16).padStart(6, "0");
    g.fillRect(0, 0, fw, fh);
    g.globalCompositeOperation = "destination-in";
    g.drawImage(fr.img, x, y, fw, fh, 0, 0, fw, fh);
    tintPixels += fw * fh;
    if (tintPixels > 48e6) { // ~190 MB of coloured copies at most; start over
      tintCache = new WeakMap(); tintPixels = fw * fh;
      tintCache.set(fr.img, byFrame = new Map()); byFrame.set(fk, per = new Map());
    }
    per.set(tint, c);
    return c;
  };
  let snapOn = false;
  const snapPx = (v) => Math.round(v * pxScale) / pxScale;
  const hex = (n) => "#" + (n & 0xffffff).toString(16).padStart(6, "0");
  const drawFrame = (fr, x, y, rot, sx, sy, alpha, tint, blend, flips) => {
    const d = fr.d, f = d.frame;
    const fw = f.w, fh = f.h;
    const src = d.sourceSize || { w: fw, h: fh }, ss = d.spriteSourceSize || { x: 0, y: 0 };
    if (snapOn && !rot && !flips && sx > 0 && sy > 0 && !d.rotated) {
      // upright sprites: put all four edges on whole device pixels, so two sprites that share an edge share the same pixels (no hairline gaps)
      const ox = -src.w / 2 + ss.x, oy = -src.h / 2 + ss.y;
      const L = snapPx(x + ox * sx), R = snapPx(x + (ox + fw) * sx), T = snapPx(y + oy * sy), B = snapPx(y + (oy + fh) * sy);
      if (R > L && B > T) {
        ctx.globalAlpha = Math.max(0, Math.min(1, alpha));
        ctx.globalCompositeOperation = blend === 1 ? "lighter" : "source-over";
        if (tint >= 0 && tint !== 0xffffff) ctx.drawImage(tinted(fr, tint), 0, 0, fw, fh, L, T, R - L, B - T);
        else ctx.drawImage(fr.img, f.x, f.y, fw, fh, L, T, R - L, B - T);
        ctx.globalAlpha = 1; ctx.globalCompositeOperation = "source-over";
      }
      return;
    }
    ctx.save();
    ctx.translate(x, y);
    if (rot) ctx.rotate(rot);
    ctx.scale((flips & 1 ? -1 : 1) * sx, (flips & 2 ? -1 : 1) * sy);
    ctx.globalAlpha = Math.max(0, Math.min(1, alpha));
    ctx.globalCompositeOperation = blend === 1 ? "lighter" : "source-over";
    const ox = -src.w / 2 + ss.x, oy = -src.h / 2 + ss.y;
    if (tint >= 0 && tint !== 0xffffff) { const c = tinted(fr, tint); ctx.drawImage(c, ox, oy); }
    else if (d.rotated) { ctx.rotate(-Math.PI / 2); ctx.drawImage(fr.img, f.x, f.y, fh, fw, -oy - fh, ox, fh, fw); }
    else ctx.drawImage(fr.img, f.x, f.y, fw, fh, ox, oy, fw, fh);
    ctx.restore();
  };

  // ---- player visuals (web/player_vis.js) ----
  const pv = PlayerVis.create({ ctx, atlas, drawFrame, tinted, loadImage, loadText, loadJson, W, H, w, onIconAtlas: (base, frames) => { for (const [n, d] of Object.entries(frames)) { const [p, l] = put("icon_" + base + "/" + n); w.wd_ui_size(p, l, d.sourceSize.w, d.sourceSize.h); } } });
  window.__pv = pv;
  // the two extra icon sheets come with the garage and the mini-icon option
  for (const name of ["player_ball_00", "player_dart_00"]) { try { const [json, img] = await Promise.all([loadText(SHEETS + name + ".json"), loadImage(SHEETS + name + ".png")]); const [np, nl] = put(name), [jp, jl] = put(json); w.wd_atlas_add(np, nl, jp, jl); const j = JSON.parse(json); let fr = j.frames || (j.textures && j.textures.flatMap((t) => t.frames)); if (Array.isArray(fr)) fr = Object.fromEntries(fr.map((f) => [f.filename, f])); atlas[name] = { frames: fr, img }; } catch (e) {} }
  await pv.init();
  if (window.__garageInit) await window.__garageInit({ w, put });
  const endfx = createEndFx({ ctx, atlas, drawFrame, W });
  const iconColor = (k, d) => { try { return parseInt(localStorage.getItem(k) || d, 16); } catch (e) { return parseInt(d, 16); } };
  let pvRestart = true, pvHidden = false;
  const pvStep = (dt) => {
    const camX = w.wd_game_scene(0), camY = w.wd_game_scene(1);
    const n = w.wd_pv_events();
    pv.players[0].hidden = pv.players[1].hidden = pvHidden;
    pv.frame(dt, { ox: -camX, oy: camY, camX, camY, menu: menuMode || !!w.wd_ui_menu_active(), paused, won: won || !!window.__pvForceWon, restart: pvRestart, endX: w.wd_game_scene(5), endY: w.wd_game_scene(4), screenW: W, events: n ? text(n).split("|") : [] });
    pvRestart = false;
  };
  // text objects: a frame named `@text` / `<size>|<text>` is drawn with the bitmap font (origin at the centre)
  const textFrames = new Map();
  const textFrame = (idx) => {
    const fn = frameNames[idx];
    if (!fn || fn[0] !== "@text") return null;
    let fr = textFrames.get(idx);
    if (fr !== undefined) return fr;
    const fnt = fonts.bigFont, bar = fn[1].indexOf("|"), size = Number(fn[1].slice(0, bar)) || 36, str = fn[1].slice(bar + 1);
    fr = null;
    if (fnt) {
      const k = size / fnt.size, lines = str.split("\n"), lh = (fnt.lineHeight || fnt.size) * k;
      const widths = lines.map((ln) => { let cur = 0, prev = null; for (const ch of ln) { const g = fnt.chars[ch.codePointAt(0)]; if (!g) continue; cur += ((prev !== null ? fnt.kern[prev + "," + ch.codePointAt(0)] || 0 : 0) + g.adv) * k; prev = ch.codePointAt(0); } return cur; });
      const cw = Math.max(1, Math.ceil(Math.max(0, ...widths)) + 2), ch = Math.max(1, Math.ceil(lh * lines.length) + 2);
      const c = document.createElement("canvas"); c.width = cw; c.height = ch;
      const g2 = c.getContext("2d");
      lines.forEach((ln, li) => { let cur = (cw - widths[li]) / 2, prev = null; for (const chr of ln) { const cp = chr.codePointAt(0), g = fnt.chars[cp]; if (!g) continue; cur += (prev !== null ? fnt.kern[prev + "," + cp] || 0 : 0) * k; g2.drawImage(fnt.img, g.x, g.y, g.w, g.h, cur + g.xo * k, li * lh + g.yo * k, g.w * k, g.h * k); cur += g.adv * k; prev = cp; } });
      c.src = "text:" + idx;
      fr = { img: c, d: { frame: { x: 0, y: 0, w: cw, h: ch }, sourceSize: { w: cw, h: ch }, spriteSourceSize: { x: 0, y: 0 } } };
    }
    textFrames.set(idx, fr);
    return fr;
  };
  const fontOf = [null, fonts.bigFont, fonts.goldFont];
  const drawBitmapText = (fnt, str, size, x, y, scale, ox, oy, color) => {
    const k = size / fnt.size;
    // lines are laid out one under the other and centred on the widest one (`setCenterAlign`)
    const lines = String(str).split("\n").map((line) => {
      let cur = 0, prev = null;
      const glyphs = [];
      for (const ch of line) {
        const id = ch.codePointAt(0), g = fnt.chars[id];
        if (prev !== null) cur += (fnt.kern[prev + "," + id] || 0);
        if (g) { glyphs.push([g, cur]); cur += g.adv; }
        prev = id;
      }
      return { glyphs, width: cur * k };
    });
    const width = Math.max(...lines.map((l) => l.width));
    const lineH = 67 * k;
    ctx.save();
    ctx.translate(x, y); ctx.scale(scale, scale);
    ctx.translate(-width * ox, -lineH * lines.length * oy);
    lines.forEach((ln, li) => {
      const off = (width - ln.width) / 2, top = li * lineH;
      for (const [g, gx] of ln.glyphs) {
        if (!g.w) continue;
        const dx = off + (gx + g.xo) * k, dy = top + g.yo * k;
        if (color !== 0xffffff) { const c = tinted({ img: fnt.img, d: { frame: { x: g.x, y: g.y, w: g.w, h: g.h } } }, color); ctx.drawImage(c, dx, dy, g.w * k, g.h * k); }
        else ctx.drawImage(fnt.img, g.x, g.y, g.w, g.h, dx, dy, g.w * k, g.h * k);
      }
    });
    ctx.restore();
  };
  let camXUi = 0, camYUi = 0, shakeUi = 0;
  let uiTable = [], uiTableLen = -1;
  const drawUi = () => {
    const n = w.wd_ui_build(0);
    if (!n) return;
    const ub = new Float64Array(w.memory.buffer, w.wd_ui_ptr(), n * 16);
    const texts = JSON.parse(text(w.wd_ui_texts()));
    const fl = w.wd_ui_frames_len(); if (fl !== uiTableLen) { uiTable = JSON.parse(text(w.wd_ui_frames())); uiTableLen = fl; }
    const table = uiTable;
    for (let k = 0; k < n; k++) {
      const o = k * 16, kind = ub[o];
      ctx.save();
      if (kind === 6) { ctx.beginPath(); ctx.rect(ub[o + 1], ub[o + 2], ub[o + 12], ub[o + 13]); ctx.clip(); continue; }
      if (kind === 7) { ctx.restore(); ctx.restore(); continue; }
      ctx.globalAlpha = Math.max(0, Math.min(1, ub[o + 6]));
      if (kind !== 0 && (ub[o + 8] & 4)) ctx.globalCompositeOperation = "lighter";
      if (kind === 0) {
        const [a, f] = table[ub[o + 13]];
        let img, d;
        if (a === "") { img = standalone[f]; d = img && { frame: { x: 0, y: 0, w: img.width, h: img.height }, sourceSize: { w: img.width, h: img.height }, spriteSourceSize: { x: 0, y: 0 } }; }
        else { const at = atlas[a]; img = at && at.img; d = at && at.frames[f]; }
        if (img && d) {
          const fl = ub[o + 8], sw2 = d.sourceSize.w, sh2 = d.sourceSize.h;
          ctx.translate(ub[o + 1] + ((fl & 8) ? -camXUi : 0), ub[o + 2] + ((fl & 8) ? camYUi : 0)); if (ub[o + 5]) ctx.rotate(ub[o + 5]);
          ctx.scale(ub[o + 3], ub[o + 4]);
          if (fl & 4) ctx.globalCompositeOperation = "lighter";
          const fr = { img, d };
          // flips mirror the frame in place (the origin is applied first)
          if (fl & 1) { ctx.translate(sw2 * (1 - 2 * ub[o + 9]), 0); ctx.scale(-1, 1); }
          if (fl & 2) { ctx.translate(0, sh2 * (1 - 2 * ub[o + 10])); ctx.scale(1, -1); }
          const dx = -sw2 * ub[o + 9] + d.spriteSourceSize.x, dy = -sh2 * ub[o + 10] + d.spriteSourceSize.y;
          const tint = ub[o + 7], crop = ub[o + 12];
          const cw = crop > 0 ? Math.min(crop, d.frame.w) : d.frame.w;
          if (crop > d.frame.w && a === "") { for (let tx2 = 0; tx2 < crop; tx2 += d.frame.w) { const pw = Math.min(d.frame.w, crop - tx2); ctx.drawImage(img, 0, 0, pw, d.frame.h, dx + tx2, dy, pw, d.frame.h); } }
          else if (tint >= 0 && tint !== 0xffffff) { const tc = tinted(fr, tint); ctx.drawImage(tc, 0, 0, cw, d.frame.h, dx, dy, cw, d.frame.h); }
          else ctx.drawImage(img, d.frame.x, d.frame.y, cw, d.frame.h, dx, dy, cw, d.frame.h);
        }
      } else if (kind === 1) {
        const t = texts[ub[o + 11]]; const size = ub[o + 12];
        const fnt = fontOf[ub[o + 15]];
        if (fnt) drawBitmapText(fnt, t, size, ub[o + 1] + ((ub[o + 8] & 8) ? -camXUi : 0), ub[o + 2] + ((ub[o + 8] & 8) ? camYUi : 0), ub[o + 3], ub[o + 9], ub[o + 10], ub[o + 14]);
        else {
          ctx.translate(ub[o + 1], ub[o + 2]); ctx.scale(ub[o + 3], ub[o + 4]);
          ctx.font = `${size}px Arial`; ctx.fillStyle = hex(ub[o + 14]);
          ctx.textAlign = ub[o + 9] === 1 ? "right" : ub[o + 9] === 0.5 ? "center" : "left"; ctx.textBaseline = ub[o + 10] === 1 ? "bottom" : ub[o + 10] === 0.5 ? "middle" : "top";
          ctx.fillText(t, 0, 0);
        }
      } else if (kind === 2) { ctx.fillStyle = hex(ub[o + 14]); ctx.fillRect(ub[o + 1], ub[o + 2], ub[o + 12], ub[o + 13]); }
      else if (kind === 3) { ctx.fillStyle = hex(ub[o + 14]); ctx.beginPath(); ctx.roundRect(ub[o + 1], ub[o + 2], ub[o + 12], ub[o + 13], ub[o + 15]); ctx.fill(); }
      else if (kind === 5) {
        const img = standalone[texts[ub[o + 11]]];
        if (img) {
          const x0 = ub[o + 1], y0 = ub[o + 2], W2 = ub[o + 12], H2 = ub[o + 13], b = ub[o + 15], tint = ub[o + 14];
          const src = tint >= 0 && tint !== 0xffffff ? tinted({ img, d: { frame: { x: 0, y: 0, w: img.width, h: img.height } } }, tint) : img;
          const iw = img.width, ih = img.height;
          const xs = [0, b, iw - b, iw], ys = [0, b, ih - b, ih];
          const xd = [x0, x0 + b, x0 + W2 - b, x0 + W2].map(snapPx), yd = [y0, y0 + b, y0 + H2 - b, y0 + H2].map(snapPx);
          for (let i = 0; i < 3; i++) for (let j = 0; j < 3; j++) ctx.drawImage(src, xs[i], ys[j], xs[i + 1] - xs[i], ys[j + 1] - ys[j], xd[i], yd[j], xd[i + 1] - xd[i], yd[j + 1] - yd[j]);
        }
      }
      else if (kind === 4) { ctx.fillStyle = hex(ub[o + 14]); ctx.beginPath(); ctx.arc(ub[o + 1], ub[o + 2], ub[o + 12], 0, Math.PI * 2); ctx.fill(); }
      ctx.restore();
    }
  };

  // ---- audio: the level's song, and the analyser the music meter is fed from ----
  const MUSIC_VOL = 0.8; // `_effectiveVolume()` with the default user volume
  const peakBuf = new Float32Array(2048);
  const ensureAudio = async () => {
    if (ac || AUTO) return;
    ac = new AudioContext();
    gain = ac.createGain(); gain.gain.value = afx.userMusic() * 0.8;
    analyser = ac.createAnalyser(); analyser.fftSize = 2048;
    gain.connect(analyser); analyser.connect(ac.destination);
    w.wd_audio_live(1);
    afx.attach(ac, gain);
    if (menuMode) afx.enterMenu();
    if (window.__songData && !songBuf) songBuf = await ac.decodeAudioData(window.__songData.slice(0));
  };
  const startMusic = (offset) => {
    if (!ac || !songBuf || menuMode) return;
    try { source && source.stop(); } catch (e) {}
    source = ac.createBufferSource(); source.buffer = songBuf; source.connect(gain);
    source.start(0, Math.max(0, Math.min(songBuf.duration - 0.1, offset)));
  };
  const stopSong = () => { try { source && source.stop(); } catch (e) {} source = null; };
  const pollSong = async () => { if (ac && !songBuf && window.__songData && !menuMode && !sliding && !paused && !wasDead) { songBuf = await ac.decodeAudioData(window.__songData.slice(0)); startMusic(info.musicOffset || 0); } };

  // ---- input ----
  let held = false;
  const down = (e) => { if (e.type === "keydown" && !["Space", "ArrowUp", "KeyW"].includes(e.code)) return; if (!menuMode && !paused && !sliding && !held) { bump("gd_totalJumps"); bumpLevel("Jumps"); } held = true; e.preventDefault(); if (!ac) ensureAudio().then(() => startMusic(info.musicOffset || 0)); };
  const up = (e) => { if (e.type === "keyup" && !["Space", "ArrowUp", "KeyW"].includes(e.code)) return; held = false; };
  addEventListener("keydown", down); addEventListener("keyup", up);
  const toGame = (e) => { const r = canvas.getBoundingClientRect(); const sc = Math.min(r.width / W, r.height / H); const ox = r.left + (r.width - W * sc) / 2, oy = r.top + (r.height - H * sc) / 2; return [(e.clientX - ox) / sc, (e.clientY - oy) / sc]; };
  canvas.addEventListener("wheel", (e) => { const [x, y] = toGame(e); w.wd_ui_wheel(x, y, e.deltaY); e.preventDefault(); }, { passive: false });
  canvas.addEventListener("pointerdown", (e) => { if (loadingLevel) return; const [x, y] = toGame(e); w.wd_ui_pointer(0, x, y); if (!w.wd_ui_menu_active()) down(e); });
  addEventListener("pointermove", (e) => { const [x, y] = toGame(e); w.wd_ui_pointer(1, x, y); });
  addEventListener("pointerup", (e) => { if (loadingLevel) return; const [x, y] = toGame(e); w.wd_ui_pointer(2, x, y); up(e); });
  let sliding = false, wasDead = false;
  const startPlaying = () => { pvRestart = true; menuMode = false; paused = false; won = false; playTime = 0; jumps = 0; w.wd_ui_close_end(); attempts = 1; practiceOn = false; afx.stopMusic(); w.wd_ui_start_game(); applyOptions(); w.wd_game_begin_slide(); sliding = true; w.wd_ui_hud(); w.wd_ui_hud_pause(0); bump("gd_totalAttempts"); bumpLevel("Attempts"); w.wd_ui_attempt(1, w.wd_game_scene(0)); if (!ac) ensureAudio(); };
  // the slide-in ends: the song starts and the pause button fades in
  const pollSlide = () => { if (!sliding) return; const ss = w.wd_game_slide_state(); if (ss === 2) { sliding = false; w.wd_ui_hud_pause(1); if (ac) startMusic(info.musicOffset || 0); else ensureAudio().then(() => startMusic(info.musicOffset || 0)); } };
  const saveBest = () => {
    const pct = Math.floor(w.wd_game_percent(0));
    const key = (practiceOn ? "practiceBestPercent_" : "bestPercent_") + "level_" + LEVEL_ID;
    try { if (pct > parseFloat(localStorage.getItem(key) || "0")) localStorage.setItem(key, String(pct)); } catch (e) {}
  };
  // coins on the level-complete screen: how many the level has, which were picked up this run, which were saved earlier
  const pushEndCoins = (o) => {
    const main = o ? o.main : levels.some((l) => l[2] === "level_" + LEVEL_ID);
    const count = o ? o.count : w.wd_game_coin_count(main ? 1 : 0);
    let saved = 0;
    try { const a = JSON.parse(localStorage.getItem(`gd_${main ? "secret" : "user"}Coins_level_${LEVEL_ID}_slots`) || "[]"); for (const sl of a) if (sl >= 0 && sl < 3) saved |= 1 << sl; } catch (e) {}
    w.wd_ui_end_coins(main ? 1 : 0, count, o ? o.run : 0, o ? o.saved : saved);
  };
  const restarted = () => { pvRestart = true; saveBest(); bump("gd_totalAttempts"); bumpLevel("Attempts"); attempts++; w.wd_ui_attempt(attempts, w.wd_game_scene(0)); startMusic(info.musicOffset || 0); };
  // ---- level editor host glue ----
  const edDown = new Set();
  addEventListener("keydown", (e) => { if (window.WD_EDITOR_ON && e.key.startsWith("Arrow")) { edDown.add(e.key); e.preventDefault(); } });
  addEventListener("keyup", (e) => edDown.delete(e.key));
  const edKeys = () => (edDown.has("ArrowLeft") ? 1 : 0) | (edDown.has("ArrowRight") ? 2 : 0) | (edDown.has("ArrowUp") ? 4 : 0) | (edDown.has("ArrowDown") ? 8 : 0);
  window.__startEditor = async (levelString, createdId) => {
    await loadLevelText("editor", levelString);
    w.wd_ui_start_game(); w.wd_menu_mode(0); menuMode = false; won = false;
    const [p, n] = put(levelString), [ip, il] = put(createdId || "");
    w.wd_ed_open(p, n, ip, il);
    window.WD_EDITOR_ON = true;
  };
  window.__stopEditor = () => { window.WD_EDITOR_ON = false; w.wd_ed_close(); };
  const doPause = () => {
    if (paused || menuMode) return;
    paused = true; if (ac) ac.suspend();
    const entry = levels.find((l) => l[2] === "level_" + LEVEL_ID);
    const [p, n] = put(entry ? entry[1] : "Level");
    w.wd_ui_pause(1, p, n, parseFloat(localStorage.getItem("bestPercent_level_" + LEVEL_ID) || "0"), parseFloat(localStorage.getItem("practiceBestPercent_level_" + LEVEL_ID) || "0"));
  };
  const doResume = () => { if (!paused) return; paused = false; w.wd_ui_pause(0, 0, 0, 0, 0); if (ac) ac.resume(); };
  const OPT_DEFAULTS = { noclip: 0, showPercentage: 1, percentDecimals: 0, showHitboxes: 0, progressBar: 0, hitboxTrail: 0, showFPS: 0, hidePauseBtn: 0, hidePracticeUI: 0, solidWaveTrail: 0, noclipAccuracy: 0, hitboxesOnDeath: 0, createObjectIds: 0, showObjectIds: 0, showCPS: 0, speedHack: 1, respawnDelay: 1, macroBot: 0, practiceMusicSync: 0, showGlow: 1, enablePortalGuide: 1, enableOrbGuide: 0, enableMiniIcon: 0, cullDistance: 3 };
  let settingsRaw = null, settingsVal = {};
  const readSettings = () => { // parsed again only when the stored text changed
    let raw = null; try { raw = localStorage.getItem("gd_settings"); } catch (e) {}
    if (raw !== settingsRaw || raw === null) { settingsRaw = raw; try { settingsVal = JSON.parse(raw || "{}") || {}; } catch (e) { settingsVal = {}; } }
    return settingsVal;
  };
  // gd_settings -> gameplay (noclip, speed hack, respawn delay, smart start pos) and HUD (percentage, bar, pause button, practice UI)
  const applyOptions = () => {
    const saved = { ...OPT_DEFAULTS, ...readSettings() };
    for (const k of Object.keys(OPT_DEFAULTS)) { const [p, n] = put(k); w.wd_ui_opt_set(p, n, Number(saved[k] === true ? 1 : saved[k] === false ? 0 : saved[k])); }
    const num = (v, d) => (Number.isFinite(Number(v)) && v !== null && v !== "" ? Number(v) : d);
    w.wd_game_set_options(saved.noclip ? 1 : 0, saved.noclipAccuracy ? 1 : 0, num(saved.speedHack, 1), num(saved.respawnDelay, 1), saved.smartStartPos ? 1 : 0);
    w.wd_ui_apply_hud();
    return saved;
  };
  if (START_PLAYING) applyOptions();
  const openOptions = () => {
    const saved = { ...OPT_DEFAULTS, ...readSettings() };
    for (const k of Object.keys(OPT_DEFAULTS)) { const [p, n] = put(k); w.wd_ui_opt_set(p, n, Number(saved[k] === true ? 1 : saved[k] === false ? 0 : saved[k])); }
    w.wd_ui_open_options();
  };
  const saveOptions = () => {
    const o = JSON.parse(text(w.wd_ui_opts_json())), cur = readSettings();
    for (const k of Object.keys(o)) cur[k] = typeof OPT_DEFAULTS[k] === "number" && ["speedHack", "respawnDelay", "cullDistance"].includes(k) ? o[k] : !!o[k];
    try { localStorage.setItem("gd_settings", JSON.stringify(cur)); } catch (e) {}
    applyOptions();
  };
  addEventListener("pointerdown", () => { if (!ac && !AUTO) ensureAudio(); });
  addEventListener("keydown", (e) => { const c = e.key === "Backspace" ? 8 : e.key === "Enter" ? 13 : e.key.length === 1 ? e.key.charCodeAt(0) : 0; if (c) w.wd_ui_key(c); });
  const handleUi = () => {
    for (;;) {
      const n = w.wd_ui_event(); if (!n) break;
      const ev = text(n);
      if (ev === "OpenLevelSelect") { pushProgress(); w.wd_ui_open_level_select(); }
      else if (ev === "CloseLevelSelect") w.wd_ui_close_level_select();
      else if (ev === "PrevLevel") w.wd_ui_switch(-1);
      else if (ev === "NextLevel") w.wd_ui_switch(1);
      else if (ev === "PlayLevel") {
        const l = visibleLevels[w.wd_ui_page()];
        if (l && !loadingLevel) { // game-scene.js: "Downloading Level Assets...", input off until the level and song are loaded, then sound, close the select, restart into the level
          const id = l[2].replace("level_", ""); loadingLevel = true;
          Promise.all([loadText(`../ref/assets/levels/${id}.txt`), loadSong(id)]).then(async ([txt]) => { await ensureAudio(); if (ac && !songBuf && window.__songData) songBuf = await ac.decodeAudioData(window.__songData.slice(0)); afx.playEffect("playSound_01", { volume: 1 }); w.wd_ui_close_level_select(); afx.stopMusic(); await loadLevelText(id, txt); w.wd_menu_mode(0); loadingLevel = false; startPlaying(); }).catch((e) => { loadingLevel = false; fail(e); });
        }
      }
      else if (ev === "Settings") { const g = (k, d) => { try { const v = localStorage.getItem(k); return v === null ? d : v; } catch (e) { return d; } }; w.wd_ui_set_options(parseFloat(g("userMusicVol", 1)), parseFloat(g("userSfxVol", 1)), g("menuMusicEnabled", "true") === "true" ? 1 : 0); w.wd_ui_open_settings(); }
      else if (ev === "Volume") { const m = w.wd_ui_option(0), sv = w.wd_ui_option(1); if (gain) gain.gain.value = m * 0.8; try { localStorage.setItem("userMusicVol", m); localStorage.setItem("userSfxVol", sv); } catch (e) {} }
      else if (ev === "ToggleMenuMusic") { try { localStorage.setItem("menuMusicEnabled", w.wd_ui_option(2) ? "true" : "false"); } catch (e) {} if (w.wd_ui_option(2)) { if (!afx.isPlaying()) afx.startMenuMusic(); } else afx.stopMusic(); }
      else if (window.WD_EVENTS[ev]) window.WD_EVENTS[ev](ev, { w, put, text, params });
      else if (ev === 'Popup("Options")') openOptions();
      else if (ev === "OptChanged") saveOptions();
      else if (ev.startsWith("Popup(")) { /* other settings popups: not ported yet */ }
      else if (ev === "LevelStats") {
        const l = visibleLevels[w.wd_ui_page()]; if (l) {
          const key = l[2], g = (k) => { try { return localStorage.getItem(k); } catch (e) { return null; } };
          const [p, n] = put(l[1] || "Unnamed");
          w.wd_pb_level_stats(p, n, parseInt(g("levelAttempts_" + key) || "0", 10) || 0, parseInt(g("levelJumps_" + key) || "0", 10) || 0, Math.floor(parseFloat(g("bestPercent_" + key) || "0")), Math.floor(parseFloat(g("practiceBestPercent_" + key) || "0")));
        }
      }
      else if (ev.startsWith("Sfx(")) afx.playEffect(ev.slice(5, -2));
      else if (ev === "Garage") { /* handled by garage.js */ }
      else if (ev === "Pause") doPause();
      else if (ev === "PauseResume") doResume();
      else if (ev === "PauseReplay") { doResume(); w.wd_ui_close_end(); won = false; w.wd_game_restart_current(); restarted(); }
      else if (ev === "PauseMenu") { afx.playEffect("quitSound_01"); sliding = false; doResume(); won = false; w.wd_ui_close_end(); if (source) { try { source.stop(); } catch (e) {} } menuMode = true; w.wd_menu_mode(1); w.wd_ui_main_menu_again(); afx.enterMenu(); }
      else if (ev === "PausePractice") { practiceOn = !practiceOn; w.wd_game_practice(0); w.wd_ui_practice(practiceOn ? 1 : 0); doResume(); doPause(); }
      else if (ev === "Checkpoint") { if (practiceOn) w.wd_game_practice(1); }
      else if (ev === "ClearCheckpoint") { if (practiceOn) w.wd_game_practice(2); }
      else if (ev.startsWith("url:")) window.open(ev.slice(4), "_blank");
    }
  };
  // gd_keybinds: start-position switcher, noclip, hitboxes (the default keys of the original)
  const kbDefaults = { startPosPrev: ["KeyQ"], startPosNext: ["KeyE"], noclip: ["KeyN"], showHitboxes: ["KeyH"] };
  const kbHas = (id, code) => { let m = {}; try { m = JSON.parse(localStorage.getItem("gd_keybinds") || "{}") || {}; } catch (e) {} return (Array.isArray(m[id]) ? m[id] : kbDefaults[id]).includes(code); };
  const toggleSetting = (k) => { const cur = readSettings(); cur[k] = !cur[k]; try { localStorage.setItem("gd_settings", JSON.stringify(cur)); } catch (e) {} applyOptions(); };
  addEventListener("keydown", (e) => {
    if (menuMode || paused || e.repeat) return;
    const st = { ...OPT_DEFAULTS, ...readSettings() };
    if (st.startPosSwitcher && (kbHas("startPosPrev", e.code) || kbHas("startPosNext", e.code))) { w.wd_game_change_start_pos(kbHas("startPosNext", e.code) ? 1 : -1); restarted(); }
    if (kbHas("noclip", e.code)) toggleSetting("noclip");
    if (kbHas("showHitboxes", e.code)) toggleSetting("showHitboxes");
  });
  addEventListener("keydown", (e) => { if (e.code === "KeyR" && !menuMode && !paused) { w.wd_game_restart_current(); restarted(); } if (e.code === "Escape" && w.wd_gar_is_open()) { w.wd_gar_close(); return; } if (e.code === "Escape") { if (paused) doResume(); else doPause(); } });

  // ---- frame ----
  let last = performance.now(), frame = 0, fps = 0, acc = 0;
  // Phaser runs with `resolution: 1` and stretches a 1138x640 canvas with CSS, which blurs the HD art; draw at the size it is shown instead.
  let pxScale = 1;
  const fitCanvas = () => {
    const s = Number(params.get("pxs")) || Math.min(2, Math.max(0.5, Math.min((innerWidth || W) / W, (innerHeight || H) / H) * (devicePixelRatio || 1)));
    const nw = Math.max(1, Math.round(W * s)), nh = Math.max(1, Math.round(H * s));
    if (canvas.width !== nw || canvas.height !== nh) { canvas.width = nw; canvas.height = nh; }
    const [shx, shy] = endfx.shake();
    pxScale = nw / W; ctx.setTransform(pxScale, 0, 0, pxScale, shx * pxScale, shy * pxScale); ctx.imageSmoothingEnabled = true; ctx.imageSmoothingQuality = "high";
  };
  const render = () => {
    fitCanvas();
    const n = w.wd_render_build();
    const buf = new Float64Array(w.memory.buffer, w.wd_render_ptr(), n);
    const camX = buf[0], camY = buf[1], count = buf[2] | 0, tiles = buf[18] | 0;
    const shakeX = buf[31] || 0, shakeY = buf[32] || 0;
    pvHidden = !!buf[27];
    // background
    ctx.globalAlpha = 1; ctx.globalCompositeOperation = "source-over";
    if (bgImg) {
      if (!bgTile || bgTile.src !== bgImg) { // the image above its own mirror (`_createMirroredBackgroundTexture`), made once per level
        const c = document.createElement("canvas"); c.width = bgImg.width; c.height = bgImg.height * 2;
        const g = c.getContext("2d"); g.drawImage(bgImg, 0, 0); g.translate(0, c.height); g.scale(1, -1); g.drawImage(bgImg, 0, 0);
        bgTile = c; bgTile.src = bgImg;
      }
      const tw = bgTile.width, th = bgTile.height;
      const tx = ((buf[20] % tw) + tw) % tw, ty = ((buf[21] % th) + th) % th; // a TileSprite shows texture pixel (x + tilePositionX, y + tilePositionY) at its (x, y)
      const e1 = 1 / pxScale;
      for (let xx = -tx; xx < W; xx += tw) for (let yy = -ty; yy < H; yy += th) ctx.drawImage(bgTile, xx, yy, tw + e1, th + e1);
      const tint = buf[22] & 0xffffff;
      if (tint !== 0xffffff) { ctx.globalCompositeOperation = "multiply"; ctx.fillStyle = hex(tint); ctx.fillRect(0, 0, W, H); ctx.globalCompositeOperation = "source-over"; }
    } else { ctx.fillStyle = hex(buf[22]); ctx.fillRect(0, 0, W, H); }
    endfx.drawBack();
    // level objects (normal layer first, then the player, then the top layer)
    const base = 40 + tiles;
    snapOn = !(buf[33] > 0);
    if (buf[33] > 0) { // level editor: static level seen through the editor camera
      const z = buf[33]; ctx.save(); ctx.scale(z, z);
      const ex = -buf[34] / z, ey = -buf[35] / z;
      for (let k = 0; k < count; k++) {
        const o = base + k * 12; const fr = frames[buf[o]] || textFrame(buf[o]); if (!fr) continue;
        const sx = (buf[o + 1] + ex) * z, sy = (buf[o + 2] + ey) * z; if (sx < -400 || sx > W + 400 || sy < -400 || sy > H + 400) continue;
        drawFrame(fr, buf[o + 1] + ex, buf[o + 2] + ey, buf[o + 3], buf[o + 4], buf[o + 5], buf[o + 6], buf[o + 7], buf[o + 8], buf[o + 9]);
      }
      ctx.restore();
      // grid (`_updateEditorGrid`)
      const gs = 60, camX = buf[34] / z, camY2 = buf[35] / z;
      ctx.strokeStyle = "rgba(0,0,0,0.4)"; ctx.lineWidth = 1; ctx.beginPath();
      for (let x = -camX % gs; x < W / z + gs; x += gs) { ctx.moveTo(x * z + 0.5, 0); ctx.lineTo(x * z + 0.5, H); }
      for (let y = (-camY2 - 20) % gs; y < H / z + gs; y += gs) { ctx.moveTo(0, y * z + 0.5); ctx.lineTo(W, y * z + 0.5); }
      ctx.stroke();
      ctx.strokeStyle = "#fff"; ctx.beginPath();
      const sl = -buf[34]; if (sl >= -50 && sl <= W + 50) { ctx.moveTo(sl + 0.5, 0); ctx.lineTo(sl + 0.5, H); }
      const gl = (-20 + 60 * 8) * z - buf[35]; if (gl >= -50 && gl <= H + 50) { ctx.moveTo(0, gl + 0.5); ctx.lineTo(W, gl + 0.5); }
      ctx.stroke();
      return buf;
    }
    const ox = -camX + shakeX, oy = camY + shakeY;
    let i = 0;
    for (; i < count; i++) {
      const o = base + i * 12;
      if (buf[o + 10] === 1) break;
      const fr = frames[buf[o]]; if (!fr) continue;
      drawFrame(fr, buf[o + 1] + ox, buf[o + 2] + oy, buf[o + 3], buf[o + 4], buf[o + 5], buf[o + 6], buf[o + 7], buf[o + 8], buf[o + 9]);
    }
    endfx.drawWall(ox, oy, w.wd_game_scene(5), w.wd_game_scene(4), iconColor("iconMainColor", "04FF00"), !menuMode && !w.wd_ui_menu_active() && !window.WD_EDITOR_ON);
    pv.drawWorld(ox, oy);
    endfx.drawPortalParticles(ox, oy);
    for (; i < count; i++) {
      const o = base + i * 12;
      const fr = frames[buf[o]] || textFrame(buf[o]); if (!fr) continue;
      drawFrame(fr, buf[o + 1] + ox, buf[o + 2] + oy, buf[o + 3], buf[o + 4], buf[o + 5], buf[o + 6], buf[o + 7], buf[o + 8], buf[o + 9]);
    }
    pv.drawTop(ox, oy);
    // ground
    ctx.globalAlpha = 1; ctx.globalCompositeOperation = "source-over";
    if (groundImg) {
      const gy = buf[15], tint = buf[23];
      const tile = tinted({ img: groundImg, d: { frame: { x: 0, y: 0, w: groundImg.width, h: groundImg.height } } }, tint);
      const tile2 = ground2Img && buf[25] ? tinted({ img: ground2Img, d: { frame: { x: 0, y: 0, w: ground2Img.width, h: ground2Img.height } } }, buf[24]) : null;
      for (let t = 0; t < tiles; t++) {
        const tx = buf[40 + t];
        const gw = tile.width + 1 / pxScale; // a device pixel of overlap hides the seam between tiles
        ctx.drawImage(tile, 0, 0, tile.width, tile.height, tx, gy, gw, tile.height);
        if (tile2) ctx.drawImage(tile2, 0, 0, tile2.width, tile2.height, tx, gy, gw, tile2.height);
        if (buf[17]) { ctx.save(); ctx.translate(tx, buf[16]); ctx.scale(1, -1); ctx.drawImage(tile, 0, 0); if (tile2) ctx.drawImage(tile2, 0, 0); ctx.restore(); }
      }
    }
    // floor line and the dark edge shadows (`_buildGround`)
    const web = atlas["GJ_WebSheet"];
    const line = web.frames["floorLine_01_001.png"], shadow = web.frames["groundSquareShadow_001.png"];
    const lineAt = (y, flipY) => {
      if (!line) return;
      const sc = (W + 400) / line.sourceSize.w;
      ctx.save(); ctx.translate(buf[19], y); ctx.scale(sc, flipY ? -1 : 1); ctx.globalCompositeOperation = "lighter";
      ctx.drawImage(web.img, line.frame.x, line.frame.y, line.frame.w, line.frame.h, -line.sourceSize.w / 2 + line.spriteSourceSize.x, line.spriteSourceSize.y, line.frame.w, line.frame.h);
      ctx.restore();
    };
    const shadowAt = (y, flipY) => {
      if (!shadow) return;
      const sw = shadow.sourceSize.w * 0.7;
      for (const right of [false, true]) {
        ctx.save(); ctx.globalAlpha = 100 / 255; ctx.translate(right ? W + 1 : -1, y); ctx.scale(right ? -0.7 : 0.7, flipY ? -1 : 1);
        ctx.drawImage(web.img, shadow.frame.x, shadow.frame.y, shadow.frame.w, shadow.frame.h, right ? -shadow.sourceSize.w + (shadow.sourceSize.w - shadow.spriteSourceSize.x - shadow.frame.w) : shadow.spriteSourceSize.x, shadow.spriteSourceSize.y, shadow.frame.w, shadow.frame.h);
        ctx.restore();
      }
    };
    lineAt(buf[15], false); shadowAt(buf[15], false);
    if (buf[17]) { lineAt(buf[16], true); shadowAt(buf[16], true); }
    pv.drawScreen();
    return buf;
  };

  // no audio yet: a steady beat stands in for the music's level so the audio-reactive sprites move
  const beat = (ms) => 0.45 + 0.35 * Math.abs(Math.sin(ms / 1000 * Math.PI * 2));
  const step = (dtMs) => {
    w.wd_trig_time(performance.now());
    if (analyser) { analyser.getFloatTimeDomainData(peakBuf); let pk = 0; for (let k = 0; k < peakBuf.length; k++) { const v = Math.abs(peakBuf[k]); if (v > pk) pk = v; } w.wd_set_audio_peak(pk, MUSIC_VOL); }
    else w.wd_set_audio_meter(beat(performance.now()));
    const r = w.wd_game_update(dtMs, held ? 1 : 0);
    if ((r >> 30) & 1) restarted(); // died and restarted
    pollSong();
    pollSlide();
  };

  if (AUTO) {
    // headless: fixed 60 fps, scripted jumps, draw the last frame
    const frames = Number(AUTO);
    for (let f = 0; f < frames; f++) {
      held = JUMP > 0 && (f % JUMP) < 8;
      w.wd_trig_time(f * 16.6667);
      w.wd_set_audio_meter(beat(f * 16.6667));
      w.wd_game_update(1000 / 60, held ? 1 : 0);
      if (params.get("pvend") && f === Number(params.get("pvend"))) { window.__pvForceWon = true; w.wd_game_set_end(w.wd_game_scene(0) + 900); } // test hook
      pvStep(1 / 60);
    }
    if (params.get("bench")) { // ?auto=N&bench=M: milliseconds per frame of the sim, the sprite build, the canvas draw and the UI draw
      const M = Number(params.get("bench")), T = {}; const tm = (k, fn) => { const t0 = performance.now(); for (let i = 0; i < M; i++) fn(); T[k] = +((performance.now() - t0) / M).toFixed(3); };
      tm("update", () => w.wd_game_update(1000 / 60, 0)); tm("build", () => w.wd_render_build()); tm("render", () => render()); tm("ui", () => drawUi());
      window.__bench = "BENCH " + JSON.stringify(T) + " sprites=" + new Float64Array(w.memory.buffer, w.wd_render_ptr(), 3)[2]; console.log(window.__bench);
    }
    if (params.get("endfx")) { // test hook: ?endfx=MS draws the level-complete effects MS milliseconds in
      endfx.begin({ px: 700, py: 380, main: 0x04ff00, second: 0x00fbff, ldm: false, onText: () => { w.wd_ui_banner(0); }, onEnd: () => {} });
      for (let ms = 0; ms < Number(params.get("endfx")); ms += 1000 / 60) { endfx.update(1000 / 60); w.wd_ui_step(1000 / 60); }
    }
    const buf = render();
    if (params.get("endfx")) { endfx.drawFront(); drawUi(); }
    hud.textContent = `frame ${frames}  x=${buf[3].toFixed(0)}  sprites=${buf[2]} ${window.__bench || ""}`;
    window.__done = true;
    return;
  }

  if (params.get("ui") === "options" && menuMode) { openOptions(); const pg = Number(params.get("page") || 0); if (pg) { w.wd_ui_pointer(0, 0, 0); } }
  if (params.get("ui") === "garage" && menuMode) { WD_EVENTS["Garage"]("Garage", { w, put, text, params }).then(() => { if (params.get("gartab")) w.wd_gar_tab(Number(params.get("gartab"))); }); }
  const SIM = Number(params.get("sim") || 0);
  const DRAG = params.get("drag") ? params.get("drag").split(",").map(Number) : null;
  if (SIM) {
    // headless script: frames at 60 fps, clicks (x,y pairs) every 10 frames, then one picture
    const cl = (params.get("clicks") || "").split(",").filter(Boolean).map(Number);
    const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
    let ci = 0;
    if (params.get("ui") === "end") { const [p, n] = put(params.get("msg") || "Awesome!"); pushEndCoins({ main: params.get("main") !== "0", count: Number(params.get("coins") || 0), run: Number(params.get("run") || 0), saved: Number(params.get("saved") || 0) }); w.wd_ui_banner(0); w.wd_ui_end_layer(37, 120, 185, 0, p, n); }
    for (let f = 0; f < SIM; f++) {
      if (cl.length && f % 10 === 5 && ci < cl.length) { const x = cl[ci++], y = cl[ci++]; w.wd_ui_pointer(0, x, y); w.wd_ui_pointer(2, x, y); }
      if (DRAG) { // drag=startX,y,endX,frames,startFrame[,hold]: press, move in even steps, release (then wait `hold` more frames)
        const [dx0, dy0, dx1, dn, df, hold] = DRAG;
        if (f === df) w.wd_ui_pointer(0, dx0, dy0);
        if (f > df && f <= df + dn) w.wd_ui_pointer(1, dx0 + (dx1 - dx0) * (f - df) / dn, dy0);
        if (f === df + dn + (hold || 0)) w.wd_ui_pointer(2, dx1, dy0);
      }
      held = JUMP > 0 && (f % JUMP) < 8;
      if (params.get("endnear") && f === 1 && !menuMode) w.wd_game_set_end(w.wd_game_scene(0) + 1100); // test hook: the end portal just ahead
      w.wd_ui_step(1000 / 60); w.wd_trig_time(f * 16.6667); w.wd_set_audio_meter(beat(f * 16.6667));
      if (window.WD_EDITOR_ON) w.wd_ed_frame(1000 / 60, edKeys());
      const rr = window.WD_EDITOR_ON ? 0 : w.wd_game_update(1000 / 60, held ? 1 : 0);
      pollSlide();
      handleUi();
      if (!menuMode && !paused) w.wd_ui_hud_update(Math.max(0, Math.min(100, w.wd_game_scene(0) * 0)));
      if (params.get("pause") && f === Number(params.get("pause"))) doPause();
      if (f % 10 === 9) await sleep(200);
    }
    await sleep(500);
    const b = render(); camXUi = b[0]; camYUi = b[1]; drawUi();
    hud.textContent = menuMode ? "" : `frame ${SIM} x=${b[3].toFixed(0)}`;
    window.__done = true;
    return;
  }
  const clicks = (params.get("clicks") || "").split(",").filter(Boolean).map(Number);
  let clickIdx = 0, tick = 0; const cpsTimes = []; let cpsHeld = false;
  const loop = (now) => {
    if (clicks.length && ++tick % 6 === 0 && clickIdx < clicks.length) { const x = clicks[clickIdx++], y = clicks[clickIdx++]; w.wd_ui_pointer(0, x, y); w.wd_ui_pointer(2, x, y); }
    const dt = Math.min(100, now - last); last = now;
    w.wd_ui_step(dt);
    if (window.WD_EDITOR_ON) w.wd_ed_frame(dt, edKeys()); else if (!paused) step(dt);
    pvStep(paused ? 0 : dt / 1000);
    if (!won && endfx.active) endfx.reset();
    if (!paused) endfx.update(dt);
    endfx.portalUpdate(paused ? 0 : dt, w.wd_game_scene(5), w.wd_game_scene(4), iconColor("iconMainColor", "04FF00"), !menuMode && !window.WD_EDITOR_ON && !readSettings().enableLDM);
    const buf = render(); camXUi = buf[0]; camYUi = buf[1];
    if (!menuMode && buf[6] && !wasDead) { afx.playEffect("explode_11", { volume: 0.65 * afx.userSfx() }); if (!(practiceOn && !readSettings().practiceMusicSync)) stopSong(); }
    wasDead = !!buf[6];
    if (!menuMode) w.wd_ui_hud_update(Math.max(0, Math.min(100, buf[30] * 100)));
    if (!menuMode && !paused) {
      if (!won) { playTime += dt / 1000; if (held && !wasHeld) jumps++; wasHeld = held; }
      if (!won && w.wd_game_percent(2)) { won = true; winClock = 0; winStarted = false; } // the player starts flying into the end wall
      // player.js `playEndAnimation` takes 1000 ms; only when it ends (the player has hit the wall) does `_levelComplete` run
      if (won && !winStarted) {
        winClock += dt;
        if (winClock >= 1000) {
          winStarted = true;
          if (!practiceOn) {
            w.wd_coins_commit(); flushStorage();
            try {
              const lid = "level_" + LEVEL_ID, set = JSON.parse(localStorage.getItem("gd_completedSet") || "[]");
              if (!set.includes(lid)) { set.push(lid); localStorage.setItem("gd_completedSet", JSON.stringify(set)); localStorage.setItem("gd_completedLevels", String(set.length)); }
            } catch (e) {}
          }
          afx.playRaw("endStart_02", 0.8 * afx.userSfx());
          try { localStorage.setItem((practiceOn ? "practiceBestPercent_" : "bestPercent_") + "level_" + LEVEL_ID, "100"); } catch (e) {}
          endfx.begin({
            px: w.wd_game_scene(5) - w.wd_game_scene(0), py: 460 - w.wd_game_scene(4) + w.wd_game_scene(1),
            main: iconColor("iconMainColor", "04FF00"), second: iconColor("iconSecondaryColor", "00FBFF"), ldm: !!readSettings().enableLDM,
            onText: () => w.wd_ui_banner(practiceOn ? 1 : 0),
            onEnd: () => { pushEndCoins(); const [p, n] = put(MESSAGES[Math.floor(Math.random() * MESSAGES.length)]); w.wd_ui_end_layer(attempts, jumps, playTime, practiceOn ? 1 : 0, p, n); },
          });
        }
      }
    }
    endfx.drawFront();
    drawUi();
    if (loadingLevel && fontOf[2]) drawBitmapText(fontOf[2], "Downloading Level Assets...", 20, W / 2, H / 2, 1, 0.5, 0.5, 0xffffff);
    handleUi();
    frame++; acc += dt; if (acc > 500) { fps = Math.round(frame * 1000 / acc); frame = 0; acc = 0; }
    { const st = { ...OPT_DEFAULTS, ...readSettings() }; if (held && !cpsHeld) cpsTimes.push(now); cpsHeld = held; while (cpsTimes.length && now - cpsTimes[0] > 1000) cpsTimes.shift();
      hud.textContent = (menuMode || w.wd_ui_menu_active()) ? "" : [st.noclip ? "Ignore Damage" : "", st.showFPS ? `FPS: ${fps}` : "", st.showCPS ? `${cpsTimes.length} CPS` : "", st.showHitboxes ? "(hitboxes: not drawn yet)" : ""].filter(Boolean).join("\n"); }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
}
main().catch(fail);
