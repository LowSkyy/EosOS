// Menu globale per app (G3): i tipi della riga di menu che Rust emette e qualche funzione pura per la barra.

export type MbAction = { kind: "eos"; value: string } | { kind: "app"; value: number[] };

export type MbItem = {
  title: string;
  shortcut: string | null;
  enabled: boolean;
  checked: boolean;
  separator: boolean;
  action: MbAction | null;
  children: MbItem[];
};

export type MbMenu = {
  title: string;
  items: MbItem[];
  /** Il menu col nome dell'app (in grassetto). */
  bold: boolean;
  /** Le voci non sono ancora state lette: si leggono quando il menu si apre. */
  lazy: boolean;
  /** Posizione fra i menu letti dall'app (null per quelli fatti da EosOS). */
  source: number | null;
};

export type MbRow = {
  hwnd: number;
  name: string;
  menus: MbMenu[];
  /** false = riga di ripiego, in attesa della lettura del menu dell'app. */
  read: boolean;
};

/** Il menu si può aprire? Quello col solo nome (sul desktop) e i menu senza voci non sono cliccabili. */
export function isOpenable(m: MbMenu): boolean {
  return m.lazy || m.items.some((i) => !i.separator);
}

/** Una riga più nuova sostituisce quella di prima solo se non è di una finestra diversa e più vecchia. */
export function acceptRow(current: MbRow | null, next: MbRow): boolean {
  if (!current) return true;
  // La stessa finestra: una riga letta vince sul ripiego, mai il contrario.
  if (current.hwnd === next.hwnd && current.read && !next.read) return false;
  return true;
}

/** Una riga della tendina, già "appiattita": una voce, il suo percorso nel modello e la profondità. */
export type MbRowItem = { item: MbItem; path: number[]; depth: number };

export const pathKey = (path: number[]): string => path.join(".");

/** Le righe da mostrare: le voci del primo livello e, sotto ogni sottomenu aperto, le sue (rientrate). Pura. */
export function flatRows(items: MbItem[], open: ReadonlySet<string>, base: number[] = [], depth = 0): MbRowItem[] {
  const out: MbRowItem[] = [];
  items.forEach((item, i) => {
    const path = [...base, i];
    out.push({ item, path, depth });
    if (item.children.length > 0 && open.has(pathKey(path))) out.push(...flatRows(item.children, open, path, depth + 1));
  });
  return out;
}

/** Si può selezionare con la tastiera: non un separatore, non una voce disattivata. */
export const selectable = (r: MbRowItem): boolean => !r.item.separator && r.item.enabled;

/** L'indice della prossima riga selezionabile da `from` (-1 = nessuna) nella direzione `dir`, senza girare
 *  in tondo oltre i bordi (su macOS il menu non gira: si ferma). Pura. */
export function moveSelection(rows: MbRowItem[], from: number, dir: 1 | -1): number {
  for (let i = from + dir; i >= 0 && i < rows.length; i += dir) if (selectable(rows[i])) return i;
  return from;
}

/** La prima riga selezionabile (o -1). */
export const firstSelectable = (rows: MbRowItem[]): number => rows.findIndex(selectable);
