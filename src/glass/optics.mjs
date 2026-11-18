// Convex bezel model informed by QuickLiquid's MIT-licensed physics notes.
// See docs/licenses/QuickLiquid.txt. Only the inward rim refracts; the center is neutral.
export function lensProfile(s, thickness, bezel, ior = 1.5) {
  if (s <= 0 || s >= 1 || thickness <= 0) return 0;
  const arc = Math.sqrt(s * (2 - s));
  const slope = thickness / bezel * (1 - s) / arc;
  const ci = 1 / Math.hypot(1, slope), si = slope * ci;
  const eta = 1 / ior, ct = Math.sqrt(1 - eta * eta * si * si);
  const tangent = (eta * ci - ct) * si;
  const vertical = -eta + (eta * ci - ct) * ci;
  return thickness * arc * Math.abs(tangent / vertical);
}

export function roundedDistance(x, y, width, height, radius) {
  const qx = Math.abs(x - width / 2) - (width / 2 - radius);
  const qy = Math.abs(y - height / 2) - (height / 2 - radius);
  const ax = Math.max(qx, 0), ay = Math.max(qy, 0), length = Math.hypot(ax, ay);
  return {
    distance: length + Math.min(Math.max(qx, qy), 0) - radius,
    nx: Math.sign(x - width / 2) * (length ? ax / length : qx > qy ? 1 : 0),
    ny: Math.sign(y - height / 2) * (length ? ay / length : qy >= qx ? 1 : 0),
  };
}

export function rasterizeLens({width, height, radius = 32, bezel = 26, thickness = 18, cap = 384}) {
  const ratio = Math.min(2, cap / Math.max(width, height));
  const w = Math.max(2, Math.round(width * ratio)), h = Math.max(2, Math.round(height * ratio));
  radius = Math.min(radius, width / 2, height / 2);
  bezel = Math.max(1, Math.min(bezel, width / 2, height / 2));
  const lut = Float32Array.from({length: 513}, (_, i) => lensProfile(i / 512, thickness, bezel));
  const peak = Math.max(...lut), data = new Uint8ClampedArray(w * h * 4);
  // Neutral 8-bit channels: the residual half-LSB is <= 0.16px at the maximum strength.
  for (let i = 0; i < data.length; i += 4) { data[i] = data[i + 1] = data[i + 2] = 128; data[i + 3] = 255; }
  let computed = 0;
  const band = Math.ceil(Math.max(radius, bezel) * ratio) + 1;
  for (let y = 0; y < Math.ceil(h / 2); y++) {
    for (let x = 0; x < Math.ceil(w / 2); x++) {
      // Skip the flat interior without evaluating its distance field.
      if (y >= band && x >= band) break;
      const {distance, nx, ny} = roundedDistance((x + .5) * width / w, (y + .5) * height / h, width, height, radius);
      if (distance >= 0 || distance <= -bezel || !peak) continue;
      const position = -distance / bezel * 512, index = Math.floor(position), fraction = position - index;
      const displacement = (lut[index] * (1 - fraction) + lut[index + 1] * fraction) / peak;
      const dx = nx * displacement * 127.5, dy = ny * displacement * 127.5;
      const a = (y * w + x) * 4, b = (y * w + w - x - 1) * 4;
      const c = ((h - y - 1) * w + x) * 4, d = ((h - y - 1) * w + w - x - 1) * 4;
      data[a] = data[c] = 127.5 - dx; data[b] = data[d] = 127.5 + dx;
      data[a + 1] = data[b + 1] = 127.5 - dy; data[c + 1] = data[d + 1] = 127.5 + dy;
      computed++;
    }
  }
  return {width:w, height:h, data, computed, peak};
}

export function relativeLuminance(r, g, b) {
  const linear = value => { const c = value / 255; return c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4; };
  return .2126 * linear(r) + .7152 * linear(g) + .0722 * linear(b);
}

// Bright backgrounds need a stronger backing. Never weaken the user's contrast choice.
export function adaptiveShade(luminance, configured) {
  return Math.min(.75, Math.max(configured, .3 + .4 * Math.max(0, Math.min(1, luminance))));
}
