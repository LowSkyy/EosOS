import { describe, expect, it } from "vitest";
import { dominant, tones } from "./tint";

describe("tint", () => {
  it("prende il colore vivace e ignora trasparenti e bianchi", () => {
    const px = [255, 0, 0, 255, 255, 0, 0, 255, 255, 255, 255, 255, 0, 255, 0, 0];
    const d = dominant(px)!;
    expect(d[0]).toBeGreaterThan(200);
    expect(d[1]).toBeLessThan(60);
  });
  it("niente pixel visibili: nessun colore", () => {
    expect(dominant([10, 10, 10, 0])).toBeNull();
  });
  it("il tema scuro da toni piu' scuri", () => {
    const lum = (s: string) => s.match(/\d+/g)!.map(Number).reduce((a, b) => a + b, 0);
    expect(lum(tones([200, 80, 40], true)[0])).toBeLessThan(lum(tones([200, 80, 40], false)[0]));
  });
});
