/** Voce del Dock, come la restituisce il comando Rust `dock_items`. */
export type DockItem = {
  id: string;
  name: string;
  path: string;
  pinned: boolean;
  running: boolean;
  /** Attiva ma senza finestre (es. ridotta nell'area di notifica). */
  background: boolean;
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
        x.background === y.background &&
        x.icon === y.icon
      );
    })
  );
}

/** Ingrandimento massimo dell'icona sotto il cursore (1 = nessuno). */
export const MAGNIFY_MAX = 1.7;
/** Raggio dell'effetto, in passi tra un'icona e la successiva. */
export const MAGNIFY_REACH = 2.4;

/**
 * Fattore di ingrandimento di un'icona a `distance` px dal cursore: campana
 * a coseno, massima sotto il cursore e 1 oltre `reach` passi (`pitch` = passo
 * tra due icone). Liscia, simmetrica e monotona decrescente.
 */
export function magnifyScale(
  distance: number,
  pitch: number,
  max = MAGNIFY_MAX,
  reach = MAGNIFY_REACH,
): number {
  const radius = reach * pitch;
  const d = Math.abs(distance);
  if (!(radius > 0) || d >= radius) return 1;
  return 1 + (max - 1) * ((1 + Math.cos((Math.PI * d) / radius)) / 2);
}

/** Toglie dall'insieme dei rimbalzi le app ormai aperte o scomparse. Pura. */
export function pruneBouncing(bouncing: ReadonlySet<string>, items: DockItem[]): Set<string> {
  const next = new Set<string>();
  for (const id of bouncing) {
    const it = items.find((i) => i.id === id);
    if (it && !it.running) next.add(id);
  }
  return next;
}

/** Stato del puntino sotto un'icona: con finestre, solo in background, oppure nessuno. */
export function dotState(item: Pick<DockItem, "running" | "background">): "running" | "background" | "none" {
  if (item.running) return "running";
  return item.background ? "background" : "none";
}

/** Impostazioni del Dock salvate da Rust. */
export type DockSettings = { autohide: boolean; glass: number; blur: boolean };

/** Opacità del vetro valida (0..1), con ripiego sullo standard per valori assurdi. */
export function clampGlass(v: unknown): number {
  return typeof v === "number" && Number.isFinite(v) ? Math.min(1, Math.max(0, v)) : 0.7;
}

/** Soglia (px) oltre la quale un clic trascinato diventa un trascinamento. */
export const DRAG_THRESHOLD = 5;

/**
 * Dove cadrebbe l'icona trascinata tra le fissate: quante delle altre icone
 * (`centers`, centri a riposo, in ordine e senza quella trascinata) stanno a
 * sinistra del cursore. Pura, testata.
 */
export function dropTarget(centers: number[], x: number): number {
  return centers.filter((c) => c < x).length;
}

/**
 * Spostamento orizzontale (px) di ogni fissata durante il trascinamento, per
 * aprire il varco nel punto di rilascio. `ids` sono le fissate nell'ordine
 * attuale, `dragged` l'icona trascinata (anche una non fissata) e `target` la
 * posizione tra le altre. Pura, testata.
 */
export function previewOffsets(
  ids: string[],
  dragged: string,
  target: number,
  pitch: number,
): Record<string, number> {
  const rest = ids.filter((i) => i !== dragged);
  const t = Math.min(Math.max(target, 0), rest.length);
  const out: Record<string, number> = {};
  ids.forEach((id, orig) => {
    if (id === dragged) return;
    const r = rest.indexOf(id);
    out[id] = ((r >= t ? r + 1 : r) - orig) * pitch;
  });
  return out;
}

/**
 * Elenco dopo lo spostamento di `id` nella posizione `target` tra le fissate (se
 * non era fissata, lo diventa). Serve a mostrare subito il risultato, senza
 * aspettare Rust. Pura, testata.
 */
export function applyMove(items: DockItem[], id: string, target: number): DockItem[] {
  const moved = items.find((i) => i.id === id);
  if (!moved) return items;
  const pinned = items.filter((i) => i.pinned && i.id !== id);
  const others = items.filter((i) => !i.pinned && i.id !== id);
  const t = Math.min(Math.max(target, 0), pinned.length);
  pinned.splice(t, 0, { ...moved, pinned: true });
  return [...pinned, ...others];
}
