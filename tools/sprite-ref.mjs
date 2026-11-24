// Reference dump of the sprites the original `_spawnObject` creates for a level, using a recording stand-in for Phaser.
// usage: node tools/sprite-ref.mjs <levelIdx> [maxObjects]
import fs from "node:fs";
import vm from "node:vm";
import { load } from "./physics-harness.mjs";

const root = new URL("..", import.meta.url).pathname;
export async function makeRecorder() {
  const H = await load();
  const { ctx } = H;
  vm.runInContext('var S = 1; var E = 0; var fs = 1000; var gs = 1001; var screenHeight = 640;', ctx);
  vm.runInContext(fs.readFileSync(root + "ref/assets/scripts/core/triggers.js", "utf8") + "\nthis.Easing = Easing; this.ColorManager = ColorManager;", ctx);
  const cfg = fs.readFileSync(root + "ref/assets/scripts/utils/config.js", "utf8");
  const a0 = cfg.indexOf("const atlasList = ["), a1 = cfg.indexOf('const CUSTOM_CUBE_KEY');
  const i0 = cfg.indexOf('window.ICON_COUNTS'), i1 = cfg.indexOf('function iconAtlasPath');
  vm.runInContext(cfg.slice(i0, i1) + cfg.slice(a0, a1), ctx);
  // atlases as Phaser builds them from the JSON files
  const atlases = {};
  for (const f of fs.readdirSync(root + "ref/assets/sheets").filter((n) => n.endsWith(".json") && !/AnimDesc/.test(n))) {
    const j = JSON.parse(fs.readFileSync(root + "ref/assets/sheets/" + f, "utf8"));
    let frames = j.frames;
    if (!frames && j.textures) frames = j.textures.flatMap((t) => t.frames); // TexturePacker multi-atlas files
    if (Array.isArray(frames)) frames = Object.fromEntries(frames.map((fr) => [fr.filename, fr]));
    atlases[f.replace(/\.json$/, "")] = frames || {};
  }
  const frameObj = (name, d) => {
    const ss = d.spriteSourceSize || { x: 0, y: 0, w: d.frame.w, h: d.frame.h };
    return { name, width: d.frame.w, height: d.frame.h, realWidth: (d.sourceSize || { w: d.frame.w }).w, realHeight: (d.sourceSize || { h: d.frame.h }).h, customData: d, rotated: !!d.rotated };
  };
  const textures = {
    getFrame: (k, f) => (atlases[k]?.[f] ? frameObj(f, atlases[k][f]) : { name: f, width: 64, height: 64, realWidth: 64, realHeight: 64, customData: {} }),
    exists: (k) => !!atlases[k],
    get: (k) => ({ has: (f) => !!atlases[k]?.[f], get: (f) => (atlases[k]?.[f] ? frameObj(f, atlases[k][f]) : undefined), getFrameNames: () => Object.keys(atlases[k] || {}) }),
  };
  const any = new Proxy(function () {}, { get: (_, k) => (k === Symbol.toPrimitive ? () => 0 : any), apply: () => any, construct: () => any });
  ctx.Phaser = new Proxy({ BlendModes: { NORMAL: 0, ADD: 1, MULTIPLY: 2, SCREEN: 3, ERASE: 17 } }, { get: (t, k) => (k in t ? t[k] : any) });
  const created = [];
  class Rec {
    constructor(kind, args) { this.kind = kind; this.args = args; this.x = args[0] ?? 0; this.y = args[1] ?? 0; this.rotation = 0; this.scaleX = 1; this.scaleY = 1; this.alpha = 1; this.visible = true; this.active = true; this.depth = 0; this.flipX = false; this.flipY = false; this.tintv = null; this.blend = 0; this.originX = 0.5; this.originY = 0.5; this.data = {}; this.list = []; this.parentContainer = null; this.scene = Rec.sceneRef; if (kind === "image") { this.atlas = args[2]; this.frameName = args[3]; } created.push(this); }
    get _raw() { return this; }
    get frame() { return { name: this.frameName }; }
    setOrigin(x, y = x) { this.originX = x; this.originY = y; return this; }
    setScale(x, y = x) { this.scaleX = x; this.scaleY = y; return this; }
    setScaleX(v) { this.scaleX = v; return this; } setScaleY(v) { this.scaleY = v; return this; }
    get scale() { return this.scaleX; } set scale(v) { this.scaleX = this.scaleY = v; }
    setAlpha(v = 1) { this.alpha = Math.max(0, Math.min(1, v)); return this; }
    get tintTopLeft() { return this.tintv ?? 0xffffff; }
    setVisible(v) { this.visible = v; return this; }
    setDepth(v) { this.depth = v; return this; }
    setFlipX(v) { this.flipX = v; return this; } setFlipY(v) { this.flipY = v; return this; }
    setTint(v) { this.tintv = v; return this; } clearTint() { this.tintv = null; return this; }
    setBlendMode(v) { this.blend = v; return this; } get blendMode() { return this.blend; }
    setPosition(x, y = x) { this.x = x; this.y = y; return this; }
    setAngle(d) { this.angle = d; return this; } get angle() { return this.rotation * 180 / Math.PI; } set angle(d) { this.rotation = (-180 + ((((d + 180) % 360) + 360) % 360)) * (Math.PI / 180); }
    setRotation(r) { this.rotation = r; return this; }
    setData(k, v) { this.data[k] = v; return this; } getData(k) { return this.data[k]; }
    setScrollFactor() { return this; } setTexture(a, f) { this.atlas = a; this.frameName = f; return this; } setFrame(f) { this.frameName = f; return this; }
    setDisplaySize() { return this; } setSize() { return this; } setInteractive() { return this; } setMask() { return this; } setName() { return this; } setDepthOffset() { return this; }
    add(c) { for (const x of Array.isArray(c) ? c : [c]) { if (x && x.parentContainer && x.parentContainer.list) { const i = x.parentContainer.list.indexOf(x); if (i >= 0) x.parentContainer.list.splice(i, 1); } this.list.push(x); if (x && typeof x === "object") x.parentContainer = this; } return this; }
    addAt(c) { this.add(c); return this; }
    off() { return this; } once() { return this; } removeFromDisplayList() { return this; } sort() { return this; } destroy() { this.active = false; } getBounds() { return { x: 0, y: 0, width: 0, height: 0 }; }
    get displayWidth() { return 0; } get displayHeight() { return 0; }
  }
  const mk = (kind) => (...args) => new Proxy(new Rec(kind, args), { get: (t, k) => (kind === "particles" && (k === "setTint" || k === "clearTint" || k === "tintTopLeft") ? undefined : k in t ? t[k] : typeof k === "symbol" || String(k).startsWith("_") ? undefined : (...a) => t) });
  const scene = new Proxy({
    textures, add: new Proxy({ image: mk("image"), container: mk("container"), sprite: mk("image"), graphics: mk("graphics"), text: mk("text"), bitmapText: mk("text"), particles: mk("particles"), rectangle: mk("rect"), tileSprite: mk("tile"), renderTexture: mk("rt"), existing() {} }, { get: (t, k) => t[k] ?? mk(String(k)) }),
    _editorPlaytestActive: false,
    tweens: { add() { return any; }, killTweensOf() {} }, time: { now: 0, delayedCall() {} }, cameras: { main: { width: 1138, height: 640 } }, sys: { game: { renderer: {} } }, events: { on() {}, once() {} }, scale: { width: 1138, height: 640 },
  }, { get: (t, k) => (k in t ? t[k] : any) });
  Rec.sceneRef = scene;
  return { H, ctx, scene, created, Rec };
}

if (process.argv[1] && process.argv[1].endsWith("sprite-ref.mjs")) {
  const { ctx, scene, created } = await makeRecorder();
  const files = fs.readdirSync(root + "ref/assets/levels").filter((f) => f.endsWith(".txt")).sort();
  const lvlIdx = Number(process.argv[2] || 0);
  const lvl = ctx.parseLevel(fs.readFileSync(root + "ref/assets/levels/" + files[lvlIdx], "utf8"));
  const cam = { _v: 0, value: 0 };
  let L;
  try { L = new ctx.LevelObject(scene, cam); } catch (e) { console.log("constructor failed:", e.stack.split("\n").slice(0, 6).join("\n")); process.exit(1); }
  const max = Number(process.argv[3] || 50);
  let n = 0, fail = 0;
  for (const lo of lvl.objects.slice(0, max)) {
    const before = created.length;
    try { L._spawnObject(lo); n++; } catch (e) { fail++; if (fail < 4) console.log("spawn failed for id", lo.id, e.stack.split("\n").slice(0, 4).join("\n")); }
    if (n < 4) console.log(lo.id, created.slice(before).filter((c) => c.kind === "image").map((c) => `${c.frameName}@(${c.x},${c.y}) rot=${c.rotation} sc=${c.scaleX},${c.scaleY} depth=${c.depth}`));
  }
  console.log(`spawned ${n} (${fail} failed), ${created.length} recorded objects`);
}
