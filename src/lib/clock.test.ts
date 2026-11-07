import { describe, expect, it } from "vitest";
import { formatClock, msToNextMinute } from "./clock";

describe("formatClock", () => {
  it("mostra giorno, data e ora a 24 ore in italiano", () => {
    const d = new Date(2026, 10, 6, 14, 32, 5); // ven 6 novembre 2026
    expect(formatClock(d)).toMatch(/^ven 6 nov {2}14:32$/);
  });

  it("riempie con zero ore e minuti", () => {
    expect(formatClock(new Date(2026, 0, 2, 3, 4))).toMatch(/03:04$/);
  });
});

describe("msToNextMinute", () => {
  it("è 60000 allo scoccare del minuto", () => {
    expect(msToNextMinute(new Date(2026, 0, 1, 10, 0, 0, 0))).toBe(60_000);
  });
  it("calcola il resto del minuto", () => {
    expect(msToNextMinute(new Date(2026, 0, 1, 10, 0, 45, 250))).toBe(14_750);
  });
});
