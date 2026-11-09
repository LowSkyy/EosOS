/** Launchpad (tutte le app in griglia): la logica pura della pagina. */

/** Toglie accenti e maiuscole per confrontare senza badarci. */
export const fold = (s: string): string => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();

export type GridApp = { id: string; name: string; icon?: string | null };

/** Le app che contengono TUTTE le parole cercate nel nome (senza ricerca, tutte), nell'ordine in cui arrivano. Pura. */
export function filterApps<T extends { name: string }>(apps: T[], query: string): T[] {
  const words = fold(query).split(/\s+/).filter(Boolean);
  if (words.length === 0) return apps;
  return apps.filter((a) => {
    const n = fold(a.name);
    return words.every((w) => n.includes(w));
  });
}

/**
 * Si muove nella griglia di `count` voci larga `cols` colonne: sinistra/destra di uno, su/giù di una riga. Non esce dai bordi e,
 * andando giù da un'ultima riga più corta, si ferma sull'ultima voce. Da fuori (-1) si entra dalla prima. Pura, testata.
 */
export function moveInGrid(current: number, count: number, cols: number, dir: "left" | "right" | "up" | "down"): number {
  if (count <= 0) return -1;
  if (current < 0) return 0;
  const c = Math.max(1, cols);
  switch (dir) {
    case "left":
      return Math.max(0, current - 1);
    case "right":
      return Math.min(count - 1, current + 1);
    case "up":
      return current - c >= 0 ? current - c : current;
    case "down": {
      const next = current + c;
      if (next < count) return next;
      // Sotto non c'è niente: se esiste una riga più in basso (più corta) ci si ferma sull'ultima voce.
      return Math.floor(current / c) < Math.floor((count - 1) / c) ? count - 1 : current;
    }
  }
}

/** Le pagine ancora da chiedere a Rust, dato quante app abbiamo e quante ce ne sono in tutto: [offset, ...]. Pura, testata. */
export function nextOffsets(have: number, total: number, page: number): number[] {
  const out: number[] = [];
  for (let o = have; o < total; o += page) out.push(o);
  return out;
}

/** Quante colonne entrano in `width` px con tessere larghe `tile` px e spazio `gap`. Pura. */
export const columnsFor = (width: number, tile: number, gap: number): number => Math.max(1, Math.floor((width + gap) / (tile + gap)));
