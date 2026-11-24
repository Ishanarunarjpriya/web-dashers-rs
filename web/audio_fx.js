// Sound effects and menu music (audio.js `playEffect`, `startMenuMusic`, `stopMusic`) on top of the page's AudioContext.
const SFX = ["explode_11", "endStart_02", "gold02", "playSound_01", "quitSound_01", "highscoreGet02", "magicExplosion"];
const get = (k, d) => { try { const v = localStorage.getItem(k); return v === null ? d : v; } catch (e) { return d; } };

export class AudioFx {
  constructor() { this.ac = null; this.musicGain = null; this.sfx = {}; this.menuBuf = null; this.menuSrc = null; this.wantMenu = false; this.loading = false; }

  /** Called once the AudioContext exists; `musicGain` is the node the music plays through (so the meter sees it). */
  attach(ac, musicGain, sink) {
    this.ac = ac; this.musicGain = musicGain; this.sink = sink || ac.destination; // the original's analyser listens to the whole sound manager, effects included
    if (this.loading) return;
    this.loading = true;
    const dec = async (url) => { try { const r = await fetch(url); if (!r.ok) return null; return await ac.decodeAudioData(await r.arrayBuffer()); } catch (e) { return null; } };
    for (const n of SFX) dec(`../ref/assets/sfx/${n}.ogg`).then((b) => { this.sfx[n] = b; });
    dec("../ref/assets/music/menuLoop.mp3").then((b) => { this.menuBuf = b; if (this.wantMenu) this.startMenuMusic(); });
  }

  userSfx() { const v = Number(get("userSfxVol", 1)); return Number.isFinite(v) ? v : 1; }
  userMusic() { const v = Number(get("userMusicVol", 1)); return Number.isFinite(v) ? v : 1; }
  menuMusicEnabled() { return get("menuMusicEnabled", null) === null ? true : get("menuMusicEnabled", "true") === "true"; }

  /** `AudioManager.playEffect(name, {volume})`: base volume times the user's sfx volume. */
  playEffect(name, opts) {
    const base = opts && "volume" in opts ? Number(opts.volume) : 1;
    this.playRaw(name, Math.max(0, (Number.isFinite(base) ? base : 1) * this.userSfx()));
  }

  /** `scene.sound.play(name, {volume})`: the volume is used as given. */
  playRaw(name, volume) {
    const ac = this.ac, buf = this.sfx[name];
    if (!ac || !buf) return;
    const src = ac.createBufferSource(); src.buffer = buf;
    const g = ac.createGain(); g.gain.value = volume;
    src.connect(g); g.connect(this.sink || ac.destination);
    src.start();
  }

  /** `startMenuMusic`: the loop at the user's music volume. Waits for the context and the file if they are not ready yet. */
  startMenuMusic() {
    this.wantMenu = true;
    if (!this.ac || !this.menuBuf || !this.musicGain) return;
    this.stopMusic(true);
    const src = this.ac.createBufferSource(); src.buffer = this.menuBuf; src.loop = true;
    this.musicGain.gain.value = this.userMusic() * 0.8;
    src.connect(this.musicGain);
    src.start();
    this.menuSrc = src;
  }

  stopMusic(keepWanted) {
    if (!keepWanted) this.wantMenu = false;
    if (this.menuSrc) { try { this.menuSrc.stop(); } catch (e) {} this.menuSrc = null; }
  }

  isPlaying() { return !!this.menuSrc; }

  /** Menu entry (`create`): start the loop if the setting allows it and it is not playing, stop it if it is off. */
  enterMenu() {
    if (this.menuMusicEnabled()) { if (!this.isPlaying()) this.startMenuMusic(); }
    else this.stopMusic();
  }
}
