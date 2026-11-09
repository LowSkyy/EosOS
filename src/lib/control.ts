/** Centro di controllo: la logica pura degli interruttori del pannello. */

export type ControlState = { wifi: boolean | null; bluetooth: boolean | null; awake: boolean };

export type TileId = "wifi" | "bluetooth" | "dnd" | "awake";

export type Tile = { id: TileId; label: string; on: boolean; sub: string };

/**
 * Gli interruttori da mostrare, nell'ordine in cui si vedono. Wi-Fi e Bluetooth ci sono solo se il PC li ha (`null` = non
 * disponibile: niente interruttore, non uno spento che non funziona). Non disturbare e Mantieni sveglio ci sono sempre. Pura, testata.
 */
export function tiles(state: ControlState | null, dnd: boolean): Tile[] {
  const out: Tile[] = [];
  if (state?.wifi != null) out.push({ id: "wifi", label: "Wi-Fi", on: state.wifi, sub: state.wifi ? "Attivo" : "Disattivato" });
  if (state?.bluetooth != null) out.push({ id: "bluetooth", label: "Bluetooth", on: state.bluetooth, sub: state.bluetooth ? "Attivo" : "Disattivato" });
  out.push({ id: "dnd", label: "Non disturbare", on: dnd, sub: dnd ? "Acceso" : "Spento" });
  out.push({ id: "awake", label: "Mantieni sveglio", on: !!state?.awake, sub: state?.awake ? "Acceso" : "Spento" });
  return out;
}

/** Lo stato dopo aver cambiato l'interruttore `id` (per mostrarlo subito, prima della risposta di Rust). Pura, testata. */
export function withToggled(state: ControlState, id: "wifi" | "bluetooth" | "awake", on: boolean): ControlState {
  return { ...state, [id]: on };
}

/** Il titolo del controllo per i lettori di schermo. Pura. */
export function tileTitle(t: Tile): string {
  return `${t.label}: ${t.on ? "acceso" : "spento"}`;
}
