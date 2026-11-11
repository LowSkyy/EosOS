/** Schermata di blocco: la logica pura (orologio, testi, attese). */

/** Cosa Rust racconta alla pagina. */
export type LockInfo = { name: string; initial: string; mode: "blank" | "password"; hello: boolean; wait_secs: number };
export type LockAttempt = { ok: boolean; wait_secs: number };

const cap = (s: string): string => (s ? s.charAt(0).toUpperCase() + s.slice(1) : s);

/** L'ora grande e il giorno sopra («Mercoledì 11 novembre»). Pura, testata. */
export function lockClock(date: Date, locale = "it-IT"): { time: string; day: string } {
  const time = date.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit", hour12: false });
  const day = cap(date.toLocaleDateString(locale, { weekday: "long", day: "numeric", month: "long" }));
  return { time, day };
}

/** «Riprova tra 5 secondi», «Riprova tra 1 minuto», «Riprova tra 2 minuti e 20 secondi». Pura, testata. */
export function waitText(secs: number): string {
  const s = Math.max(0, Math.ceil(Number.isFinite(secs) ? secs : 0));
  if (s === 0) return "";
  const sec = (n: number) => `${n} second${n === 1 ? "o" : "i"}`;
  if (s < 60) return `Riprova tra ${sec(s)}`;
  const m = Math.floor(s / 60);
  const r = s % 60;
  const min = `${m} minut${m === 1 ? "o" : "i"}`;
  return r === 0 ? `Riprova tra ${min}` : `Riprova tra ${min} e ${sec(r)}`;
}

/** Il testo sotto il campo dopo un tentativo sbagliato. Pura, testata. */
export function errorText(wait: number): string {
  return wait > 0 ? waitText(wait) : "Password errata";
}

/** Si puo' inviare? Serve almeno un carattere, niente tentativo in corso e nessuna attesa. Pura, testata. */
export function canSubmit(password: string, busy: boolean, wait: number): boolean {
  return password.length > 0 && !busy && wait <= 0;
}

/** Dopo quanti ms senza muovere il mouse ne' toccare i tasti la schermata torna al solo orologio. */
export const IDLE_BACK_MS = 15000;
