import { describe, expect, it } from "vitest";
import {
  clampSelection,
  combine,
  filterLabel,
  wantsFiles,
  fallbackLetter,
  filterForKey,
  glyphFor,
  kindLabel,
  toggleFilter,
  INPUT_HEIGHT,
  knownIds,
  LauncherItem,
  MAX_HEIGHT,
  mergeIcons,
  moveSelection,
  pointerMoved,
  ROW_HEIGHT,
  windowHeight,
} from "./launcher";

const item = (id: string, icon: string | null = null): LauncherItem => ({ id, name: id, kind: "app", icon });

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

describe("filters", () => {
  it("toggles a tab on and off", () => {
    expect(toggleFilter(null, "app")).toBe("app");
    expect(toggleFilter("app", "app")).toBeNull();
    expect(toggleFilter("app", "action")).toBe("action");
    expect(toggleFilter("file", "file")).toBeNull();
  });
  it("maps Ctrl+digit to a tab", () => {
    expect(filterForKey("1")).toBe("app");
    expect(filterForKey("2")).toBe("file");
    expect(filterForKey("3")).toBe("action");
    expect(filterForKey("4")).toBe("clip");
    expect(filterForKey("5")).toBeUndefined();
    expect(filterForKey("a")).toBeUndefined();
  });
});

describe("glyphFor and kindLabel", () => {
  it("uses symbols for calc and known actions, a letter otherwise", () => {
    expect(glyphFor({ id: "calc", kind: "calc", name: "84" })).toBe("=");
    expect(glyphFor({ id: "action:shutdown", kind: "action", name: "Spegni" })).toBe("⏻");
    expect(glyphFor({ id: "action:nuova", kind: "action", name: "Nuova" })).toBe("N");
  });
  it("labels each kind and the confirmation step", () => {
    expect(kindLabel({ kind: "app" }, false)).toBe("App");
    expect(kindLabel({ kind: "action" }, false)).toBe("Azione");
    expect(kindLabel({ kind: "calc" }, false)).toBe("Invio per copiare");
    expect(kindLabel({ kind: "action" }, true)).toBe("Invio per confermare");
    expect(kindLabel({ kind: "file" }, false)).toBe("File");
    expect(kindLabel({ kind: "clip" }, false)).toBe("Appunti");
    expect(kindLabel({ kind: "setting" }, false)).toBe("Invio per attivare");
  });
});

const mk = (n: number, kind: LauncherItem["kind"]): LauncherItem[] =>
  Array.from({ length: n }, (_, i) => ({ id: `${kind}${i}`, name: `${kind}${i}`, kind, icon: null }));

describe("combine", () => {
  it("shows only files on the file tab and only fast results on the other tabs", () => {
    expect(combine(mk(5, "app"), mk(3, "file"), "file").map((i) => i.kind)).toEqual(["file", "file", "file"]);
    expect(combine(mk(5, "app"), mk(3, "file"), "app")).toHaveLength(5);
    expect(combine(mk(5, "app"), mk(3, "file"), "action").every((i) => i.kind === "app")).toBe(true);
    expect(combine(mk(4, "clip"), mk(3, "file"), "clip").every((i) => i.kind === "clip")).toBe(true);
  });
  it("fills the rest of the list with files on the all tab", () => {
    const r = combine(mk(3, "app"), mk(10, "file"), null);
    expect(r).toHaveLength(8);
    expect(r.filter((i) => i.kind === "file")).toHaveLength(5);
    expect(r.slice(0, 3).every((i) => i.kind === "app")).toBe(true);
  });
  it("always leaves at least two rows for files when there are any", () => {
    const r = combine(mk(8, "app"), mk(10, "file"), null);
    expect(r).toHaveLength(8);
    expect(r.filter((i) => i.kind === "file")).toHaveLength(2);
    expect(combine(mk(8, "app"), mk(1, "file"), null).filter((i) => i.kind === "file")).toHaveLength(1);
  });
  it("copes with empty inputs", () => {
    expect(combine([], [], null)).toEqual([]);
    expect(combine([], mk(3, "file"), null)).toHaveLength(3);
    expect(combine(mk(2, "app"), [], null)).toHaveLength(2);
  });
});

describe("wantsFiles and filterLabel", () => {
  it("needs the right tab and two characters", () => {
    expect(wantsFiles("ab", null)).toBe(true);
    expect(wantsFiles(" a ", null)).toBe(false);
    expect(wantsFiles("", "file")).toBe(false);
    expect(wantsFiles("relazione", "app")).toBe(false);
    expect(wantsFiles("relazione", "action")).toBe(false);
    expect(wantsFiles("relazione", "clip")).toBe(false);
    expect(wantsFiles("😀😀", "file")).toBe(true);
  });
  it("labels tabs", () => {
    expect(filterLabel("file")).toBe("File");
    expect(filterLabel("action")).toBe("Azioni");
    expect(filterLabel("app")).toBe("App");
    expect(filterLabel("clip")).toBe("Appunti");
  });
});

describe("pointerMoved", () => {
  it("ignores the first event and synthetic events at the same position", () => {
    expect(pointerMoved(null, { x: 100, y: 100 })).toBe(false);
    expect(pointerMoved({ x: 100, y: 100 }, { x: 100, y: 100 })).toBe(false);
    expect(pointerMoved({ x: 100, y: 100 }, { x: 101, y: 100 })).toBe(false);
  });
  it("accepts a real movement in any direction", () => {
    expect(pointerMoved({ x: 100, y: 100 }, { x: 103, y: 100 })).toBe(true);
    expect(pointerMoved({ x: 100, y: 100 }, { x: 100, y: 96 })).toBe(true);
    expect(pointerMoved({ x: 100, y: 100 }, { x: 102, y: 102 })).toBe(false);
    expect(pointerMoved({ x: 100, y: 100 }, { x: 103, y: 103 })).toBe(true);
  });
});
