import { describe, expect, it } from "vitest";
import { TAG_PREFIX, TRASH, columnChain, isRealFolder, taggedWith, trashEntry } from "./finder";

describe("finder, fase 2", () => {
  it("distingue cartelle vere, Cestino ed etichette", () => {
    expect(isRealFolder("C:\\Dati")).toBe(true);
    expect(isRealFolder(TRASH)).toBe(false);
    expect(isRealFolder(TAG_PREFIX + "red")).toBe(false);
  });
  it("trova gli elementi con una etichetta", () => {
    expect(taggedWith({ "C:\\a": ["red", "blue"], "C:\\b": ["blue"], "C:\\c": [] }, "blue")).toEqual(["C:\\a", "C:\\b"]);
  });
  it("mostra le voci del Cestino come voci normali", () => {
    const e = trashEntry({ id: "C:\\$Recycle.Bin\\S\\$R1.txt", name: "nota.txt", origin: "C:\\Dati\\nota.txt", dir: false, size: 3, deleted: 99, ext: "txt" });
    expect(e).toMatchObject({ name: "nota.txt", path: "C:\\$Recycle.Bin\\S\\$R1.txt", modified: 99, ext: "txt" });
  });
  it("costruisce la catena delle colonne", () => {
    expect(columnChain("C:\\Users\\Anna")).toEqual(["C:\\", "C:\\Users", "C:\\Users\\Anna"]);
  });
});
