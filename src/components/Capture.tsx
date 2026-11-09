import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { clampPoint, isSelection, selection, sizeLabel, type Box, type Point } from "../lib/capture";

/**
 * Il velo per la selezione della cattura dello schermo (finestra "capture", vista ?view=capture): si trascina un rettangolo; al rilascio la
 * pagina manda a Rust solo il rettangolo (px CSS) e la scala. Esc, il tasto destro o un clic senza trascinare annullano. Rust nasconde il
 * velo prima di scattare, quindi non finisce nell'immagine.
 */
export default function Capture() {
  const root = useRef<HTMLDivElement>(null);
  const start = useRef<Point | null>(null);
  const [box, setBox] = useState<Box | null>(null);

  const cancel = () => {
    start.current = null;
    setBox(null);
    void invoke("capture_cancel");
  };

  useEffect(() => {
    const open = listen("capture-open", () => {
      start.current = null;
      setBox(null);
      root.current?.focus();
    });
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        cancel();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      void open.then((f) => f());
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  const point = (e: React.PointerEvent): Point => clampPoint({ x: e.clientX, y: e.clientY }, window.innerWidth, window.innerHeight);

  const down = (e: React.PointerEvent) => {
    if (e.button !== 0) {
      cancel();
      return;
    }
    try {
      e.currentTarget.setPointerCapture(e.pointerId);
    } catch {
      // Un puntatore non più attivo: si trascina comunque, senza la cattura del puntatore.
    }
    start.current = point(e);
    setBox({ ...start.current, w: 0, h: 0 });
  };
  const move = (e: React.PointerEvent) => {
    if (start.current) setBox(selection(start.current, point(e)));
  };
  const up = (e: React.PointerEvent) => {
    if (!start.current) return;
    const b = selection(start.current, point(e));
    start.current = null;
    const dpr = window.devicePixelRatio || 1;
    if (!isSelection(b, dpr)) {
      cancel();
      return;
    }
    setBox(null);
    void invoke("capture_region", { x: b.x, y: b.y, w: b.w, h: b.h, dpr });
  };

  const dpr = window.devicePixelRatio || 1;
  return (
    <div
      className="capture"
      ref={root}
      tabIndex={-1}
      role="application"
      aria-label="Trascina per scegliere l'area da catturare. Esc per annullare."
      onPointerDown={down}
      onPointerMove={move}
      onPointerUp={up}
      onContextMenu={(e) => {
        e.preventDefault();
        cancel();
      }}
    >
      {!box && <div className="capture-hint">Trascina per scegliere l'area · Esc annulla</div>}
      {box && (
        <>
          <div className="capture-box" style={{ left: box.x, top: box.y, width: box.w, height: box.h }} />
          <div className="capture-size" style={{ left: box.x, top: Math.max(box.y - 26, 4) }}>
            {sizeLabel(box, dpr)}
          </div>
        </>
      )}
    </div>
  );
}
