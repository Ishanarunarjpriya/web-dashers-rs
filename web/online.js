// Online levels: the Geometry Dash server calls behind the Search, Featured, Saved and Daily / Weekly / Event screens (game-scene.js
// `_openOnlineLevelsScene`, `_fetchSpecialLevel`, `_playSelectedOnlineLevel`). The screens themselves are drawn by online.rs; this file
// asks the servers (through the same proxies the original uses), turns the answers into JSON for it, and starts a downloaded level.
(() => {
  const E = window.WD_EVENTS, SECRET = "Wmfd2893gb7";
  // The proxy of the Geometry Dash web port (its Cloudflare Worker, see "Geometry Dash Webport/server/cloudflare-worker.js"): GET or POST
  // `<worker>/?url=<target>` is fetched server side and handed back with CORS headers. It only lets boomlings.com, the GD CDN and Newgrounds through.
  const PROXY = "https://gd-proxy.theblehlollipop.workers.dev/?url=";
  const GD = "https://www.boomlings.com/database/";
  for (const [n, f] of [["GJ_button02", "GJ_button_02"], ["loadingCircle", "loadingCircle"], ["GJ_square01", "GJ_square01"], ["GJ_square02", "GJ_square02"], ["GJ_square04", "GJ_square04"], ["GJ_square05", "GJ_square05"]]) window.WD_ASSETS.push([n, `../ref/assets/sprites/${f}.png`]);

  // one form POST to a GD server page through the proxy
  const post = async (path, body) => {
    const ctrl = new AbortController(), timer = setTimeout(() => ctrl.abort(), 15000);
    try {
      const r = await fetch(PROXY + GD + path, { method: "POST", headers: { "Content-Type": "application/x-www-form-urlencoded" }, body, signal: ctrl.signal });
      if (!r.ok) throw new Error("HTTP " + r.status);
      return await r.text();
    } finally { clearTimeout(timer); }
  };
  const kv = (str, sep) => { const p = str.split(sep), o = {}; for (let i = 0; i + 1 < p.length; i += 2) o[p[i]] = p[i + 1]; return o; };
  const b64 = (s) => { try { return atob((s || "").replace(/-/g, "+").replace(/_/g, "/")); } catch (e) { return ""; } };
  const ls = (k) => { try { return localStorage.getItem(k); } catch (e) { return null; } };

  // the difficulty face index of the original: 0 N/A .. 5 insane, 6-10 the demons, 11 auto
  const diffOf = (m) => {
    if (parseInt(m["25"]) === 1) return 11;
    if (parseInt(m["17"]) === 1) {
      const d9 = parseInt(m["9"]), d43 = parseInt(m["43"]);
      if (!isNaN(d9) && d9 >= 1 && d9 <= 5) return [7, 8, 6, 9, 10][d9 - 1] ?? 8;
      if (!isNaN(d43)) { const mp = { 3: 7, 4: 8, 0: 6, 5: 9, 6: 10 }; return mp.hasOwnProperty(d43) ? mp[d43] : 8; }
      return 8;
    }
    return Math.min(5, Math.max(0, Math.round((parseInt(m["9"]) || 0) / 10)));
  };
  const allLevels = () => window.__allLevels || [];
  const officialName = (n) => (allLevels()[n] ? allLevels()[n][1] : "Unknown");
  const toLevel = (m, author, songName) => {
    const id = m["1"] || "";
    return {
      id, name: (m["2"] || "Unknown").trim(), author: author || "Unknown", desc: b64(m["3"]), songName: songName || "", customSong: (m["35"] || "").trim(),
      difficulty: diffOf(m), stars: parseInt(m["18"]) || 0, downloads: parseInt(m["10"]) || 0, likes: parseInt(m["14"]) || 0, length: parseInt(m["15"]) || 0,
      featured: (parseInt(m["19"]) || 0) > 0 ? 1 : 0, epic: parseInt(m["42"]) || 0, coins: parseInt(m["37"]) || 0,
      best: parseFloat(ls("bestPercent_level_" + id) || "0") || 0, bestPractice: parseFloat(ls("practiceBestPercent_level_" + id) || "0") || 0, viewed: ls("viewedLevel_" + id) ? 1 : 0,
    };
  };
  // `getGJLevels21.php`: levels # users # songs # page info # hash
  const parseList = (raw) => {
    const sec = raw.split("#"), players = {}, songs = {};
    for (const ps of (sec[1] || "").split("|").filter(Boolean)) { const p = ps.split(":"); if (p.length >= 2) players[p[0]] = p[1]; }
    for (const ss of (sec[2] || "").split("~:~").filter(Boolean)) { const sm = kv(ss, "~|~"); if (sm["1"]) songs[sm["1"]] = sm["2"] || ""; }
    const info = (sec[3] || "0:0:10").split(":");
    const levels = (sec[0] || "").split("|").filter(Boolean).map((s) => {
      const m = kv(s, ":"), c = (m["35"] || "").trim(), custom = c && c !== "0";
      return toLevel(m, players[m["6"]], custom ? (songs[c] || "Song #" + c) : officialName(parseInt(m["12"]) || 0));
    });
    return { levels, total: parseInt(info[0]) || 0, offset: parseInt(info[1]) || 0, count: parseInt(info[2]) || 10 };
  };

  const send = (wd, fn, s, ...rest) => { const [p, l] = wd.put(s); wd.w[fn](p, l, ...rest); };
  const results = (wd, levels, total, offset, count) => { window.__onlineList = levels; send(wd, "wd_on_results", JSON.stringify(levels), total, offset, count); };
  const fail = (wd, e) => { console.warn("online", e); send(wd, "wd_on_error", navigator.onLine === false ? "You are offline" : "Could not reach the servers"); };

  // a level asked for by its number: the level itself with its author (the original asks gdbrowser.com for it)
  const byId = async (id) => {
    const raw = await post("downloadGJLevel22.php", `levelID=${id}&secret=${SECRET}`);
    if (!raw || raw === "-1" || !raw.includes(":")) return null;
    const m = kv(raw.split("#")[0], ":"), c = (m["35"] || "").trim(), custom = c && c !== "0";
    let author = "Unknown", song = custom ? "Song #" + c : officialName(parseInt(m["12"]) || 0);
    for (const seg of raw.split("#").slice(1)) { const pm = /^\d+:([^:#|]+):\d+$/.exec(seg); if (pm) { author = pm[1]; break; } }
    try { const r = await fetch(`https://gdbrowser.com/api/level/${m["1"] || id}`); if (r.ok) { const j = await r.json(); if (j.author) author = j.author; if (j.songName) song = j.songName; } } catch (e) {}
    return toLevel(m, author, song);
  };

  const load = async (wd) => {
    const kind = wd.w.wd_on_kind(), page = wd.w.wd_on_page();
    try {
      if (kind === 2) { // saved levels live in this browser
        const saved = JSON.parse(localStorage.getItem("gd_saved_online_levels") || "[]");
        const list = saved.map((s) => { const id = String(s.id).replace(/^online_/, ""); return { id, name: s.name, author: s.author, desc: "", songName: s.songName || "", customSong: s.customSongID || "", difficulty: s.difficulty || 0, stars: s.stars || 0, downloads: s.downloads || 0, likes: s.likes || 0, length: s.length || 0, featured: s.featured ? 1 : 0, epic: s.epic || 0, coins: s.coins || 0, best: parseFloat(ls("bestPercent_level_" + id) || "0") || 0, bestPractice: parseFloat(ls("practiceBestPercent_level_" + id) || "0") || 0, viewed: 1 }; });
        results(wd, list, list.length, 0, Math.max(1, list.length)); return;
      }
      const q = wd.text(wd.w.wd_on_query()).trim();
      if (kind === 0 && /^\d+$/.test(q)) { const l = await byId(q); results(wd, l ? [l] : [], l ? -1 : 0, 0, 1); return; } // total -1: a single level, no page controls
      const params = { secret: SECRET, page };
      if (kind === 1) params.type = 6;
      else if (kind >= 10) params.type = kind - 10;
      else if (q) { params.type = 0; params.str = q; } else params.type = 2;
      const diff = wd.text(wd.w.wd_on_diff());
      if (diff) { params.diff = diff; const dt = wd.w.wd_on_demon(); if (dt) params.demonFilter = dt; }
      const body = Object.entries(params).map(([k, v]) => `${k}=${encodeURIComponent(v)}`).join("&");
      const raw = await post("getGJLevels21.php", body);
      if (!raw || raw === "-1") { results(wd, [], 0, 0, 10); return; }
      const r = parseList(raw);
      results(wd, r.levels, r.total, r.offset, r.count);
    } catch (e) { fail(wd, e); }
  };
  for (const ev of ["OnSearch", "OnFeatured", "OnSaved", "OnPage", "OnPrev", "OnNext", "OnRefresh"]) E[`Pop("${ev}", 0)`] = (e, wd) => load(wd);
  for (const t of [1, 2, 3, 4, 7, 11]) E[`Pop("OnQuick", ${t})`] = (e, wd) => load(wd);

  // daily / weekly / event (`_fetchSpecialLevel`)
  const SPECIAL = [{ dlId: -1, listType: 21 }, { dlId: -2, listType: 22 }, { dlId: -3, listType: 23 }];
  for (let k = 0; k < 3; k++) {
    E[`Pop("OnDaily", ${k})`] = async (ev, wd) => {
      const C = SPECIAL[k];
      try {
        let lvl = null;
        try { const raw = await post("getGJLevels21.php", `secret=${SECRET}&page=0&type=${C.listType}`); if (raw && raw !== "-1") lvl = parseList(raw).levels[0] || null; } catch (e) { console.warn("list request failed", e); }
        if (!lvl) {
          const raw = await post("downloadGJLevel22.php", `levelID=${C.dlId}&secret=${SECRET}`);
          if (!raw || raw === "-1" || !raw.includes(":")) throw new Error("no level");
          const segs = raw.split("#"), m = kv(segs[0], ":");
          let author = "Unknown";
          for (const seg of segs.slice(1)) { const pm = /^\d+:([^:#|]+):\d+$/.exec(seg); if (pm) { author = pm[1]; break; } }
          lvl = toLevel(m, author, officialName(parseInt(m["12"]) || 0));
        }
        try { const t = await post("getGJDailyLevel.php", `type=${k}&secret=${SECRET}`); const n = parseInt((t || "").split("|")[0]); if (n > 0) lvl.dailyNumber = n; } catch (e) {}
        window.__special = lvl; window.__onlineList = [lvl]; send(wd, "wd_on_special", JSON.stringify(lvl));
      } catch (e) { fail(wd, e); }
    };
  }
  E[`Pop("OnSpecialPlay", 0)`] = (ev, wd) => { const l = window.__special; if (l) { window.__onlineList = [l]; try { localStorage.setItem("viewedLevel_" + l.id, "1"); } catch (e) {} saveLevel(l); songInfo(wd, l); } };

  // `_saveOnlineLevelToSavedList`
  const saveLevel = (l, extra = {}) => {
    try {
      const key = "gd_saved_online_levels", all = JSON.parse(localStorage.getItem(key) || "[]");
      if (all.some((s) => String(s.id).replace(/^online_/, "") === String(l.id))) return;
      all.unshift({ id: String(l.id), name: l.name || "Online Level", author: l.author || "Unknown", customSongID: l.customSong || null, songName: l.songName || "Unknown", difficulty: l.difficulty || 0, downloads: l.downloads || 0, likes: l.likes || 0, stars: l.stars || 0, coins: l.coins || 0, length: l.length || 0, featured: !!l.featured, epic: l.epic || 0, savedAt: Date.now(), levelString: null, ...extra });
      localStorage.setItem(key, JSON.stringify(all));
    } catch (e) {}
  };

  // the real song name and artist of a custom song (`getGJSongInfo.php`), shown in the play menu's song box
  const songInfo = async (wd, l) => {
    if (!l || !l.customSong || l.customSong === "0") return;
    try {
      const t = await post("getGJSongInfo.php", `songID=${l.customSong}&secret=${SECRET}`);
      if (!t || t === "-1") return;
      const m = kv(t, "~|~");
      send2(wd, "wd_on_song", (m["2"] || "").trim(), (m["4"] || "Unknown").trim());
    } catch (e) { console.warn("song info", e); }
  };
  const send2 = (wd, fn, a, b) => { const [p, l] = wd.put(a), [q, m] = wd.put(b); wd.w[fn](p, l, q, m); };
  E['Pop("OnDeleteSaved", 0)'] = () => { try { localStorage.removeItem("gd_saved_online_levels"); } catch (e) {} };

  // opening the play menu marks the level as viewed and saves it (`_openPlayMenu`)
  for (let i = 0; i < 40; i++) {
    E[`Pop("OnView", ${i})`] = (ev, wd) => { const l = window.__onlineList && window.__onlineList[i]; if (l) { try { localStorage.setItem("viewedLevel_" + l.id, "1"); } catch (e) {} saveLevel(l); songInfo(wd, l); } };
    E[`Pop("OnPlay", ${i})`] = (ev, wd) => { const l = window.__onlineList && window.__onlineList[i]; if (l) playOnline(wd, l); };
    E[`Pop("OnCopy", ${i})`] = (ev, wd) => { const l = window.__onlineList && window.__onlineList[i]; if (l) copyToEditor(wd, l); };
  }

  const download = async (l) => {
    const raw = await post("downloadGJLevel22.php", `levelID=${l.id}&secret=${SECRET}`);
    if (!raw || raw === "-1" || !raw.includes(":")) throw new Error("level not found");
    const m = kv(raw.split("#")[0], ":");
    if (!m["4"]) throw new Error("no level data");
    return m;
  };
  // `_duplicateOnlineLevelToEditor`: a copy in My Levels
  const copyToEditor = async (wd, l) => {
    try {
      const m = await download(l), ED = window.WD_ED; if (!ED) return;
      const levels = ED.read(), id = ED.nextLocalId ? ED.nextLocalId(levels) : "local_" + Date.now();
      levels.push({ createdId: id, levelName: l.name, description: l.desc || "", song: l.songName || "", levelLength: l.length || 0, status: "Copy", version: 1, levelString: m["4"], lastModified: Date.now() });
      ED.write(levels);
    } catch (e) { fail(wd, e); }
  };

  // downloads a level and starts it: the level string is base64 + gzip (the core reads it as it is)
  const playOnline = async (wd, l) => {
    if (window.__onlineBusy) return; window.__onlineBusy = true;
    try {
      const m = await download(l), text = m["4"];
      const official = parseInt(m["12"]) || 0, custom = (m["35"] || "").trim();
      saveLevel(l, { levelString: text, officialSong: official });
      let songData = null;
      try {
        if (custom && custom !== "0") {
          const info = await post("getGJSongInfo.php", `songID=${custom}&secret=${SECRET}`);
          const url = decodeURIComponent((info.split("~|~10~|~")[1] || "").split("~|~")[0]);
          if (url) { const r = await fetch(PROXY + url); if (r.ok) songData = await r.arrayBuffer(); }
        } else { const e = allLevels()[official]; if (e) { const r = await fetch(`../ref/assets/music/${e[4] || String(e[1]).replaceAll(" ", "")}.mp3`); if (r.ok) songData = await r.arrayBuffer(); } }
      } catch (e) { console.warn("song", e); }
      await window.__playOnline({ id: String(l.id), name: l.name, text, songData, offset: parseFloat(m["45"] || "0") || 0 });
    } catch (e) { fail(wd, e); } finally { window.__onlineBusy = false; }
  };
})();
