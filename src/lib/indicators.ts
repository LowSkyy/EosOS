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

export type Volume = { percent: number; muted: boolean; icon: string; title: string };

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
      label: b.label,
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
