// The loading screen of loading-screen.js: the blue level background, RobTop and Geometry Dash logos, a progress bar, a random tip in the
// gold font, and (where the original shows the Phaser logo) the Rust logo. Drawn on the 2D canvas while the rest of the game loads.
const W = 1138, H = 640, CX = W / 2, CY = H / 2;

const loadImage = (url) => new Promise((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = () => rej(new Error("image " + url)); i.src = url; });
const loadText = async (url) => { const r = await fetch(url); if (!r.ok) throw new Error(url + " " + r.status); return r.text(); };

const parseSheet = (j) => {
  let frames = j.frames;
  if (!frames && j.textures) frames = j.textures.flatMap((t) => t.frames);
  if (Array.isArray(frames)) frames = Object.fromEntries(frames.map((f) => [f.filename, f]));
  return frames || {};
};
const parseFont = (fnt) => {
  const chars = {}, kern = {}; let size = 0;
  for (const line of fnt.split("\n")) {
    const kv = Object.fromEntries([...line.matchAll(/(\w+)=("[^"]*"|\S+)/g)].map((m) => [m[1], m[2].replace(/"/g, "")]));
    if (line.startsWith("info ")) size = Number(kv.size);
    else if (line.startsWith("char ")) chars[kv.id] = { x: +kv.x, y: +kv.y, w: +kv.width, h: +kv.height, xo: +kv.xoffset, yo: +kv.yoffset, adv: +kv.xadvance };
    else if (line.startsWith("kerning ")) kern[kv.first + "," + kv.second] = +kv.amount;
  }
  return { chars, kern, size };
};

/** Shows the screen and returns `{ progress(v), done() }`; `ctx` is the 2D context of the game canvas. */
export async function showLoading(ctx, SHEETS) {
  const [bg, bar, rust, goldImg, goldFnt, webImg, webJson, launchImg, launchJson, src] = await Promise.all([
    loadImage("../ref/assets/game-bg/game_bg_01_001-hd.png"), loadImage("../ref/assets/sprites/sliderBar.png"), loadImage("rust_logo.svg"),
    loadImage("../ref/assets/fonts/goldFont.png"), loadText("../ref/assets/fonts/goldFont.fnt"),
    loadImage(SHEETS + "GJ_WebSheet.png"), loadText(SHEETS + "GJ_WebSheet.json"), loadImage(SHEETS + "GJ_LaunchSheet.png"), loadText(SHEETS + "GJ_LaunchSheet.json"),
    loadText("../ref/assets/scripts/core/loading-screen.js").catch(() => ""),
  ]);
  const web = parseSheet(JSON.parse(webJson)), launch = parseSheet(JSON.parse(launchJson)), font = parseFont(goldFnt);
  let messages = ["Loading..."];
  try { const m = src.match(/LOADING_MESSAGES\s*=\s*(\[[\s\S]*?\n\s*\]);/); if (m) messages = new Function("return " + m[1])(); } catch (e) {}
  const msg = messages[Math.floor(Math.random() * messages.length)];

  // the background is tinted blue (0x0066ff) and scaled to cover the screen
  const tinted = document.createElement("canvas"); tinted.width = bg.width; tinted.height = bg.height;
  { const g = tinted.getContext("2d"); g.drawImage(bg, 0, 0); g.globalCompositeOperation = "multiply"; g.fillStyle = "#0066ff"; g.fillRect(0, 0, bg.width, bg.height); g.globalCompositeOperation = "destination-in"; g.drawImage(bg, 0, 0); }
  const bgScale = Math.max(W / bg.width, H / bg.height);

  const sprite = (sheet, img, name, x, y, scale) => {
    const d = sheet[name]; if (!d) return;
    const f = d.frame, src = d.sourceSize || { w: f.w, h: f.h }, ss = d.spriteSourceSize || { x: 0, y: 0 };
    ctx.save(); ctx.translate(x, y); ctx.scale(scale, scale);
    ctx.drawImage(img, f.x, f.y, f.w, f.h, -src.w / 2 + ss.x, -src.h / 2 + ss.y, f.w, f.h);
    ctx.restore();
  };
  const text = (str, x, y, size) => { // bitmap text, centred on (x, y)
    const k = size / font.size; let cur = 0, prev = null; const glyphs = [];
    for (const ch of str) { const id = ch.codePointAt(0), g = font.chars[id]; if (prev !== null) cur += font.kern[prev + "," + id] || 0; if (g) { glyphs.push([g, cur]); cur += g.adv; } prev = id; }
    const width = cur * k, lineH = 67 * k;
    for (const [g, gx] of glyphs) { if (!g.w) continue; ctx.drawImage(goldImg, g.x, g.y, g.w, g.h, x - width / 2 + (gx + g.xo) * k, y - lineH / 2 + g.yo * k, g.w * k, g.h * k); }
  };

  let value = 0, alive = true;
  const draw = () => {
    ctx.save(); ctx.setTransform(1, 0, 0, 1, 0, 0); ctx.clearRect(0, 0, W, H); ctx.restore();
    ctx.fillStyle = "#000"; ctx.fillRect(0, 0, W, H);
    ctx.save(); ctx.translate(CX, CY); ctx.scale(bgScale, bgScale); ctx.drawImage(tinted, -bg.width / 2, -bg.height / 2); ctx.restore();
    const sx = CX - 105 - 100, sy = CY + 110;
    const fw = Math.max(0, Math.min(380, value * 380));
    if (fw > 0) for (let x = 0; x < fw; x += 64) ctx.drawImage(bar, 0, 0, Math.min(64, fw - x), 14, sx + x, sy - 2 - 7, Math.min(64, fw - x), 14); // a tile strip 14 px high, as the TileSprite
    sprite(web, webImg, "slidergroove.png", CX, sy, 1);
    text(msg, CX, CY + 187, 30);
    sprite(launch, launchImg, "RobTopLogoBig_001.png", CX, CY - 120, 0.8);
    sprite(web, webImg, "GJ_logo_001.png", CX, CY, 1.1);
    // the Rust logo takes the corner where the original has the Phaser logo
    const size = 150, rs = size / rust.height;
    ctx.save(); ctx.translate(W + 8 - size / 2, H + 12 - size / 2); ctx.scale(rs, rs); ctx.drawImage(rust, -rust.width / 2, -rust.height / 2, rust.width, rust.height); ctx.restore();
  };
  const loop = () => { if (!alive) return; draw(); requestAnimationFrame(loop); };
  requestAnimationFrame(loop);
  return {
    progress(v) { value = Math.max(value, Math.min(1, v)); },
    /** the bar fills up, then after 200 ms (`delayedCall(200, ...)`) the screen goes */
    done() { value = 1; return new Promise((r) => setTimeout(() => { alive = false; ctx.save(); ctx.setTransform(1, 0, 0, 1, 0, 0); ctx.clearRect(0, 0, W, H); ctx.restore(); r(); }, 200)); },
  };
}
