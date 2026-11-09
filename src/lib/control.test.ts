import { describe, expect, it } from "vitest";
import { tiles, tileTitle, withToggled } from "./control";

describe("Control center tiles", () => {
  it("shows Wi-Fi and Bluetooth only when the PC has them", () => {
    expect(tiles({ wifi: null, bluetooth: null, awake: false }, false).map((t) => t.id)).toEqual(["dnd", "awake"]);
    expect(tiles({ wifi: false, bluetooth: null, awake: false }, false).map((t) => t.id)).toEqual(["wifi", "dnd", "awake"]);
    expect(tiles({ wifi: true, bluetooth: true, awake: true }, true).map((t) => t.id)).toEqual(["wifi", "bluetooth", "dnd", "awake"]);
  });
  it("still offers the always-available switches before the state arrives", () => {
    expect(tiles(null, true).map((t) => [t.id, t.on])).toEqual([
      ["dnd", true],
      ["awake", false],
    ]);
  });
  it("reports on/off in the label and the subtitle", () => {
    const [wifi] = tiles({ wifi: true, bluetooth: null, awake: false }, false);
    expect(wifi).toEqual({ id: "wifi", label: "Wi-Fi", on: true, sub: "Attivo" });
    expect(tileTitle(wifi)).toBe("Wi-Fi: acceso");
    expect(tileTitle(tiles(null, false)[0])).toBe("Non disturbare: spento");
  });
  it("applies a toggle without touching the rest", () => {
    const s = { wifi: false, bluetooth: true, awake: false };
    expect(withToggled(s, "wifi", true)).toEqual({ wifi: true, bluetooth: true, awake: false });
    expect(s.wifi).toBe(false);
  });
});
