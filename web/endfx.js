// The effects of a completed level, in the order of game-scene.js `_levelComplete` -> `_showCompleteEffect` -> `_showCompleteText` -> `_showEndLayer`:
//   0 ms     circles at the end portal, eight light rays behind the level, camera shake for 1950 ms
//   1950 ms  "Level Complete" banner, two particle bursts, circles, ten fireworks (triggers.js `circleEffect` / `particleEffect`)
//   3450 ms  the end layer
export function createEndFx({ ctx, atlas, drawFrame, W }) {
  const T = 460;
  const rnd = (a, b) => a + Math.random() * (b - a);
  const quadOut = (v) => v * (2 - v);
  const hex = (n) => "#" + (n & 0xffffff).toString(16).padStart(6, "0");
  let t = 0, active = false, o = null, queue = [], circles = [], parts = [], rays = [], emitters = [], rayFade = null, shakeUntil = 0;
  let onText = null, onEnd = null;
  const sq = () => { const a = atlas["GJ_WebSheet"]; const d = a && a.frames["square.png"]; return d ? { img: a.img, d } : null; };

  const at = (ms, fn) => queue.push({ abs: t + ms, fn });
  // circleEffect(scene, x, y, radius, radius2, duration, filled, mode, color)
  const circle = (x, y, r0, r1, dur, filled, mode, color) => circles.push({ x, y, r0, r1, dur, filled, mode, color, t: 0 });

  const makeRays = (x, y, color) => { // _showCompleteEffect: eight beams, angles -135..-45 shuffled, each opening over ~180 ms
    const n = 8, step = 90 / n, ang = Array.from({ length: n }, (_, i) => -135 + i * step);
    for (let i = n - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [ang[i], ang[j]] = [ang[j], ang[i]]; }
    const len = Math.round(Math.sqrt(W * W + 102400)) + 65;
    let last = 0;
    for (let i = 0; i < n; i++) {
      const delay = Math.max(0, i * 195 + 40 + 40 * (Math.random() * 2 - 1));
      rays.push({
        x, y, color, delay, len, w1: 60 + 40 * (Math.random() * 2 - 1), dur: 180 + 40 * (Math.random() * 2 - 1),
        alpha: Math.min(1, Math.max(0, 155 / 255 + (100 / 255) * (Math.random() * 2 - 1))),
        angle: ang[i] + step * Math.random() + 180, fade: -1, fadeDelay: 0, fadeDur: 0,
      });
      if (delay > last) last = delay;
    }
    at(last + 400, () => { for (const r of rays) { r.fade = 0; r.fadeDelay = Math.random() * 200; r.fadeDur = 400 + (Math.random() * 2 - 1) * 100; } });
  };

  // an emitter that emits `quantity` particles every frame until `stopAfter` have been made
  const emitter = (x, y, ex, ey, quantity, stopAfter, cfg) => emitters.push({ x, y, ex, ey, quantity, left: stopAfter, ...cfg });
  const firework = (color1, color2) => { // triggers.js `particleEffect`
    const x = 200 + (W - 400) * Math.random(), y = 200 + Math.random() * 240;
    circle(x, y, 40, 140 + Math.random() * 60, 500, true, true, color2);
    emitter(x, y, [-20, 20], [-20, 20], 1, 25, { speed: [520, 920], life: [0, 500], s0: 0.4, s1: 0.13, a0: 1, a1: 0, color: color1 });
  };

  const showText = () => {
    const cx = W / 2;
    onText && onText();
    for (const col of [o.main, 0xffffff]) emitter(cx, 250, [-800, 800], [-80, 80], 50, 200, { speed: [300, 700], life: [0, 1000], s0: 0.4, s1: 0.13, a0: 1, a1: 1, color: col });
    circle(o.px, o.py, 10, W, 800, true, false, o.main);
    circle(cx, 250, 10, 1000, 800, true, false, o.main);
    for (let i = 0; i < 5; i++) at(i * 50, () => circle(o.px, o.py, 10, W, 500, false, true, o.main));
    if (!o.ldm) for (let i = 0; i < 10; i++) at(Math.max(0, i * 150 + (Math.random() * 160 - 80)), () => firework(o.main, o.second));
    at(1500, () => onEnd && onEnd());
  };

  const api = {
    get active() { return active; },
    /** `px, py`: the end portal on screen; colours are the player's main / secondary colour */
    begin(opts) {
      api.reset(); active = true; o = opts; onText = opts.onText; onEnd = opts.onEnd;
      for (let i = 0; i < 5; i++) at(i * 50, () => circle(o.px, o.py, 10, W, 500, false, true, o.main));
      circle(o.px, o.py, 10, 1000, 500, true, false, o.main);
      makeRays(o.px + 60, o.py, o.main);
      shakeUntil = 1950;
      at(1950, showText);
    },
    reset() { t = 0; active = false; o = null; queue = []; circles = []; parts = []; rays = []; emitters = []; shakeUntil = 0; },
    /** camera shake offset (Phaser: a random point in +-intensity * size each frame) */
    shake() { return active && t < shakeUntil ? [(Math.random() * 2 - 1) * 0.004 * W, (Math.random() * 2 - 1) * 0.004 * 640] : [0, 0]; },
    update(dtMs) {
      if (!active) return;
      t += dtMs;
      for (let guard = 0; guard < 8; guard++) { // events may schedule more events
        const ready = queue.filter((q) => q.abs <= t);
        if (!ready.length) break;
        queue = queue.filter((q) => q.abs > t);
        for (const q of ready) q.fn();
      }
      for (const c of circles) c.t += dtMs;
      circles = circles.filter((c) => c.t < c.dur);
      for (const r of rays) { r.delay -= dtMs; if (r.fade >= 0) r.fadeDelay -= dtMs; if (r.fade >= 0 && r.fadeDelay <= 0) r.fade += dtMs; if (r.delay < 0) r.age = (r.age || 0) + dtMs; }
      rays = rays.filter((r) => !(r.fade >= 0 && r.fade >= r.fadeDur));
      for (const e of emitters) {
        const n = Math.min(e.quantity, e.left); e.left -= n;
        for (let i = 0; i < n; i++) {
          const a = rnd(0, 360) * Math.PI / 180, sp = rnd(e.speed[0], e.speed[1]);
          parts.push({ x: e.x + rnd(e.ex[0], e.ex[1]), y: e.y + rnd(e.ey[0], e.ey[1]), vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, age: 0, life: rnd(e.life[0], e.life[1]), e });
        }
      }
      emitters = emitters.filter((e) => e.left > 0);
      const s = dtMs / 1000;
      parts = parts.filter((p) => { p.age += dtMs; p.x += p.vx * s; p.y += p.vy * s; return p.age < p.life; });
      if (!queue.length && !circles.length && !parts.length && !rays.length && !emitters.length && t > 3600) active = false;
    },
    /** the light rays (depth -1: behind the level) */
    drawBack() {
      if (!active) return;
      for (const r of rays) {
        if (r.delay > 0) continue;
        const k = Math.min(1, (r.age || 0) / r.dur), e = quadOut(k), h = 1 + (r.len - 1) * e, w = 2 + (r.w1 - 2) * e, w0 = 2 + (w - 2) / 4;
        const al = r.fade >= 0 && r.fadeDelay <= 0 ? r.alpha * (1 - Math.min(1, r.fade / r.fadeDur)) : r.alpha;
        ctx.save(); ctx.globalCompositeOperation = "lighter"; ctx.globalAlpha = al; ctx.fillStyle = hex(r.color);
        ctx.translate(r.x, r.y); ctx.rotate(r.angle * Math.PI / 180);
        ctx.beginPath(); ctx.moveTo(-w0 / 2, 0); ctx.lineTo(w0 / 2, 0); ctx.lineTo(w / 2, h); ctx.lineTo(-w / 2, h); ctx.closePath(); ctx.fill();
        ctx.restore();
      }
    },
    /** circles and particles (depths 55-59: above the level, below the end layer) */
    drawFront() {
      if (!active) return;
      for (const c of circles) {
        const tt = c.t / c.dur, te = c.filled && !c.mode ? quadOut(tt) : tt;
        const r = c.r0 + (c.r1 - c.r0) * te, al = c.mode ? (te < 0.5 ? te * 2 : (1 - te) * 2) : 1 - te;
        if (al <= 0) continue;
        ctx.save(); ctx.globalCompositeOperation = "lighter"; ctx.globalAlpha = Math.min(1, al);
        ctx.beginPath(); ctx.arc(c.x, c.y, Math.max(0, r), 0, Math.PI * 2);
        if (c.filled) { ctx.fillStyle = hex(c.color); ctx.fill(); } else { ctx.lineWidth = 4; ctx.strokeStyle = hex(c.color); ctx.stroke(); }
        ctx.restore();
      }
      const fr = sq(); if (!fr) return;
      for (const p of parts) {
        const k = p.life > 0 ? p.age / p.life : 1, e = p.e;
        const sc = e.s0 + (e.s1 - e.s0) * k, al = e.a0 + (e.a1 - e.a0) * k;
        drawFrame(fr, p.x, p.y, 0, sc, sc, al, e.color, 1, 0);
      }
    },
  };
  return api;
}
