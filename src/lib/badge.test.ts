import { describe, expect, it } from "vitest";
import { badgeAria, badgeMap, sameBadges } from "./badge";

describe("Dock badges", () => {
  it("builds the id → label map and ignores empty badges", () => {
    expect(
      badgeMap([
        { id: "a", count: 3, label: "3" },
        { id: "b", count: 0, label: "0" },
        { id: "c", count: 120, label: "99+" },
        { id: "d", count: 2, label: "" },
      ]),
    ).toEqual({ a: "3", c: "99+" });
    expect(badgeMap([])).toEqual({});
  });
  it("compares maps by content", () => {
    expect(sameBadges({}, {})).toBe(true);
    expect(sameBadges({ a: "1" }, { a: "1" })).toBe(true);
    expect(sameBadges({ a: "1" }, { a: "2" })).toBe(false);
    expect(sameBadges({ a: "1" }, { b: "1" })).toBe(false);
    expect(sameBadges({ a: "1" }, {})).toBe(false);
  });
  it("describes the badge for screen readers", () => {
    expect(badgeAria("Discord")).toBe("Discord");
    expect(badgeAria("Discord", "1")).toBe("Discord, 1 notifica");
    expect(badgeAria("Discord", "3")).toBe("Discord, 3 notifiche");
    expect(badgeAria("Discord", "99+")).toBe("Discord, più di 99 notifiche");
  });
});
