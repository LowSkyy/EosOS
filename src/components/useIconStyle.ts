import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Theme } from "./iconart";

export type IconStyle = { mac: boolean; theme: Theme; /** Semafori a sinistra (macOS) o a destra: lo stesso lato delle finestre delle app. */ left: boolean };

/** Il tema da una scelta («auto», «light», «dark») e dall'aspetto scuro del sistema. Pura, testata. */
export function resolveTheme(choice: string | undefined, systemDark: boolean): Theme {
  if (choice === "dark") return "dark";
  if (choice === "light") return "light";
  return systemDark ? "dark" : "light";
}

/** Lo stile delle icone (macOS o originali, chiaro o scuro): lo legge da Rust e lo segue quando cambia nelle Impostazioni. */
export function useIconStyle(): IconStyle {
  // Si parte dal tema del sistema (non dal chiaro): cosi' una finestra scura non lampeggia di bianco all'apertura.
  const [st, setSt] = useState<IconStyle>(() => ({ mac: true, left: false, theme: resolveTheme(undefined, typeof window !== "undefined" && !!window.matchMedia?.("(prefers-color-scheme: dark)").matches) }));
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    let choice: string | undefined;
    let mac = true;
    let left = false;
    const apply = () => setSt({ mac, left, theme: resolveTheme(choice, mq.matches) });
    const take = (v: { icons?: boolean; icon_theme?: string; lights_left?: boolean } | null) => {
      if (!v) return;
      mac = v.icons !== false;
      left = v.lights_left === true;
      choice = v.icon_theme;
      apply();
    };
    let unlisten: (() => void) | undefined;
    let gone = false;
    invoke<{ icons?: boolean; icon_theme?: string }>("dock_settings").then(take).catch(() => {});
    listen<{ icons?: boolean; icon_theme?: string }>("dock-settings-changed", (e) => take(e.payload)).then((fn) => (gone ? fn() : (unlisten = fn)));
    mq.addEventListener("change", apply);
    return () => {
      gone = true;
      unlisten?.();
      mq.removeEventListener("change", apply);
    };
  }, []);
  return st;
}
