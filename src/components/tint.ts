import { useEffect, useState } from "react";

/** Colore dominante di un'icona (media pesata sui pixel visibili e colorati), come [r, g, b]. Pura, testata. */
export function dominant(px: ArrayLike<number>): [number, number, number] | null {
  let r = 0, g = 0, b = 0, wsum = 0;
  for (let i = 0; i + 3 < px.length; i += 4) {
    const a = px[i + 3] / 255;
    if (a < 0.5) continue;
    const mx = Math.max(px[i], px[i + 1], px[i + 2]);
    const mn = Math.min(px[i], px[i + 1], px[i + 2]);
    // I pixel molto chiari o grigi contano poco: il colore dell'app e' quello vivace.
    const w = a * (0.15 + (mx - mn) / 255) * (mx < 235 || mx - mn > 30 ? 1 : 0.2);
    r += px[i] * w; g += px[i + 1] * w; b += px[i + 2] * w; wsum += w;
  }
  return wsum > 0 ? [r / wsum, g / wsum, b / wsum] : null;
}

/** Due toni (alto, basso) dello sfondo della tessera dal colore dominante. `dark`: toni scuri. Pura, testata. */
export function tones([r, g, b]: [number, number, number], dark: boolean): [string, string] {
  const mix = (k: number, w: number) => `rgb(${[r, g, b].map((c) => Math.round(c * k + 255 * w * (1 - k))).join(",")})`;
  return dark ? [mix(0.55, 0), mix(0.28, 0)] : [mix(0.92, 0.5), mix(0.72, 0.15)];
}

const CACHE = new Map<string, [number, number, number] | null>();

/** Il colore dominante dell'icona `src` (data URL), calcolato una volta sola per icona. */
export function useDominant(src: string | null): [number, number, number] | null {
  const [c, setC] = useState<[number, number, number] | null>(src ? (CACHE.get(src) ?? null) : null);
  useEffect(() => {
    if (!src) return setC(null);
    if (CACHE.has(src)) return setC(CACHE.get(src) ?? null);
    let live = true;
    const img = new Image();
    img.onload = () => {
      const cv = document.createElement("canvas");
      cv.width = cv.height = 24;
      const cx = cv.getContext("2d", { willReadFrequently: true });
      let col: [number, number, number] | null = null;
      if (cx) {
        cx.drawImage(img, 0, 0, 24, 24);
        col = dominant(cx.getImageData(0, 0, 24, 24).data);
      }
      CACHE.set(src, col);
      if (live) setC(col);
    };
    img.src = src;
    return () => { live = false; };
  }, [src]);
  return c;
}
