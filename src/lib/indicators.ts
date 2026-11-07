/** Stato degli indicatori di sistema, come lo manda Rust (`indicators_get`, evento `indicators-changed`). */
export type Battery = {
  percent: number;
  charging: boolean;
  plugged: boolean;
  minutes_left: number | null;
  icon: string;
  /** Sempre la percentuale ("85%"). */
  label: string;
  title: string;
};

export type Network = {
  kind: "wifi" | "ethernet" | "cellular" | "none";
  online: "internet" | "limited" | "none";
  name: string | null;
  bars: number | null;
  icon: string;
  title: string;
};

export type Volume = {
  percent: number;
  muted: boolean;
  /** Nome del dispositivo di uscita predefinito, se noto. */
  device: string | null;
  icon: string;
  title: string;
};

export type IndicatorsState = {
  battery: Battery | null;
  network: Network | null;
  volume: Volume | null;
};

export type IndicatorKey = "battery" | "network" | "volume";

/** Una voce da mostrare nella barra. */
export type IndicatorItem = {
  key: IndicatorKey;
  icon: string;
  /** Testo accanto all'icona (solo la batteria: la percentuale, sempre visibile). */
  label: string | null;
  title: string;
  /** Va evidenziata (batteria quasi scarica, rete assente o senza Internet). */
  alert: boolean;
};

/**
 * Le voci da mostrare, nell'ordine della barra di macOS (batteria, rete, audio). Un indicatore
 * assente (desktop senza batteria, nessuna uscita audio) non compare. Pura, testata.
 */
export function indicatorItems(s: IndicatorsState | null): IndicatorItem[] {
  if (!s) return [];
  const out: IndicatorItem[] = [];
  if (s.battery) {
    const b = s.battery;
    out.push({
      key: "battery",
      icon: b.icon,
      // Con la percentuale spenta nelle impostazioni Rust manda l'etichetta vuota: non si mostra nulla.
      label: b.label || null,
      title: b.title,
      alert: b.icon === "battery-critical",
    });
  }
  if (s.network) {
    const n = s.network;
    out.push({ key: "network", icon: n.icon, label: null, title: n.title, alert: n.online !== "internet" });
  }
  if (s.volume) {
    out.push({ key: "volume", icon: s.volume.icon, label: null, title: s.volume.title, alert: false });
  }
  return out;
}

/** L'id che Rust si aspetta per aprire le Impostazioni dell'indicatore. Pura, testata. */
export function openId(key: IndicatorKey): "battery" | "network" | "sound" {
  return key === "volume" ? "sound" : key;
}

/** Quanto è piena la batteria (0..1), anche con valori assurdi. Pura, testata. */
export function batteryFill(percent: number): number {
  return Number.isFinite(percent) ? Math.min(Math.max(percent, 0), 100) / 100 : 0;
}

/** Quante onde Wi-Fi sono accese (0-3) per un'icona di rete, o `null` se non è un'icona Wi-Fi. */
export function wifiWaves(icon: string): number | null {
  switch (icon) {
    case "wifi-1":
      return 1;
    case "wifi-2":
      return 2;
    case "wifi-3":
      return 3;
    case "wifi-alert":
      return 0;
    default:
      return null;
  }
}

/** Quante onde ha l'altoparlante (0-3) per un'icona di volume. Pura, testata. */
export function volumeWaves(icon: string): number {
  switch (icon) {
    case "volume-1":
      return 1;
    case "volume-2":
      return 2;
    case "volume-3":
      return 3;
    default:
      return 0;
  }
}

/** Tutte le icone che Rust può mandare (da tenere allineate a `indicators.rs`; il test le controlla). */
export const KNOWN_ICONS = [
  "battery-critical",
  "battery-low",
  "battery-half",
  "battery-high",
  "battery-full",
  "battery-charging",
  "battery-plugged",
  "wifi-1",
  "wifi-2",
  "wifi-3",
  "wifi-alert",
  "ethernet",
  "ethernet-alert",
  "cellular",
  "net-off",
  "volume-0",
  "volume-1",
  "volume-2",
  "volume-3",
  "volume-muted",
] as const;

/** Un valore di volume come lo vede l'utente: percentuale e silenzio. */
export type VolumeValue = { percent: number; muted: boolean };

/** Dopo un tuo gesto sul cursore o sul silenzio, per quanto si ignora lo stato che arriva dal sistema. */
export const INPUT_HOLD_MS = 2500;

/**
 * Quale volume mostrare nel pannello: subito dopo un gesto dell'utente vale il suo valore (lo stato
 * del sistema arriva con un ritardo di circa 1,5 s e farebbe tornare il cursore indietro); poi
 * quello del sistema, così un cambio fatto altrove (tasti del portatile, cuffie) si vede. Pura, testata.
 */
export function reconcileVolume(local: VolumeValue | null, incoming: VolumeValue | null, sinceInputMs: number): VolumeValue | null {
  if (local && sinceInputMs < INPUT_HOLD_MS) return local;
  return incoming ?? local;
}

/**
 * Manda al massimo un valore ogni `ms` millisecondi, ma l'ULTIMO valore arriva sempre: trascinando
 * il cursore non si inonda il sistema di richieste, e il valore finale non si perde. Testata con
 * timer finti.
 */
export function makeLatestSender<T>(send: (v: T) => void, ms: number): (v: T) => void {
  let last = Number.NEGATIVE_INFINITY;
  let pending: { v: T } | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  const flush = () => {
    timer = null;
    if (pending) {
      const { v } = pending;
      pending = null;
      last = Date.now();
      send(v);
    }
  };
  return (v: T) => {
    pending = { v };
    const wait = last + ms - Date.now();
    if (wait <= 0) {
      if (timer) clearTimeout(timer);
      flush();
    } else if (!timer) {
      timer = setTimeout(flush, wait);
    }
  };
}

/** Una riga etichetta/valore del pannello. */
export type DetailRow = { label: string; value: string };

/** Righe di dettaglio della rete per il pannello. Pura, testata. */
export function networkDetails(n: Network): DetailRow[] {
  const kind = { wifi: "Wi-Fi", ethernet: "Cavo di rete", cellular: "Rete mobile", none: "Nessuna" }[n.kind];
  const state = { internet: "Connesso a Internet", limited: "Connesso, senza Internet", none: "Non connesso" }[n.online];
  const rows: DetailRow[] = [];
  if (n.kind !== "none") rows.push({ label: "Rete", value: n.name?.trim() || kind });
  rows.push({ label: "Tipo", value: kind });
  rows.push({ label: "Stato", value: state });
  if (n.kind === "wifi" && n.bars !== null) rows.push({ label: "Segnale", value: `${n.bars} su 5` });
  return rows;
}

/** Righe di dettaglio della batteria per il pannello. Pura, testata. */
export function batteryDetails(b: Battery): DetailRow[] {
  const state = b.charging ? "In carica" : b.plugged ? "Collegata alla rete" : "A batteria";
  const rows: DetailRow[] = [{ label: "Stato", value: state }];
  if (b.minutes_left !== null && !b.plugged) {
    const h = Math.floor(b.minutes_left / 60);
    const m = b.minutes_left % 60;
    rows.push({ label: "Tempo rimanente", value: h > 0 ? (m > 0 ? `${h} h ${m} min` : `${h} h`) : `${m} min` });
  }
  return rows;
}

/** Titolo del pannello per ogni indicatore. */
export function panelTitle(key: IndicatorKey): string {
  return key === "battery" ? "Batteria" : key === "network" ? "Rete" : "Audio";
}

/** Etichetta del pulsante che apre le Impostazioni di Windows. */
export function settingsLabel(key: IndicatorKey): string {
  return key === "battery" ? "Impostazioni batteria…" : key === "network" ? "Impostazioni di rete…" : "Impostazioni audio…";
}
