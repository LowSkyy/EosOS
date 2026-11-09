import { describe, expect, it } from "vitest";
import { columnsFor, filterApps, moveInGrid, nextOffsets } from "./launchpad";

const apps = ["Calcolatrice", "Chrome", "Discord", "Esplora file", "Impostazioni", "Visual Studio Code"].map((n, i) => ({ id: `id${i}`, name: n }));

describe("Launchpad logic", () => {
  it("filters by every word, ignoring accents and case", () => {
    expect(filterApps(apps, "")).toHaveLength(6);
    expect(filterApps(apps, "  ")).toHaveLength(6);
    expect(filterApps(apps, "CHR").map((a) => a.name)).toEqual(["Chrome"]);
    expect(filterApps(apps, "visual code").map((a) => a.name)).toEqual(["Visual Studio Code"]);
    expect(filterApps(apps, "c").map((a) => a.name)).toEqual(["Calcolatrice", "Chrome", "Discord", "Visual Studio Code"]);
    expect(filterApps(apps, "zzz")).toEqual([]);
  });
  it("moves in the grid without leaving it", () => {
    // 7 voci, 3 colonne: righe [0 1 2] [3 4 5] [6]
    expect(moveInGrid(-1, 7, 3, "down")).toBe(0);
    expect(moveInGrid(0, 7, 3, "left")).toBe(0);
    expect(moveInGrid(6, 7, 3, "right")).toBe(6);
    expect(moveInGrid(1, 7, 3, "down")).toBe(4);
    expect(moveInGrid(4, 7, 3, "up")).toBe(1);
    expect(moveInGrid(1, 7, 3, "up")).toBe(1);
    // Giù da una colonna senza voce sotto: si ferma sull'ultima voce.
    expect(moveInGrid(5, 7, 3, "down")).toBe(6);
    expect(moveInGrid(6, 7, 3, "down")).toBe(6);
    expect(moveInGrid(3, 7, 3, "right")).toBe(4);
    expect(moveInGrid(0, 0, 3, "down")).toBe(-1);
    expect(moveInGrid(2, 5, 0, "down")).toBe(3); // colonne minime: 1
  });
  it("asks Rust only for the missing pages", () => {
    expect(nextOffsets(0, 130, 64)).toEqual([0, 64, 128]);
    expect(nextOffsets(64, 130, 64)).toEqual([64, 128]);
    expect(nextOffsets(130, 130, 64)).toEqual([]);
    expect(nextOffsets(0, 0, 64)).toEqual([]);
  });
  it("counts the columns", () => {
    expect(columnsFor(1000, 110, 10)).toBe(8);
    expect(columnsFor(50, 110, 10)).toBe(1);
  });
});
