import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { parseFrame } from "../lib/backdrop";
import { boxStyle, cardLetter, cellTitle, desktopAction, desktopLabel, LABEL_H, neighbour, type McView, type McWindow } from "../lib/mc";

/**
 * Panoramica di Mission Control per UN monitor (finestra "mc-N", vista ?view=mc&m=N). Rust la apre,
 * manda la vista (celle e miniature in px CSS) e disegna le miniature vive sopra questa pagina con
 * `DwmRegisterThumbnail`: per questo qui ogni cella lascia libero il rettangolo della miniatura e
 * disegna etichetta e bordo attorno. Le finestre minimizzate (e quelle senza miniatura) sono schede
 * con l'icona e il titolo. La pagina conosce le finestre solo per `id`: attivare o chiudere lo fa Rust.
 */
export default function Mc() {
  const monitor = Number(new URLSearchParams(window.location.search).get("m") ?? 0);
  const [view, setView] = useState<McView | null>(null);
  const [hover, setHover] = useState<number | null>(null);
  // In chiusura (Rust sta riportando le miniature sulle finestre): sfondo ed etichette si dissolvono.
  const [closing, setClosing] = useState(false);
  const bg = useRef<HTMLCanvasElement>(null);

  // Lo scatto dello sfondo (fatto da Rust prima di mostrare la panoramica) si disegna una volta sola.
  const paintBackdrop = async () => {
    try {
      const f = parseFrame(await invoke<ArrayBuffer>("mc_backdrop", { index: monitor }));
      const c = bg.current;
      if (!f || !c) return;
      c.width = f.w;
      c.height = f.h;
      c.getContext("2d")?.putImageData(new ImageData(f.data, f.w, f.h), 0, 0);
      document.documentElement.setAttribute("data-mc-bg", "on");
    } catch {
      /* senza sfondo resta solo l'attenuazione */
    }
  };

  useEffect(() => {
    const open = listen<McView>("mc-open", (e) => {
      if (e.payload.index !== monitor) return;
      setView(e.payload);
      setHover(null);
      setClosing(false);
      void paintBackdrop();
    });
    const update = listen<McView>("mc-update", (e) => {
      if (e.payload.index === monitor) setView(e.payload);
    });
    const closingEv = listen("mc-closing", () => setClosing(true));
    // Alla chiusura la pagina dimentica titoli, icone e sfondo.
    const close = listen("mc-close", () => {
      setView(null);
      setHover(null);
      setClosing(false);
      document.documentElement.removeAttribute("data-mc-bg");
      const c = bg.current;
      if (c) c.getContext("2d")?.clearRect(0, 0, c.width, c.height);
    });
    return () => {
      for (const p of [open, update, closingEv, close]) void p.then((f) => f());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [monitor]);

  const windows = view?.windows ?? [];

  // Tastiera: Esc chiude, frecce spostano la selezione, Invio attiva la finestra evidenziata.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        void invoke("mc_close");
        return;
      }
      const dir = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down" }[e.key] as "left" | "right" | "up" | "down" | undefined;
      if (dir && windows.length > 0) {
        e.preventDefault();
        const from = hover === null ? -1 : windows.findIndex((w) => w.id === hover);
        if (from < 0) return setHover(windows[0].id);
        const next = neighbour(windows.map((w) => w.cell), from, dir);
        if (next !== null) setHover(windows[next].id);
      } else if (e.key === "Enter" && hover !== null) {
        e.preventDefault();
        void invoke("mc_activate", { id: hover });
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [windows, hover]);

  const activate = (w: McWindow) => void invoke("mc_activate", { id: w.id });

  return (
    <div className="mc-root" data-open={view !== null} data-closing={closing} onClick={() => !closing && void invoke("mc_close")}>
      <canvas className="mc-bg" ref={bg} aria-hidden />
      <div className="mc-dim" aria-hidden />
      {view && view.desktops.length > 0 && (
        <div className="mc-spaces" role="group" aria-label="Desktop virtuali" onClick={(e) => e.stopPropagation()}>
          {view.desktops.map((d, i) => (
            <button
              key={d.id || i}
              type="button"
              className="mc-space"
              data-current={d.current}
              aria-label={desktopLabel(d, i, view.desktops.length)}
              aria-current={d.current}
              onClick={() => {
                const a = desktopAction(d, i);
                void (a.kind === "close" ? invoke("mc_close") : invoke("mc_switch_desktop", { index: a.index }));
              }}
            >
              <span className="mc-space-num">{i + 1}</span>
              <span className="mc-space-name">{d.name}</span>
            </button>
          ))}
          <button type="button" className="mc-space mc-space-new" aria-label="Nuovo desktop" title="Nuovo desktop" onClick={() => void invoke("mc_new_desktop")}>
            +
          </button>
        </div>
      )}
      {windows.map((w) => {
        const card = w.minimized || w.thumb === null;
        return (
          <div
            key={w.id}
            className="mc-cell"
            role="button"
            tabIndex={0}
            aria-label={`${cellTitle(w)} (${w.app})${w.minimized ? ", minimizzata" : ""}`}
            data-id={w.id}
            data-hover={hover === w.id}
            data-minimized={w.minimized}
            data-card={card}
            style={boxStyle(w.cell)}
            onMouseEnter={() => setHover(w.id)}
            onMouseLeave={() => setHover((h) => (h === w.id ? null : h))}
            onClick={(e) => {
              e.stopPropagation();
              activate(w);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                activate(w);
              }
            }}
          >
            {card && (
              <div className="mc-card" style={{ bottom: LABEL_H }}>
                {w.icon ? <img className="mc-card-icon" src={w.icon} alt="" draggable={false} /> : <span className="mc-card-letter">{cardLetter(w.app)}</span>}
                {w.minimized && <span className="mc-card-badge">Minimizzata</span>}
              </div>
            )}
            <div className="mc-label" style={{ height: LABEL_H }}>
              {w.icon ? <img className="mc-label-icon" src={w.icon} alt="" draggable={false} /> : <span className="mc-label-letter">{cardLetter(w.app)}</span>}
              <span className="mc-label-title" title={cellTitle(w)}>{cellTitle(w)}</span>
              <button
                type="button"
                className="mc-close"
                aria-label={`Chiudi ${cellTitle(w)}`}
                title="Chiudi la finestra"
                onClick={(e) => {
                  e.stopPropagation();
                  void invoke("mc_close_window", { id: w.id });
                }}
              >
                \u00d7
              </button>
            </div>
          </div>
        );
      })}
    </div>
  );
}
