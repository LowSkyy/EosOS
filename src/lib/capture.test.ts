import { describe, expect, it } from "vitest";
import { clampPoint, isSelection, selection, sizeLabel } from "./capture";

describe("Capture selection", () => {
  it("builds the box from two corners in any direction", () => {
    expect(selection({ x: 10, y: 20 }, { x: 110, y: 70 })).toEqual({ x: 10, y: 20, w: 100, h: 50 });
    expect(selection({ x: 110, y: 70 }, { x: 10, y: 20 })).toEqual({ x: 10, y: 20, w: 100, h: 50 });
    expect(selection({ x: 10, y: 70 }, { x: 110, y: 20 })).toEqual({ x: 10, y: 20, w: 100, h: 50 });
    expect(selection({ x: 5, y: 5 }, { x: 5, y: 5 })).toEqual({ x: 5, y: 5, w: 0, h: 0 });
  });
  it("treats a tiny drag as a click, counting physical pixels", () => {
    expect(isSelection({ x: 0, y: 0, w: 2, h: 40 }, 1)).toBe(false);
    expect(isSelection({ x: 0, y: 0, w: 40, h: 3 }, 1)).toBe(false);
    expect(isSelection({ x: 0, y: 0, w: 4, h: 4 }, 1)).toBe(true);
    expect(isSelection({ x: 0, y: 0, w: 3, h: 3 }, 1.5)).toBe(true); // 4,5 px fisici
    expect(isSelection({ x: 0, y: 0, w: 0, h: 0 }, 2)).toBe(false);
  });
  it("shows the size in physical pixels", () => {
    expect(sizeLabel({ x: 0, y: 0, w: 200, h: 100 }, 1.5)).toBe("300 × 150");
    expect(sizeLabel({ x: 0, y: 0, w: 10.4, h: 5.6 }, 1)).toBe("10 × 6");
  });
  it("keeps a point inside the window", () => {
    expect(clampPoint({ x: -5, y: 900 }, 800, 600)).toEqual({ x: 0, y: 600 });
    expect(clampPoint({ x: 400, y: 300 }, 800, 600)).toEqual({ x: 400, y: 300 });
    expect(clampPoint({ x: 10, y: 10 }, -1, -1)).toEqual({ x: 0, y: 0 });
  });
});
