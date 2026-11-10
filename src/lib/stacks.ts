/** Pile del Dock: i tipi che Rust manda e le piccole funzioni pure del pannello. */

/** Una pila nel Dock (cartella + icona). */
export type StackItem = { index: number; title: string; icon: string | null };

export type StackEntry = { id: string; name: string; is_dir: boolean; icon: string | null };

/** Contenuto del pannello di una pila. */
export type StackReply = { title: string; state: "ok" | "empty" | "missing"; entries: StackEntry[]; total: number };

/** Due elenchi di pile uguali? Evita rendering inutili ad ogni lettura. */
export function sameStacks(a: StackItem[], b: StackItem[]): boolean {
  return a.length === b.length && a.every((x, i) => x.index === b[i].index && x.title === b[i].title && x.icon === b[i].icon);
}

/** «+12 altri» sotto la griglia quando la cartella ha più file di quelli mostrati; vuoto altrimenti. */
export function stackSummary(r: StackReply | null): string {
  if (!r || r.state !== "ok") return "";
  const hidden = r.total - r.entries.length;
  return hidden > 0 ? `+${hidden} ${hidden === 1 ? "altro" : "altri"}` : "";
}

/** Il nome del pannello per i lettori di schermo. */
export function stackAria(r: StackReply | null): string {
  return r?.title ? `Pila ${r.title}` : "Pila";
}

/** Una risposta più vecchia non deve sovrascrivere una più nuova. */
export const isLatest = (mine: number, current: number): boolean => mine === current;

/**
 * Accorcia ogni «parola» troppo lunga di un nome di file (senza spazi non puo' andare a capo e verrebbe spezzata a meta') tenendo l'inizio e
 * l'estensione: «CheraxLoader.exe» → «Chera….exe». Le parole tutte maiuscole sono piu' larghe e hanno 2 caratteri in meno. Le parole corte e
 * gli spazi restano com'erano. Pura, testata.
 */
export function shortName(name: string, max = 10): string {
  return name
    .split(" ")
    .map((w) => {
      const limit = /^[^a-z]*$/.test(w) ? max - 2 : max;
      if (w.length <= limit) return w;
      const dot = w.lastIndexOf(".");
      const ext = dot > 0 && w.length - dot <= 6 ? w.slice(dot) : "";
      const base = ext ? w.slice(0, dot) : w;
      return base.slice(0, Math.max(2, limit - ext.length - 1)) + "…" + ext;
    })
    .join(" ");
}
