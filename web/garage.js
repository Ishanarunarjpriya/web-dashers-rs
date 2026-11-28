// Host side of the icon kit (garage.rs): loads icon atlases on request, stores the choices like game-scene.js does
// (localStorage keys iconMainColor, iconSecondaryColor, iconCurrentPlayer/Ship/Ball/Wave/Bird/Spider/Robot, playerName, customCubeIconSrc).
(function () {
  const LS = {
    get(k, d) { try { const v = localStorage.getItem(k); return v === null ? d : v; } catch (e) { return d; } },
    set(k, v) { try { localStorage.setItem(k, v); } catch (e) {} },
    del(k) { try { localStorage.removeItem(k); } catch (e) {} },
  };
  const idlePatterns = {
    robot: [/^Robot_idle_001\.png$/, /^Robot_idle_\d+\.png$/, /^Robot_idle01_\d+\.png$/, /^Robot_idle02_\d+\.png$/, /^Robot_run_001\.png$/, /^Robot_run_\d+\.png$/],
    spider: [/^Spider_idle01_001\.png$/, /^Spider_idle01_\d+\.png$/, /^Spider_idle02_\d+\.png$/, /^Spider_idle_\d+\.png$/, /^Spider_walk_\d+\.png$/],
  };
  const suffix = (s) => parseInt((String(s).match(/_(\d+)\.png$/) || [0, 0])[1], 10) || 0;
  let animsSent = false;
  const sendAnims = async (wd) => {
    if (animsSent) return; animsSent = true;
    for (const [i, kind] of [[0, "robot"], [1, "spider"]]) {
      try {
        const desc = await (await fetch(`../ref/assets/sheets/${kind === "robot" ? "Robot" : "Spider"}_AnimDesc.json`)).json();
        const keys = Object.keys(desc.animationContainer).sort((a, b) => suffix(a) - suffix(b));
        let frame = null;
        for (const re of idlePatterns[kind]) { const m = keys.find((k) => re.test(k)); if (m) { frame = desc.animationContainer[m]; break; } }
        frame = frame || desc.animationContainer[keys[0]];
        const pair = (v, fx, fy) => { const m = String(v ?? "").match(/\{\s*([-+]?\d*\.?\d+)\s*,\s*([-+]?\d*\.?\d+)\s*\}/); return m ? [parseFloat(m[1]), parseFloat(m[2])] : [fx, fy]; };
        const lines = [];
        for (const key of Object.keys(frame)) {
          if (!key.startsWith("sprite_")) continue;
          const sd = frame[key], tag = parseInt(sd.tag || "0", 10) || 0, pos = pair(sd.position, 0, 0), sc = pair(sd.scale, 1, 1), fl = pair(sd.flipped, 0, 0);
          lines.push([tag, sd.texture, pos[0], pos[1], sc[0], sc[1], fl[0] ? 1 : 0, fl[1] ? 1 : 0, parseFloat(sd.zValue || tag || "0") || 0, parseFloat(sd.rotation || "0") || 0].join("|"));
        }
        const [p, l] = wd.put(lines.join("\n")); wd.w.wd_gar_anim(i, p, l);
      } catch (e) { /* the grid falls back to plain frames */ }
    }
  };
  const setKV = (wd, k, v) => { const [kp, kl] = wd.put(k), [vp, vl] = wd.put(String(v)); wd.w.wd_gar_set(kp, kl, vp, vl); };
  const hex6 = (n) => n.toString(16).padStart(6, "0");

  WD_EVENTS["Garage"] = async (ev, wd) => {
    await sendAnims(wd);
    setKV(wd, "iconMainColor", LS.get("iconMainColor", "04FF00"));
    setKV(wd, "iconSecondaryColor", LS.get("iconSecondaryColor", "00FBFF"));
    setKV(wd, "iconCurrentPlayer", LS.get("iconCurrentPlayer", "player_01")); setKV(wd, "iconCurrentShip", LS.get("iconCurrentShip", "ship_01"));
    setKV(wd, "iconCurrentBall", LS.get("iconCurrentBall", "player_ball_01")); setKV(wd, "iconCurrentWave", LS.get("iconCurrentWave", "dart_01"));
    setKV(wd, "iconCurrentBird", LS.get("iconCurrentBird", "bird_01")); setKV(wd, "iconCurrentSpider", LS.get("iconCurrentSpider", "spider_01"));
    setKV(wd, "iconCurrentRobot", LS.get("iconCurrentRobot", "robot_01")); setKV(wd, "iconCurrentSwing", LS.get("iconCurrentSwing", "swing_01"));
    setKV(wd, "iconGlowColor", LS.get("iconGlowColor", "7DFF00")); setKV(wd, "iconGlowEnabled", LS.get("iconGlowEnabled", "false"));
    setKV(wd, "iconCurrentTrail", LS.get("iconCurrentTrail", "1")); setKV(wd, "iconCurrentShipFire", LS.get("iconCurrentShipFire", "1")); setKV(wd, "iconCurrentDeathEffect", LS.get("iconCurrentDeathEffect", "1"));
    setKV(wd, "iconExplode", LS.get("iconExplode", "true")); setKV(wd, "iconSlowRobotRun", LS.get("iconSlowRobotRun", "false")); setKV(wd, "iconFastRobotRun", LS.get("iconFastRobotRun", "false")); setKV(wd, "iconFastSpiderRun", LS.get("iconFastSpiderRun", "false"));
    setKV(wd, "secretCoins", parseInt(LS.get("gd_totalsecretcoins", "0"), 10) || 0); setKV(wd, "userCoins", parseInt(LS.get("gd_totalusercoins", "0"), 10) || 0);
    let name = String(LS.get("playerName", "Player")).replace(/\r|\n/g, "").slice(0, 20); if (!name.trim()) name = "Player";
    setKV(wd, "playerName", name);
    setKV(wd, "customCubeIcon", window.__pv && window.__pv.hasCustomCube() ? "1" : "0");
    wd.w.wd_gar_open();
  };
  WD_EVENTS['Pop("GarageNeed", 0)'] = async (ev, wd) => {
    const n = wd.w.wd_gar_need(); const need = n ? JSON.parse(wd.text(n)) : [];
    const pv = window.__pv;
    await Promise.all(need.map(async (b) => { const ok = await pv.loadIconAtlas(b); const [p, l] = wd.put(b); wd.w.wd_gar_ready(p, l, ok ? 1 : 0); }));
    wd.w.wd_gar_rebuild();
  };
  WD_EVENTS['Pop("GarageSave", 0)'] = (ev, wd) => {
    const n = wd.w.wd_gar_changes(); const lines = n ? wd.text(n).split("\n") : [];
    for (const line of lines) {
      const i = line.indexOf("="); if (i < 0) continue;
      const k = line.slice(0, i), v = line.slice(i + 1);
      if (k === "customCubeIconSrc") { if (!v) { LS.del("customCubeIconSrc"); window.__pv.setCustomCube(null); } else LS.set(k, v); }
      else LS.set(k, v);
    }
    window.__pv.reloadCfg();
  };
  WD_EVENTS['Pop("GaragePickFile", 0)'] = (ev, wd) => {
    const input = document.createElement("input"); input.type = "file"; input.accept = "image/*"; input.style.display = "none"; document.body.appendChild(input);
    input.addEventListener("change", () => {
      const file = input.files && input.files[0]; input.remove();
      if (!file) return;
      if (file.size > 8 * 1024 * 1024) { alert("That image is too large. Please pick one under 8MB."); return; }
      const reader = new FileReader();
      reader.onload = () => {
        const img = new Image();
        img.onload = () => {
          const size = 256, cv = document.createElement("canvas"); cv.width = cv.height = size;
          const g = cv.getContext("2d"), sc = Math.max(size / img.width, size / img.height), dw = img.width * sc, dh = img.height * sc;
          g.drawImage(img, (size - dw) / 2, (size - dh) / 2, dw, dh);
          const src = cv.toDataURL("image/png");
          window.__pv.setCustomCube(src).then(() => {
            LS.set("customCubeIconSrc", src);
            window.__standalone["custom_cube_icon_hd"] = cv;
            const [p, l] = wd.put("/custom_cube_icon_hd"); wd.w.wd_ui_size(p, l, size, size);
            setKV(wd, "customCubeIcon", "1"); wd.w.wd_gar_rebuild();
          }).catch(() => alert("Could not import that image. Please try another file."));
        };
        img.onerror = () => alert("Could not read that image. Please try another file.");
        img.src = reader.result;
      };
      reader.onerror = () => alert("Could not read that file. Please try another one.");
      reader.readAsDataURL(file);
    });
    input.click();
  };
  // an imported cube from an earlier visit
  window.__garageInit = async (wd) => {
    const src = LS.get("customCubeIconSrc", null);
    if (!src) return;
    try {
      await window.__pv.setCustomCube(src);
      const img = new Image(); img.src = src; await img.decode();
      window.__standalone["custom_cube_icon_hd"] = img;
      const [p, l] = wd.put("/custom_cube_icon_hd"); wd.w.wd_ui_size(p, l, img.width, img.height);
    } catch (e) {}
  };
})();
