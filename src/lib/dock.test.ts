import { describe, expect, it } from "vitest";
import {
  MAGNIFY_MAX,
  applyMove,
  dropTarget,
  previewOffsets,
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

describe("dropTarget", () => {
  it("conta le altre icone a sinistra del cursore", () => {
    const centers = [30, 90, 150];
    expect(dropTarget(centers, 0)).toBe(0);
    expect(dropTarget(centers, 31)).toBe(1);
    expect(dropTarget(centers, 100)).toBe(2);
    expect(dropTarget(centers, 999)).toBe(3);
    expect(dropTarget([], 50)).toBe(0);
  });
});

describe("previewOffsets", () => {
  const ids = ["A", "B", "C", "D"];
  it("trascinando A in avanti, le icone scavalcate si spostano a sinistra", () => {
    // A (posizione 0) verso la posizione 2 tra le altre [B, C, D]
    expect(previewOffsets(ids, "A", 2, 62)).toEqual({ B: -62, C: -62, D: 0 });
  });
  it("trascinando D indietro, le icone scavalcate si spostano a destra", () => {
    expect(previewOffsets(ids, "D", 1, 62)).toEqual({ A: 0, B: 62, C: 62 });
  });
  it("lasciandola dov'è non muove nessuno", () => {
    expect(previewOffsets(ids, "B", 1, 62)).toEqual({ A: 0, C: 0, D: 0 });
  });
  it("una non fissata trascinata tra le fissate apre un varco", () => {
    expect(previewOffsets(ids, "X", 1, 62)).toEqual({ A: 0, B: 62, C: 62, D: 62 });
  });
  it("limita il bersaglio fuori range", () => {
    expect(previewOffsets(ids, "A", 99, 62)).toEqual({ B: -62, C: -62, D: -62 });
    expect(previewOffsets(ids, "D", -5, 62)).toEqual({ A: 62, B: 62, C: 62 });
  });
});

describe("applyMove", () => {
  const list = [
    item("A", { pinned: true }),
    item("B", { pinned: true }),
    item("C", { pinned: true }),
    item("X", { running: true }),
  ];
  const ids = (l: DockItem[]) => l.map((i) => i.id);
  it("riordina le fissate", () => {
    expect(ids(applyMove(list, "A", 2))).toEqual(["B", "C", "A", "X"]);
    expect(ids(applyMove(list, "C", 0))).toEqual(["C", "A", "B", "X"]);
  });
  it("una app aperta trascinata tra le fissate viene fissata", () => {
    const out = applyMove(list, "X", 1);
    expect(ids(out)).toEqual(["A", "X", "B", "C"]);
    expect(out[1].pinned).toBe(true);
  });
  it("un id sconosciuto lascia l'elenco com'è", () => {
    expect(applyMove(list, "boh", 0)).toBe(list);
  });
});
