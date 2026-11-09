import { describe, expect, it } from "vitest";
import { EOS_ITEMS, nextIndex } from "./eosmenu";

describe("EosOS menu", () => {
  it("has a single entry that opens the general settings", () => {
    expect(EOS_ITEMS.map((i) => i.id)).toEqual(["settings"]);
    expect(EOS_ITEMS[0].label).toBe("Impostazioni…");
  });
  it("moves with the arrows without wrapping and enters from the right end", () => {
    expect(nextIndex(-1, 5, 1)).toBe(0);
    expect(nextIndex(-1, 5, -1)).toBe(4);
    expect(nextIndex(0, 5, -1)).toBe(0);
    expect(nextIndex(4, 5, 1)).toBe(4);
    expect(nextIndex(2, 5, 1)).toBe(3);
    expect(nextIndex(2, 5, -1)).toBe(1);
    expect(nextIndex(0, 0, 1)).toBe(-1);
  });
});
