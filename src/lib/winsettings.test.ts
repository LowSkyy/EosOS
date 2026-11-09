import { describe, expect, it } from "vitest";
import { filterPages, fold, groupPages, type WinPage } from "./winsettings";

const p = (id: string, group: string, label: string, hint: string): WinPage => ({ id, group, label, hint });
const pages = [
  p("display", "Sistema", "Schermo", "Luminosità, scala, risoluzione"),
  p("sound", "Sistema", "Audio", "Uscita, ingresso, volume"),
  p("wifi", "Rete e Internet", "Wi-Fi", "Reti wireless disponibili"),
  p("update", "Windows Update", "Windows Update", "Cerca e installa gli aggiornamenti"),
];

describe("Windows settings list", () => {
  it("folds accents and case", () => {
    expect(fold("Luminosità")).toBe("luminosita");
    expect(fold("SCHERMO")).toBe("schermo");
  });
  it("shows everything without a query and needs every word with one", () => {
    expect(filterPages(pages, "")).toHaveLength(4);
    expect(filterPages(pages, "   ")).toHaveLength(4);
    expect(filterPages(pages, "luminosita").map((x) => x.id)).toEqual(["display"]);
    expect(filterPages(pages, "VOLUME uscita").map((x) => x.id)).toEqual(["sound"]);
    expect(filterPages(pages, "volume schermo")).toEqual([]);
    expect(filterPages(pages, "sistema").map((x) => x.id)).toEqual(["display", "sound"]);
    expect(filterPages(pages, "aggiornamenti").map((x) => x.id)).toEqual(["update"]);
    expect(filterPages(pages, "zzz")).toEqual([]);
  });
  it("groups in order", () => {
    const g = groupPages(pages);
    expect(g.map((x) => x.group)).toEqual(["Sistema", "Rete e Internet", "Windows Update"]);
    expect(g[0].pages.map((x) => x.id)).toEqual(["display", "sound"]);
    expect(groupPages([])).toEqual([]);
  });
});
