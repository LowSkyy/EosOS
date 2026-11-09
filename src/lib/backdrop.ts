/** Intestazione di un fotogramma dello sfondo: vedi `backdrop::encode` in Rust. */
export const FRAME_HEADER = 28;

/** Fotogramma dello schermo dietro il Dock, già ridotto di `factor`. */
export type Frame = {
  seq: number;
  /** Origine della cattura rispetto all'angolo della finestra del Dock (px fisici). */
  ox: number;
  oy: number;
  /** Dimensioni in pixel del fotogramma ridotto. */
  w: number;
  h: number;
  factor: number;
  /** RGBA opaco. */
  data: Uint8ClampedArray<ArrayBuffer>;
};

/**
 * Decodifica la risposta di `dock_backdrop`. Restituisce `null` se è vuota (nessun
 * fotogramma nuovo) o malformata (intestazione corta, dimensioni che non tornano).
 */
export function parseFrame(buf: ArrayBuffer): Frame | null {
  if (buf.byteLength < FRAME_HEADER) return null;
  const v = new DataView(buf);
  const w = v.getUint32(16, true);
  const h = v.getUint32(20, true);
  const factor = v.getUint32(24, true);
  if (w === 0 || h === 0 || factor === 0) return null;
  if (buf.byteLength !== FRAME_HEADER + w * h * 4) return null;
  return {
    seq: Number(v.getBigUint64(0, true)),
    ox: v.getInt32(8, true),
    oy: v.getInt32(12, true),
    w,
    h,
    factor,
    data: new Uint8ClampedArray(buf.slice(FRAME_HEADER)),
  };
}

/**
 * Parte del fotogramma (in pixel del fotogramma) che sta dietro la capsula e un
 * contorno di `margin` px CSS. `pill` è in px CSS rispetto alla finestra, `dpr` è
 * la scala dello schermo. Pura, testata.
 */
export function sourceRect(
  pill: { left: number; top: number; width: number; height: number },
  margin: number,
  dpr: number,
  frame: Pick<Frame, "ox" | "oy" | "factor">,
) {
  const k = dpr / frame.factor;
  return {
    sx: (pill.left - margin) * k - frame.ox / frame.factor,
    sy: (pill.top - margin) * k - frame.oy / frame.factor,
    sw: (pill.width + 2 * margin) * k,
    sh: (pill.height + 2 * margin) * k,
  };
}

/** Luminosità media (0 = nero, 1 = bianco) di un fotogramma RGBA, campionando un pixel ogni `step` (per non pesare). Pura. */
export function meanLuminance(data: Uint8ClampedArray, step = 16): number {
  let sum = 0;
  let n = 0;
  for (let i = 0; i + 2 < data.length; i += 4 * step) {
    // Luminanza relativa approssimata (Rec. 709) sui valori di schermo.
    sum += 0.2126 * data[i] + 0.7152 * data[i + 1] + 0.0722 * data[i + 2];
    n++;
  }
  return n === 0 ? 0.5 : sum / n / 255;
}

/**
 * La tinta della barra per la luminosità dello sfondo: il vetro «liquido» si adatta a ciò che c'è dietro. Con il tema
 * scuro (testo chiaro) lo sfondo chiaro riceve una tinta più scura, così il testo si legge; con il tema chiaro (testo
 * scuro) lo sfondo scuro riceve una tinta più chiara. L'opacità resta sempre nel vetro (mai pieno). Pura, testata.
 */
export function barTint(dark: boolean, lum: number): string {
  const l = Math.min(1, Math.max(0, Number.isFinite(lum) ? lum : 0.5));
  const alpha = (dark ? 0.26 + 0.44 * l : 0.64 - 0.34 * l).toFixed(2);
  return dark ? `rgba(30, 30, 32, ${alpha})` : `rgba(246, 246, 248, ${alpha})`;
}

/** Luminanza relativa (lineare) del testo chiaro #f5f5f7 e di quello scuro #1c1c1e. */
export const LIGHT_TEXT = 0.91;
export const DARK_TEXT = 0.012;

/** Da luminosità «di schermo» (0..1) a luminanza lineare, come nel calcolo del contrasto WCAG. Pura. */
export const linear = (c: number): number => (c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4));

/** Luminosità (0..1) di un colore «r, g, b». Pura. */
const lumOf = (r: number, g: number, b: number): number => (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;

/**
 * Tinta e colore del testo della barra per lo sfondo dietro di lei. Il testo si sceglie dalla luminosità EFFETTIVA (sfondo
 * mescolato con la tinta), non dal tema: così resta leggibile su qualunque sfondo, anche con il tema chiaro su uno sfondo scuro
 * (o al contrario). Pura, testata.
 */
export function barColors(dark: boolean, lum: number): { tint: string; fg: string; fgMuted: string; effective: number } {
  const l = Math.min(1, Math.max(0, Number.isFinite(lum) ? lum : 0.5));
  const alpha = dark ? 0.26 + 0.44 * l : 0.64 - 0.34 * l;
  const tintLum = dark ? lumOf(30, 30, 32) : lumOf(246, 246, 248);
  const effective = l * (1 - alpha) + tintLum * alpha;
  // Il testo (chiaro o scuro) si sceglie dal contrasto vero: la luminosità va prima portata da valori di schermo a lineari.
  const bg = linear(effective);
  const withDark = (bg + 0.05) / (DARK_TEXT + 0.05);
  const withLight = (LIGHT_TEXT + 0.05) / (bg + 0.05);
  const lightBackground = withDark >= withLight;
  return {
    tint: barTint(dark, l),
    fg: lightBackground ? "#1c1c1e" : "#f5f5f7",
    fgMuted: lightBackground ? "#3a3a3c" : "#c7c7cc",
    effective,
  };
}
