// Host side of popups group B (StartPos, Keybinds, Vault, level Stats, Stats screen).
// Register extra standalone textures:  WD_ASSETS.push([name, "../ref/assets/sprites/<file>.png"]);
// Handle events from the wasm UI (names are Debug strings, e.g. 'Popup("Songs")' or 'Pop("Songs", 2)'):
//   WD_EVENTS[eventName] = (ev, wd) => { ... };   wd = { w (wasm exports), put(str)->[ptr,len], text(n)->string }
(() => {
  for (const [name, file] of [["GJ_button04", "GJ_button_04"], ["GJ_button06", "GJ_button_06"], ["GJ_square02", "GJ_square02"]]) {
    WD_ASSETS.push([name, `../ref/assets/sprites/${file}.png`]);
  }
  let W = null; // latest wd (set by the first event; the key listener below needs it)
  const lsGet = (k, d) => { try { const v = localStorage.getItem(k); return v === null ? d : v; } catch (e) { return d; } };
  const lsSet = (k, v) => { try { localStorage.setItem(k, v); } catch (e) {} };
  const readSettings = () => { try { return JSON.parse(lsGet("gd_settings", "{}")) || {}; } catch (e) { return {}; } };
  let seeded = false;
  const seed = (wd) => { if (!seeded) { seeded = true; wd.w.wd_pb_seed(Math.random()); } };

  // ---- StartPos: gd_settings.startPosSwitcher / smartStartPos / startPosOpacity
  const loadStartPos = (wd) => {
    const s = readSettings();
    const op = Math.min(1, Math.max(0.05, Number(s.startPosOpacity) || 1));
    wd.w.wd_pb_startpos(s.startPosSwitcher ? 1 : 0, s.smartStartPos ? 1 : 0, op);
    wd.w.wd_pb_refresh();
  };
  const saveStartPos = (wd) => {
    const s = readSettings();
    s.startPosSwitcher = !!wd.w.wd_pb_startpos_get(0);
    s.smartStartPos = !!wd.w.wd_pb_startpos_get(1);
    s.startPosOpacity = wd.w.wd_pb_startpos_get(2);
    lsSet("gd_settings", JSON.stringify(s));
  };

  // ---- Keybinds: gd_keybinds = { id: [KeyboardEvent.code, ...] } (old Phaser key names are converted)
  const legacy = (name) => {
    const direct = { SPACE: "Space", UP: "ArrowUp", DOWN: "ArrowDown", LEFT: "ArrowLeft", RIGHT: "ArrowRight", ESC: "Escape", SHIFT: "ShiftLeft", CTRL: "ControlLeft", ALT: "AltLeft",
      ENTER: "Enter", TAB: "Tab", BACKSPACE: "Backspace", DELETE: "Delete", INSERT: "Insert", HOME: "Home", END: "End", PAGE_UP: "PageUp", PAGE_DOWN: "PageDown", CAPS_LOCK: "CapsLock",
      PAUSE: "Pause", COMMA: "Comma", PERIOD: "Period", FORWARD_SLASH: "Slash", BACK_SLASH: "Backslash", COLON: "Semicolon", SEMICOLON: "Semicolon", QUOTES: "Quote",
      OPEN_BRACKET: "BracketLeft", CLOSED_BRACKET: "BracketRight", MINUS: "Minus", PLUS: "Equal", BACKTICK: "Backquote" };
    if (direct[name]) return direct[name];
    const d = ["ZERO", "ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN", "EIGHT", "NINE"].indexOf(name);
    if (d !== -1) return "Digit" + d;
    if (/^[A-Z]$/.test(name)) return "Key" + name;
    return name;
  };
  const loadKeybinds = (wd) => {
    let saved = null;
    try { saved = JSON.parse(lsGet("gd_keybinds", "null")); } catch (e) {}
    if (saved) {
      for (const id of Object.keys(saved)) {
        if (!Array.isArray(saved[id])) continue;
        const keys = [...new Set(saved[id].map(legacy))].slice(0, 4).join(",");
        const [ip, il] = wd.put(id), [kp, kl] = wd.put(keys);
        wd.w.wd_pb_kb_set(ip, il, kp, kl);
      }
    }
    wd.w.wd_pb_refresh();
  };
  const saveKeybinds = (wd) => lsSet("gd_keybinds", wd.text(wd.w.wd_pb_kb_json()));

  // ---- Stats screen numbers (what the original keeps in localStorage)
  const loadStats = (wd) => {
    const n = (k) => parseInt(lsGet(k, "0"), 10) || 0;
    wd.w.wd_pb_stats(n("gd_totalJumps"), 1, n("gd_completedLevels"), n("gd_totalDeaths"), n("gd_totalsecretcoins"), n("gd_totalusercoins"));
  };

  // ---- Vault music (secretLoop), at the user's music volume * 0.8
  let vaultAudio = null;
  const vaultMusic = (on) => {
    if (vaultAudio) { try { vaultAudio.pause(); } catch (e) {} vaultAudio = null; }
    if (!on) return;
    try {
      vaultAudio = new Audio("../ref/assets/music/secretLoop.mp3");
      vaultAudio.loop = true;
      vaultAudio.volume = Math.max(0, Math.min(1, (parseFloat(lsGet("userMusicVol", "1")) || 0) * 0.8));
      vaultAudio.play().catch(() => {});
    } catch (e) {}
  };

  const E = window.WD_EVENTS;
  const wrap = (fn) => (ev, wd) => { W = wd; seed(wd); fn(wd, ev); };
  E['Popup("StartPos")'] = wrap(loadStartPos);
  E['Popup("Keys")'] = wrap(loadKeybinds);
  E["Vault"] = wrap(() => {});
  E["Stats"] = wrap(loadStats);
  E['Pop("sp_save", 0)'] = wrap(saveStartPos);
  E['Pop("kb_save", 0)'] = wrap(saveKeybinds);
  E['Pop("va_music", 1)'] = wrap(() => vaultMusic(true));
  E['Pop("va_music", 0)'] = wrap(() => vaultMusic(false));

  // ---- keys: capture for keybind editing, search box and vault code box
  addEventListener("keydown", (e) => {
    if (!W) return;
    const [cp, cl] = W.put(e.code || ""), [kp, kl] = W.put(e.key || "");
    const used = W.w.wd_pb_key(cp, cl, kp, kl, (e.ctrlKey || e.metaKey) ? 1 : 0);
    if (!used) return;
    e.preventDefault(); e.stopImmediatePropagation();
    if (e.ctrlKey || e.metaKey) {
      const k = (e.key || "").toLowerCase();
      if (k === "v" && navigator.clipboard) navigator.clipboard.readText().then((t) => { const [p, l] = W.put(t); W.w.wd_pb_paste(p, l); }).catch(() => {});
      else if (k === "c" && navigator.clipboard) navigator.clipboard.writeText(W.text(W.w.wd_pb_copy_text())).catch(() => {});
    }
  }, true);
})();
