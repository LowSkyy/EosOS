import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** Ogni quanti millisecondi si chiede a Rust quante app girano in background (Rust riusa l'ultima lettura). */
const COUNT_POLL_MS = 5000;

/**
 * Le app in background (area di notifica: NVIDIA Broadcast, G HUB, AnyDesk…) nella barra in alto: un'icona a forma di freccia su con tre
 * puntini, accanto agli indicatori. Compare solo se ce n'è almeno una (e se la voce è accesa nelle Impostazioni); un clic apre o chiude il
 * pannello con l'elenco, sotto l'icona.
 */
export default function TrayButton() {
  const btn = useRef<HTMLButtonElement>(null);
  const [count, setCount] = useState(0);

  useEffect(() => {
    let gone = false;
    const ask = () =>
      invoke<number>("tray_count")
        .then((n) => !gone && setCount((cur) => (cur === n ? cur : n)))
        .catch(() => undefined);
    void ask();
    const t = setInterval(ask, COUNT_POLL_MS);
    return () => {
      gone = true;
      clearInterval(t);
    };
  }, []);

  if (count <= 0) return null;

  const open = () => {
    const r = btn.current?.getBoundingClientRect();
    if (!r) return;
    void invoke("panel_toggle", { key: "tray", left: r.left, top: r.top, width: r.width, height: r.height });
  };
  const label = `App in background (${count})`;
  return (
    <button ref={btn} type="button" className="tray-btn" aria-label={label} aria-haspopup="dialog" title={label} onClick={open}>
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden>
        <path d="M3.5 9.5 8 5l4.5 4.5" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="4.5" cy="12.6" r="1" fill="currentColor" />
        <circle cx="8" cy="12.6" r="1" fill="currentColor" />
        <circle cx="11.5" cy="12.6" r="1" fill="currentColor" />
      </svg>
    </button>
  );
}
