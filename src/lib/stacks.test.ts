import { describe, expect, it } from "vitest";
import { isLatest, sameStacks, stackAria, stackSummary, type StackReply } from "./stacks";

const reply = (over: Partial<StackReply> = {}): StackReply => ({ title: "Download", state: "ok", entries: [], total: 0, ...over });
const entry = (i: number) => ({ id: `stk:${i}`, name: `f${i}`, is_dir: false, icon: null });

describe("stack helpers", () => {
  it("says how many files are not shown", () => {
    expect(stackSummary(null)).toBe("");
    expect(stackSummary(reply({ entries: [entry(0)], total: 1 }))).toBe("");
    expect(stackSummary(reply({ entries: [entry(0)], total: 2 }))).toBe("+1 altro");
    expect(stackSummary(reply({ entries: [entry(0), entry(1)], total: 14 }))).toBe("+12 altri");
    expect(stackSummary(reply({ state: "empty", total: 0 }))).toBe("");
    expect(stackSummary(reply({ state: "missing", total: 5 }))).toBe("");
  });
  it("names the panel for screen readers", () => {
    expect(stackAria(null)).toBe("Pila");
    expect(stackAria(reply({ title: "" }))).toBe("Pila");
    expect(stackAria(reply())).toBe("Pila Download");
  });
  it("compares stack lists", () => {
    const a = [{ index: 0, title: "Download", icon: "x" }];
    expect(sameStacks(a, [{ ...a[0] }])).toBe(true);
    expect(sameStacks(a, [])).toBe(false);
    expect(sameStacks(a, [{ ...a[0], icon: "y" }])).toBe(false);
    expect(sameStacks(a, [{ ...a[0], title: "Doc" }])).toBe(false);
  });
  it("only the newest answer counts", () => {
    expect(isLatest(3, 3)).toBe(true);
    expect(isLatest(2, 3)).toBe(false);
  });
});
