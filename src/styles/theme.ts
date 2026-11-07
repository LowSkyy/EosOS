export type ThemePref = "system" | "light" | "dark";

const STORAGE_KEY = "eosos.theme";

export function isThemePref(v: unknown): v is ThemePref {
  return v === "system" || v === "light" || v === "dark";
}

/** Applica la preferenza: "system" segue Windows (nessun attributo), le altre la forzano. */
export function applyThemePref(pref: ThemePref, root: HTMLElement = document.documentElement) {
  if (pref === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", pref);
}

/** Preferenza iniziale: ?theme=... (utile per prove), poi localStorage, poi sistema. */
export function readThemePref(
  search: string = window.location.search,
  storage: Pick<Storage, "getItem"> = localStorage,
): ThemePref {
  const fromQuery = new URLSearchParams(search).get("theme");
  if (isThemePref(fromQuery)) return fromQuery;
  const stored = storage.getItem(STORAGE_KEY);
  return isThemePref(stored) ? stored : "system";
}

export function saveThemePref(pref: ThemePref, storage: Pick<Storage, "setItem"> = localStorage) {
  storage.setItem(STORAGE_KEY, pref);
}

export function initTheme() {
  applyThemePref(readThemePref());
}
