/** Una finestra nella panoramica, come la manda Rust: misure in px CSS relativi al monitor. */
export type McWindow = {
  id: number;
  title: string;
  app: string;
  minimized: boolean;
  icon: string | null;
  /** La cella nella griglia: x, y, larghezza, altezza. */
  cell: [number, number, number, number];
  /** Dove Windows disegna la miniatura viva (sopra la pagina); `null` = si disegna una scheda con l'icona. */
  thumb: [number, number, number, number] | null;
};

/** Un desktop virtuale nella fascia in alto. */
export type McDesktop = { id: string; name: string; current: boolean };

/** La panoramica di un monitor. */
export type McView = {
  index: number;
  scale: number;
  width: number;
  height: number;
  windows: McWindow[];
  desktops: McDesktop[];
};

/** Altezza della fascia dell'etichetta in fondo alla cella (px CSS): da tenere uguale a `LABEL_H` di mc.rs. */
export const LABEL_H = 38;

/** Stile di posizionamento assoluto di un rettangolo [x, y, w, h]. Pura, testata. */
export function boxStyle(r: [number, number, number, number]): { left: number; top: number; width: number; height: number } {
  return { left: r[0], top: r[1], width: r[2], height: r[3] };
}

/** Testo da mostrare come titolo di una cella: il titolo della finestra o, se è vuoto, il nome dell'app. Pura, testata. */
export function cellTitle(w: Pick<McWindow, "title" | "app">): string {
  const t = w.title.trim();
  return t !== "" ? t : w.app;
}

/** Iniziale dell'app per la scheda senza icona. Pura, testata. */
export function cardLetter(app: string): string {
  const c = [...app.trim()][0];
  return c ? c.toUpperCase() : "?";
}

/**
 * Indice della cella da selezionare spostandosi con le frecce (`dir`) a partire da `from`: la cella più
 * vicina nella direzione scelta, a pari distanza la più allineata. `null` se non c'è nessuna in quella
 * direzione. Pura, testata.
 */
export function neighbour(cells: [number, number, number, number][], from: number, dir: "left" | "right" | "up" | "down"): number | null {
  const c = cells[from];
  if (!c) return null;
  const cx = c[0] + c[2] / 2;
  const cy = c[1] + c[3] / 2;
  let best: number | null = null;
  let bestScore = Number.POSITIVE_INFINITY;
  cells.forEach((o, i) => {
    if (i === from) return;
    const ox = o[0] + o[2] / 2;
    const oy = o[1] + o[3] / 2;
    const dx = ox - cx;
    const dy = oy - cy;
    const ahead = dir === "right" ? dx > 1 : dir === "left" ? dx < -1 : dir === "down" ? dy > 1 : dy < -1;
    if (!ahead) return;
    const along = dir === "left" || dir === "right" ? Math.abs(dx) : Math.abs(dy);
    const across = dir === "left" || dir === "right" ? Math.abs(dy) : Math.abs(dx);
    const score = along + across * 3;
    if (score < bestScore) {
      bestScore = score;
      best = i;
    }
  });
  return best;
}

/**
 * Cosa fa il clic su un desktop della fascia: sul desktop corrente chiude la panoramica, su un altro ci
 * passa (l'indice è la posizione nell'elenco). Pura, testata.
 */
export function desktopAction(d: McDesktop, index: number): { kind: "close" } | { kind: "switch"; index: number } {
  return d.current ? { kind: "close" } : { kind: "switch", index };
}

/** Nome accessibile di un desktop: numero, nome e se è quello corrente. Pura, testata. */
export function desktopLabel(d: McDesktop, index: number, total: number): string {
  return `${d.name}, desktop ${index + 1} di ${total}${d.current ? ", corrente" : ""}`;
}
