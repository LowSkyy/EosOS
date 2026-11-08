import { describe, expect, it } from "vitest";
import { addName, asList, byKeys, choiceLabel, comboFromEvent, folderName, inSection, moveItem, pickSection, removeName, sectionsOf, type KeyLike, type SettingEntry } from "./settings";

const e = (key: string, section: string, over: Partial<SettingEntry> = {}): SettingEntry => ({
  key,
  section,
  section_label: section.toUpperCase(),
  label: key,
  help: "",
  kind: { type: "bool" },
  live: true,
  value: false,
  ...over,
});
const entries = [e("a.1", "dock"), e("a.2", "dock"), e("b.1", "bar"), e("c.1", "launcher")];

describe("sections", () => {
  it("lists each section once, in the order Rust gives", () => {
    expect(sectionsOf(entries)).toEqual([
      { id: "dock", label: "DOCK" },
      { id: "bar", label: "BAR" },
      { id: "launcher", label: "LAUNCHER" },
    ]);
    expect(sectionsOf([])).toEqual([]);
  });
  it("filters by section keeping the order", () => {
    expect(inSection(entries, "dock").map((x) => x.key)).toEqual(["a.1", "a.2"]);
    expect(inSection(entries, "nope")).toEqual([]);
  });
  it("falls back to the first section when the wanted one does not exist", () => {
    expect(pickSection(entries, "bar")).toBe("bar");
    expect(pickSection(entries, "xx")).toBe("dock");
    expect(pickSection(entries, null)).toBe("dock");
    expect(pickSection([], "dock")).toBe("");
  });
  it("shows search results in the order of the keys and ignores unknown ones", () => {
    expect(byKeys(entries, ["c.1", "zz", "a.2"]).map((x) => x.key)).toEqual(["c.1", "a.2"]);
  });
});

describe("lists", () => {
  it("adds a name once, ignoring case and blanks", () => {
    expect(addName(["Chrome"], " discord ")).toEqual(["Chrome", "discord"]);
    expect(addName(["Chrome"], "CHROME")).toEqual(["Chrome"]);
    expect(addName(["Chrome"], "   ")).toEqual(["Chrome"]);
  });
  it("removes a name", () => {
    expect(removeName(["a", "b"], "a")).toEqual(["b"]);
    expect(removeName(["a"], "z")).toEqual(["a"]);
  });
  it("reads anything that is not a list as empty", () => {
    expect(asList(["a"])).toEqual(["a"]);
    expect(asList(true)).toEqual([]);
    expect(asList(null)).toEqual([]);
  });
});

describe("choiceLabel", () => {
  const kind = { type: "choice" as const, options: [["clear", "Vetro trasparente"], ["solid", "Vetro opaco"]] as [string, string][] };
  it("shows the label of the current option, or the raw id if unknown", () => {
    expect(choiceLabel(kind, "solid")).toBe("Vetro opaco");
    expect(choiceLabel(kind, "boh")).toBe("boh");
    expect(choiceLabel({ type: "bool" }, true)).toBe("");
  });
});

const ev = (over: Partial<KeyLike>): KeyLike => ({ key: "k", code: "KeyK", ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...over });

describe("comboFromEvent", () => {
  it("writes the combination in the order Ctrl, Alt, Shift, Win, key", () => {
    expect(comboFromEvent(ev({ ctrlKey: true, altKey: true }))).toBe("Ctrl+Alt+K");
    expect(comboFromEvent(ev({ ctrlKey: true, shiftKey: true, metaKey: true, altKey: true }))).toBe("Ctrl+Alt+Shift+Win+K");
    expect(comboFromEvent(ev({ code: "ArrowUp", key: "ArrowUp", ctrlKey: true, altKey: true }))).toBe("Ctrl+Alt+Up");
    expect(comboFromEvent(ev({ code: "Space", key: " ", altKey: true }))).toBe("Alt+Space");
    expect(comboFromEvent(ev({ code: "F5", key: "F5", ctrlKey: true }))).toBe("Ctrl+F5");
    expect(comboFromEvent(ev({ code: "Digit7", key: "7", ctrlKey: true }))).toBe("Ctrl+7");
  });
  it("uses the physical key, not the letter a layout produces", () => {
    expect(comboFromEvent(ev({ code: "KeyQ", key: "a", ctrlKey: true }))).toBe("Ctrl+Q");
  });
  it("waits while only modifiers are down", () => {
    for (const key of ["Control", "Alt", "Shift", "Meta"]) expect(comboFromEvent(ev({ key, code: "Left", ctrlKey: true }))).toBeNull();
  });
  it("refuses a bare key or Shift alone, and keys that are not allowed", () => {
    expect(comboFromEvent(ev({}))).toBeNull();
    expect(comboFromEvent(ev({ shiftKey: true }))).toBeNull();
    expect(comboFromEvent(ev({ code: "Enter", key: "Enter", ctrlKey: true }))).toBeNull();
    expect(comboFromEvent(ev({ code: "F13", key: "F13", ctrlKey: true }))).toBeNull();
    expect(comboFromEvent(ev({ code: "Tab", key: "Tab", altKey: true }))).toBeNull();
  });
});

describe("moveItem / folderName", () => {
  it("swaps with the neighbour and stops at the edges", () => {
    expect(moveItem(["a", "b", "c"], 1, -1)).toEqual(["b", "a", "c"]);
    expect(moveItem(["a", "b", "c"], 1, 1)).toEqual(["a", "c", "b"]);
    const l = ["a", "b"];
    expect(moveItem(l, 0, -1)).toBe(l);
    expect(moveItem(l, 1, 1)).toBe(l);
    expect(moveItem(l, 5, 1)).toBe(l);
  });
  it("shows only the last part of a path", () => {
    expect(folderName("D:\\Progetti\\EosOS")).toBe("EosOS");
    expect(folderName("D:\\")).toBe("D:");
    expect(folderName("E:/Dati/")).toBe("Dati");
    expect(folderName("")).toBe("");
  });
});
