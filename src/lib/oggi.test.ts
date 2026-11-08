import { describe, expect, it } from "vitest";
import { hhmm, isLatest, isOverdue, openCount, remindersTitle, weatherNotice, weatherSpoken, type Reminder, type Weather } from "./oggi";

const r = (id: number, time: string | null, done = false): Reminder => ({ id, text: `t${id}`, time, done });
const w = (over: Partial<Weather> = {}): Weather => ({ state: "ok", place: "Roma", temp: 10, min: 9, max: 17, label: "Nuvoloso", glyph: "☁️", message: null, ...over });

describe("Oggi widget helpers", () => {
  it("formats the time with two digits on a 24 hour clock", () => {
    expect(hhmm(new Date(2026, 10, 9, 7, 5))).toBe("07:05");
    expect(hhmm(new Date(2026, 10, 9, 23, 59))).toBe("23:59");
    expect(hhmm(new Date(2026, 10, 9, 0, 0))).toBe("00:00");
  });
  it("marks only open timed reminders in the past as overdue", () => {
    expect(isOverdue(r(1, "08:00"), "09:00")).toBe(true);
    expect(isOverdue(r(1, "09:00"), "09:00")).toBe(false);
    expect(isOverdue(r(1, "10:00"), "09:00")).toBe(false);
    expect(isOverdue(r(1, "08:00", true), "09:00")).toBe(false);
    expect(isOverdue(r(1, null), "09:00")).toBe(false);
  });
  it("counts what is left to do", () => {
    expect(openCount([])).toBe(0);
    expect(openCount([r(1, null), r(2, null, true), r(3, "09:00")])).toBe(2);
    expect(remindersTitle([])).toBe("Promemoria");
    expect(remindersTitle([r(1, null, true)])).toBe("Promemoria");
    expect(remindersTitle([r(1, null), r(2, null)])).toBe("Promemoria (2)");
  });
  it("speaks the weather and explains what is missing", () => {
    expect(weatherSpoken(w())).toBe("Roma: 10° Nuvoloso, minima 9°, massima 17°");
    expect(weatherSpoken(w({ state: "off" }))).toBe("");
    expect(weatherSpoken(w({ temp: null }))).toBe("");
    expect(weatherNotice(null)).toBe("");
    expect(weatherNotice(w())).toBe("");
    expect(weatherNotice(w({ state: "off" }))).toBe("");
    expect(weatherNotice(w({ state: "nocity" }))).toContain("scegli la città");
    expect(weatherNotice(w({ state: "error", message: "Non trovo la città «Xx»." }))).toBe("Non trovo la città «Xx».");
    expect(weatherNotice(w({ state: "error", message: null }))).toBe("Meteo non disponibile.");
  });
  it("only the newest answer counts", () => {
    expect(isLatest(2, 2)).toBe(true);
    expect(isLatest(1, 2)).toBe(false);
  });
});
