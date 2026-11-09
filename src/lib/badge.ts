/** Badge delle notifiche sulle icone del Dock: la logica pura (Rust manda già il testo pronto, «5» o «99+»). */

export type Badge = { id: string; count: number; label: string };

/** Da elenco a mappa id → testo. I conteggi nulli o i testi vuoti non fanno badge. Pura, testata. */
export function badgeMap(list: Badge[]): Record<string, string> {
  const out: Record<string, string> = {};
  for (const b of list) if (b.count > 0 && b.label) out[b.id] = b.label;
  return out;
}

/** Le due mappe dicono la stessa cosa? Serve a non riscrivere lo stato (e non ridisegnare il Dock) senza motivo. Pura, testata. */
export function sameBadges(a: Record<string, string>, b: Record<string, string>): boolean {
  const ka = Object.keys(a);
  return ka.length === Object.keys(b).length && ka.every((k) => a[k] === b[k]);
}

/** Il nome accessibile dell'icona: con un badge dice anche quante notifiche ci sono («Discord, 3 notifiche»). Pura, testata. */
export function badgeAria(name: string, label?: string): string {
  if (!label) return name;
  const n = parseInt(label, 10);
  const many = label.endsWith("+");
  if (many) return `${name}, più di ${n} notifiche`;
  return `${name}, ${n} ${n === 1 ? "notifica" : "notifiche"}`;
}
