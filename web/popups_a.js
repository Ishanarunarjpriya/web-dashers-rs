// Host side of popups group A (How To Play, Newgrounds, Credits, News, Help, Songs).
for (const n of ["tutorial_01", "tutorial_02", "tutorial_03", "tutorial_04", "tutorial_05", "loadingCircle", "Phaserlogo", "GJ_square02", "GJ_button04"]) {
  WD_ASSETS.push([n, `../ref/assets/sprites/${n}.png`]);
}
// GJ_button04's file is GJ_button_04.png
WD_ASSETS.splice(WD_ASSETS.findIndex((a) => a[0] === "GJ_button04"), 1, ["GJ_button04", "../ref/assets/sprites/GJ_button_04.png"]);
// the Help popup shows the saved Low Detail Mode checkbox: load it before the (delayed) open
WD_EVENTS['Popup("Help")'] = (ev, wd) => {
  let v = 0;
  try { v = JSON.parse(localStorage.getItem("gd_settings") || "{}").enableLDM ? 1 : 0; } catch (e) {}
  const [p, n] = wd.put("enableLDM"); wd.w.wd_ui_opt_set(p, n, v);
};
