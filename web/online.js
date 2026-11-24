// Online levels: the Geometry Dash server calls behind the Search, Featured, Saved and Daily / Weekly / Event screens (game-scene.js
// `_openOnlineLevelsScene`, `_fetchSpecialLevel`, the play-online-level code). The screens themselves are drawn by online.rs; this file
// asks the servers (through the same proxies the original uses), turns the answers into JSON for it, and starts a downloaded level.
(() => {
  const E = window.WD_EVENTS, SECRET = "Wmfd2893gb7";
  const HOSTS = [
    "https://gd-proxy.webdashers.workers.dev", "https://cold-grass-d871.gn-math-business.workers.dev", "https://webd.ktrio839.workers.dev",
    "https://webdashers.webdashersdevelopement.workers.dev", "https://fallbackgdproxy.webdashers.workers.dev", "https://fallback2.webdashers.workers.dev",
  ];
  let hostIdx = 0;
  for (const [n, f] of [["GJ_button02", "GJ_button_02"], ["loadingCircle", "loadingCircle"], ["GJ_square01", "GJ_square01"]]) window.WD_ASSETS.push([n, `../ref/assets/sprites/${f}.png`]);

  // one form POST, trying the next proxy when one is over its limit or down (index.html does the same around `fetch`)
  const post = async (path, body) => {
    let last = null;
    for (let i = 0; i < HOSTS.length; i++) {
      const k = (hostIdx + i) % HOSTS.length, ctrl = new AbortController(), timer = setTimeout(() => ctrl.abort(), 12000);
      try {
        const r = await fetch(`${HOSTS[k]}/${path}`, { method: "POST", headers: { "Content-Type": "application/x-www-form-urlencoded" }, body, signal: ctrl.signal });
        if (!r.ok) { last = new Error("HTTP " + r.status); continue; }
        hostIdx = k; return await r.text();
      } catch (e) { last = e; } finally { clearTimeout(timer); }
    }
    throw last || new Error("offline");
  };
  const kv = (str, sep) => { const p = str.split(sep), o = {}; for (let i = 0; i + 1 < p.length; i += 2) o[p[i]] = p[i + 1]; return o; };
  const b64 = (s) => { try { return decodeURIComponent(escape(atob((s || "").replace(/-/g, "+").replace(/_/g, "/")))); } catch (e) { try { return atob(s || ""); } catch (e2) { return ""; } } };

  // difficulty face (diffIcon_NN): 0 N/A .. 5 insane, 6-10 the demons, auto
  const faceOf = (m) => {
    let idx;
    if (parseInt(m["25"]) === 1) return "diffIcon_auto_btn_001.png";
    if (parseInt(m["17"]) === 1) {
      const d9 = parseInt(m["9"]), d43 = parseInt(m["43"]);
      if (!isNaN(d9) && d9 >= 1 && d9 <= 5) idx = [7, 8, 6, 9, 10][d9 - 1] ?? 8;
      else if (!isNaN(d43)) { const mp = { 3: 7, 4: 8, 0: 6, 5: 9, 6: 10 }; idx = mp.hasOwnProperty(d43) ? mp[d43] : 8; } else idx = 8;
    } else idx = Math.min(5, Math.max(0, Math.round((parseInt(m["9"]) || 0) / 10)));
    return `diffIcon_${String(idx).padStart(2, "0")}_btn_001.png`;
  };
  const officialName = (levels, n) => (levels && levels[n]) ? levels[n][1] : "Unknown";
  const toLevel = (m, author, songName) => ({
    id: m["1"] || "", name: (m["2"] || "Unknown").trim(), author: author || "Unknown", desc: b64(m["3"]), face: faceOf(m),
    stars: parseInt(m["18"]) || 0, downloads: parseInt(m["10"]) || 0, likes: parseInt(m["14"]) || 0, length: parseInt(m["15"]) || 0, song: songName || "",
    featured: (parseInt(m["19"]) || 0) > 0 ? 1 : 0, epic: parseInt(m["42"]) || 0, coins: parseInt(m["37"]) || 0, customSong: (m["35"] || "").trim(),
  });
  // `getGJLevels21.php`: levels # users # songs # page info # hash
  const parseList = (raw, allLevels) => {
    const sec = raw.split("#"), players = {}, songs = {};
    for (const ps of (sec[1] || "").split("|").filter(Boolean)) { const p = ps.split(":"); if (p.length >= 2) players[p[0]] = p[1]; }
    for (const ss of (sec[2] || "").split("~:~").filter(Boolean)) { const sm = kv(ss, "~|~"); if (sm["1"]) songs[sm["1"]] = sm["2"] || ""; }
    const info = (sec[3] || "0:0:10").split(":"), total = parseInt(info[0]) || 0, size = parseInt(info[2]) || 10;
    const levels = (sec[0] || "").split("|").filter(Boolean).map((s) => {
      const m = kv(s, ":"), c = (m["35"] || "").trim(), custom = c && c !== "0";
      return toLevel(m, players[m["6"]], custom ? (songs[c] || "Song #" + c) : officialName(allLevels, parseInt(m["12"]) || 0));
    });
    return { levels, pages: Math.max(1, Math.ceil(total / size)) };
  };

  const send = (wd, fn, s, ...rest) => { const [p, l] = wd.put(s); wd.w[fn](p, l, ...rest); };
  const allLevels = () => (window.__allLevels || []);
  const fail = (wd, e) => { console.warn("online", e); send(wd, "wd_on_error", navigator.onLine === false ? "You are offline" : "Could not reach the servers"); };

  const load = async (wd) => {
    const kind = wd.w.wd_on_kind(), page = wd.w.wd_on_page();
    try {
      if (kind === 2) { // saved levels live in this browser
        const saved = JSON.parse(localStorage.getItem("gd_saved_online_levels") || "[]");
        const list = saved.map((s) => ({ id: String(s.id).replace(/^online_/, ""), name: s.name, author: s.author, desc: "", face: `diffIcon_${String(Math.min(10, s.difficulty || 0)).padStart(2, "0")}_btn_001.png`, stars: s.stars || 0, downloads: s.downloads || 0, likes: s.likes || 0, length: s.length || 0, song: s.songName || "", featured: s.featured ? 1 : 0, epic: s.epic || 0, coins: s.coins || 0 }));
        window.__onlineList = list; send(wd, "wd_on_results", JSON.stringify(list), 1); return;
      }
      const q = wd.text(wd.w.wd_on_query()).trim();
      const body = kind === 1 ? `secret=${SECRET}&page=${page}&type=6` : `secret=${SECRET}&page=${page}&type=0&str=${encodeURIComponent(q)}`;
      const raw = await post("getGJLevels21.php", body);
      if (!raw || raw === "-1") { window.__onlineList = []; send(wd, "wd_on_results", "[]", 1); return; }
      const { levels, pages } = parseList(raw, allLevels());
      window.__onlineList = levels;
      send(wd, "wd_on_results", JSON.stringify(levels), kind === 1 ? 10 : pages);
    } catch (e) { fail(wd, e); }
  };
  E['Pop("OnSearch", 0)'] = (ev, wd) => load(wd);
  E['Pop("OnFeatured", 0)'] = (ev, wd) => load(wd);
  E['Pop("OnSaved", 0)'] = (ev, wd) => load(wd);
  E['Pop("OnPage", 0)'] = (ev, wd) => load(wd);

  // daily / weekly / event (`_fetchSpecialLevel`)
  const SPECIAL = [{ dlId: -1, listType: 21, numType: 0 }, { dlId: -2, listType: 22, numType: 1 }, { dlId: -3, listType: 23, numType: 2 }];
  for (let k = 0; k < 3; k++) {
    E[`Pop("OnDaily", ${k})`] = async (ev, wd) => {
      const C = SPECIAL[k];
      try {
        let lvl = null;
        try {
          const raw = await post("getGJLevels21.php", `secret=${SECRET}&page=0&type=${C.listType}`);
          if (raw && raw !== "-1") lvl = parseList(raw, allLevels()).levels[0] || null;
        } catch (e) { console.warn("list request failed", e); }
        if (!lvl) {
          const raw = await post("downloadGJLevel22.php", `levelID=${C.dlId}&secret=${SECRET}`);
          if (!raw || raw === "-1" || !raw.includes(":")) throw new Error("no level");
          const segs = raw.split("#"), m = kv(segs[0], ":");
          let author = "Unknown";
          for (const seg of segs.slice(1)) { const pm = /^\d+:([^:#|]+):\d+$/.exec(seg); if (pm) { author = pm[1]; break; } }
          lvl = toLevel(m, author, officialName(allLevels(), parseInt(m["12"]) || 0));
        }
        window.__special = lvl;
        send(wd, "wd_on_special", JSON.stringify(lvl));
      } catch (e) { fail(wd, e); }
    };
    E[`Pop("OnSpecialPlay", 0)`] = (ev, wd) => { if (window.__special) playOnline(wd, window.__special); };
  }

  // keeps a level in the saved list (`_saveOnlineLevelToSavedList`)
  const saveLevel = (l, extra = {}) => {
    try {
      const key = "gd_saved_online_levels", all = JSON.parse(localStorage.getItem(key) || "[]");
      if (all.some((s) => String(s.id).replace(/^online_/, "") === String(l.id))) return;
      all.unshift({ id: String(l.id), name: l.name || "Online Level", author: l.author || "Unknown", songName: l.song || "Unknown", difficulty: parseInt((/diffIcon_(\d+)/.exec(l.face) || [])[1] || 0, 10), downloads: l.downloads || 0, likes: l.likes || 0, stars: l.stars || 0, coins: l.coins || 0, length: l.length || 0, featured: !!l.featured, epic: l.epic || 0, savedAt: Date.now(), levelString: null, ...extra });
      localStorage.setItem(key, JSON.stringify(all));
    } catch (e) {}
  };

  // downloads a level and starts it: `downloadGJLevel22.php`, the level string is base64 + gzip (the core reads it as it is)
  const playOnline = async (wd, l) => {
    if (window.__onlineBusy) return; window.__onlineBusy = true;
    try {
      const raw = await post("downloadGJLevel22.php", `levelID=${l.id}&secret=${SECRET}`);
      if (!raw || raw === "-1" || !raw.includes(":")) throw new Error("level not found");
      const m = kv(raw.split("#")[0], ":"), text = m["4"];
      if (!text) throw new Error("no level data");
      const official = parseInt(m["12"]) || 0, custom = (m["35"] || "").trim();
      saveLevel(l, { levelString: text, officialSong: official });
      let songData = null;
      try {
        if (custom && custom !== "0") {
          const info = await post("getGJSongInfo.php", `songID=${custom}&secret=${SECRET}`);
          const url = decodeURIComponent((info.split("~|~10~|~")[1] || "").split("~|~")[0]);
          if (url) { const r = await fetch(HOSTS[hostIdx] + "/audio-proxy?url=" + encodeURIComponent(url)); if (r.ok) songData = await r.arrayBuffer(); }
        } else { const e = allLevels()[official]; if (e) { const r = await fetch(`../ref/assets/music/${e[4] || String(e[1]).replaceAll(" ", "")}.mp3`); if (r.ok) songData = await r.arrayBuffer(); } }
      } catch (e) { console.warn("song", e); }
      await window.__playOnline({ id: String(l.id), name: l.name, text, songData, offset: parseFloat(m["45"] || "0") || 0 });
    } catch (e) { fail(wd, e); } finally { window.__onlineBusy = false; }
  };
  for (let i = 0; i < 40; i++) { // the buttons of the list and the level view carry the row index
    E[`Pop("OnPlay", ${i})`] = (ev, wd) => { const l = window.__onlineList && window.__onlineList[i]; if (l) playOnline(wd, l); };
    E[`Pop("OnSave", ${i})`] = (ev, wd) => { const l = window.__onlineList && window.__onlineList[i]; if (l) saveLevel(l); };
  }
})();
