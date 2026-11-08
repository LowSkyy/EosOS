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
