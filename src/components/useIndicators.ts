import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { IndicatorsState } from "../lib/indicators";

/**
 * Stato di batteria, rete e volume: lo chiede a Rust all'avvio (`indicators_get`) e poi ascolta
 * `indicators-changed`, che parte solo quando qualcosa cambia.
 */
export function useIndicators(): IndicatorsState | null {
  const [state, setState] = useState<IndicatorsState | null>(null);
  useEffect(() => {
    let alive = true;
    invoke<IndicatorsState>("indicators_get")
      .then((s) => alive && setState((prev) => prev ?? s))
      .catch(() => {});
    const un = listen<IndicatorsState>("indicators-changed", (e) => alive && setState(e.payload));
    return () => {
      alive = false;
      void un.then((f) => f());
    };
  }, []);
  return state;
}
