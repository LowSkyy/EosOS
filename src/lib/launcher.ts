export type ItemKind = "app" | "action" | "calc";

/** Scheda attiva: solo le app, solo le azioni, o tutto (`null`). */
export type Filter = "app" | "action" | null;

/** Un risultato del Launcher, come lo manda Rust (`launcher_search`). */
export type LauncherItem = {
  id: string;
  name: string;
  /** "app", "action" (azione di sistema) o "calc" (risultato di un calcolo). */
  kind: ItemKind;
  /** Testo secondario: per il calcolo, l'espressione digitata. */
  detail?: string | null;
  /** Data URL dell'icona; `null` se il frontend l'ha già (vedi `mergeIcons`). */
  icon: string | null;
};

/** Altezza della riga del campo di ricerca e di ogni risultato, in px CSS (vedi styles.css). */
export const INPUT_HEIGHT = 72;
export const ROW_HEIGHT = 52;
export const LIST_PADDING = 8;
/** Limiti dell'altezza della finestra: devono coincidere con `MIN_HEIGHT`/`MAX_HEIGHT` di launcherwin.rs. */
export const MAX_HEIGHT = 640;

/**
 * Nuova riga selezionata dopo un tasto freccia (`delta` = +1 o -1), con giro da un capo
 * all'altro. Con l'elenco vuoto la selezione resta 0. Pura, testata.
 */
export function moveSelection(index: number, delta: number, len: number): number {
  if (len <= 0) return 0;
  return (((index + delta) % len) + len) % len;
}

/** Tiene la selezione dentro l'elenco quando i risultati cambiano. Pura, testata. */
export function clampSelection(index: number, len: number): number {
  if (len <= 0) return 0;
  return Math.min(Math.max(index, 0), len - 1);
}

/**
 * Altezza della finestra per `count` risultati. Con zero risultati ma una ricerca in corso
 * serve una riga per dire che non c'è nulla; con campo vuoto e nessun risultato resta solo il
 * campo. Pura, testata.
 */
export function windowHeight(count: number, hasQuery: boolean): number {
  const rows = count > 0 ? count : hasQuery ? 1 : 0;
  if (rows === 0) return INPUT_HEIGHT;
  return Math.min(INPUT_HEIGHT + LIST_PADDING * 2 + rows * ROW_HEIGHT, MAX_HEIGHT);
}

/** Icone già ricevute: id -> data URL. Solo cresce (i risultati cambiano a ogni tasto). */
export type IconCache = Map<string, string>;

/**
 * Memorizza le icone nuove e riempie con la cache quelle che Rust non ha rimandato perché già
 * note. Pura nei confronti di `items` (ne restituisce una copia), testata.
 */
export function mergeIcons(items: LauncherItem[], cache: IconCache): LauncherItem[] {
  for (const it of items) if (it.icon) cache.set(it.id, it.icon);
  return items.map((it) => ({ ...it, icon: it.icon ?? cache.get(it.id) ?? null }));
}

/** Gli id di cui si ha già l'icona, da dire a Rust perché non li rimandi. */
export function knownIds(cache: IconCache): string[] {
  return [...cache.keys()];
}

/** Iniziale da mostrare al posto di un'icona mancante. Pura, testata. */
export function fallbackLetter(name: string): string {
  const c = [...name.trim()][0];
  return c ? c.toUpperCase() : "?";
}

/**
 * Scheda dopo aver premuto la scorciatoia `wanted` (Ctrl+1 = app, Ctrl+2 = azioni): ripremere
 * quella già attiva torna a "tutto". Pura, testata.
 */
export function toggleFilter(current: Filter, wanted: Exclude<Filter, null>): Filter {
  return current === wanted ? null : wanted;
}

/** Scheda associata a un tasto numerico con Ctrl, o `undefined` se non è una scheda. Pura, testata. */
export function filterForKey(key: string): Exclude<Filter, null> | undefined {
  return key === "1" ? "app" : key === "2" ? "action" : undefined;
}

/** Simbolo al posto dell'icona per le voci che non hanno un'icona di app. Pura, testata. */
export function glyphFor(item: Pick<LauncherItem, "id" | "kind" | "name">): string {
  if (item.kind === "calc") return "=";
  const byId: Record<string, string> = {
    "action:lock": "🔒",
    "action:sleep": "🌙",
    "action:restart": "↻",
    "action:shutdown": "⏻",
    "action:settings": "⚙",
    "action:downloads": "⬇",
    "action:documents": "📄",
    "action:desktop": "🖥",
  };
  return byId[item.id] ?? fallbackLetter(item.name);
}

/** Etichetta a destra della riga. Pura, testata. */
export function kindLabel(item: Pick<LauncherItem, "kind">, confirming: boolean): string {
  if (confirming) return "Invio per confermare";
  return item.kind === "calc" ? "Invio per copiare" : item.kind === "action" ? "Azione" : "App";
}
