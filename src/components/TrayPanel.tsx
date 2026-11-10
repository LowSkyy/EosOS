import { forwardRef, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type TrayApp = { id: string; name: string; icon: string | null };

/**
 * «App in background» (finestra "panel", dall'icona nella barra in alto): le app che girano ridotte nell'area di notifica (NVIDIA Broadcast, G HUB,
 * AnyDesk…) come il vassoio delle icone nascoste di Windows: celle compatte con la sola icona; il nome dell'app sotto il mouse (o selezionata con la
 * tastiera) compare in una riga fissa sotto la griglia, così il pannello non cambia altezza. Un clic la porta davanti (o la riavvia: le app a istanza
 * unica mostrano la loro finestra). La pagina manda solo l'id che ha ricevuto; il percorso lo ricava Rust. Frecce per muoversi, Invio apre, Esc chiude.
 */
const TrayPanel = forwardRef<HTMLDivElement, { opens: number }>(function TrayPanel({ opens }, ref) {
  const [apps, setApps] = useState<TrayApp[] | null>(null);
  const [error, setError] = useState("");
  const [hot, setHot] = useState<string | null>(null);
  const list = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setError("");
    setApps(null);
    setHot(null);
    invoke<TrayApp[]>("tray_apps")
      .then(setApps)
      .catch(() => setApps([]));
    window.setTimeout(() => list.current?.querySelector<HTMLButtonElement>("button")?.focus(), 120);
  }, [opens]);

  const open = async (a: TrayApp) => {
    setError("");
    const ok = await invoke<boolean>("tray_open", { id: a.id }).catch(() => false);
    if (ok) await invoke("panel_hide").catch(() => undefined);
    else setError("Non riesco ad aprirla: forse si è appena chiusa.");
  };

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key !== "ArrowRight" && e.key !== "ArrowLeft" && e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const buttons = Array.from(list.current?.querySelectorAll<HTMLButtonElement>("button") ?? []);
    const cur = buttons.findIndex((b) => b === document.activeElement);
    const step = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : -1;
    const next = cur < 0 ? (step === 1 ? 0 : buttons.length - 1) : Math.min(buttons.length - 1, Math.max(0, cur + step));
    buttons[next]?.focus();
  };

  const count = apps?.length ?? 0;
  const caption = hot ?? (count > 0 ? `${count} app in background` : "");
  return (
    <div className="tray" ref={ref} role="group" aria-label="App in background" data-phase={opens % 2 ? "a" : "b"} onKeyDown={onKey}>
      <h2 className="panel-title">App in background</h2>
      <div className="tray-grid" ref={list} onMouseLeave={() => setHot(null)}>
        {(apps ?? []).map((a) => (
          <button
            key={a.id}
            type="button"
            className="tray-app"
            aria-label={a.name}
            onClick={() => void open(a)}
            onMouseEnter={() => setHot(a.name)}
            onFocus={() => setHot(a.name)}
            onBlur={() => setHot(null)}
          >
            {a.icon ? (
              <img className="tray-icon" src={a.icon} alt="" draggable={false} />
            ) : (
              <span className="tray-icon tray-icon-fallback" aria-hidden>
                {a.name.charAt(0).toUpperCase()}
              </span>
            )}
          </button>
        ))}
      </div>
      {apps !== null && apps.length === 0 ? (
        <p className="tray-empty">Nessuna app in background.</p>
      ) : (
        <div className="tray-caption" data-hot={hot !== null} aria-live="polite">
          {caption}
        </div>
      )}
      {error && (
        <p className="control-error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
});

export default TrayPanel;
