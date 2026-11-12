import { describe, expect, it } from "vitest";
import { arrange, artFor, badgeFor, clickSelect, crumbs, hasThumb, kindLabel, sizeLabel, usesShellIcon, type Entry } from "./finder";

const e = (name: string, over: Partial<Entry> = {}): Entry => ({ name, path: `C:\\x\\${name}`, dir: false, size: 0, modified: 0, ext: name.includes(".") ? name.split(".").pop()!.toLowerCase() : "", hidden: false, ...over });

describe("finder", () => {
  it("sceglie l'icona dal tipo di file", () => {
    expect(artFor("png")).toBe("image");
    expect(artFor("docx")).toBe("word");
    expect(artFor("rs")).toBe("code");
    expect(artFor("boh")).toBe("file");
    expect(hasThumb("jpg") && hasThumb("mp4") && !hasThumb("txt")).toBe(true);
    expect(usesShellIcon("exe") && usesShellIcon("lnk") && !usesShellIcon("png")).toBe(true);
  });

  it("descrive il tipo e la dimensione", () => {
    expect(kindLabel({ dir: true, ext: "" })).toBe("Cartella");
    expect(kindLabel({ dir: false, ext: "png" })).toBe("Immagine PNG");
    expect(kindLabel({ dir: false, ext: "exe" })).toBe("Applicazione");
    expect(kindLabel({ dir: false, ext: "pdf" })).toBe("Documento PDF");
    expect(kindLabel({ dir: false, ext: "xyz" })).toBe("File XYZ");
    expect(kindLabel({ dir: false, ext: "" })).toBe("Documento");
    expect(sizeLabel(512)).toBe("512 byte");
    expect(sizeLabel(1536)).toBe("1,5 KB");
    expect(sizeLabel(5 * 1024 * 1024 * 1024)).toBe("5,0 GB");
  });

  it("ordina con le cartelle prima, nasconde i file nascosti e filtra", () => {
    const list = [e("b.txt", { size: 10 }), e("Zeta", { dir: true }), e("a10.txt", { size: 5 }), e("a2.txt", { size: 50 }), e(".segreto", { hidden: true })];
    expect(arrange(list, { key: "name", asc: true }, false, "").map((x) => x.name)).toEqual(["Zeta", "a2.txt", "a10.txt", "b.txt"]);
    expect(arrange(list, { key: "size", asc: false }, false, "").map((x) => x.name)).toEqual(["Zeta", "a2.txt", "b.txt", "a10.txt"]);
    expect(arrange(list, { key: "name", asc: true }, true, "").length).toBe(5);
    expect(arrange(list, { key: "name", asc: true }, false, "A1").map((x) => x.name)).toEqual(["a10.txt"]);
  });

  it("spezza il percorso in livelli", () => {
    expect(crumbs("C:\\Users\\Anna")).toEqual([
      { name: "C:", path: "C:\\" },
      { name: "Users", path: "C:\\Users" },
      { name: "Anna", path: "C:\\Users\\Anna" },
    ]);
    expect(crumbs("D:\\")).toEqual([{ name: "D:", path: "D:\\" }]);
  });

  it("seleziona con clic, Ctrl e Maiusc", () => {
    const order = ["a", "b", "c", "d"];
    expect(clickSelect(order, ["a"], "a", "c", false, false).selected).toEqual(["c"]);
    expect(clickSelect(order, ["a"], "a", "c", true, false).selected).toEqual(["a", "c"]);
    expect(clickSelect(order, ["a", "c"], "c", "c", true, false).selected).toEqual(["a"]);
    expect(clickSelect(order, ["b"], "b", "d", false, true).selected).toEqual(["b", "c", "d"]);
    expect(clickSelect(order, ["d"], "d", "b", false, true).selected).toEqual(["b", "c", "d"]);
  });

  it("riconosce le cartelle dell'utente", () => {
    const places = [
      { name: "Documenti", path: "C:\\Users\\Anna\\Documents", kind: "documents" as const },
      { name: "Disco", path: "C:\\", kind: "drive" as const },
    ];
    expect(badgeFor("c:\\users\\anna\\documents\\", places)).toBe("documents");
    expect(badgeFor("C:\\", places)).toBeUndefined();
    expect(badgeFor("C:\\Altro", places)).toBeUndefined();
  });
});
