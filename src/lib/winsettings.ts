/** Le Impostazioni di Windows dentro quelle di EosOS: i tipi che Rust manda e le funzioni pure dell'elenco. */

export type WinPage = { id: string; group: string; label: string; hint: string };

export type WinGroup = { group: string; pages: WinPage[] };

/** Toglie accenti e maiuscole per confrontare senza badarci. */
export const fold = (s: string): string => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();

/** Le pagine che contengono TUTTE le parole cercate (nel nome, nella descrizione o nel gruppo). Senza ricerca, tutte. Pura. */
export function filterPages(pages: WinPage[], query: string): WinPage[] {
  const words = fold(query).split(/\s+/).filter(Boolean);
  if (words.length === 0) return pages;
  return pages.filter((p) => {
    const hay = fold(`${p.label} ${p.hint} ${p.group}`);
    return words.every((w) => hay.includes(w));
  });
}

/** Le pagine raggruppate nell'ordine in cui compaiono (i gruppi di Rust sono già contigui). Pura. */
export function groupPages(pages: WinPage[]): WinGroup[] {
  const out: WinGroup[] = [];
  for (const p of pages) {
    const last = out[out.length - 1];
    if (last && last.group === p.group) last.pages.push(p);
    else out.push({ group: p.group, pages: [p] });
  }
  return out;
}
