# Web Dashers in Rust

A port of https://github.com/web-dashers/web-dashers.github.io (copy in `ref/`, no license upstream).
Every ported piece is checked against the original JavaScript with a differential test in `tools/`.

Build (no Xcode license needed, wasm only):

    export RUSTUP_HOME=$PWD/.rustup CARGO_HOME=$PWD/.cargo PATH=$PWD/.cargo/bin:$PATH
    cargo build --release --target wasm32-unknown-unknown
    node tools/compare-parse.mjs
    node tools/compare-jump.mjs
    node tools/compare-colliders.mjs
    node tools/compare-collisions.mjs 1 50000
    node tools/compare-run.mjs 1 20 2000 - start
    node tools/compare-triggers.mjs 1 - 3000          # shipped levels
    FUZZ=20 node tools/compare-triggers.mjs 1 - 3000  # random trigger-heavy levels
    node tools/compare-sprites.mjs                    # which sprites every object creates (visual half of _spawnObject)

## Status
- [x] Level parsing (`level.js` parseLevel/parseObject): identical on all 31 shipped levels (373,603 objects) + edge cases
- [x] Cube jump/gravity update (`updateJump`, cube path + dash + slope boost): identical on 200,000 random states (`node tools/compare-jump.mjs`)
- [x] Jump updates for ship, wave, ball, UFO and robot, plus ground rotation and gravity flip: identical on 700,000 random states
- [x] Spider jump update (teleport search over level objects)
- [x] Player collisions: `checkCollisions` complete (solids, hazards, portals incl. teleport, pads, orbs incl. dash orbs, coins, speed portals, slopes, floor/ceiling, mode enter/exit): identical on 700k+ random cases against real level colliders (`node tools/compare-collisions.mjs <seed> <n> [types] [collide|jump]`)
- [x] Per-frame step loop for one player (`game.rs`: input, quantised substeps, jump, move, collisions, rotation, fly bounds): identical frame by frame on whole runs of real levels (`node tools/compare-run.mjs <seed> <runs> <frames>`)
- [x] Attempt start (player reset, start speed `kA4` and game mode `kA2`, x = -centerX): checked in `compare-run.mjs ... start`
- [x] Restart after death (`Game::restart`: object/trigger reset, the exact fields `PlayerState.reset`/`PlayerObject.reset` clear): `RESTART=1 node tools/compare-run.mjs <seed> 25 4000 - start` keeps running after each death (988 restarts identical)
- [x] Synced with upstream up to 10ba54b (2026-11-25): dash orbs keep spinning on the ground, spider teleports land on slopes + hold after a spider orb, cube/ceiling death rule, rotated solid bounds, dual wave gravity sync, dual ball spawn lock, dual fly bounds, new news/credits text. `ref/assets/scripts` is the new upstream code; assets stay in the old folder layout. Differential runs (single, dual, restarts, start options) identical.
- [x] Dual mode (`pdual.rs`: enable/disable incl. the shared nearby-buffer quirk, gravity/mirror sync, ball overlap, fly bounds), level start options, start positions + trigger fast-forward, death/restart, end-of-level detection, camera (`Game::update`): identical on 1,400+ restarts and ~300k dual-mode frames (`DUAL=1`, `PORTAL=1`, `OPTS=1 RESTART=1`, `CAMERA=1` in `compare-run.mjs`)
- [x] Practice mode checkpoints (`save_checkpoint` / `respawn_from_checkpoint`, incl. dual): `PRACTICE=1 CAMERA=1 node tools/compare-run.mjs <seed> 20 4000 - start` (7,000+ respawns identical)
- [x] Object definitions (generated Rust table from `allObjects.js`, `node tools/dump-objects.mjs`) and collider creation for every object type (`_spawnObject`'s collision half, slopes, portals, pads, orbs, coins, speed portals): identical on all 373,596 objects / 362,595 colliders of the 31 shipped levels (`node tools/compare-colliders.mjs`)
- [x] `_normalizeTeleportPortals` (object 749), collider groups, `Game::from_level` (colliders + trigger lists built from a level string; collider rows identical to the original's)
- [x] Triggers that move things: move, rotate, spawn (incl. random delay), stop, toggle, follow, follow-Y, with touch variants, group transforms, collision-section re-bucketing, and the update order of `game-scene.js` (`triggers.rs`, `node tools/compare-triggers.mjs`). Group centres use sprite anchors that the test supplies; the real sprite spawn (rendering) will provide them.
- [x] Also pickup, count, instant count, collision (+blocks), on-death and touch (1595) triggers: identical on the shipped levels and on 120+ random trigger-heavy levels (`FUZZ=40 node tools/compare-triggers.mjs <seed> - 2500`; levels where the original itself overflows its stack are skipped)
- [x] Visual spawn (`visual.rs`, object definitions in `data/objects_full.json`, atlas frames, glow, overlays, children, saws, coins, teleport exits, colour channels, groups): the sprites of all 373,596 objects of the 31 shipped levels (524,577 sprites, every position/rotation/scale/tint/flag) are identical to the original's (`node tools/compare-sprites.mjs`, which runs the real `_spawnObject` against a recording stand-in for Phaser). Not yet: text objects, BeastAnim objects, trigger editor visuals.
- [x] Colour channels: `ColorManager` (`colors.rs`, `node tools/compare-colors.mjs`: 1M+ reads identical), level header colours/blending/background/ground (`settings.rs`, `node tools/compare-settings.mjs`: all 31 levels), colour and channel-pulse triggers (in the trigger tests, incl. random levels), per-sprite tint rules (`TINT=1 node tools/compare-sprites.mjs`)
- [x] Alpha (group opacity), group pulse, shake (`applyShakeToOffsets`), player visibility, animation (commands are queued for the renderer) and enter-effect triggers: state machines identical in `compare-triggers.mjs` (shipped + random levels). `fast_forward_triggers` and the restart resets include them.
- [x] Ground/ceiling tile layout and the fly-mode floor animation (`ground.rs`, `node tools/compare-ground.mjs`: 600k frames identical); scrolling background and the background/ground art-id helpers (`background.rs`, `node tools/compare-background.mjs`)
- [x] The sprite world (`fx.rs`): what triggers do to the real sprites of a level (group move/rotate transforms incl. the group-centre cache, toggle visibility, alpha, enter/exit effects, invisible-block fade, colour channels with the original's dirty/partial-update cache, group pulses, channel blending, pad emitters as culled group members). `REAL=1 node tools/compare-triggers.mjs` lets the original `LevelObject.loadLevel` run against a recording stand-in for Phaser and compares every sprite's position, rotation, scale, alpha, visibility, tint and blend frame by frame (all shipped levels and random trigger-heavy levels identical).
- [x] First playable front end (`web/`): `python3 -m http.server 8123` in this folder, then open `http://localhost:8123/web/` (query: `?level=21`, `?auto=100&jump=30` for a headless scripted run). The wasm game does everything; `web/main.js` only draws the buffer it hands over (Canvas 2D). Player icon is a placeholder (cube/ball/ship default icons).
- [x] Menus and UI (`ui.rs` + `web/main.js`): main menu (logo, play/garage/creator, bottom buttons, social icons, rainbow level scrolling behind it), level select (cards, difficulty icons, coins, progress bars, arrows, dots), in-game HUD (attempt label, percent, pause button, practice checkpoint buttons), pause overlay, level-complete banner and end screen. Tween engine with Phaser's easings, bouncy buttons, hit-testing, BMFont text. Not checked against the original numerically (the original's loader hangs headless); layouts are ported from `game-scene.js` and compared by eye. Settings screen (sliders, menu-music checkbox, button grid) done, Options popup (3 tabs, toggles, number boxes, info boxes, saved to gd_settings; most options not wired to gameplay yet), How To Play, Newgrounds, Credits, News, Help, Songs, StartPos, Keybinds, Vault, level Stats popup and Stats screen done (popups_a/b.rs + popups_a/b.js, scroll areas with clipping). Still missing: creator (level editor) and garage (icon kit) screens, card drag/slide animation, level-select song download, star/coin award animations, slide-in start.
- [ ] Still open: text objects, BeastAnim objects, player drawing/particles/trails, mirror portals, animated sprites, audio, menus, editor, online levels
- [ ] Rendering, audio, UI
- [x] Editor, rest of `level-editor.js` (checked with scripted clicks in headless Chrome, not against the original frame by frame): playtest (play / pause / stop, green path, death crosses), pause menu with saving, level settings (colours, speed, mode, start options, background, ground, song), colour picker, group / layer popup, layer selector, move / spawn / colour trigger popups, start position and text object popups, paged My Levels list. Not done: the colour channel list behind the "+" of the colour trigger popup, the stats screen scrolling, Escape closing popups.
- [ ] Level editor, online levels

## Notes
- `Math.sin/cos/...` are imported from the host JS engine (`jsmath.rs`), because Rust's versions differ from V8's in the last digit and the game is sensitive to that.
- The physics structs are defined with a macro (`slots!`) so the test harness can compare every field automatically.
