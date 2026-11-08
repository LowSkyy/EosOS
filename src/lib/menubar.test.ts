import { describe, expect, it } from "vitest";
import { acceptRow, isOpenable, type MbMenu, type MbRow } from "./menubar";

const menu = (over: Partial<MbMenu> = {}): MbMenu => ({ title: "File", items: [], bold: false, lazy: false, ...over });
const item = (over = {}) => ({ title: "Apri", shortcut: null, enabled: true, checked: false, separator: false, action: null, children: [], ...over });
const row = (over: Partial<MbRow> = {}): MbRow => ({ hwnd: 1, name: "App", menus: [], read: false, ...over });

describe("isOpenable", () => {
  it("only menus with something to show can be opened", () => {
    expect(isOpenable(menu())).toBe(false);
    expect(isOpenable(menu({ items: [item({ separator: true })] }))).toBe(false);
    expect(isOpenable(menu({ items: [item()] }))).toBe(true);
    expect(isOpenable(menu({ lazy: true }))).toBe(true);
  });
});

describe("acceptRow", () => {
  it("accepts anything when there is nothing yet", () => {
    expect(acceptRow(null, row())).toBe(true);
  });
  it("never replaces a read row of the same window with its fallback", () => {
    expect(acceptRow(row({ read: true }), row({ read: false }))).toBe(false);
    expect(acceptRow(row({ read: false }), row({ read: true }))).toBe(true);
  });
  it("a different window always wins", () => {
    expect(acceptRow(row({ hwnd: 1, read: true }), row({ hwnd: 2, read: false }))).toBe(true);
  });
});
