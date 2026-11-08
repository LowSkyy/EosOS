// Centro notifiche (N4): i tipi che Rust manda al pannello e qualche funzione pura per mostrarli.

export type Notif = { id: number; title: string; body: string; time_ms: number; ago: string };
export type NotifGroup = { app_id: string; app_name: string; items: Notif[]; hidden: number };
export type NotifReply = { state: "off" | "ok" | "error"; groups: NotifGroup[]; total: number; message: string | null };

/** Ogni quanti millisecondi si rilegge l'elenco mentre il pannello è aperto. */
export const REFRESH_MS = 4000;

/** Il testo del pulsante «×» di una notifica, per i lettori di schermo. */
export const dismissLabel = (g: NotifGroup, n: Notif): string => `Rimuovi la notifica «${n.title || n.body}» di ${g.app_name}`;

/** «+3 altre» sotto un gruppo che ha notifiche oltre il tetto; vuoto se non ce ne sono. */
export const hiddenLabel = (g: NotifGroup): string => (g.hidden > 0 ? `+${g.hidden} ${g.hidden === 1 ? "altra" : "altre"}` : "");

/** L'etichetta del totale per l'intestazione («Notifiche», «Notifiche (7)»). */
export const headingLabel = (total: number): string => (total > 0 ? `Notifiche (${total})` : "Notifiche");

/**
 * Una risposta più vecchia non sovrascrive una più nuova: ogni richiesta ha un numero crescente e vale solo se è
 * ancora l'ultima partita. Serve perché la lettura può essere lenta e le richieste si sovrappongono.
 */
export function isLatest(requested: number, latest: number): boolean {
  return requested === latest;
}

/** Tolta una notifica dall'elenco già mostrato (risposta immediata, prima che la rilettura confermi). Pura. */
export function withoutNotif(groups: NotifGroup[], id: number): NotifGroup[] {
  return groups
    .map((g) => ({ ...g, items: g.items.filter((n) => n.id !== id) }))
    .filter((g) => g.items.length > 0 || g.hidden > 0);
}

/** Tolto un intero gruppo dall'elenco già mostrato. Pura. */
export const withoutApp = (groups: NotifGroup[], appId: string): NotifGroup[] => groups.filter((g) => g.app_id.toLowerCase() !== appId.toLowerCase());
