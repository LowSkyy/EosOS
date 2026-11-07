import { describe, expect, it } from "vitest";
import { boxStyle, cardLetter, cellTitle, neighbour } from "./mc";

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
