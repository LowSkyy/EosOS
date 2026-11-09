import { describe, expect, it } from "vitest";
import { columnsFor, filterApps, layoutFor, moveInGrid, moveInPages, nextOffsets, pageCount, pageOf, pageSlice } from "./launchpad";

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

describe("Launchpad pages", () => {
  it("fits columns and rows in the area", () => {
    expect(layoutFor(1000, 600, 110, 120, 10)).toEqual({ cols: 8, rows: 4 });
    expect(layoutFor(50, 50, 110, 120, 10)).toEqual({ cols: 1, rows: 1 });
  });
  it("counts and slices the pages", () => {
    expect(pageCount(0, 20)).toBe(1);
    expect(pageCount(20, 20)).toBe(1);
    expect(pageCount(21, 20)).toBe(2);
    expect(pageCount(5, 0)).toBe(5);
    expect(pageOf(0, 20)).toBe(0);
    expect(pageOf(19, 20)).toBe(0);
    expect(pageOf(20, 20)).toBe(1);
    expect(pageSlice([1, 2, 3, 4, 5], 1, 2)).toEqual([3, 4]);
    expect(pageSlice([1, 2, 3, 4, 5], 2, 2)).toEqual([5]);
    expect(pageSlice([1, 2, 3], 5, 2)).toEqual([]);
  });
  it("moves inside a page and across pages on the same row", () => {
    // 3 colonne x 2 righe = 6 per pagina; 10 app: pagina 0 = 0..5, pagina 1 = 6..9.
    const m = (i: number, d: "left" | "right" | "up" | "down") => moveInPages(i, 10, 3, 2, d);
    expect(m(-1, "down")).toBe(0);
    expect(m(0, "right")).toBe(1);
    expect(m(2, "right")).toBe(6); // bordo destro della riga 0 -> pagina 1, riga 0, colonna 0
    expect(m(5, "right")).toBe(9); // riga 1 -> pagina 1 riga 1 colonna 0 = indice 9
    expect(m(6, "left")).toBe(2); // indietro: bordo destro della stessa riga della pagina precedente
    expect(m(9, "left")).toBe(5);
    expect(m(0, "left")).toBe(0);
    expect(m(0, "down")).toBe(3);
    expect(m(3, "up")).toBe(0);
    expect(m(0, "up")).toBe(0);
    expect(m(3, "down")).toBe(3); // ultima riga: non cambia pagina
  });
  it("stops on the last app when there is nothing below or to the right", () => {
    // 8 app, 3 x 2: pagina 1 = 6,7 (due app, riga 0).
    expect(moveInPages(7, 8, 3, 2, "right")).toBe(7);
    expect(moveInPages(6, 8, 3, 2, "down")).toBe(7);
    expect(moveInPages(7, 8, 3, 2, "down")).toBe(7);
    expect(moveInPages(0, 0, 3, 2, "right")).toBe(-1);
  });
});
