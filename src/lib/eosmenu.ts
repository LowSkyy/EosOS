/** Le voci del menu EosOS (il «menu mela») e la navigazione con le frecce. */

export type EosItem = {
  id: string;
  label: string;
  hint?: string;
  separatorBefore?: boolean;
  danger?: boolean;
};

/** Le voci, nell'ordine in cui si vedono. Gli id sono quelli che Rust conosce (`eosmenu_run`). Una sola: le impostazioni generali di EosOS (che includono quelle di Windows). */
export const EOS_ITEMS: EosItem[] = [{ id: "settings", label: "Impostazioni…" }];

/** Posizione successiva muovendosi di `delta` (+1 giù, -1 su), senza girare oltre le estremità. Da fuori (-1) si entra dal primo/ultimo. Pura. */
export function nextIndex(current: number, count: number, delta: 1 | -1): number {
  if (count <= 0) return -1;
  if (current < 0) return delta === 1 ? 0 : count - 1;
  return Math.min(count - 1, Math.max(0, current + delta));
}
