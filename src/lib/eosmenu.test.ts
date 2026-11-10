import { describe, expect, it } from "vitest";
import { EOS_ITEMS, nextIndex } from "./eosmenu";

describe("EosOS menu", () => {
  it("has unique ids and the dangerous items ask for confirmation", () => {
    const ids = EOS_ITEMS.map((i) => i.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const id of ["restart", "shutdown"]) expect(EOS_ITEMS.find((i) => i.id === id)?.hint).toBe("chiede conferma");
    expect(ids).toEqual(["about", "settings", "winsettings", "launchpad", "mission", "lock", "sleep", "restart", "shutdown", "quit"]);
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
