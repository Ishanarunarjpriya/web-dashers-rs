// Host side of the creator menu / level editor: storage (`created_levels`), key forwarding, level import/export.
(() => {
  const NEW_LEVEL_STRING = "H4sIAAAAAAAACq1QwRHDMAhbyO0hwIlzfWWGDsAAXaHD10Z-9Ff3Ln4gG4GMeD2tFYRLaEBrWGitARCUwKTHDbEFRCT2wF3yBOrXvYVEC7wRKSi6JoirBY8FwdHB9iVJjZ5ckP1rlf19taIv7pLGh-wP43XROPq9z9mOtX1uS7LldcKKzPx41ZKwEbz0yPueUSfPF9qApx3kMlrGJE7PSBbCIlYpy5QVuheMciE0AgiaoFRUihk5I2ec0Knp1PTK9slxYDM2OIFmjL8bv-1mBmB6YrvO4UErHR4fJXMaP9sDAAA=";
  const read = () => { try { return JSON.parse(localStorage.getItem("created_levels") || "[]") || []; } catch (e) { return []; } };
  const write = (v) => { try { localStorage.setItem("created_levels", JSON.stringify(v)); } catch (e) {} };
  const nextLocalId = () => {
    let max = 0;
    for (const l of read()) if (typeof l.createdId === "string") { const n = parseInt(l.createdId.split("_")[1], 10); if (!isNaN(n) && n > max) max = n; }
    return "local_" + (max + 1);
  };
  const push = (wd) => { const [p, n] = wd.put(JSON.stringify(read())); wd.w.wd_em_levels(p, n); };
  const viewId = (wd) => { const n = wd.w.wd_em_view_id(); return wd.text(n); };
  const E = window.WD_EVENTS;
  E['Creator'] = (ev, wd) => push(wd);
  E['Pop("EmNew", 0)'] = (ev, wd) => {
    const levels = read();
    let counter = 0;
    while (levels.some((l) => l.levelName === "Unnamed " + counter)) counter++;
    const lvl = { levelName: "Unnamed " + counter, song: "Stereo Madness", songId: -1, levelId: null, levelString: NEW_LEVEL_STRING, levelLength: 0, normalBest: 0, practiceBest: 0, description: "", version: 1, status: "Unverified", createdId: nextLocalId() };
    levels.push(lvl); write(levels); push(wd);
    const [p, n] = wd.put(lvl.createdId); wd.w.wd_em_open_new(p, n);
  };
  const field = (ev, wd) => {
    const which = parseInt(ev.match(/(\d+)\)$/)[1], 10);
    const id = viewId(wd); if (!id) return;
    const levels = read(); const l = levels.find((x) => x.createdId === id); if (!l) return;
    l[which === 0 ? "levelName" : "description"] = wd.text(wd.w.wd_em_field(which));
    write(levels);
  };
  E['Pop("EmField", 0)'] = field; E['Pop("EmField", 1)'] = field;
  E['Pop("EmDelete", 0)'] = (ev, wd) => {
    const id = viewId(wd); const l = read().find((x) => x.createdId === id); if (!l) return;
    if (!confirm(`Are you sure you want to delete ${l.levelName}?`)) return;
    write(read().filter((x) => x.createdId !== id)); push(wd); wd.w.wd_em_after_delete();
  };
  addEventListener("keydown", (e) => {
    const wd = window.__WD; if (!wd || !wd.w.wd_em_key) return;
    if (e.key === "Backspace") wd.w.wd_em_key(8); else if (e.key === "Enter") wd.w.wd_em_key(13);
    else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) wd.w.wd_em_key(e.key.codePointAt(0));
  });
  for (let i = 1; i <= 8; i++) window.WD_ASSETS.push(["tab" + i, `../ref/assets/sprites/tab${i}.png`]);
  for (const n of ["GJ_moveBtn", "GJ_moveSBtn", "slidergroove2"]) window.WD_ASSETS.push([n, `../ref/assets/sprites/${n}.png`]);
  E['Pop("EmEdit", 0)'] = (ev, wd) => {
    const id = viewId(wd); const l = read().find((x) => x.createdId === id); if (!l) return;
    window.__edId = l.createdId; window.__startEditor(l.levelString, l.createdId);
  };
  // Pause button: stand-in for the pause menu (not ported) = "Save and Exit": store the level string, back to the page start.
  E['Pop("EdPause", 0)'] = (ev, wd) => {
    const id = window.__edId; const levels = read(); const l = levels.find((x) => x.createdId === id);
    if (l) { l.levelString = wd.text(wd.w.wd_ed_level_string()); l.lastModified = Date.now(); write(levels); }
    location.reload();
  };
  window.WD_ED = { read, write, push, nextLocalId, NEW_LEVEL_STRING, viewId };
})();
// test hooks: ?ui=creator | mylevels | levelview  (&seed=N to add N sample levels)
(() => {
  const q = new URLSearchParams(location.search), ui = q.get("ui");
  if (!["creator", "mylevels", "levelview", "editor"].includes(ui)) return;
  const t0 = performance.now();
  const t = setInterval(() => {
    const wd = window.__WD; if (!wd || !wd.w.wd_em_open_creator || !wd.w.wd_ui_ready() || performance.now() - t0 < 1500) return;
    clearInterval(t);
    const seed = parseInt(q.get("seed") || "0", 10);
    if (seed && !window.WD_ED.read().length) { const ls = []; for (let i = 1; i <= seed; i++) ls.push({ levelName: "Test Level " + (i % 3), song: "Stereo Madness", songId: -1, levelId: null, levelString: window.WD_ED.NEW_LEVEL_STRING, levelLength: i % 6, description: "", version: 1, status: "Unverified", createdId: "local_" + i }); window.WD_ED.write(ls); }
    window.WD_ED.push(wd);
    if (ui === "creator") wd.w.wd_em_open_creator();
    else if (ui === "mylevels") wd.w.wd_em_open_my_levels();
    else if (ui === "editor") { fetch("../ref/assets/levels/" + (q.get("edlevel") || "1") + ".txt").then((r) => r.text()).then((t) => window.__startEditor(t, "local_1")); }
    else { const [p, n] = wd.put("local_1"); wd.w.wd_em_open_new(p, n); }
  }, 50);
})();
