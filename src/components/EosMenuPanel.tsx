import { forwardRef, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EOS_ITEMS, nextIndex, type EosItem } from "../lib/eosmenu";

type About = { version: string; windows: string };

/**
 * Il «menu mela» di EosOS (a sinistra della barra, nella finestra "panel"): informazioni, impostazioni di EosOS e di Windows,
 * Mission Control, blocca, sospendi, riavvia, spegni, esci. Frecce su/giù per muoversi, Invio per scegliere, Esc per chiudere.
 * La pagina manda solo l'id fisso della voce: Rust fa il resto (e per spegnere o riavviare chiede il «Sì» a Windows).
 */
const EosMenuPanel = forwardRef<HTMLDivElement, { opens: number }>(function EosMenuPanel({ opens }, ref) {
  const [about, setAbout] = useState<About | null>(null);
  const [showAbout, setShowAbout] = useState(false);
  const [error, setError] = useState("");
  const list = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setShowAbout(false);
    setError("");
    invoke<About | null>("eosmenu_about")
      .then(setAbout)
      .catch(() => undefined);
    // Il focus va alla prima voce all'apertura.
    window.setTimeout(() => list.current?.querySelector<HTMLButtonElement>("button")?.focus(), 80);
  }, [opens]);

  const run = async (item: EosItem) => {
    setError("");
    if (item.id === "about") {
      setShowAbout((v) => !v);
      return;
    }
    const ok = await invoke<boolean>("eosmenu_run", { id: item.id }).catch(() => false);
    if (ok && item.id !== "quit") await invoke("panel_hide").catch(() => undefined);
    else if (!ok) setError("Non riesco a farlo.");
  };

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const buttons = Array.from(list.current?.querySelectorAll<HTMLButtonElement>("button") ?? []);
    const cur = buttons.findIndex((b) => b === document.activeElement);
    buttons[nextIndex(cur, buttons.length, e.key === "ArrowDown" ? 1 : -1)]?.focus();
  };

  return (
    <div className="eosmenu" ref={ref} role="menu" aria-label="Menu EosOS" data-phase={opens % 2 ? "a" : "b"} onKeyDown={onKey}>
      <div ref={list}>
        {EOS_ITEMS.map((it, i) => (
          <div key={it.id}>
            {it.separatorBefore && <div className="eosmenu-sep" role="separator" />}
            <button type="button" role="menuitem" className="eosmenu-item" data-danger={it.danger ? "true" : undefined} onClick={() => void run(it)} aria-expanded={it.id === "about" ? showAbout : undefined} data-index={i}>
              <span>{it.label}</span>
              {it.hint && <span className="eosmenu-hint">{it.hint}</span>}
            </button>
            {it.id === "about" && showAbout && (
              <div className="eosmenu-about" role="note">
                <div>EosOS {about?.version ?? ""}</div>
                {about?.windows && <div>{about.windows}</div>}
              </div>
            )}
          </div>
        ))}
      </div>
      {error && (
        <p className="eosmenu-error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
});

export default EosMenuPanel;
