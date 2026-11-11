import { describe, expect, it } from "vitest";
import { canSubmit, errorText, lockClock, waitText } from "./lock";

describe("lock screen helpers", () => {
  it("writes the time and the day in Italian, day capitalized", () => {
    const c = lockClock(new Date(2026, 10, 11, 21, 7));
    expect(c.time).toBe("21:07");
    expect(c.day).toBe("Mercoledì 11 novembre");
    expect(lockClock(new Date(2026, 0, 1, 0, 5)).time).toBe("00:05");
  });
  it("says how long to wait in seconds and minutes", () => {
    expect(waitText(0)).toBe("");
    expect(waitText(-3)).toBe("");
    expect(waitText(Number.NaN)).toBe("");
    expect(waitText(1)).toBe("Riprova tra 1 secondo");
    expect(waitText(5)).toBe("Riprova tra 5 secondi");
    expect(waitText(59.2)).toBe("Riprova tra 1 minuto"); // 59,2 arrotonda per eccesso a 60 s
    expect(waitText(60)).toBe("Riprova tra 1 minuto");
    expect(waitText(80)).toBe("Riprova tra 1 minuto e 20 secondi");
    expect(waitText(300)).toBe("Riprova tra 5 minuti");
    expect(waitText(121)).toBe("Riprova tra 2 minuti e 1 secondo");
  });
  it("shows the wait after too many errors and a plain message otherwise", () => {
    expect(errorText(0)).toBe("Password errata");
    expect(errorText(5)).toBe("Riprova tra 5 secondi");
  });
  it("only submits a non-empty password, when idle and not waiting", () => {
    expect(canSubmit("a", false, 0)).toBe(true);
    expect(canSubmit("", false, 0)).toBe(false);
    expect(canSubmit("a", true, 0)).toBe(false);
    expect(canSubmit("a", false, 3)).toBe(false);
  });
});
