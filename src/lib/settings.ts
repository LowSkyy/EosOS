// Impostazioni di EosOS (S2): i tipi che Rust manda alla finestra e qualche funzione pura per mostrarli.

export type SettingValue = boolean | string | string[];

export type SettingKind =
  | { type: "bool" }
  | { type: "choice"; options: [string, string][] }
  | { type: "hotkey" }
  | { type: "names"; max: number }
  | { type: "folders"; max: number };

export type SettingEntry = {
  key: string;
  section: string;
  section_label: string;
  label: string;
  help: string;
  kind: SettingKind;
  /** Cambiarla ha effetto subito (altrimenti al prossimo avvio). */
  live: boolean;
  value: SettingValue | null;
};

export type SetReply = { ok: boolean; live: boolean; error: string | null };

export type SectionInfo = { id: string; label: string };

/** Le sezioni nell'ordine in cui Rust le elenca, senza ripetizioni. */
export function sectionsOf(entries: SettingEntry[]): SectionInfo[] {
  const out: SectionInfo[] = [];
  for (const e of entries) if (!out.some((s) => s.id === e.section)) out.push({ id: e.section, label: e.section_label });
  return out;
}

/** Le impostazioni di una sezione, nell'ordine dell'elenco. */
export const inSection = (entries: SettingEntry[], section: string): SettingEntry[] => entries.filter((e) => e.section === section);

/** Le impostazioni con le chiavi date, nell'ordine in cui le chiavi arrivano (il risultato di una ricerca). */
export function byKeys(entries: SettingEntry[], keys: string[]): SettingEntry[] {
  return keys.map((k) => entries.find((e) => e.key === k)).filter((e): e is SettingEntry => !!e);
}

/** La sezione richiesta se esiste, altrimenti la prima. Pura. */
export function pickSection(entries: SettingEntry[], wanted: string | null | undefined): string {
  const secs = sectionsOf(entries);
  return secs.find((s) => s.id === wanted)?.id ?? secs[0]?.id ?? "";
}

/** L'elenco con `name` in più, se non c'è già (senza badare alle maiuscole) e non è vuoto. Rust ripulisce comunque. */
export function addName(list: string[], name: string): string[] {
  const n = name.trim();
  if (!n || list.some((x) => x.toLowerCase() === n.toLowerCase())) return list;
  return [...list, n];
}

/** L'elenco senza `name`. */
export const removeName = (list: string[], name: string): string[] => list.filter((x) => x !== name);

/** Il valore come lista, per un tipo elenco (un valore assente o di forma strana vale vuoto). */
export const asList = (v: SettingValue | null): string[] => (Array.isArray(v) ? v : []);

/** Il testo da mostrare per una scelta: l'etichetta dell'opzione corrente (o l'id se sconosciuto). */
export function choiceLabel(kind: SettingKind, value: SettingValue | null): string {
  if (kind.type !== "choice" || typeof value !== "string") return "";
  return kind.options.find(([id]) => id === value)?.[1] ?? value;
}
