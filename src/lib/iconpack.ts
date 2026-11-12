/** Pacchetto di icone in stile macOS per il Dock: quali app di sistema hanno un'icona disegnata apposta. */

export type PackId = "finder" | "settings" | "notes" | "calculator" | "paint" | "terminal" | "monitor" | "snip" | "store" | "photos" | "camera" | "clock";

/** Nome del file senza percorso né estensione, minuscolo. Pura, testata. */
export function exeStem(path: string): string {
  const file = path.split(/[\\/]/).pop() ?? "";
  return file.replace(/\.(exe|lnk)$/i, "").toLowerCase();
}

const BY_STEM: Record<string, PackId> = {
  explorer: "finder",
  systemsettings: "settings",
  notepad: "notes",
  calc: "calculator",
  calculatorapp: "calculator",
  calculator: "calculator",
  mspaint: "paint",
  paint: "paint",
  cmd: "terminal",
  powershell: "terminal",
  pwsh: "terminal",
  windowsterminal: "terminal",
  wt: "terminal",
  taskmgr: "monitor",
  snippingtool: "snip",
  screenclippinghost: "snip",
  winstore_app: "store",
  windowscamera: "camera",
  microsoftphotos: "photos",
  photos: "photos",
};

/** Per nome visibile (le app dello Store non hanno un eseguibile utile). */
const BY_NAME: [RegExp, PackId][] = [
  [/^(esplora (file|risorse)|file explorer|explorer)$/i, "finder"],
  [/^(impostazioni|settings)$/i, "settings"],
  [/^(blocco note|notepad)$/i, "notes"],
  [/^(calcolatrice|calculator)$/i, "calculator"],
  [/^paint$/i, "paint"],
  [/^(terminale|terminal|prompt dei comandi|windows powershell|powershell)$/i, "terminal"],
  [/^(gestione attivit|task manager)/i, "monitor"],
  [/^(strumento di cattura|snipping tool)$/i, "snip"],
  [/^(microsoft store)$/i, "store"],
  [/^(foto|photos)$/i, "photos"],
  [/^(fotocamera|camera)$/i, "camera"],
  [/^(orologio|clock)$/i, "clock"],
];

/** L'icona disegnata apposta per una voce del Dock, se c'è. Pura, testata. */
export function packFor(item: { name: string; path: string }): PackId | null {
  const stem = exeStem(item.path);
  if (BY_STEM[stem]) return BY_STEM[stem];
  const hit = BY_NAME.find(([re]) => re.test(item.name.trim()));
  return hit ? hit[1] : null;
}
