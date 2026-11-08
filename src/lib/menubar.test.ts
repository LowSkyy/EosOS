import { describe, expect, it } from "vitest";
import { acceptRow, firstSelectable, flatRows, isOpenable, moveSelection, pathKey, type MbItem, type MbMenu, type MbRow } from "./menubar";

const menu = (over: Partial<MbMenu> = {}): MbMenu => ({ title: "File", items: [], bold: false, lazy: false, source: null, ...over });
const item = (over: Partial<MbItem> = {}): MbItem => ({ title: "Apri", shortcut: null, enabled: true, checked: false, separator: false, action: null, children: [], ...over });
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

describe("flatRows / selection", () => {
  const sub = item({ title: "Recenti", children: [item({ title: "uno" }), item({ title: "due", enabled: false })] });
  const items = [item({ title: "Nuovo" }), item({ separator: true, title: "" }), sub, item({ title: "Esci" })];

  it("shows submenu entries only when that submenu is open", () => {
    expect(flatRows(items, new Set()).map((r) => r.item.title)).toEqual(["Nuovo", "", "Recenti", "Esci"]);
    const open = flatRows(items, new Set([pathKey([2])]));
    expect(open.map((r) => r.item.title)).toEqual(["Nuovo", "", "Recenti", "uno", "due", "Esci"]);
    expect(open[3]).toMatchObject({ path: [2, 0], depth: 1 });
  });

  it("moves over selectable rows only and stops at the edges", () => {
    const rows = flatRows(items, new Set([pathKey([2])]));
    expect(firstSelectable(rows)).toBe(0);
    expect(moveSelection(rows, 0, 1)).toBe(2); // salta il separatore
    expect(moveSelection(rows, 2, 1)).toBe(3);
    expect(moveSelection(rows, 3, 1)).toBe(5); // salta "due", disattivata
    expect(moveSelection(rows, 5, 1)).toBe(5); // non gira
    expect(moveSelection(rows, 0, -1)).toBe(0);
    expect(moveSelection(rows, 2, -1)).toBe(0);
  });

  it("an empty or all-disabled list has nothing to select", () => {
    expect(firstSelectable([])).toBe(-1);
    expect(firstSelectable(flatRows([item({ enabled: false }), item({ separator: true })], new Set()))).toBe(-1);
  });
});
