import { describe, expect, it } from "vitest";
import {
  MAGNIFY_MAX,
  clampGlass,
  dotState,
  MAGNIFY_REACH,
  magnifyScale,
  pruneBouncing,
  sameItems,
  splitDock,
  type DockItem,
} from "./dock";

const item = (id: string, over: Partial<DockItem> = {}): DockItem => ({
  id,
  name: id,
  path: `C:\\${id}.exe`,
  pinned: false,
  running: false,
  background: false,
  icon: null,
  ...over,
});

describe("splitDock", () => {
  it("separa fissate e aperte mantenendo l'ordine", () => {
    const { pinned, others } = splitDock([
      item("a", { pinned: true }),
      item("b", { pinned: true, running: true }),
      item("c", { running: true }),
    ]);
    expect(pinned.map((i) => i.id)).toEqual(["a", "b"]);
    expect(others.map((i) => i.id)).toEqual(["c"]);
  });

  it("gestisce l'elenco vuoto", () => {
    expect(splitDock([])).toEqual({ pinned: [], others: [] });
  });
});

describe("sameItems", () => {
  it("riconosce elenchi identici", () => {
    expect(sameItems([item("a"), item("b")], [item("a"), item("b")])).toBe(true);
  });

  it("nota un cambio di stato, di icona o di lunghezza", () => {
    expect(sameItems([item("a")], [item("a", { running: true })])).toBe(false);
    expect(sameItems([item("a")], [item("a", { icon: "data:x" })])).toBe(false);
    expect(sameItems([item("a")], [item("a"), item("b")])).toBe(false);
    expect(sameItems([item("a")], [item("a", { background: true })])).toBe(false);
  });
});

describe("magnifyScale", () => {
  const pitch = 62;

  it("è massima sotto il cursore e vale 1 lontano", () => {
    expect(magnifyScale(0, pitch)).toBeCloseTo(MAGNIFY_MAX, 6);
    expect(magnifyScale(pitch * MAGNIFY_REACH, pitch)).toBe(1);
    expect(magnifyScale(10_000, pitch)).toBe(1);
  });

  it("è simmetrica e decresce con la distanza", () => {
    expect(magnifyScale(-40, pitch)).toBeCloseTo(magnifyScale(40, pitch), 9);
    let prev = Infinity;
    for (let d = 0; d <= pitch * MAGNIFY_REACH; d += 7) {
      const s = magnifyScale(d, pitch);
      expect(s).toBeLessThanOrEqual(prev + 1e-9);
      expect(s).toBeGreaterThanOrEqual(1);
      prev = s;
    }
  });

  it("le icone vicine crescono meno di quella sotto il cursore", () => {
    const peak = magnifyScale(0, pitch);
    const next = magnifyScale(pitch, pitch);
    expect(next).toBeGreaterThan(1);
    expect(next).toBeLessThan(peak);
  });

  it("con passo non valido non ingrandisce", () => {
    expect(magnifyScale(0, 0)).toBe(1);
    expect(magnifyScale(0, NaN)).toBe(1);
  });
});

describe("pruneBouncing", () => {
  it("toglie le app già aperte o scomparse e tiene quelle in avvio", () => {
    const items = [item("a", { running: true }), item("b", { running: false })];
    expect([...pruneBouncing(new Set(["a", "b", "zzz"]), items)]).toEqual(["b"]);
  });
});

describe("dotState", () => {
  it("distingue finestre aperte, solo background e chiusa", () => {
    expect(dotState({ running: true, background: false })).toBe("running");
    expect(dotState({ running: false, background: true })).toBe("background");
    expect(dotState({ running: false, background: false })).toBe("none");
    // Con finestre vale sempre il puntino normale.
    expect(dotState({ running: true, background: true })).toBe("running");
  });
});

describe("clampGlass", () => {
  it("limita a 0..1 e scarta valori non numerici", () => {
    expect(clampGlass(0.4)).toBe(0.4);
    expect(clampGlass(3)).toBe(1);
    expect(clampGlass(-1)).toBe(0);
    expect(clampGlass(Number.NaN)).toBe(0.7);
    expect(clampGlass("x")).toBe(0.7);
    expect(clampGlass(undefined)).toBe(0.7);
  });
});
