import { useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

/** Il pulsante del Centro di controllo nella barra (i due cursori): un clic apre o chiude il pannello con gli interruttori rapidi. */
export default function ControlButton() {
  const btn = useRef<HTMLButtonElement>(null);
  const open = () => {
    const r = btn.current?.getBoundingClientRect();
    if (!r) return;
    void invoke("panel_toggle", { key: "control", left: r.left, top: r.top, width: r.width, height: r.height });
  };
  return (
    <button ref={btn} type="button" className="control-btn" aria-label="Centro di controllo" aria-haspopup="dialog" title="Centro di controllo" onClick={open}>
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden>
        <rect x="1.5" y="3" width="13" height="4" rx="2" fill="none" stroke="currentColor" strokeWidth="1.3" />
        <circle cx="5" cy="5" r="1.3" fill="currentColor" />
        <rect x="1.5" y="9" width="13" height="4" rx="2" fill="none" stroke="currentColor" strokeWidth="1.3" />
        <circle cx="11" cy="11" r="1.3" fill="currentColor" />
      </svg>
    </button>
  );
}
