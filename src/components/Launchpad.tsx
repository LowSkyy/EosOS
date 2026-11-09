import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { columnsFor, filterApps, moveInGrid, nextOffsets, type GridApp } from "../lib/launchpad";

const PAGE = 48;
const TILE = 116;
const GAP = 8;

type Page = { total: number; items: GridApp[] };

/**
 * Launchpad: tutte le app in griglia, in ordine alfabetico, a tutto schermo (è la finestra del Launcher in un'altra modalità).
 * Si scrive per filtrare, frecce per muoversi, Invio apre l'app scelta (o la prima), Esc o un clic nello sfondo chiude.
 * Le app arrivano a pagine da Rust (le prime subito, le icone man mano); la pagina manda solo l'id di un'app che ha ricevuto.
 */
export default function Launchpad({ opens }: { opens: number }) {
  const [apps, setApps] = useState<GridApp[]>([]);
  const [query, setQuery] = useState("");
  const [sel, setSel] = useState(0);
  const [loading, setLoading] = useState(true);
  const icons = useRef(new Map<string, string>());
  const input = useRef<HTMLInputElement>(null);
  const grid = useRef<HTMLDivElement>(null);
  const epoch = useRef(0);

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

  const shown = useMemo(() => filterApps(apps, query), [apps, query]);
  useEffect(() => setSel((s) => Math.min(Math.max(0, s), Math.max(0, shown.length - 1))), [shown.length]);
  useEffect(() => {
    grid.current?.querySelector<HTMLElement>(`[data-i="${sel}"]`)?.scrollIntoView({ block: "nearest" });
  }, [sel]);

  const hide = () => void invoke("launcher_hide");
  const open = async (app: GridApp | undefined) => {
    if (!app) return;
    hide();
    await invoke("launcher_launch", { id: app.id });
  };

  const cols = () => columnsFor(grid.current?.clientWidth ?? 900, TILE, GAP);
  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.nativeEvent.isComposing) return;
    const dir = ({ ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down" } as const)[e.key as "ArrowLeft"];
    if (dir) {
      e.preventDefault();
      setSel((s) => moveInGrid(s, shown.length, cols(), dir));
    } else if (e.key === "Enter") {
      e.preventDefault();
      void open(shown[sel]);
    } else if (e.key === "Escape") {
      e.preventDefault();
      hide();
    }
  };

  return (
    <div className="launchpad" onKeyDown={onKeyDown} onClick={(e) => e.target === e.currentTarget && hide()}>
      <input ref={input} className="launchpad-search" type="search" value={query} placeholder="Cerca" aria-label="Cerca tra le app" spellCheck={false} autoComplete="off" onChange={(e) => setQuery(e.target.value)} />
      <div className="launchpad-grid" ref={grid} role="listbox" aria-label="Tutte le app" onClick={(e) => e.target === e.currentTarget && hide()}>
        {shown.map((a, i) => (
          <button key={a.id} type="button" role="option" aria-selected={i === sel} className="launchpad-app" data-i={i} data-selected={i === sel} title={a.name} onClick={() => void open(a)} onMouseMove={() => i !== sel && setSel(i)}>
            {a.icon ? <img className="launchpad-icon" src={a.icon} alt="" draggable={false} /> : <span className="launchpad-icon launchpad-icon-fallback">{a.name.charAt(0).toUpperCase()}</span>}
            <span className="launchpad-name">{a.name}</span>
          </button>
        ))}
        {!loading && shown.length === 0 && <p className="launchpad-empty">Nessuna app corrisponde.</p>}
      </div>
    </div>
  );
}
