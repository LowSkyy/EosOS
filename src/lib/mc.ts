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

/** Cosa si può fare con i desktop virtuali su questa versione di Windows. */
export type McVd = {
  /** Le interfacce non documentate sono usabili (un profilo ha superato la verifica di coerenza). */
  internal: boolean;
  move_window: boolean;
  remove_any: boolean;
  rename: boolean;
  /** Perché le funzioni avanzate non ci sono. */
  reason: string | null;
};

/** La panoramica di un monitor. */
export type McView = {
  index: number;
  scale: number;
  width: number;
  height: number;
  windows: McWindow[];
  desktops: McDesktop[];
  vd: McVd;
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

/** Di quanti pixel si deve muovere il puntatore perché un clic diventi un trascinamento. */
export const DRAG_THRESHOLD = 8;

/** Il puntatore si è mosso abbastanza da iniziare un trascinamento? Pura, testata. */
export function dragStarted(from: { x: number; y: number }, to: { x: number; y: number }, threshold = DRAG_THRESHOLD): boolean {
  return Math.hypot(to.x - from.x, to.y - from.y) >= threshold;
}

/**
 * Si può rimuovere questo desktop? Con il backend interno qualunque (se ce n'è più di uno); senza, solo il
 * corrente (Windows lo chiude con una scorciatoia). Pura, testata.
 */
export function canRemove(vd: Pick<McVd, "remove_any">, d: McDesktop, total: number): boolean {
  return total > 1 && (vd.remove_any || d.current);
}

/** Il nome che si manda a Rust per un desktop rinominato: senza spazi ai bordi, al più 60 caratteri; vuoto = nessuna modifica. Pura, testata. */
export function cleanDesktopName(raw: string): string {
  return [...raw.trim()].slice(0, 60).join("");
}

/** Riga da mostrare sotto la fascia quando le funzioni avanzate dei desktop non ci sono. Pura, testata. */
export function advancedHint(vd: McVd): string | null {
  return vd.internal ? null : "Spostare finestre, rinominare e rimuovere desktop qualsiasi non è disponibile su questa versione di Windows.";
}
