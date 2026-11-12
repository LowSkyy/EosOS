import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FILE_ART, FOLDER_GLYPHS, driveArt, folderArt, type FolderBadge, type Theme } from "./iconart";
import { useIconStyle } from "./useIconStyle";
import { arrange, artFor, badgeFor, clickSelect, crumbs, hasThumb, kindLabel, sizeLabel, usesShellIcon, type Entry, type Listing, type Place, type Sort, type SortKey } from "../lib/finder";

/** Finder di EosOS: la finestra per cartelle e file con l'aspetto del Finder di macOS. La parte Rust e' `finder.rs`. */

// ---------------------------------------------------------------- icone e anteprime dalla shell (poche alla volta, ricordate)

const images = new Map<string, string | null>();
const queue: (() => void)[] = [];
let running = 0;

function shellImage(key: string, path: string, size: number, thumb: boolean): Promise<string | null> {
  if (images.has(key)) return Promise.resolve(images.get(key) ?? null);
  return new Promise((resolve) => {
    const run = () => {
      running++;
      invoke<string | null>("finder_image", { path, size, thumb })
        .catch(() => null)
        .then((v) => {
          images.set(key, v);
          resolve(v);
          running--;
          queue.shift()?.();
        });
    };
    if (running < 4) run();
    else queue.push(run);
  });
}

/** L'immagine della shell per una voce, chiesta solo quando la voce entra nella parte visibile. */
function useShellImage(ref: React.RefObject<HTMLElement | null>, entry: Entry, size: number): string | null {
  const thumb = hasThumb(entry.ext);
  const wanted = !entry.dir && (thumb || usesShellIcon(entry.ext));
  const key = `${thumb ? "t" : "i"}|${size}|${entry.path}|${entry.modified}`;
  const [src, setSrc] = useState<string | null>(images.get(key) ?? null);
  useEffect(() => {
    setSrc(images.get(key) ?? null);
    const el = ref.current;
    if (!wanted || images.has(key) || !el) return;
    let live = true;
    const io = new IntersectionObserver((seen) => {
      if (seen.some((s) => s.isIntersecting)) {
        io.disconnect();
        void shellImage(key, entry.path, size, thumb).then((v) => live && setSrc(v));
      }
    });
    io.observe(el);
    return () => {
      live = false;
      io.disconnect();
    };
  }, [key, wanted, entry.path, size, thumb, ref]);
  return wanted ? src : null;
}

function Art({ children }: { children: React.ReactNode }) {
  return (
    <svg viewBox="0 0 100 100" width="100%" height="100%" aria-hidden>
      {children}
    </svg>
  );
}

function EntryIcon({ entry, theme, badge, size }: { entry: Entry; theme: Theme; badge?: FolderBadge; size: number }) {
  const ref = useRef<HTMLSpanElement>(null);
  const src = useShellImage(ref, entry, size);
  return (
    <span ref={ref} className="fd-icon" data-thumb={src && hasThumb(entry.ext) ? "1" : undefined}>
      {entry.dir ? <Art>{folderArt(theme, badge)}</Art> : src ? <img src={src} alt="" draggable={false} /> : <Art>{FILE_ART[artFor(entry.ext)](theme)}</Art>}
    </span>
  );
}

/** Il simbolo di una voce della barra laterale (lo stesso impresso sulle cartelle, piccolo e a tinta unita). */
function PlaceGlyph({ kind }: { kind: FolderBadge }) {
  if (kind === "drive") {
    return (
      <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden>
        <path d="M5 4h10l3 8v3a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 2 15v-3z" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" />
        <path d="M2.5 12h15" stroke="currentColor" strokeWidth="1.5" />
        <circle cx="14.5" cy="14.2" r="1" fill="currentColor" />
      </svg>
    );
  }
  return (
    <svg viewBox="33 43 34 32" width="16" height="16" fill="currentColor" stroke="currentColor" strokeWidth="0" aria-hidden>
      {FOLDER_GLYPHS[kind]}
    </svg>
  );
}

const DATE = new Intl.DateTimeFormat("it-IT", { day: "numeric", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit" });

type Menu = { x: number; y: number; target: Entry | null };

export default function Finder() {
  const { theme } = useIconStyle();
  const first = useMemo(() => new URLSearchParams(window.location.search).get("path") ?? "C:\\", []);
  const [hist, setHist] = useState<{ stack: string[]; i: number }>({ stack: [first], i: 0 });
  const path = hist.stack[hist.i];
  const [listing, setListing] = useState<Listing | null>(null);
  const [places, setPlaces] = useState<Place[]>([]);
  const [view, setView] = useState<"icons" | "list">(() => (localStorage.getItem("finder.view") === "list" ? "list" : "icons"));
  const [sort, setSort] = useState<Sort>({ key: "name", asc: true });
  const [query, setQuery] = useState("");
  const [showHidden, setShowHidden] = useState(false);
  const [sel, setSel] = useState<string[]>([]);
  const anchor = useRef<string | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [menu, setMenu] = useState<Menu | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [clip, setClip] = useState(0);
  const [active, setActive] = useState(true);
  const body = useRef<HTMLDivElement>(null);
  const search = useRef<HTMLInputElement>(null);

  const load = useCallback(
    (keep?: string[]) => {
      invoke<Listing | null>("finder_list", { path })
        .then((l) => {
          setListing(l);
          setSel(keep ?? []);
        })
        .catch(() => setListing({ path, name: path, parent: null, entries: [], error: "Non riesco a leggere la cartella." }));
    },
    [path],
  );

  useEffect(() => {
    setListing(null);
    setQuery("");
    setRenaming(null);
    load();
  }, [load]);

  useEffect(() => {
    invoke<Place[]>("finder_places").then(setPlaces).catch(() => {});
    invoke<number>("finder_clip_count").then(setClip).catch(() => {});
  }, []);

  useEffect(() => {
    document.documentElement.dataset.view = "finder";
    const on = () => {
      setActive(true);
      // Tornando alla finestra il contenuto puo' essere cambiato (file creati o tolti da altre app).
      invoke<Listing | null>("finder_list", { path }).then((l) => l && setListing(l)).catch(() => {});
      invoke<number>("finder_clip_count").then(setClip).catch(() => {});
    };
    const off = () => setActive(false);
    window.addEventListener("focus", on);
    window.addEventListener("blur", off);
    return () => {
      window.removeEventListener("focus", on);
      window.removeEventListener("blur", off);
    };
  }, [path]);

  useEffect(() => {
    localStorage.setItem("finder.view", view);
  }, [view]);

  useEffect(() => {
    if (!note) return;
    const t = setTimeout(() => setNote(null), 4000);
    return () => clearTimeout(t);
  }, [note]);

  const shown = useMemo(() => arrange(listing?.entries ?? [], sort, showHidden, query), [listing, sort, showHidden, query]);
  const order = useMemo(() => shown.map((e) => e.path), [shown]);
  const chosen = useMemo(() => shown.filter((e) => sel.includes(e.path)), [shown, sel]);

  const go = useCallback((p: string) => setHist((h) => (h.stack[h.i] === p ? h : { stack: [...h.stack.slice(0, h.i + 1), p], i: h.i + 1 })), []);
  const back = useCallback(() => setHist((h) => (h.i > 0 ? { ...h, i: h.i - 1 } : h)), []);
  const forward = useCallback(() => setHist((h) => (h.i < h.stack.length - 1 ? { ...h, i: h.i + 1 } : h)), []);
  const up = useCallback(() => listing?.parent && go(listing.parent), [listing, go]);

  const open = useCallback(
    (e: Entry) => {
      if (e.dir) go(e.path);
      else void invoke("finder_open", { path: e.path });
    },
    [go],
  );

  const fail = (err: unknown) => setNote(typeof err === "string" ? err : "Operazione non riuscita.");
  const trash = useCallback(() => {
    if (!chosen.length) return;
    invoke("finder_trash", { paths: chosen.map((e) => e.path) })
      .catch(fail)
      .finally(() => load());
  }, [chosen, load]);
  const copy = useCallback(
    (cut: boolean) => {
      if (!chosen.length) return;
      invoke<number>("finder_clip", { paths: chosen.map((e) => e.path), cut }).then((n) => {
        setClip(n);
        setNote(`${n} ${n === 1 ? "elemento" : "elementi"} ${cut ? "da spostare" : "copiati"}: scegli la cartella e incolla.`);
      });
    },
    [chosen],
  );
  const paste = useCallback(() => {
    invoke("finder_paste", { dest: path })
      .catch(fail)
      .finally(() => {
        load();
        invoke<number>("finder_clip_count").then(setClip).catch(() => {});
      });
  }, [path, load]);
  const newFolder = useCallback(() => {
    invoke<string>("finder_new_folder", { dir: path })
      .then((made) => {
        load([made]);
        setRenaming(made);
      })
      .catch(fail);
  }, [path, load]);
  const rename = useCallback(
    (e: Entry, name: string) => {
      setRenaming(null);
      if (!name.trim() || name === e.name) return;
      invoke<string>("finder_rename", { path: e.path, name })
        .then((np) => load([np]))
        .catch(fail);
    },
    [load],
  );

  const columns = () => {
    const grid = body.current?.querySelector<HTMLElement>(".fd-grid");
    const item = grid?.querySelector<HTMLElement>(".fd-item");
    return grid && item ? Math.max(1, Math.floor(grid.clientWidth / item.offsetWidth)) : 1;
  };

  useEffect(() => {
    const key = (ev: KeyboardEvent) => {
      const inField = (ev.target as HTMLElement | null)?.tagName === "INPUT";
      const ctrl = ev.ctrlKey || ev.metaKey;
      if (ev.key === "Escape") {
        if (menu) setMenu(null);
        else if (inField) (ev.target as HTMLInputElement).blur();
        else setSel([]);
        return;
      }
      if (ctrl && ev.key.toLowerCase() === "f") {
        ev.preventDefault();
        search.current?.focus();
        return;
      }
      if (ctrl && ev.key.toLowerCase() === "w") {
        ev.preventDefault();
        void invoke("finder_win", { action: "close" });
        return;
      }
      if (inField) return;
      const k = ev.key;
      const stop = () => ev.preventDefault();
      if (ctrl && k.toLowerCase() === "n" && ev.shiftKey) return stop(), newFolder();
      if (ctrl && k.toLowerCase() === "n") return stop(), void invoke("finder_window", { path });
      if (ctrl && k.toLowerCase() === "a") return stop(), setSel(order);
      if (ctrl && k.toLowerCase() === "c") return stop(), copy(false);
      if (ctrl && k.toLowerCase() === "x") return stop(), copy(true);
      if (ctrl && k.toLowerCase() === "v") return stop(), paste();
      if (ctrl && k === "1") return stop(), setView("icons");
      if (ctrl && k === "2") return stop(), setView("list");
      if (ctrl && ev.shiftKey && (k === "." || k === ":")) return stop(), setShowHidden((v) => !v);
      if ((ctrl && k === "ArrowUp") || (ev.altKey && k === "ArrowUp")) return stop(), void up();
      if ((ctrl && (k === "ArrowDown" || k.toLowerCase() === "o")) && chosen.length) return stop(), chosen.forEach(open);
      if ((ev.altKey && k === "ArrowLeft") || k === "BrowserBack") return stop(), back();
      if ((ev.altKey && k === "ArrowRight") || k === "BrowserForward") return stop(), forward();
      if (k === "Delete" || (ctrl && k === "Backspace")) return stop(), trash();
      if (k === "Backspace") return stop(), back();
      if ((k === "Enter" || k === "F2") && chosen.length === 1) return stop(), setRenaming(chosen[0].path);
      if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(k) && order.length) {
        stop();
        const cols = view === "list" ? 1 : columns();
        const cur = anchor.current ? order.indexOf(anchor.current) : -1;
        const step = k === "ArrowLeft" ? -1 : k === "ArrowRight" ? 1 : k === "ArrowUp" ? -cols : cols;
        const next = cur < 0 ? 0 : Math.min(order.length - 1, Math.max(0, cur + step));
        const r = clickSelect(order, sel, anchor.current, order[next], false, ev.shiftKey);
        if (!ev.shiftKey) anchor.current = r.anchor;
        setSel(r.selected);
        body.current?.querySelector(`[data-path="${CSS.escape(order[next])}"]`)?.scrollIntoView({ block: "nearest" });
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [menu, order, sel, chosen, view, path, open, back, forward, up, trash, copy, paste, newFolder]);

  const pick = (ev: React.MouseEvent, e: Entry) => {
    ev.stopPropagation();
    setMenu(null);
    const r = clickSelect(order, sel, anchor.current, e.path, ev.ctrlKey || ev.metaKey, ev.shiftKey);
    anchor.current = r.anchor;
    setSel(r.selected);
  };
  const context = (ev: React.MouseEvent, e: Entry | null) => {
    ev.preventDefault();
    ev.stopPropagation();
    if (e && !sel.includes(e.path)) {
      setSel([e.path]);
      anchor.current = e.path;
    }
    if (!e) setSel([]);
    setMenu({ x: Math.min(ev.clientX, window.innerWidth - 230), y: Math.min(ev.clientY, window.innerHeight - 300), target: e });
  };

  const title = places.find((p) => p.path.replace(/\\$/, "").toLowerCase() === path.replace(/\\$/, "").toLowerCase())?.name ?? listing?.name ?? "";
  const trail = crumbs(path);
  const sortBy = (key: SortKey) => setSort((s) => (s.key === key ? { key, asc: !s.asc } : { key, asc: key === "name" || key === "kind" }));
  const head = (key: SortKey, label: string) => (
    <button type="button" className="fd-th" data-key={key} data-on={sort.key === key || undefined} onClick={() => sortBy(key)}>
      {label}
      {sort.key === key && <span className="fd-arrow">{sort.asc ? "▲" : "▼"}</span>}
    </button>
  );

  const nameCell = (e: Entry) =>
    renaming === e.path ? (
      <input
        className="fd-rename"
        defaultValue={e.name}
        autoFocus
        spellCheck={false}
        onFocus={(ev) => {
          const dot = e.dir ? -1 : e.name.lastIndexOf(".");
          ev.target.setSelectionRange(0, dot > 0 ? dot : e.name.length);
        }}
        onClick={(ev) => ev.stopPropagation()}
        onDoubleClick={(ev) => ev.stopPropagation()}
        onBlur={(ev) => rename(e, ev.target.value)}
        onKeyDown={(ev) => {
          if (ev.key === "Enter") (ev.target as HTMLInputElement).blur();
          if (ev.key === "Escape") {
            ev.stopPropagation();
            setRenaming(null);
          }
        }}
      />
    ) : (
      <span className="fd-name">{e.name}</span>
    );

  const item = (label: string, run: () => void, off = false, keys = "") => (
    <button
      type="button"
      className="fd-menu-item"
      disabled={off}
      onClick={() => {
        setMenu(null);
        run();
      }}
    >
      <span>{label}</span>
      {keys && <kbd>{keys}</kbd>}
    </button>
  );

  return (
    <div className="fd" data-theme={theme} data-active={active || undefined} onClick={() => setMenu(null)} onContextMenu={(ev) => ev.preventDefault()}>
      <aside className="fd-side">
        <div className="fd-lights" data-tauri-drag-region>
          <button type="button" className="fd-light" data-k="close" aria-label="Chiudi" onClick={() => void invoke("finder_win", { action: "close" })} />
          <button type="button" className="fd-light" data-k="min" aria-label="Riduci a icona" onClick={() => void invoke("finder_win", { action: "min" })} />
          <button type="button" className="fd-light" data-k="zoom" aria-label="Ingrandisci" onClick={() => void invoke("finder_win", { action: "zoom" })} />
        </div>
        <nav className="fd-places">
          <div className="fd-group">Preferiti</div>
          {places
            .filter((p) => p.kind !== "drive")
            .map((p) => (
              <button type="button" key={p.path} className="fd-place" data-on={p.path === path || undefined} onClick={() => go(p.path)}>
                <PlaceGlyph kind={p.kind} />
                <span>{p.name}</span>
              </button>
            ))}
          <div className="fd-group">Posizioni</div>
          {places
            .filter((p) => p.kind === "drive")
            .map((p) => (
              <button type="button" key={p.path} className="fd-place" data-on={p.path === path || undefined} onClick={() => go(p.path)}>
                <PlaceGlyph kind="drive" />
                <span>{p.name}</span>
              </button>
            ))}
        </nav>
      </aside>

      <main className="fd-main">
        <header className="fd-bar" data-tauri-drag-region>
          <div className="fd-nav">
            <button type="button" className="fd-btn" aria-label="Indietro" disabled={hist.i === 0} onClick={back}>
              <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden>
                <path d="M10 3L5 8l5 5" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </button>
            <button type="button" className="fd-btn" aria-label="Avanti" disabled={hist.i >= hist.stack.length - 1} onClick={forward}>
              <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden>
                <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </button>
          </div>
          <h1 className="fd-title" data-tauri-drag-region>
            {title}
          </h1>
          <div className="fd-tools">
            <div className="fd-seg" role="group" aria-label="Vista">
              <button type="button" className="fd-btn" data-on={view === "icons" || undefined} aria-label="Icone" onClick={() => setView("icons")}>
                <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden>
                  <path d="M2.5 2.5h4.5v4.5H2.5zM9 2.5h4.5v4.5H9zM2.5 9h4.5v4.5H2.5zM9 9h4.5v4.5H9z" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinejoin="round" />
                </svg>
              </button>
              <button type="button" className="fd-btn" data-on={view === "list" || undefined} aria-label="Elenco" onClick={() => setView("list")}>
                <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden>
                  <path d="M2.5 4h11M2.5 8h11M2.5 12h11" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
                </svg>
              </button>
            </div>
            <button type="button" className="fd-btn fd-round" aria-label="Nuova cartella" title="Nuova cartella" onClick={newFolder}>
              <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden>
                <path d="M8 3.5v9M3.5 8h9" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
              </svg>
            </button>
            <label className="fd-search">
              <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden>
                <circle cx="7" cy="7" r="4.3" fill="none" stroke="currentColor" strokeWidth="1.6" />
                <path d="M10.3 10.3l3.2 3.2" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
              </svg>
              <input ref={search} value={query} placeholder="Cerca" spellCheck={false} onChange={(ev) => setQuery(ev.target.value)} />
            </label>
          </div>
        </header>

        <div className="fd-body" ref={body} onClick={() => setSel([])} onContextMenu={(ev) => context(ev, null)}>
          {listing?.error ? (
            <div className="fd-empty">{listing.error}</div>
          ) : listing && shown.length === 0 ? (
            <div className="fd-empty">{query ? "Nessun elemento corrisponde alla ricerca." : "Questa cartella è vuota."}</div>
          ) : view === "icons" ? (
            <div className="fd-grid">
              {shown.map((e) => (
                <div key={e.path} className="fd-item" data-path={e.path} data-on={sel.includes(e.path) || undefined} data-hidden={e.hidden || undefined} onClick={(ev) => pick(ev, e)} onDoubleClick={() => open(e)} onContextMenu={(ev) => context(ev, e)}>
                  <EntryIcon entry={e} theme={theme} badge={e.dir ? badgeFor(e.path, places) : undefined} size={128} />
                  {nameCell(e)}
                </div>
              ))}
            </div>
          ) : (
            <div className="fd-list">
              <div className="fd-head">
                {head("name", "Nome")}
                {head("modified", "Data di modifica")}
                {head("size", "Dimensioni")}
                {head("kind", "Tipo")}
              </div>
              {shown.map((e) => (
                <div key={e.path} className="fd-row" data-path={e.path} data-on={sel.includes(e.path) || undefined} data-hidden={e.hidden || undefined} onClick={(ev) => pick(ev, e)} onDoubleClick={() => open(e)} onContextMenu={(ev) => context(ev, e)}>
                  <span className="fd-cell fd-cell-name">
                    <EntryIcon entry={e} theme={theme} badge={e.dir ? badgeFor(e.path, places) : undefined} size={32} />
                    {nameCell(e)}
                  </span>
                  <span className="fd-cell">{e.modified ? DATE.format(new Date(e.modified * 1000)) : "--"}</span>
                  <span className="fd-cell fd-num">{e.dir ? "--" : sizeLabel(e.size)}</span>
                  <span className="fd-cell">{kindLabel(e)}</span>
                </div>
              ))}
            </div>
          )}
        </div>

        <footer className="fd-foot">
          <div className="fd-path">
            {trail.map((c, i) => (
              <React.Fragment key={c.path}>
                {i > 0 && <span className="fd-sep">›</span>}
                <button type="button" className="fd-crumb" onClick={() => go(c.path)}>
                  <span className="fd-crumb-icon">
                    <Art>{i === 0 ? driveArt(theme) : folderArt(theme, badgeFor(c.path, places))}</Art>
                  </span>
                  {i === 0 ? (places.find((p) => p.path.toLowerCase() === c.path.toLowerCase())?.name ?? c.name) : c.name}
                </button>
              </React.Fragment>
            ))}
          </div>
          <div className="fd-count">{chosen.length ? `${chosen.length} di ${shown.length} selezionati` : `${shown.length} ${shown.length === 1 ? "elemento" : "elementi"}`}</div>
        </footer>
      </main>

      {menu && (
        <div className="fd-menu" style={{ left: menu.x, top: menu.y }} onClick={(ev) => ev.stopPropagation()}>
          {menu.target ? (
            <>
              {item("Apri", () => chosen.forEach(open))}
              {menu.target.dir && item("Apri in una nuova finestra", () => void invoke("finder_window", { path: menu.target?.path }))}
              <hr />
              {item("Rinomina", () => menu.target && setRenaming(menu.target.path), chosen.length !== 1, "Invio")}
              {item("Copia", () => copy(false), false, "Ctrl C")}
              {item("Taglia", () => copy(true), false, "Ctrl X")}
              {item("Sposta nel Cestino", trash, false, "Canc")}
              <hr />
              {item("Mostra in Esplora file", () => menu.target && void invoke("finder_reveal", { path: menu.target.path }))}
            </>
          ) : (
            <>
              {item("Nuova cartella", newFolder, false, "Ctrl ⇧ N")}
              {item("Incolla", paste, clip === 0, "Ctrl V")}
              <hr />
              {item(view === "icons" ? "Vista a elenco" : "Vista a icone", () => setView(view === "icons" ? "list" : "icons"))}
              {item(showHidden ? "Nascondi gli elementi nascosti" : "Mostra gli elementi nascosti", () => setShowHidden((v) => !v))}
              {item("Nuova finestra", () => void invoke("finder_window", { path }), false, "Ctrl N")}
              <hr />
              {item("Mostra in Esplora file", () => void invoke("finder_reveal", { path }))}
            </>
          )}
        </div>
      )}
      {note && <div className="fd-note">{note}</div>}
    </div>
  );
}
