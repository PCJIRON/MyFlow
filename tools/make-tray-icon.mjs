// Generates src-tauri/icons/tray32.rgba — a 32x32 straight-alpha RGBA tray
// mark (accent disc + white microphone) with no image-library dependency.
import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const SIZE = 32;
const SS = 4; // supersampling per axis
const ACCENT = [90, 78, 232]; // #5a4ee8 — MyFlow brand blue-violet
const WHITE = [255, 255, 255];

function roundedRect(x, y, rx, ry, w, h, r) {
  const nx = Math.min(Math.max(x, rx + r), rx + w - r);
  const ny = Math.min(Math.max(y, ry + r), ry + h - r);
  return Math.hypot(x - nx, y - ny) <= r;
}

function disc(x, y) {
  return Math.hypot(x - 16, y - 16) <= 15.2;
}

// microphone capsule, pickup arc and stand
const capsule = (x, y) => roundedRect(x, y, 13.2, 7.6, 5.6, 11.0, 2.8);
const arc = (x, y) => {
  const d = Math.hypot(x - 16, y - 13.4);
  return y >= 13.4 && d >= 6.4 && d <= 7.9;
};
const stem = (x, y) => x >= 15.15 && x <= 16.85 && y >= 21.2 && y <= 24.6;

function sample(x, y) {
  if (!disc(x, y)) return [ACCENT, 0];
  if (capsule(x, y) || arc(x, y) || stem(x, y)) return [WHITE, 1];
  return [ACCENT, 1];
}

const out = Buffer.alloc(SIZE * SIZE * 4);
for (let py = 0; py < SIZE; py++) {
  for (let px = 0; px < SIZE; px++) {
    let ar = 0, ag = 0, ab = 0, aa = 0;
    for (let sy = 0; sy < SS; sy++) {
      for (let sx = 0; sx < SS; sx++) {
        const [c, a] = sample(px + (sx + 0.5) / SS, py + (sy + 0.5) / SS);
        if (a <= 0) continue;
        ar += c[0] * a;
        ag += c[1] * a;
        ab += c[2] * a;
        aa += a;
      }
    }
    const i = (py * SIZE + px) * 4;
    if (aa > 0) {
      out[i] = Math.round(ar / aa);
      out[i + 1] = Math.round(ag / aa);
      out[i + 2] = Math.round(ab / aa);
      out[i + 3] = Math.round((aa / (SS * SS)) * 255);
    }
  }
}

const here = dirname(fileURLToPath(import.meta.url));
const target = join(here, "..", "src-tauri", "icons", "tray32.rgba");
writeFileSync(target, out);
console.log(`wrote ${target} (${out.length} bytes)`);
