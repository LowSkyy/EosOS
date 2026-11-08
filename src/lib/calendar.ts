// Calendario sotto l'orologio (N2): tutta la logica delle date, pura e senza dipendere dal fuso né dalla lingua del
// sistema. Le date si trattano come terne (anno, mese 0-11, giorno) e si calcolano con `Date.UTC`, così un cambio
// d'ora legale non sposta mai un giorno. La settimana comincia da lunedì (uso italiano).

export const MONTHS_IT = ["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"];
export const WEEKDAYS_SHORT = ["lun", "mar", "mer", "gio", "ven", "sab", "dom"];
export const WEEKDAYS_LONG = ["lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato", "domenica"];

export type Ymd = { y: number; m0: number; d: number };
export type YearMonth = { y: number; m0: number };

export type Cell = Ymd & {
  /** Il giorno è del mese mostrato (gli altri sono quelli dei mesi vicini, in grigio). */
  inMonth: boolean;
  isToday: boolean;
  /** Sabato o domenica. */
  weekend: boolean;
};

export type Row = { week: number; cells: Cell[] };

export const isLeap = (y: number): boolean => (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0;

export function daysInMonth(y: number, m0: number): number {
  return [31, isLeap(y) ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][((m0 % 12) + 12) % 12];
}

/** Giorni dal 1970-01-01 (indipendente dal fuso). */
const dayNumber = (y: number, m0: number, d: number): number => Math.floor(Date.UTC(y, m0, d) / 86_400_000);

/** Giorno della settimana con lunedì = 0 … domenica = 6. */
export function weekdayMon0(y: number, m0: number, d: number): number {
  // 1970-01-01 era un giovedì (3 con lunedì = 0).
  return (((dayNumber(y, m0, d) + 3) % 7) + 7) % 7;
}

/** Il giorno `delta` giorni dopo (o prima) di quello dato. */
export function addDays(date: Ymd, delta: number): Ymd {
  const t = new Date(Date.UTC(date.y, date.m0, date.d + delta));
  return { y: t.getUTCFullYear(), m0: t.getUTCMonth(), d: t.getUTCDate() };
}

/** Numero di settimana ISO 8601 (la settimana 1 è quella con il primo giovedì dell'anno). */
export function isoWeek(date: Ymd): number {
  // Il giovedì della stessa settimana decide l'anno e il numero.
  const thursday = addDays(date, 3 - weekdayMon0(date.y, date.m0, date.d));
  const jan1 = dayNumber(thursday.y, 0, 1);
  return Math.floor((dayNumber(thursday.y, thursday.m0, thursday.d) - jan1) / 7) + 1;
}

/** Il mese `delta` mesi dopo (o prima), anche attraverso l'anno. */
export function shiftMonth(ym: YearMonth, delta: number): YearMonth {
  const total = ym.y * 12 + ym.m0 + delta;
  return { y: Math.floor(total / 12), m0: ((total % 12) + 12) % 12 };
}

/** «novembre 2026». */
export const monthTitle = (ym: YearMonth): string => `${MONTHS_IT[ym.m0]} ${ym.y}`;

/** «sabato 8 novembre 2026», per i lettori di schermo. */
export function dateLabel(d: Ymd): string {
  return `${WEEKDAYS_LONG[weekdayMon0(d.y, d.m0, d.d)]} ${d.d} ${MONTHS_IT[d.m0]} ${d.y}`;
}

/** Il mese come sei settimane da sette giorni (con i giorni dei mesi vicini a riempire), con i numeri di settimana. */
export function monthGrid(ym: YearMonth, today: Ymd): Row[] {
  const first: Ymd = { y: ym.y, m0: ym.m0, d: 1 };
  const start = addDays(first, -weekdayMon0(ym.y, ym.m0, 1));
  const rows: Row[] = [];
  for (let r = 0; r < 6; r++) {
    const cells: Cell[] = [];
    for (let c = 0; c < 7; c++) {
      const day = addDays(start, r * 7 + c);
      cells.push({ ...day, inMonth: day.y === ym.y && day.m0 === ym.m0, isToday: day.y === today.y && day.m0 === today.m0 && day.d === today.d, weekend: c >= 5 });
    }
    // La settimana ISO di una riga si legge dal suo giovedì.
    rows.push({ week: isoWeek(cells[3]), cells });
  }
  return rows;
}

/** La data di oggi come terna, dall'orologio del sistema. */
export const todayOf = (now: Date): Ymd => ({ y: now.getFullYear(), m0: now.getMonth(), d: now.getDate() });

/** Millisecondi fino alla prossima mezzanotte locale (almeno 1 s, per non girare a vuoto). */
export function msUntilMidnight(now: Date): number {
  const next = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1, 0, 0, 0, 0);
  return Math.max(1000, next.getTime() - now.getTime());
}
