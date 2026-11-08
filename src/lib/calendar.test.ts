import { describe, expect, it } from "vitest";
import { addDays, dateLabel, daysInMonth, isLeap, isoWeek, monthGrid, monthTitle, msUntilMidnight, parseWeekStart, shiftMonth, todayOf, weekdayHeaders, weekdayIndex, weekdayMon0 } from "./calendar";

describe("leap years and month lengths", () => {
  it("knows the century rules", () => {
    expect([2024, 2000, 2028].every(isLeap)).toBe(true);
    expect([2023, 1900, 2100, 2026].some(isLeap)).toBe(false);
  });
  it("gives the right number of days", () => {
    expect(daysInMonth(2024, 1)).toBe(29);
    expect(daysInMonth(2026, 1)).toBe(28);
    expect(daysInMonth(2026, 10)).toBe(30);
    expect(daysInMonth(2026, 11)).toBe(31);
    expect(daysInMonth(2026, 12)).toBe(31); // dicembre + 1 gira sul mese successivo (gennaio)
    expect(daysInMonth(2026, -1)).toBe(31);
  });
});

describe("weekdays", () => {
  it("starts the week on Monday", () => {
    expect(weekdayMon0(2026, 10, 8)).toBe(6); // 8 novembre 2026 e' domenica
    expect(weekdayMon0(2026, 10, 9)).toBe(0); // lunedi
    expect(weekdayMon0(1970, 0, 1)).toBe(3); // giovedi
    expect(weekdayMon0(2000, 1, 29)).toBe(1); // martedi
    expect(weekdayMon0(1969, 11, 31)).toBe(2); // prima del 1970
  });
  it("writes dates for screen readers", () => {
    expect(dateLabel({ y: 2026, m0: 10, d: 8 })).toBe("domenica 8 novembre 2026");
    expect(dateLabel({ y: 2024, m0: 1, d: 29 })).toBe("giovedì 29 febbraio 2024");
  });
});

describe("ISO weeks", () => {
  it("matches known dates", () => {
    expect(isoWeek({ y: 2026, m0: 0, d: 1 })).toBe(1); // giovedi: settimana 1
    expect(isoWeek({ y: 2025, m0: 11, d: 29 })).toBe(1); // lunedi 29 dic 2025: gia' settimana 1 del 2026
    expect(isoWeek({ y: 2021, m0: 0, d: 1 })).toBe(53); // venerdi 1 gen 2021: ultima settimana del 2020
    expect(isoWeek({ y: 2026, m0: 10, d: 8 })).toBe(45);
    expect(isoWeek({ y: 2026, m0: 11, d: 31 })).toBe(53);
    expect(isoWeek({ y: 2024, m0: 11, d: 30 })).toBe(1); // lunedi 30 dic 2024: settimana 1 del 2025
    expect(isoWeek({ y: 2020, m0: 11, d: 31 })).toBe(53);
  });
});

describe("month arithmetic", () => {
  it("crosses the year in both directions", () => {
    expect(shiftMonth({ y: 2026, m0: 11 }, 1)).toEqual({ y: 2027, m0: 0 });
    expect(shiftMonth({ y: 2026, m0: 0 }, -1)).toEqual({ y: 2025, m0: 11 });
    expect(shiftMonth({ y: 2026, m0: 5 }, 14)).toEqual({ y: 2027, m0: 7 });
    expect(shiftMonth({ y: 2026, m0: 5 }, -30)).toEqual({ y: 2023, m0: 11 });
    expect(shiftMonth({ y: 2026, m0: 5 }, 0)).toEqual({ y: 2026, m0: 5 });
  });
  it("adds days across month and year edges", () => {
    expect(addDays({ y: 2026, m0: 11, d: 31 }, 1)).toEqual({ y: 2027, m0: 0, d: 1 });
    expect(addDays({ y: 2024, m0: 1, d: 28 }, 1)).toEqual({ y: 2024, m0: 1, d: 29 });
    expect(addDays({ y: 2026, m0: 2, d: 1 }, -1)).toEqual({ y: 2026, m0: 1, d: 28 });
  });
  it("titles the month in Italian", () => {
    expect(monthTitle({ y: 2026, m0: 10 })).toBe("novembre 2026");
    expect(monthTitle({ y: 2027, m0: 0 })).toBe("gennaio 2027");
  });
});

describe("monthGrid", () => {
  const today = { y: 2026, m0: 10, d: 8 };
  const grid = monthGrid({ y: 2026, m0: 10 }, today);

  it("is always six rows of seven, Monday first", () => {
    expect(grid).toHaveLength(6);
    for (const r of grid) expect(r.cells).toHaveLength(7);
    // novembre 2026 comincia di domenica: la prima riga parte da lunedi 26 ottobre.
    expect(grid[0].cells[0]).toMatchObject({ y: 2026, m0: 9, d: 26, inMonth: false });
    expect(grid[0].cells[6]).toMatchObject({ y: 2026, m0: 10, d: 1, inMonth: true });
  });
  it("marks exactly one today, in the right cell", () => {
    const t = grid.flatMap((r) => r.cells).filter((c) => c.isToday);
    expect(t).toHaveLength(1);
    expect(t[0]).toMatchObject({ d: 8, m0: 10, inMonth: true });
  });
  it("marks Saturdays and Sundays", () => {
    for (const r of grid) r.cells.forEach((c, i) => expect(c.weekend).toBe(i >= 5));
  });
  it("covers every day of the month once and in order", () => {
    const days = grid.flatMap((r) => r.cells).filter((c) => c.inMonth).map((c) => c.d);
    expect(days).toEqual(Array.from({ length: 30 }, (_, i) => i + 1));
  });
  it("numbers the weeks (ISO) from the Thursday of each row", () => {
    expect(grid.map((r) => r.week)).toEqual([44, 45, 46, 47, 48, 49]);
  });
  it("does not mark today when looking at another month", () => {
    const g = monthGrid({ y: 2026, m0: 11 }, today);
    expect(g.flatMap((r) => r.cells).some((c) => c.isToday)).toBe(false);
  });
  it("handles February of a leap year and a month that fits in five weeks", () => {
    const f = monthGrid({ y: 2024, m0: 1 }, today).flatMap((r) => r.cells).filter((c) => c.inMonth);
    expect(f).toHaveLength(29);
    // febbraio 2027 comincia di lunedi e finisce di domenica: cinque righe piene e la sesta tutta di marzo.
    const g = monthGrid({ y: 2027, m0: 1 }, today);
    expect(g[0].cells[0]).toMatchObject({ m0: 1, d: 1, inMonth: true });
    expect(g[5].cells.every((c) => !c.inMonth)).toBe(true);
  });
  it("labels the first week of the year correctly across the year edge", () => {
    const g = monthGrid({ y: 2026, m0: 0 }, today);
    expect(g[0].week).toBe(1);
    expect(g[0].cells[0]).toMatchObject({ y: 2025, m0: 11, d: 29 });
  });
});

describe("midnight", () => {
  it("counts the time to the next local midnight", () => {
    expect(msUntilMidnight(new Date(2026, 10, 8, 23, 59, 0, 0))).toBe(60_000);
    expect(msUntilMidnight(new Date(2026, 10, 8, 0, 0, 0, 0))).toBe(24 * 3600 * 1000);
    expect(msUntilMidnight(new Date(2026, 10, 8, 23, 59, 59, 999))).toBe(1000); // mai meno di un secondo
  });
  it("reads today from the system clock", () => {
    expect(todayOf(new Date(2026, 10, 8, 14, 30))).toEqual({ y: 2026, m0: 10, d: 8 });
  });
});

describe("week starting on Sunday", () => {
  const today = { y: 2026, m0: 10, d: 8 };
  it("rotates the headers and the weekday index", () => {
    expect(weekdayHeaders("mon")).toEqual(["lun", "mar", "mer", "gio", "ven", "sab", "dom"]);
    expect(weekdayHeaders("sun")).toEqual(["dom", "lun", "mar", "mer", "gio", "ven", "sab"]);
    expect(weekdayIndex(2026, 10, 8, "sun")).toBe(0); // domenica
    expect(weekdayIndex(2026, 10, 9, "sun")).toBe(1); // lunedi
    expect(weekdayIndex(2026, 10, 8, "mon")).toBe(6);
    expect(weekdayIndex(2026, 10, 14, "sun")).toBe(6); // sabato
  });
  it("starts the grid on a Sunday and marks Sunday and Saturday as weekend", () => {
    const g = monthGrid({ y: 2026, m0: 10 }, today, "sun");
    // novembre 2026 comincia di domenica: la prima riga e' gia' quella del 1 novembre.
    expect(g[0].cells[0]).toMatchObject({ y: 2026, m0: 10, d: 1, inMonth: true });
    for (const r of g) r.cells.forEach((c, i) => expect(c.weekend).toBe(i === 0 || i === 6));
    expect(g).toHaveLength(6);
  });
  it("keeps the ISO week of each row (read from its Thursday)", () => {
    const g = monthGrid({ y: 2026, m0: 10 }, today, "sun");
    // la riga 1-7 novembre (dom-sab) ha il giovedi' 5 novembre: settimana ISO 45.
    expect(g[0].week).toBe(45);
    expect(g.map((r) => r.week)).toEqual([45, 46, 47, 48, 49, 50]);
  });
  it("covers every day once and marks one today", () => {
    const g = monthGrid({ y: 2026, m0: 10 }, today, "sun").flatMap((r) => r.cells);
    expect(g.filter((c) => c.inMonth).map((c) => c.d)).toEqual(Array.from({ length: 30 }, (_, i) => i + 1));
    expect(g.filter((c) => c.isToday)).toHaveLength(1);
  });
  it("defaults to Monday and ignores strange values", () => {
    expect(parseWeekStart("sun")).toBe("sun");
    for (const v of ["mon", "", "domenica", null, undefined, 3]) expect(parseWeekStart(v)).toBe("mon");
    expect(monthGrid({ y: 2026, m0: 10 }, today)[0].cells[0]).toMatchObject({ m0: 9, d: 26 });
  });
});
