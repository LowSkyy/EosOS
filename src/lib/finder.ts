/** Finder di EosOS: la parte pura (tipi di file, ordinamento, testi). La pagina e' `components/Finder.tsx`. */

import type { FileId, FolderBadge } from "../components/iconart";

export type Entry = { name: string; path: string; dir: boolean; size: number; modified: number; ext: string; hidden: boolean };
export type Listing = { path: string; name: string; parent: string | null; entries: Entry[]; error: string | null };
export type Place = { name: string; path: string; kind: FolderBadge };
export type SortKey = "name" | "modified" | "size" | "kind";
export type Sort = { key: SortKey; asc: boolean };

const KINDS: Record<Exclude<FileId, "folder" | "file">, string[]> = {
  text: ["txt", "log", "md", "ini", "cfg", "rtf"],
  image: ["png", "jpg", "jpeg", "gif", "bmp", "webp", "tif", "tiff", "heic", "svg", "ico"],
  audio: ["mp3", "wav", "flac", "m4a", "ogg", "aac", "wma"],
  video: ["mp4", "mkv", "avi", "mov", "wmv", "webm", "m4v"],
  zip: ["zip", "rar", "7z", "tar", "gz"],
  pdf: ["pdf"],
  word: ["doc", "docx", "odt"],
  excel: ["xls", "xlsx", "csv", "ods"],
  powerpoint: ["ppt", "pptx", "odp"],
  code: ["js", "ts", "tsx", "jsx", "py", "rs", "json", "html", "css", "xml", "cs", "cpp", "c", "h", "java", "sh", "ps1", "bat", "yml", "yaml", "toml"],
};

const BY_EXT = new Map<string, FileId>(Object.entries(KINDS).flatMap(([id, exts]) => exts.map((e) => [e, id as FileId] as [string, FileId])));

/** L'icona disegnata per un'estensione (minuscola, senza punto). Pura, testata. */
export function artFor(ext: string): FileId {
  return BY_EXT.get(ext) ?? "file";
}

/** Di questi file si mostra l'anteprima vera al posto dell'icona. Pura, testata. */
export function hasThumb(ext: string): boolean {
  return ["png", "jpg", "jpeg", "gif", "bmp", "webp", "tif", "tiff", "heic", "mp4", "mkv", "mov", "avi", "wmv", "webm", "m4v"].includes(ext);
}

/** Di questi file si usa l'icona che gli da' Windows (programmi, collegamenti: ognuno ha la sua). Pura, testata. */
export function usesShellIcon(ext: string): boolean {
  return ["exe", "lnk", "url", "msi", "appref-ms"].includes(ext);
}

const KIND_NAMES: Partial<Record<FileId, string>> = { text: "Testo", image: "Immagine", audio: "Audio", video: "Filmato", zip: "Archivio", pdf: "Documento PDF", word: "Documento", excel: "Foglio di calcolo", powerpoint: "Presentazione", code: "Codice" };

/** Il «Tipo» da mostrare nell'elenco. Pura, testata. */
export function kindLabel(e: Pick<Entry, "dir" | "ext">): string {
  if (e.dir) return "Cartella";
  if (e.ext === "exe") return "Applicazione";
  if (e.ext === "lnk" || e.ext === "url") return "Collegamento";
  if (e.ext === "pdf") return "Documento PDF";
  const base = KIND_NAMES[artFor(e.ext)];
  if (!e.ext) return "Documento";
  return base ? `${base} ${e.ext.toUpperCase()}` : `File ${e.ext.toUpperCase()}`;
}

/** Dimensione leggibile (base 1024, virgola all'italiana). Pura, testata. */
export function sizeLabel(bytes: number): string {
  if (bytes < 1024) return `${bytes} byte`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = bytes / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v >= 100 ? Math.round(v) : v.toFixed(1).replace(".", ",")} ${units[i]}`;
}

/** Le voci visibili, nell'ordine scelto: le cartelle sempre prima, poi la chiave (a parita', il nome). Pura, testata. */
export function arrange(entries: Entry[], sort: Sort, showHidden: boolean, query: string): Entry[] {
  const q = query.trim().toLowerCase();
  const cmpName = (a: Entry, b: Entry) => a.name.localeCompare(b.name, "it", { numeric: true, sensitivity: "base" });
  const by: Record<SortKey, (a: Entry, b: Entry) => number> = {
    name: cmpName,
    modified: (a, b) => a.modified - b.modified,
    size: (a, b) => a.size - b.size,
    kind: (a, b) => kindLabel(a).localeCompare(kindLabel(b), "it"),
  };
  return entries
    .filter((e) => (showHidden || !e.hidden) && (!q || e.name.toLowerCase().includes(q)))
    .sort((a, b) => Number(b.dir) - Number(a.dir) || (sort.asc ? 1 : -1) * by[sort.key](a, b) || cmpName(a, b));
}

/** Le briciole del percorso: nome e percorso di ogni livello, dall'unita' in giu'. Pura, testata. */
export function crumbs(path: string): { name: string; path: string }[] {
  const parts = path.split(/[\\/]+/).filter(Boolean);
  const out: { name: string; path: string }[] = [];
  let acc = "";
  parts.forEach((p, i) => {
    acc = i === 0 ? `${p}\\` : `${acc}${acc.endsWith("\\") ? "" : "\\"}${p}`;
    out.push({ name: p, path: acc });
  });
  return out;
}

/** La selezione dopo un clic: semplice, con Ctrl (aggiunge/toglie) o con Maiusc (intervallo dall'ultimo scelto). Pura, testata. */
export function clickSelect(order: string[], selected: string[], anchor: string | null, hit: string, ctrl: boolean, shift: boolean): { selected: string[]; anchor: string } {
  if (shift && anchor && order.includes(anchor)) {
    const [a, b] = [order.indexOf(anchor), order.indexOf(hit)].sort((x, y) => x - y);
    return { selected: order.slice(a, b + 1), anchor };
  }
  if (ctrl) {
    return { selected: selected.includes(hit) ? selected.filter((p) => p !== hit) : [...selected, hit], anchor: hit };
  }
  return { selected: [hit], anchor: hit };
}

/** Il simbolo da imprimere sulla cartella, se e' una delle cartelle dell'utente. Pura, testata. */
export function badgeFor(path: string, places: Place[]): FolderBadge | undefined {
  const norm = (p: string) => p.replace(/[\\/]+$/, "").toLowerCase();
  return places.find((pl) => pl.kind !== "drive" && norm(pl.path) === norm(path))?.kind;
}
