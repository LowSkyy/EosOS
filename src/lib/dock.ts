/** Voce del Dock, come la restituisce il comando Rust `dock_items`. */
export type DockItem = {
  id: string;
  name: string;
  path: string;
  pinned: boolean;
  running: boolean;
  icon: string | null;
};

/** Divide le voci in fissate e non fissate (aperte): tra i due gruppi va un separatore. */
export function splitDock(items: DockItem[]): { pinned: DockItem[]; others: DockItem[] } {
  return {
    pinned: items.filter((i) => i.pinned),
    others: items.filter((i) => !i.pinned),
  };
}

/** Due elenchi sono equivalenti per il rendering? Evita rendering inutili ad ogni sondaggio. */
export function sameItems(a: DockItem[], b: DockItem[]): boolean {
  return (
    a.length === b.length &&
    a.every((x, i) => {
      const y = b[i];
      return (
        x.id === y.id &&
        x.name === y.name &&
        x.pinned === y.pinned &&
        x.running === y.running &&
        x.icon === y.icon
      );
    })
  );
}
