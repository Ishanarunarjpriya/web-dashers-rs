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
  // `_importGMD`: a .gmd (GD's XML export) or .gdlvl (a Web Dashers level string) becomes a created level
  const inflateLevel = async (str) => {
    try {
      let b64 = String(str).trim().replace(/-/g, "+").replace(/_/g, "/"); while (b64.length % 4) b64 += "=";
      const bin = atob(b64), bytes = new Uint8Array(bin.length); for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
      return await new Response(stream).text();
    } catch (err) { return String(str); }
  };
  E['Pop("EmImport", 0)'] = (ev, wd) => {
    const input = document.createElement("input"); input.type = "file"; input.accept = ".gmd,.gdlvl";
    input.onchange = (e) => {
      const file = e.target.files[0]; if (!file) return;
      const isGdlvl = file.name.toLowerCase().endsWith(".gdlvl");
      const reader = new FileReader();
      reader.onload = async (event) => {
        try {
          const content = isGdlvl ? await inflateLevel(event.target.result) : event.target.result;
          const xml = new DOMParser().parseFromString(content, "text/xml");
          const x = { name: "Imported Level", data: "", version: 1, length: 0, id: "NA", desc: "", officialSongId: 0, customSongId: 0 };
          xml.querySelectorAll("key, k").forEach((node) => {
            const k = node.textContent, v = node.nextElementSibling; if (!v) return;
            const val = v.textContent;
            if (k === "k2") x.name = val; if (k === "k4") x.data = val; if (k === "k1") x.id = val;
            if (k === "k23") x.length = parseInt(val) || 0; if (k === "k16") x.version = parseInt(val) || 1;
            if (k === "k8") x.officialSongId = parseInt(val) || 0; if (k === "k45") x.customSongId = parseInt(val) || 0;
            if (k === "k3") { try { x.desc = atob(val); } catch (err) { x.desc = val; } }
          });
          if (!x.data) throw new Error("No level string found.");
          let songName = "Stereo Madness", songId = -1;
          if (x.customSongId > 0) { songId = x.customSongId; songName = `NG#${x.customSongId}`; }
          else { songId = -x.officialSongId - 1; try { songName = (window.__allLevels || window.allLevels)[x.officialSongId][1]; } catch (err) { songName = "Unknown"; } }
          const levels = read();
          const lvl = { levelName: x.name, song: songName, songId, levelId: (x.id === "0" || !x.id) ? "NA" : x.id, levelString: x.data, levelLength: x.length, normalBest: 0, practiceBest: 0, description: x.desc || "", version: x.version, status: "Unverified", createdId: nextLocalId() };
          levels.push(lvl); write(levels); push(wd);
          const [p, n] = wd.put(lvl.createdId); wd.w.wd_em_open_new(p, n);
        } catch (err) { console.error("GMD Import Error:", err); alert("Failed to parse .gmd: " + err.message); }
      };
      reader.readAsText(file);
    };
    input.click();
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
  // ---- pause menu (level-editor.js `_saveEditorLevel` and the buttons of `_initEditorPauseMenu`) ----
  // `GameCacheManager.setItemWithQuotaRecovery`: a full storage drops the cached downloads and tries once more
  const setWithRecovery = (key, value) => {
    try { localStorage.setItem(key, value); }
    catch (e) {
      if (e.name !== "QuotaExceededError" && e.code !== 22 && e.code !== 1014) throw e;
      if (window.gameCache && window.gameCache.clearCache) window.gameCache.clearCache();
      else Object.keys(localStorage).forEach((k) => { if (k.startsWith("webdash_cache_")) localStorage.removeItem(k); });
      localStorage.setItem(key, value);
    }
  };
  const saveEditor = (wd) => {
    const id = window.__edId; const levels = read(); const l = levels.find((x) => x.createdId === id);
    if (!l) return true;
    l.levelString = wd.text(wd.w.wd_ed_level_string()); l.lastModified = Date.now();
    const pid = wd.w.wd_ed_pending_song(); // the song picked in the level settings (`_editorPendingSongId`)
    if (!Number.isNaN(pid)) { l.songId = pid; l.song = wd.text(wd.w.wd_ed_pending_song_name()) || l.song; }
    try { setWithRecovery("created_levels", JSON.stringify(levels)); }
    catch (e) { console.error("Failed to save editor level", e); alert("Could not save this level because browser storage is full. Free up browser storage and try again."); return false; }
    wd.w.wd_ed_clear_pending_song();
    return true;
  };
  // back to the main page: the level view of this level opens again (and starts when it was "Save and Play")
  const leave = (play) => { try { sessionStorage.setItem("wd_ed_return", JSON.stringify({ id: window.__edId, play: !!play })); } catch (e) {} location.reload(); };
  E['Pop("EdArt", 0)'] = E['Pop("EdArt", 1)'] = () => { window.__edArtDirty = true; }; // background or ground changed in the level settings
  E['Pop("EdPm", 1)'] = (ev, wd) => { if (saveEditor(wd)) leave(true); };
  E['Pop("EdPm", 2)'] = (ev, wd) => { if (saveEditor(wd)) leave(false); };
  E['Pop("EdPm", 3)'] = (ev, wd) => { saveEditor(wd); };
  E['Pop("EdPm", 4)'] = () => leave(false);
  E['Pop("EdPmGlow", 0)'] = (ev, wd) => { try { const c = JSON.parse(localStorage.getItem("gd_settings") || "{}"); c.showEditorGlow = !!wd.w.wd_ed_glow(); localStorage.setItem("gd_settings", JSON.stringify(c)); } catch (e) {} };
  // "Play" of the level view: a created level starts like an online one (its song is an official one by name, or a downloaded custom one)
  E['Pop("EmPlay", 0)'] = async (ev, wd) => {
    const id = viewId(wd); const l = read().find((x) => x.createdId === id); if (!l || !window.__playOnline) return;
    let songData = null;
    try {
      if (l.songId > 0 && window.__songById) songData = await window.__songById(l.songId);
      else { const e = (window.__allLevels || []).find((x) => x[1] === l.song); if (e) { const r = await fetch(`../ref/assets/music/${e[4] || String(e[1]).replaceAll(" ", "")}.mp3`); if (r.ok) songData = await r.arrayBuffer(); } }
    } catch (e) { console.warn("song", e); }
    window.__playOnline({ id: String(l.createdId), name: l.levelName, text: l.levelString, songData, offset: 0, created: l.createdId });
  };
  // after leaving the editor: the level view of the level that was open
  (() => {
    let ret = null; try { ret = JSON.parse(sessionStorage.getItem("wd_ed_return") || "null"); sessionStorage.removeItem("wd_ed_return"); } catch (e) {}
    if (!ret) return;
    const t = setInterval(() => {
      const wd = window.__WD; if (!wd || !wd.w.wd_em_open_new || !wd.w.wd_ui_ready()) return;
      clearInterval(t);
      push(wd);
      const [p, n] = wd.put(ret.id); wd.w.wd_em_open_new(p, n);
      if (ret.play) setTimeout(() => E['Pop("EmPlay", 0)']('Pop("EmPlay", 0)', wd), 300);
    }, 50);
  })();
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
