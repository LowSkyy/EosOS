/** Impostazioni di Windows dentro EosOS: tipi e logica pura delle schede. */

export type SysInfo = { device: string; user: string; os: string; cpu: string; memory: string; uptime: string };
export type Drive = { letter: string; label: string; total: string; free: string; used_percent: number };
export type Display = { name: string; primary: boolean; width: number; height: number; scale: number; x: number; y: number };
export type Theme = { apps_light: boolean; system_light: boolean; transparency: boolean };
export type Power = { screen_ac: number; screen_dc: number; sleep_ac: number; sleep_dc: number; has_battery: boolean };
export type StartupApp = { name: string; command: string; enabled: boolean };

/** Le scelte dei tempi (minuti; 0 = mai): le stesse di Rust. */
export const TIMEOUTS = [0, 1, 2, 3, 5, 10, 15, 20, 25, 30, 45, 60, 120, 180, 240, 300] as const;

/** «Mai», «1 minuto», «15 minuti», «2 ore»… Pura, testata. */
export function timeoutLabel(minutes: number): string {
  if (minutes === 0) return "Mai";
  if (minutes === 1) return "1 minuto";
  if (minutes < 60) return `${minutes} minuti`;
  const h = minutes / 60;
  return Number.isInteger(h) ? (h === 1 ? "1 ora" : `${h} ore`) : `${minutes} minuti`;
}

/** Le righe di descrizione di uno schermo: «1920 × 1080 · 100%», con «principale» se lo è. Pura, testata. */
export function displayLine(d: Display): string {
  const pct = Math.round(d.scale * 100);
  return `${d.width} × ${d.height} · ${pct}%${d.primary ? " · principale" : ""}`;
}

/** Il nome breve di un comando di avvio: solo il file, senza cartelle né argomenti. Pura, testata. */
export function commandName(cmd: string): string {
  const t = cmd.trim();
  const m = /^"([^"]+)"/.exec(t);
  const path = m ? m[1] : t.split(/\s+-{1,2}\w/)[0];
  const file = path.split(/[\\/]/).pop() ?? path;
  return file.trim() || cmd;
}

/** Le app che partono con Windows, quelle accese prima e poi per nome. Pura, testata. */
export function sortStartup(apps: StartupApp[]): StartupApp[] {
  return [...apps].sort((a, b) => Number(b.enabled) - Number(a.enabled) || a.name.localeCompare(b.name, "it"));
}
