import { describe, expect, it } from "vitest";
import { isThemePref, readThemePref } from "./theme";

const store = (v: string | null) => ({ getItem: () => v });

describe("readThemePref", () => {
  it("usa il sistema di default", () => {
    expect(readThemePref("", store(null))).toBe("system");
  });
  it("legge la preferenza salvata", () => {
    expect(readThemePref("", store("dark"))).toBe("dark");
  });
  it("il parametro ?theme= ha la precedenza", () => {
    expect(readThemePref("?theme=light", store("dark"))).toBe("light");
  });
  it("ignora valori non validi", () => {
    expect(readThemePref("?theme=pippo", store("boh"))).toBe("system");
    expect(isThemePref("pippo")).toBe(false);
  });
});
