// Differential test: PlayerObject.updateJump (JS original) vs the Rust port, every mode except spider.
import { load } from "./physics-harness.mjs";

const { layout, runJs, runRust } = await load();
let seed = Number(process.argv[2] || 12345);
const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296);
const pick = (a) => a[Math.floor(rnd() * a.length)];
const SP = [9.30222544655, 11.540004, 14.3488938625, 17.3333393414, 21.3333407279, 12.5, 0];
const MODES = ["cube", "cube", "cube", "fly", "wave", "ball", "ufo", "robot"];

const N = Number(process.argv[3] || 300000);
let bad = 0;
for (let t = 0; t < N; t++) {
  const mode = pick(MODES);
  const v = {};
  for (const f of layout) {
    const nm = f.name;
    if (f.type === "bool") v[nm] = rnd() < 0.5 ? 1 : 0;
    else if (f.type === "Option<f64>") v[nm] = rnd() < 0.5 ? NaN : rnd() * 30 - 15;
    else v[nm] = rnd() * 4 - 2;
  }
  v.y_velocity = pick([0, rnd() * 80 - 40, rnd() * 4 - 2, -30, 30, 22.360064, -22.360064, 16, -12.8, 14]);
  v.pending_velocity = rnd() < 0.1 ? rnd() * 20 - 10 : NaN;
  v.is_dashing = rnd() < 0.1 ? 1 : 0;
  v.is_swing = rnd() < 0.1 ? 1 : 0;
  v.slope_contact_frames = Math.floor(rnd() * 100);
  v.prev_slope_angle = pick([0, rnd() * 6 - 3, 0.5, 1e-9]);
  v.rotation = rnd() * 20 - 10;
  v.on_slope_angle = rnd() < 0.5 ? NaN : rnd() * 3 - 1.5;
  v.robot_accel = pick([0, rnd() * 2]);
  v.gd_flip_tick = rnd() < 0.5 ? NaN : Math.floor(rnd() * 1000);
  for (const k of ["gd_last_slope_obj", "gd_last_slope_top", "gd_last_slope_up", "gd_last_slope_y_pos", "active_slope_obj", "prev_slope_obj", "last_land_object", "active_slope_angle", "gd_mode_sig", "gd_mode_tick", "last_collision_world_x", "last_collision_world_y", "slope_entry_x"]) v[k] = NaN;
  v.slope_exit_grace = 0; v["g.slope_exit_cap"] = Infinity; v["g.slope_swing_cap"] = 13;
  for (const m of ["is_flying", "is_wave", "is_ball", "is_ufo", "is_spider", "is_robot"]) v[m] = 0;
  if (mode === "fly") v.is_flying = 1; else if (mode === "wave") v.is_wave = 1; else if (mode === "ball") v.is_ball = 1;
  else if (mode === "ufo") v.is_ufo = 1; else if (mode === "robot") v.is_robot = 1;
  v["g.player_speed"] = pick(SP);
  v["g.slope_jump_boost"] = rnd() < 0.85 ? 1 : 0;
  v["g.slope_ramp_time"] = pick([0.1, 0.05, 0.3]); v["g.slope_ramp_floor"] = pick([0.4, 0.2]);
  v["g.slope_launch_num"] = pick([0.8, 1]); v["g.slope_launch_cap"] = pick([1.1, 2]); v["g.slope_launch_gain"] = pick([1.4, 1]);
  v["e.is_dual"] = rnd() < 0.3 ? 1 : 0; v["e.physics_tick"] = Math.floor(rnd() * 5000); v["e.orb_touch"] = rnd() < 0.4 ? 1 : 0;
  const dt = pick([0.9 / 240, 0.9 / 960, rnd() * 0.05, 0, 0.0037]);

  const js = runJs(v, (o) => o.updateJump(dt));
  const rs = runRust(v, 0, dt);
  const diff = Object.keys(js).filter((k) => k !== "__log" && !k.startsWith("e.") && !k.startsWith("g.") && !(Object.is(js[k], rs[k]) || (Number.isNaN(js[k]) && Number.isNaN(rs[k])) || js[k] === rs[k]));
  if (diff.length && bad++ < 5) console.log("MISMATCH", mode, "dt", dt, diff.map((k) => `${k}: rust=${rs[k]} js=${js[k]}`).join("; "), "\n input:", JSON.stringify(v));
}
console.log(`${N} random states across modes: ${bad ? bad + " MISMATCHED" : "all identical"}`);
process.exit(bad ? 1 : 0);
