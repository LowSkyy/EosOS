/** Formatta data e ora per la barra, es. "ven 7 nov  14:32" (locale it-IT). */
export function formatClock(date: Date, locale = "it-IT"): string {
  const day = new Intl.DateTimeFormat(locale, {
    weekday: "short",
    day: "numeric",
    month: "short",
  })
    .format(date)
    .replace(/\./g, "");
  const time = new Intl.DateTimeFormat(locale, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
  return `${day}  ${time}`;
}

/** Millisecondi da aspettare per arrivare all'inizio del minuto successivo. */
export function msToNextMinute(now: Date): number {
  return 60_000 - (now.getSeconds() * 1000 + now.getMilliseconds());
}
