// Random trigger-heavy levels shared by the differential tests (every ported trigger kind, dense groups).
export function makeFuzz(rnd) {
return function fuzzLevel() {
  const parts = ["kA4,0"];
  const pick = (a) => a[Math.floor(rnd() * a.length)];
  const grp = () => 1 + Math.floor(rnd() * 8);
  const num = (lo, hi) => (lo + rnd() * (hi - lo)).toFixed(3);
  const n = 40 + Math.floor(rnd() * 160);
  for (let i = 0; i < n; i++) {
    const x = Math.floor(rnd() * 3000) * 0.5, y = Math.floor(rnd() * 40) * 0.5;
    const base = ["1", "ID", "2", x, "3", y];
    const kind = rnd();
    if (kind < 0.45) {
      const id = pick([1, 1, 1, 8, 8, 103, 35, 36, 10, 13, 48, 200, 201, 11, 12, 38]);
      const o = ["1", id, "2", x, "3", y];
      const gs = Array.from({ length: Math.floor(rnd() * 3) }, grp);
      if (gs.length) o.push("57", gs.join("."));
      if (rnd() < 0.2) o.push("6", Math.floor(rnd() * 4) * 90);
      parts.push(o.join(","));
      continue;
    }
    const id = pick([901, 901, 901, 1346, 1346, 1268, 1268, 1616, 1049, 1049, 1814, 1347, 1347, 1812, 1595, 1817, 1611, 1811, 1815, 1816, 1816, 1816, 29, 30, 899, 105, 221, 1006, 1006, 1007, 1007, 1007, 1520, 1520, 1612, 1613, 1585, 1585, 23, 24, 25, 26, 27, 28, 55, 56, 57, 58, 59]);
    const o = ["1", id, "2", id === 1816 ? 300 + 15 * Math.floor(rnd() * 20) : x, "3", id === 1816 ? 15 * Math.floor(rnd() * 6) : y];
    const add = (k, v) => o.push(String(k), String(v));
    add(51, grp());
    if (rnd() < 0.4) add(11, 1);
    if (rnd() < 0.3) add(62, 1);
    if (rnd() < 0.3) add(57, Array.from({ length: 1 + Math.floor(rnd() * 2) }, grp).join("."));
    if (rnd() < 0.6) add(10, num(0, 3));
    if (id === 901) { add(28, num(-90, 90)); add(29, num(-90, 90)); add(30, Math.floor(rnd() * 20)); add(85, num(0.5, 4)); if (rnd() < 0.15) add(pick([58, 59, 302, 303]), 1); }
    if (id === 1346) { add(68, num(-360, 360)); add(69, Math.floor(rnd() * 3)); add(70, rnd() < 0.3 ? 1 : 0); add(71, grp()); add(30, Math.floor(rnd() * 20)); add(85, num(0.5, 4)); }
    if (id === 1268) { add(63, num(0, 2)); if (rnd() < 0.3) add(556, num(0, 1)); }
    if (id === 1049) add(56, rnd() < 0.5 ? 1 : 0);
    if (id === 1814) { add(90, num(0, 3)); add(91, num(0, 1)); add(92, num(-30, 30)); add(105, num(0, 5)); }
    if (id === 1347) { add(71, grp()); add(72, num(-2, 2)); add(73, num(-2, 2)); }
    if (id === 29 || id === 30 || id === 899 || id === 105 || id === 221) { add(7, Math.floor(rnd() * 256)); add(8, Math.floor(rnd() * 256)); add(9, Math.floor(rnd() * 256)); add(14, rnd() < 0.3 ? 1 : 0); if (id > 100) { add(23, pick([1, 2, 3, 4, 1002, 1003, 1004, 1009, 8, 20])); add(17, rnd() < 0.3 ? 1 : 0); add(35, num(0, 1)); if (rnd() < 0.3) add(50, pick([1, 2, 3, 4, 8, 20])); if (rnd() < 0.3) add(49, `${Math.floor(rnd() * 360)}a${num(0, 2)}a${num(0, 2)}a0a0`); } }
    if (id === 1007) add(35, num(-0.3, 1.3));
    if (id === 1520) { add(75, num(-5, 120)); add(84, num(0, 0.6)); }
    if (id === 1585) add(76, Math.floor(rnd() * 12));
    if (id === 1006) { add(52, rnd() < 0.7 ? 0 : 1); add(7, Math.floor(rnd() * 256)); add(8, Math.floor(rnd() * 256)); add(9, Math.floor(rnd() * 256)); add(45, num(0, 1)); add(46, num(0, 1)); add(47, num(0, 1)); if (rnd() < 0.3) { add(48, 1); add(49, `${Math.floor(rnd() * 360)}a${num(0, 2)}a${num(0, 2)}a0a0`); } if (rnd() < 0.3) add(50, pick([1, 2, 3, 4, 8, 20])); }
    if (id === 1812) add(56, rnd() < 0.5 ? 1 : 0);
    if (id === 1595) { add(56, Math.floor(rnd() * 3)); add(81, rnd() < 0.3 ? 1 : 0); add(82, rnd() < 0.4 ? 1 : 0); }
    if (id === 1817) { add(80, 1 + Math.floor(rnd() * 4)); add(77, Math.floor(rnd() * 4)); }
    if (id === 1611 || id === 1811) { add(80, 1 + Math.floor(rnd() * 4)); add(77, Math.floor(rnd() * 5)); add(56, rnd() < 0.7 ? 1 : 0); if (id === 1611) add(104, rnd() < 0.3 ? 1 : 0); else add(88, Math.floor(rnd() * 3)); }
    if (id === 1815) { add(80, 1 + Math.floor(rnd() * 3)); add(95, 1 + Math.floor(rnd() * 3)); add(56, rnd() < 0.5 ? 1 : 0); add(93, rnd() < 0.3 ? 1 : 0); }
    if (id === 1816) { add(80, 1 + Math.floor(rnd() * 3)); add(94, rnd() < 0.6 ? 1 : 0); }
    parts.push(o.join(","));
  }
  return parts.join(";");
};
}
