import { describe, expect, it } from "vitest";
import { FRAME_HEADER, parseFrame, sourceRect } from "./backdrop";

/** Costruisce un fotogramma come lo serializza Rust. */
function make(seq: number, ox: number, oy: number, w: number, h: number, factor: number, px?: number[]) {
  const data = px ?? new Array(w * h * 4).fill(7);
  const buf = new ArrayBuffer(FRAME_HEADER + data.length);
  const v = new DataView(buf);
  v.setBigUint64(0, BigInt(seq), true);
  v.setInt32(8, ox, true);
  v.setInt32(12, oy, true);
  v.setUint32(16, w, true);
  v.setUint32(20, h, true);
  v.setUint32(24, factor, true);
  new Uint8Array(buf, FRAME_HEADER).set(data);
  return buf;
}

describe("parseFrame", () => {
  it("legge intestazione e pixel", () => {
    const f = parseFrame(make(42, -48, 12, 2, 1, 2, [1, 2, 3, 255, 4, 5, 6, 255]));
    expect(f).not.toBeNull();
    expect(f).toMatchObject({ seq: 42, ox: -48, oy: 12, w: 2, h: 1, factor: 2 });
    expect([...f!.data]).toEqual([1, 2, 3, 255, 4, 5, 6, 255]);
  });

  it("scarta risposte vuote o malformate", () => {
    expect(parseFrame(new ArrayBuffer(0))).toBeNull(); // nessun fotogramma nuovo
    expect(parseFrame(new ArrayBuffer(10))).toBeNull(); // intestazione corta
    expect(parseFrame(make(1, 0, 0, 0, 5, 2, []))).toBeNull(); // larghezza zero
    expect(parseFrame(make(1, 0, 0, 2, 2, 0))).toBeNull(); // fattore zero
    // Dimensioni dichiarate che non corrispondono ai byte ricevuti.
    const bad = make(1, 0, 0, 3, 3, 2, new Array(8).fill(0));
    expect(parseFrame(bad)).toBeNull();
  });
});

describe("sourceRect", () => {
  const frame = { ox: -48, oy: -48, factor: 2 };

  it("a scala 100% la capsula a (72, 90) cade al punto giusto del fotogramma", () => {
    // Il fotogramma parte 48 px fisici a sinistra e sopra la finestra, ridotto di 2.
    const r = sourceRect({ left: 72, top: 90, width: 400, height: 78 }, 28, 1, frame);
    expect(r.sx).toBeCloseTo((72 - 28) / 2 + 24, 9); // 22 + 24
    expect(r.sy).toBeCloseTo((90 - 28) / 2 + 24, 9);
    expect(r.sw).toBeCloseTo((400 + 56) / 2, 9);
    expect(r.sh).toBeCloseTo((78 + 56) / 2, 9);
  });

  it("a scala 150% i px CSS diventano più pixel fisici", () => {
    const a = sourceRect({ left: 100, top: 100, width: 200, height: 80 }, 0, 1, frame);
    const b = sourceRect({ left: 100, top: 100, width: 200, height: 80 }, 0, 1.5, frame);
    expect(b.sw).toBeCloseTo(a.sw * 1.5, 9);
    expect(b.sh).toBeCloseTo(a.sh * 1.5, 9);
  });

  it("muovendo la capsula a destra, l'area sorgente si sposta a destra", () => {
    const a = sourceRect({ left: 100, top: 90, width: 400, height: 78 }, 28, 1, frame);
    const b = sourceRect({ left: 130, top: 90, width: 400, height: 78 }, 28, 1, frame);
    expect(b.sx - a.sx).toBeCloseTo(15, 9);
    expect(b.sy).toBe(a.sy);
  });
});

describe("bar glass tint", () => {
  it("measures the mean luminance", async () => {
    const { meanLuminance } = await import("./backdrop");
    const white = new Uint8ClampedArray(4 * 64).fill(255);
    const black = new Uint8ClampedArray(4 * 64).fill(0);
    expect(meanLuminance(white, 1)).toBeCloseTo(1, 5);
    expect(meanLuminance(black, 1)).toBe(0);
    expect(meanLuminance(new Uint8ClampedArray(0))).toBe(0.5);
    const grey = new Uint8ClampedArray(4 * 64).fill(128);
    expect(meanLuminance(grey, 4)).toBeCloseTo(128 / 255, 2);
  });
  it("makes the tint stronger when the text would be hard to read", async () => {
    const { barTint } = await import("./backdrop");
    const a = (s: string) => Number(/, ([\d.]+)\)$/.exec(s)![1]);
    // Tema scuro (testo chiaro): più chiaro lo sfondo, più scura la tinta.
    expect(a(barTint(true, 1))).toBeGreaterThan(a(barTint(true, 0)));
    // Tema chiaro (testo scuro): più scuro lo sfondo, più chiara la tinta.
    expect(a(barTint(false, 0))).toBeGreaterThan(a(barTint(false, 1)));
    // Resta vetro: mai opaco.
    for (const dark of [true, false]) for (const l of [0, 0.5, 1]) expect(a(barTint(dark, l))).toBeLessThanOrEqual(0.7);
    expect(barTint(true, 0.5)).toMatch(/^rgba\(30, 30, 32, /);
    expect(barTint(false, 0.5)).toMatch(/^rgba\(246, 246, 248, /);
    // Valori assurdi non rompono nulla.
    expect(barTint(true, NaN)).toBe(barTint(true, 0.5));
    expect(a(barTint(true, 99))).toBe(a(barTint(true, 1)));
  });
});

describe("bar text colour", () => {
  it("keeps the text readable on any background, whatever the theme", async () => {
    const { barColors, linear, LIGHT_TEXT, DARK_TEXT } = await import("./backdrop");
    for (const dark of [true, false]) {
      for (const lum of [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1]) {
        const c = barColors(dark, lum);
        const bg = linear(c.effective);
        const textLight = c.fg === "#f5f5f7";
        const textL = textLight ? LIGHT_TEXT : DARK_TEXT;
        const ratio = (Math.max(bg, textL) + 0.05) / (Math.min(bg, textL) + 0.05);
        // WCAG: almeno 4,5:1 per il testo normale, per ogni sfondo e ogni tema.
        expect(ratio, `tema ${dark ? "scuro" : "chiaro"}, sfondo ${lum}`).toBeGreaterThan(4.5);
      }
    }
  });
  it("uses the tint of barTint", async () => {
    const { barColors, barTint } = await import("./backdrop");
    expect(barColors(true, 0.3).tint).toBe(barTint(true, 0.3));
    expect(barColors(false, 0.3).tint).toBe(barTint(false, 0.3));
    expect(barColors(true, NaN).effective).toBeCloseTo(barColors(true, 0.5).effective, 6);
  });
});
