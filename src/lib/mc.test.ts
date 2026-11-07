import { describe, expect, it } from "vitest";
import { advancedHint, boxStyle, canRemove, cardLetter, cellTitle, cleanDesktopName, desktopAction, desktopLabel, dragStarted, neighbour } from "./mc";

describe("boxStyle", () => {
  it("turns a rectangle into absolute positioning", () => {
    expect(boxStyle([10, 20, 300, 200])).toEqual({ left: 10, top: 20, width: 300, height: 200 });
  });
});

describe("cellTitle and cardLetter", () => {
  it("falls back to the app name for untitled windows", () => {
    expect(cellTitle({ title: "Relazione.docx - Word", app: "Word" })).toBe("Relazione.docx - Word");
    expect(cellTitle({ title: "   ", app: "Word" })).toBe("Word");
    expect(cellTitle({ title: "", app: "Calcolatrice" })).toBe("Calcolatrice");
  });
  it("uses the first character of the app", () => {
    expect(cardLetter("calcolatrice")).toBe("C");
    expect(cardLetter("  \u00e9diteur")).toBe("\u00c9");
    expect(cardLetter("")).toBe("?");
  });
});

describe("neighbour", () => {
  // Griglia 3 + 2: [0 1 2] sopra, [3 4] sotto.
  const cells: [number, number, number, number][] = [
    [0, 0, 100, 100],
    [120, 0, 100, 100],
    [240, 0, 100, 100],
    [60, 120, 100, 100],
    [180, 120, 100, 100],
  ];
  it("moves to the closest cell in each direction", () => {
    expect(neighbour(cells, 0, "right")).toBe(1);
    expect(neighbour(cells, 1, "right")).toBe(2);
    expect(neighbour(cells, 2, "left")).toBe(1);
    expect(neighbour(cells, 0, "down")).toBe(3);
    expect(neighbour(cells, 2, "down")).toBe(4);
    expect(neighbour(cells, 3, "up")).toBe(0);
    // La cella 4 e' esattamente fra la 1 e la 2: una qualunque delle due va bene, ma sempre una di quelle.
    expect([1, 2]).toContain(neighbour(cells, 4, "up"));
  });
  it("returns null when there is nothing in that direction", () => {
    expect(neighbour(cells, 0, "left")).toBeNull();
    expect(neighbour(cells, 0, "up")).toBeNull();
    expect(neighbour(cells, 2, "right")).toBeNull();
    expect(neighbour(cells, 3, "down")).toBeNull();
  });
  it("handles degenerate input", () => {
    expect(neighbour([], 0, "right")).toBeNull();
    expect(neighbour(cells, 9, "right")).toBeNull();
    expect(neighbour([[0, 0, 10, 10]], 0, "right")).toBeNull();
  });
});

describe("desktops bar", () => {
  const cur = { id: "{A}", name: "Desktop 1", current: true };
  const other = { id: "{B}", name: "Lavoro", current: false };
  it("closes on the current desktop and switches on the others", () => {
    expect(desktopAction(cur, 0)).toEqual({ kind: "close" });
    expect(desktopAction(other, 1)).toEqual({ kind: "switch", index: 1 });
  });
  it("names each desktop for screen readers", () => {
    expect(desktopLabel(cur, 0, 2)).toBe("Desktop 1, desktop 1 di 2, corrente");
    expect(desktopLabel(other, 1, 2)).toBe("Lavoro, desktop 2 di 2");
  });
});

describe("dragging and desktop management", () => {
  it("starts a drag only after a real movement", () => {
    expect(dragStarted({ x: 10, y: 10 }, { x: 10, y: 10 })).toBe(false);
    expect(dragStarted({ x: 10, y: 10 }, { x: 14, y: 14 })).toBe(false);
    expect(dragStarted({ x: 10, y: 10 }, { x: 18, y: 10 })).toBe(true);
    expect(dragStarted({ x: 0, y: 0 }, { x: -6, y: -6 })).toBe(true);
  });
  const cur = { id: "{A}", name: "1", current: true };
  const other = { id: "{B}", name: "2", current: false };
  it("allows removing any desktop only with the internal backend, the current one otherwise", () => {
    expect(canRemove({ remove_any: true }, other, 2)).toBe(true);
    expect(canRemove({ remove_any: false }, other, 2)).toBe(false);
    expect(canRemove({ remove_any: false }, cur, 2)).toBe(true);
    expect(canRemove({ remove_any: true }, cur, 1)).toBe(false);
    expect(canRemove({ remove_any: false }, cur, 1)).toBe(false);
  });
  it("cleans a typed desktop name", () => {
    expect(cleanDesktopName("  Lavoro  ")).toBe("Lavoro");
    expect(cleanDesktopName("   ")).toBe("");
    expect(cleanDesktopName("x".repeat(100))).toHaveLength(60);
    expect([...cleanDesktopName("\u{1F600}".repeat(80))]).toHaveLength(60);
  });
  it("explains the missing advanced features", () => {
    const base = { internal: true, move_window: true, remove_any: true, rename: true, reason: null };
    expect(advancedHint(base)).toBeNull();
    expect(advancedHint({ ...base, internal: false })).toContain("non è disponibile");
  });
});
