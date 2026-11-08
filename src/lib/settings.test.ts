import { describe, expect, it } from "vitest";
import { addName, asList, byKeys, choiceLabel, inSection, pickSection, removeName, sectionsOf, type SettingEntry } from "./settings";

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
