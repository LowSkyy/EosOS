import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type ActiveApp = { name: string; pid: number };

export default function ActiveAppName() {
  const [name, setName] = useState("");

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    // Prima registra l'ascolto, poi legge lo stato corrente: nessun evento perso.
    listen<ActiveApp>("foreground-changed", (e) => setName(e.payload.name)).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    invoke<ActiveApp | null>("get_active_app").then((app) => {
      if (app && !cancelled) setName((cur) => cur || app.name);
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return <span className="active-app">{name}</span>;
}
