import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { acceptRow, isOpenable, type MbRow } from "../lib/menubar";

/** Il rettangolo di un titolo, nelle unità che Rust si aspetta (px CSS relativi alla finestra della barra). */
function rectArgs(el: HTMLElement) {
  const r = el.getBoundingClientRect();
  return { left: r.left, top: r.top, width: r.width, height: r.height };
}

/** Il nome dell'app in primo piano e la fila dei suoi menu (G3: solo titoli; la tendina arriva in G4). */
export default function AppMenus() {
  const [row, setRow] = useState<MbRow | null>(null);
  const cur = useRef<MbRow | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    const take = (next: MbRow) => {
      if (!acceptRow(cur.current, next)) return;
      cur.current = next;
      setRow(next);
    };
    // Prima registra l'ascolto, poi legge lo stato corrente: nessun evento perso.
    listen<MbRow>("menubar-row", (e) => take(e.payload)).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    invoke<MbRow | null>("menubar_current").then((r) => {
      if (r && !cancelled && !cur.current) take(r);
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  if (!row) return <span className="active-app" />;
  return (
    <nav className="app-menus" aria-label="Menu dell'app">
      {row.menus.map((m, i) => (
        <button
          type="button"
          key={`${i}-${m.title}`}
          className={`app-menu${m.bold ? " app-menu-name" : ""}${isOpenable(m) ? " app-menu-openable" : ""}`}
          disabled={!isOpenable(m)}
          data-menu-index={i}
          onClick={(e) => void invoke("menubar_open", { index: i, ...rectArgs(e.currentTarget) })}
          onMouseEnter={(e) => isOpenable(m) && void invoke("menubar_hover", { index: i, ...rectArgs(e.currentTarget) })}
        >
          {m.title}
        </button>
      ))}
    </nav>
  );
}
