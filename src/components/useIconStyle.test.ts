import { describe, expect, it } from "vitest";
import { resolveTheme } from "./useIconStyle";

describe("tema delle icone", () => {
  it("la scelta esplicita vince sul sistema", () => {
    expect(resolveTheme("dark", false)).toBe("dark");
    expect(resolveTheme("light", true)).toBe("light");
  });
  it("in automatico segue il sistema", () => {
    expect(resolveTheme("auto", true)).toBe("dark");
    expect(resolveTheme(undefined, false)).toBe("light");
  });
});
