/** Widget «Oggi» nel pannello sotto l'orologio: i tipi che Rust manda e le piccole funzioni pure della pagina. */

export type Reminder = { id: number; text: string; time: string | null; done: boolean };

export type Weather = {
  state: "off" | "nocity" | "ok" | "error";
  place: string;
  temp: number | null;
  min: number | null;
  max: number | null;
  label: string;
  glyph: string;
  message: string | null;
};

/** «HH:MM» di un istante, con le ore a 24 ore. Pura. */
export function hhmm(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** Da fare, con un'ora, e l'ora è già passata (`now` = «HH:MM» di adesso). Pura. */
export function isOverdue(r: Reminder, now: string): boolean {
  return !r.done && r.time !== null && r.time < now;
}

/** Quanti promemoria restano da fare. */
export const openCount = (list: Reminder[]): number => list.filter((r) => !r.done).length;

/** Intestazione del blocco: «Promemoria» o «Promemoria (3)». */
export const remindersTitle = (list: Reminder[]): string => {
  const n = openCount(list);
  return n > 0 ? `Promemoria (${n})` : "Promemoria";
};

/** Il testo del meteo per i lettori di schermo, per esempio «Roma: 10° Nuvoloso, minima 9°, massima 17°». Pura. */
export function weatherSpoken(w: Weather): string {
  if (w.state !== "ok" || w.temp === null || w.min === null || w.max === null) return "";
  return `${w.place}: ${w.temp}° ${w.label}, minima ${w.min}°, massima ${w.max}°`;
}

/** Il messaggio da mostrare al posto del meteo quando non c'è (vuoto = non mostrare niente). Pura. */
export function weatherNotice(w: Weather | null): string {
  if (!w) return "";
  if (w.state === "nocity") return "Meteo acceso: scegli la città nelle Impostazioni.";
  if (w.state === "error") return w.message ?? "Meteo non disponibile.";
  return "";
}

/** Una risposta più vecchia non deve sovrascrivere una più nuova. */
export const isLatest = (mine: number, current: number): boolean => mine === current;
