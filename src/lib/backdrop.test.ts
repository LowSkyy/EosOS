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
