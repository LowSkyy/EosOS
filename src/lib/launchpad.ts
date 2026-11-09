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

/** Quante colonne e righe entrano nell'area `width` x `height` con tessere `tileW` x `tileH` e spazio `gap`; mai meno di 1 x 1. Pura, testata. */
export function layoutFor(width: number, height: number, tileW: number, tileH: number, gap: number): { cols: number; rows: number } {
  return { cols: columnsFor(width, tileW, gap), rows: Math.max(1, Math.floor((height + gap) / (tileH + gap))) };
}

/** Quante pagine servono per `count` app a `perPage` per pagina (almeno 1). Pura, testata. */
export function pageCount(count: number, perPage: number): number {
  return Math.max(1, Math.ceil(count / Math.max(1, perPage)));
}

/** In che pagina sta l'app di posizione `index`. Pura, testata. */
export function pageOf(index: number, perPage: number): number {
  return Math.floor(Math.max(0, index) / Math.max(1, perPage));
}

/** Le app di una pagina. Pura, testata. */
export function pageSlice<T>(items: T[], page: number, perPage: number): T[] {
  const p = Math.max(1, perPage);
  return items.slice(page * p, page * p + p);
}

/**
 * Si muove nella griglia a pagine (`cols` x `rows` per pagina, in ordine per righe): su e giù restano nella pagina; sinistra dal bordo sinistro
 * e destra dal bordo destro passano alla pagina accanto, sulla stessa riga. Non esce dalla fine né dall'inizio. Da fuori (-1) si entra dalla prima.
 * Pura, testata.
 */
export function moveInPages(current: number, count: number, cols: number, rows: number, dir: "left" | "right" | "up" | "down"): number {
  if (count <= 0) return -1;
  if (current < 0) return 0;
  const c = Math.max(1, cols);
  const perPage = c * Math.max(1, rows);
  const page = Math.floor(current / perPage);
  const pos = current % perPage;
  const col = pos % c;
  const row = Math.floor(pos / c);
  const clamp = (i: number) => Math.min(count - 1, Math.max(0, i));
  switch (dir) {
    case "left":
      if (col > 0) return current - 1;
      return page > 0 ? (page - 1) * perPage + row * c + (c - 1) : current;
    case "right": {
      if (col < c - 1) return current + 1 < count ? current + 1 : current;
      const next = (page + 1) * perPage + row * c;
      return next < count ? next : current;
    }
    case "up":
      return row > 0 ? current - c : current;
    case "down": {
      if (row >= rows - 1) return current;
      const below = current + c;
      if (below < count) return below;
      // Sotto non c'è niente: si ferma sull'ultima app della pagina, se sta più in basso.
      const last = clamp(Math.min(count - 1, page * perPage + perPage - 1));
      return last > current ? last : current;
    }
  }
}
