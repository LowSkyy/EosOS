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
