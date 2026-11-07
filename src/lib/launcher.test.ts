import { describe, expect, it } from "vitest";
import {
  clampSelection,
  fallbackLetter,
  INPUT_HEIGHT,
  knownIds,
  LauncherItem,
  MAX_HEIGHT,
  mergeIcons,
  moveSelection,
  ROW_HEIGHT,
  windowHeight,
} from "./launcher";

const item = (id: string, icon: string | null = null): LauncherItem => ({ id, name: id, icon });

describe("moveSelection", () => {
  it("wraps in both directions", () => {
    expect(moveSelection(0, 1, 3)).toBe(1);
    expect(moveSelection(2, 1, 3)).toBe(0);
    expect(moveSelection(0, -1, 3)).toBe(2);
  });
  it("stays at 0 with an empty list", () => {
    expect(moveSelection(0, 1, 0)).toBe(0);
    expect(moveSelection(5, -1, 0)).toBe(0);
  });
  it("handles a single result", () => {
    expect(moveSelection(0, 1, 1)).toBe(0);
    expect(moveSelection(0, -1, 1)).toBe(0);
  });
});

describe("clampSelection", () => {
  it("keeps the index inside the list", () => {
    expect(clampSelection(5, 3)).toBe(2);
    expect(clampSelection(-1, 3)).toBe(0);
    expect(clampSelection(1, 3)).toBe(1);
    expect(clampSelection(4, 0)).toBe(0);
  });
});

describe("windowHeight", () => {
  it("is just the field with no query and no results", () => {
    expect(windowHeight(0, false)).toBe(INPUT_HEIGHT);
  });
  it("reserves one row for the no-results message", () => {
    expect(windowHeight(0, true)).toBe(INPUT_HEIGHT + 16 + ROW_HEIGHT);
  });
  it("grows with the results and is capped", () => {
    expect(windowHeight(3, true)).toBe(INPUT_HEIGHT + 16 + 3 * ROW_HEIGHT);
    expect(windowHeight(8, true)).toBeLessThanOrEqual(MAX_HEIGHT);
    expect(windowHeight(500, true)).toBe(MAX_HEIGHT);
  });
});

describe("mergeIcons", () => {
  it("remembers new icons and fills the ones Rust did not resend", () => {
    const cache = new Map<string, string>();
    mergeIcons([item("a", "data:a")], cache);
    const out = mergeIcons([item("a"), item("b", "data:b"), item("c")], cache);
    expect(out.map((i) => i.icon)).toEqual(["data:a", "data:b", null]);
    expect(knownIds(cache).sort()).toEqual(["a", "b"]);
  });
  it("does not mutate its input", () => {
    const input = [item("a")];
    const cache = new Map([["a", "data:a"]]);
    mergeIcons(input, cache);
    expect(input[0].icon).toBeNull();
  });
  it("keeps icons of results that disappeared", () => {
    const cache = new Map<string, string>();
    mergeIcons([item("a", "data:a")], cache);
    mergeIcons([item("b", "data:b")], cache);
    expect(cache.has("a")).toBe(true);
  });
});

describe("fallbackLetter", () => {
  it("uses the first character uppercased", () => {
    expect(fallbackLetter("calcolatrice")).toBe("C");
    expect(fallbackLetter("  éclair")).toBe("É");
    expect(fallbackLetter("😀 faccina")).toBe("😀");
  });
  it("falls back to a question mark", () => {
    expect(fallbackLetter("")).toBe("?");
    expect(fallbackLetter("   ")).toBe("?");
  });
});
