import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { filterApps, layoutFor, moveInPages, nextOffsets, pageCount, pageOf, pageSlice, type GridApp } from "../lib/launchpad";

/** Quante app si chiedono a Rust per volta (le prime arrivano subito, le altre man mano). */
const PAGE = 48;
/** Misure di una tessera (px CSS): larghezza, altezza e spazio. */
const TILE_W = 140;
const TILE_H = 152;
const GAP = 18;

type Page = { total: number; items: GridApp[] };

/**
 * Launchpad: le applicazioni (non processi, file o prompt dei comandi) in griglia a tutto schermo, in ordine alfabetico, a PAGINE come in
 * macOS. Si scrive per filtrare; frecce per muoversi (anche da una pagina all'altra), PagSu/PagGiù o la rotellina o un trascinamento
 * per cambiare pagina, i puntini in basso per saltare; Invio apre l'app scelta (o la prima); Esc cancella la ricerca e poi chiude; un clic
 * nello sfondo chiude. Le app arrivano a pagine da Rust; la pagina manda solo l'id di un'app che ha ricevuto.
 */
export default function Launchpad({ opens }: { opens: number }) {
  const [apps, setApps] = useState<GridApp[]>([]);
  const [query, setQuery] = useState("");
  const [sel, setSel] = useState(0);
  const [loading, setLoading] = useState(true);
  const [area, setArea] = useState({ w: 1100, h: 560 });
  const icons = useRef(new Map<string, string>());
  const input = useRef<HTMLInputElement>(null);
  const viewport = useRef<HTMLDivElement>(null);
  const epoch = useRef(0);
  const wheelAt = useRef(0);
  const drag = useRef<{ x: number; moved: boolean } | null>(null);

  const loadAll = useCallback(async () => {
    const mine = ++epoch.current;
    setLoading(true);
    let all: GridApp[] = [];
    let total = 0;
    let offset = 0;
    do {
      const have = Array.from(icons.current.keys());
      const page = await invoke<Page>("launcher_grid", { offset, limit: PAGE, have }).catch(() => null);
      if (mine !== epoch.current || !page) break;
      total = page.total;
      for (const it of page.items) if (it.icon) icons.current.set(it.id, it.icon);
      all = [...all, ...page.items.map((it) => ({ ...it, icon: it.icon ?? icons.current.get(it.id) ?? null }))];
      setApps(all);
      offset += page.items.length;
      if (page.items.length === 0) break;
    } while (nextOffsets(offset, total, PAGE).length > 0);
    if (mine === epoch.current) setLoading(false);
  }, []);

  // A ogni apertura: campo vuoto e a fuoco, elenco ricaricato.
  useEffect(() => {
    setQuery("");
    setSel(0);
    void loadAll();
    window.setTimeout(() => input.current?.focus(), 60);
  }, [opens, loadAll]);

  // L'area delle pagine segue la finestra.
  useEffect(() => {
    const el = viewport.current;
    if (!el) return;
    const read = () => setArea({ w: el.clientWidth, h: el.clientHeight });
    read();
    const ro = new ResizeObserver(read);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const shown = useMemo(() => filterApps(apps, query), [apps, query]);
  const { cols, rows } = layoutFor(area.w, area.h, TILE_W, TILE_H, GAP);
  const perPage = cols * rows;
  const pages = pageCount(shown.length, perPage);
  const page = Math.min(pageOf(sel, perPage), pages - 1);

  useEffect(() => setSel((s) => Math.min(Math.max(0, s), Math.max(0, shown.length - 1))), [shown.length]);

  const hide = () => void invoke("launcher_hide");
  const open = async (app: GridApp | undefined) => {
    if (!app) return;
    hide();
    await invoke("launcher_launch", { id: app.id });
  };

  /** Va a una pagina: seleziona la stessa posizione (o l'ultima app) della pagina nuova. */
  const goPage = (p: number) => {
    const target = Math.min(Math.max(0, p), pages - 1);
    if (target === page) return;
    const first = target * perPage;
    const keep = first + (sel % perPage);
    setSel(Math.min(keep, shown.length - 1));
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.nativeEvent.isComposing) return;
    const dir = ({ ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down" } as const)[e.key as "ArrowLeft"];
    if (dir) {
      e.preventDefault();
      setSel((s) => moveInPages(s, shown.length, cols, rows, dir));
    } else if (e.key === "PageDown") {
      e.preventDefault();
      goPage(page + 1);
    } else if (e.key === "PageUp") {
      e.preventDefault();
      goPage(page - 1);
    } else if (e.key === "Enter") {
      e.preventDefault();
      void open(shown[sel]);
    } else if (e.key === "Escape") {
      e.preventDefault();
      if (query) setQuery("");
      else hide();
    }
  };

  // Rotellina: una pagina per scatto (con un po' di pausa, perché i trackpad ne mandano a raffica).
  const onWheel = (e: React.WheelEvent) => {
    const now = Date.now();
    if (now - wheelAt.current < 350) return;
    const d = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    if (Math.abs(d) < 8) return;
    wheelAt.current = now;
    goPage(page + (d > 0 ? 1 : -1));
  };

  // Trascinamento orizzontale: oltre 60 px cambia pagina; un clic senza trascinare apre l'app.
  const onPointerDown = (e: React.PointerEvent) => {
    drag.current = { x: e.clientX, moved: false };
  };
  const onPointerMove = (e: React.PointerEvent) => {
    if (drag.current && Math.abs(e.clientX - drag.current.x) > 8) drag.current.moved = true;
  };
  const onPointerUp = (e: React.PointerEvent) => {
    const d = drag.current;
    drag.current = null;
    if (d?.moved && Math.abs(e.clientX - d.x) > 60) goPage(page + (e.clientX < d.x ? 1 : -1));
  };

  const empty = !loading && shown.length === 0;
  return (
    <div className="launchpad" onKeyDown={onKeyDown} onWheel={onWheel} onClick={(e) => e.target === e.currentTarget && hide()}>
      <div className="launchpad-searchbox">
        <svg className="launchpad-searchicon" width="15" height="15" viewBox="0 0 16 16" aria-hidden>
          <circle cx="6.8" cy="6.8" r="4.6" fill="none" stroke="currentColor" strokeWidth="1.6" />
          <path d="M10.4 10.4 14 14" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
        </svg>
        <input ref={input} className="launchpad-search" type="search" value={query} placeholder="Cerca" aria-label="Cerca tra le app" spellCheck={false} autoComplete="off" onChange={(e) => setQuery(e.target.value)} />
      </div>
      <div className="launchpad-viewport" ref={viewport} onClick={(e) => e.target === e.currentTarget && hide()} onPointerDown={onPointerDown} onPointerMove={onPointerMove} onPointerUp={onPointerUp}>
        <div className="launchpad-grid" role="listbox" aria-label="Applicazioni" style={{ transform: `translateX(${-page * 100}%)` }} onClick={(e) => e.target === e.currentTarget && hide()}>
          {Array.from({ length: pages }, (_, p) => (
            <div
              key={p}
              className="launchpad-page"
              data-page={p}
              aria-hidden={p !== page}
              style={{ gridTemplateColumns: `repeat(${cols}, ${TILE_W}px)`, gridTemplateRows: `repeat(${rows}, ${TILE_H}px)`, gap: GAP }}
              onClick={(e) => e.target === e.currentTarget && hide()}
            >
              {pageSlice(shown, p, perPage).map((a, k) => {
                const i = p * perPage + k;
                return (
                  <button
                    key={a.id}
                    type="button"
                    role="option"
                    aria-selected={i === sel}
                    aria-label={a.name}
                    tabIndex={p === page ? 0 : -1}
                    className="launchpad-app"
                    data-i={i}
                    data-selected={i === sel}
                    onClick={() => !drag.current?.moved && void open(a)}
                    onMouseMove={() => i !== sel && setSel(i)}
                  >
                    {a.icon ? <img className="launchpad-icon" src={a.icon} alt="" draggable={false} /> : <span className="launchpad-icon launchpad-icon-fallback">{a.name.charAt(0).toUpperCase()}</span>}
                    <span className="launchpad-name">{a.name}</span>
                  </button>
                );
              })}
            </div>
          ))}
        </div>
        {empty && <p className="launchpad-empty">Nessuna app corrisponde.</p>}
      </div>
      <div className="launchpad-dots" role="tablist" aria-label="Pagine" style={{ visibility: pages > 1 ? "visible" : "hidden" }}>
        {Array.from({ length: pages }, (_, p) => (
          <button key={p} type="button" role="tab" aria-selected={p === page} aria-label={`Pagina ${p + 1} di ${pages}`} className="launchpad-dot" data-active={p === page} tabIndex={-1} onClick={() => goPage(p)} />
        ))}
      </div>
    </div>
  );
}
