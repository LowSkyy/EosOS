import { describe, expect, it } from "vitest";
import { sameItems, splitDock, type DockItem } from "./dock";

const item = (id: string, over: Partial<DockItem> = {}): DockItem => ({
  id,
  name: id,
  path: `C:\\${id}.exe`,
  pinned: false,
  running: false,
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
  });
});
