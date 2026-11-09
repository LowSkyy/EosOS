import { useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * Il pulsante del menu EosOS (il «menu mela»): il simbolo a sinistra della barra. Un clic apre (o chiude) il pannello con le
 * voci di sistema; la posizione la comunica alla finestra del pannello come per gli indicatori.
 */
export default function EosButton() {
  const btn = useRef<HTMLButtonElement>(null);
  const open = () => {
    const r = btn.current?.getBoundingClientRect();
    if (!r) return;
    void invoke("panel_toggle", { key: "eos", left: r.left, top: r.top, width: r.width, height: r.height });
  };
  return (
    <button ref={btn} type="button" className="eos-btn" aria-label="Menu EosOS" aria-haspopup="menu" onClick={open}>
      <svg width="15" height="15" viewBox="0 0 16 16" aria-hidden>
        <circle cx="8" cy="8" r="6.2" fill="none" stroke="currentColor" strokeWidth="1.6" />
        <path d="M8 1.8a6.2 6.2 0 0 0 0 12.4z" fill="currentColor" />
      </svg>
    </button>
  );
}
