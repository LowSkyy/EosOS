import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { FILE_ART, FOLDER_GLYPHS, driveArt, folderArt, type FolderBadge, type Theme } from "./iconart";
import { useIconStyle } from "./useIconStyle";
import { Lights, type LightKind } from "./Lights";
import {
  TAGS,
  TAG_PREFIX,
  TRASH,
  arrange,
  artFor,
  badgeFor,
  clickSelect,
  columnChain,
  crumbs,
  hasThumb,
  isRealFolder,
  kindLabel,
  sizeLabel,
  taggedWith,
  trashEntry,
  usesShellIcon,
  type Entry,
  type Listing,
  type Place,
  type Sort,
  type SortKey,
  type TrashItem,
  type ViewMode,
} from "../lib/finder";

/** Finder di EosOS: la finestra per cartelle e file con l'aspetto del Finder di macOS. La parte Rust e' `finder.rs` e `finder2.rs`. */

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
function PlaceGlyph({ kind }: { kind: Place["kind"] }) {
  if (kind === "drive") {
    return (
      <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden>
        <path d="M5 4h10l3 8v3a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 2 15v-3z" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" />
        <path d="M2.5 12h15" stroke="currentColor" strokeWidth="1.5" />
        <circle cx="14.5" cy="14.2" r="1" fill="currentColor" />
      </svg>
    );
  }
  if (kind === "trash") {
    return (
      <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden>
        <path d="M4 5.5h12M8 5.5V4h4v1.5M5.5 5.5l.8 10.2a1.5 1.5 0 0 0 1.5 1.3h4.4a1.5 1.5 0 0 0 1.5-1.3l.8-10.2" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    );
  }
  return (
    <svg viewBox="33 43 34 32" width="16" height="16" fill="currentColor" stroke="currentColor" strokeWidth="0" aria-hidden>
      {FOLDER_GLYPHS[kind]}
    </svg>
  );
}

function TagDots({ colors }: { colors?: string[] }) {
  if (!colors?.length) return null;
  return (
    <span className="fd-tags">
      {colors.map((c) => (
        <i key={c} style={{ background: TAGS.find((t) => t.id === c)?.color }} />
      ))}
    </span>
  );
}

const DATE = new Intl.DateTimeFormat("it-IT", { day: "numeric", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit" });
const when = (s: number) => (s ? DATE.format(new Date(s * 1000)) : "--");

type Menu = { x: number; y: number; target: Entry | null };
type Info = { size: number; items: number; partial: boolean; created: number; modified: number; accessed: number; readonly: boolean; hidden: boolean };
type Sheet = { kind: "goto" } | { kind: "confirm"; text: string; ok: string; run: () => void } | { kind: "info"; entry: Entry; info: Info | null };
type Peek = { text: string | null; image: string | null };

const VIEWS: ViewMode[] = ["icons", "list", "columns", "gallery"];

export default function Finder() {
  const { theme, left } = useIconStyle();
  const win = useCallback((action: LightKind) => void invoke("finder_win", { action }), []);
  const first = useMemo(() => new URLSearchParams(window.location.search).get("path") ?? "C:\\", []);
  const [hist, setHist] = useState<{ stack: string[]; i: number }>({ stack: [first], i: 0 });
  const path = hist.stack[hist.i];
  const real = isRealFolder(path);
  const inTrash = path === TRASH;
  const tagColor = path.startsWith(TAG_PREFIX) ? path.slice(TAG_PREFIX.length) : null;
  const [listing, setListing] = useState<Listing | null>(null);
  const [trashItems, setTrashItems] = useState<TrashItem[]>([]);
  const [places, setPlaces] = useState<Place[]>([]);
  const [tags, setTags] = useState<Record<string, string[]>>({});
  const [view, setView] = useState<ViewMode>(() => {
    const v = localStorage.getItem("finder.view") as ViewMode | null;
    return v && VIEWS.includes(v) ? v : "icons";
  });
  const [sort, setSort] = useState<Sort>({ key: "name", asc: true });
  const [query, setQuery] = useState("");
  const [deep, setDeep] = useState(false);
  const [found, setFound] = useState<Entry[] | null>(null);
  const [searching, setSearching] = useState(false);
  const [showHidden, setShowHidden] = useState(false);
  const [sel, setSel] = useState<string[]>([]);
  const anchor = useRef<string | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [menu, setMenu] = useState<Menu | null>(null);
  const [sheet, setSheet] = useState<Sheet | null>(null);
  const [peek, setPeek] = useState<{ entry: Entry; data: Peek | null } | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [clip, setClip] = useState(0);
  const [active, setActive] = useState(true);
  const [dropOn, setDropOn] = useState<string | null>(null);
  const [cols, setCols] = useState<Record<string, Entry[]>>({});
  const body = useRef<HTMLDivElement>(null);
  const search = useRef<HTMLInputElement>(null);

  // ---------------------------------------------------------------- contenuto

  const load = useCallback(
    (keep?: string[]) => {
      if (path === TRASH) {
        invoke<TrashItem[]>("finder_trash_list")
          .then((t) => {
            setTrashItems(t);
            setListing({ path, name: "Cestino", parent: null, entries: t.map(trashEntry), error: null });
            setSel(keep ?? []);
          })
          .catch(() => setListing({ path, name: "Cestino", parent: null, entries: [], error: "Non riesco a leggere il Cestino." }));
        return;
      }
      if (path.startsWith(TAG_PREFIX)) {
        const color = path.slice(TAG_PREFIX.length);
        invoke<Record<string, string[]>>("finder_tags")
          .then((t) => {
            setTags(t);
            return invoke<Entry[]>("finder_entries", { paths: taggedWith(t, color) });
          })
          .then((entries) => {
            setListing({ path, name: TAGS.find((t) => t.id === color)?.name ?? color, parent: null, entries, error: null });
            setSel(keep ?? []);
          })
          .catch(() => setListing({ path, name: color, parent: null, entries: [], error: "Non riesco a leggere le etichette." }));
        return;
      }
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
    setFound(null);
    setRenaming(null);
    setPeek(null);
    load();
  }, [load]);

  const refreshMeta = useCallback(() => {
    invoke<Place[]>("finder_places").then(setPlaces).catch(() => {});
    invoke<number>("finder_clip_count").then(setClip).catch(() => {});
    invoke<Record<string, string[]>>("finder_tags").then(setTags).catch(() => {});
  }, []);
  useEffect(refreshMeta, [refreshMeta]);

  useEffect(() => {
    document.documentElement.dataset.view = "finder";
    const on = () => {
      setActive(true);
      // Tornando alla finestra il contenuto puo' essere cambiato (file creati o tolti da altre app, appunti nuovi).
      load(sel);
      invoke<number>("finder_clip_count").then(setClip).catch(() => {});
    };
    const off = () => setActive(false);
    window.addEventListener("focus", on);
    window.addEventListener("blur", off);
    return () => {
      window.removeEventListener("focus", on);
      window.removeEventListener("blur", off);
    };
  }, [load, sel]);

  useEffect(() => {
    localStorage.setItem("finder.view", view);
  }, [view]);

  useEffect(() => {
    if (!note) return;
    const t = setTimeout(() => setNote(null), 4000);
    return () => clearTimeout(t);
  }, [note]);

  // Ricerca nelle sottocartelle (dopo una breve pausa nella scrittura).
  useEffect(() => {
    if (!deep || !query.trim() || !real) {
      setFound(null);
      setSearching(false);
      return;
    }
    let live = true;
    setSearching(true);
    const t = setTimeout(() => {
      invoke<Entry[]>("finder_search", { root: path, query })
        .then((r) => live && setFound(r))
        .catch(() => live && setFound([]))
        .finally(() => live && setSearching(false));
    }, 300);
    return () => {
      live = false;
      clearTimeout(t);
    };
  }, [deep, query, path, real]);

  // Vista a colonne: il contenuto di ogni cartella della catena.
  const chain = useMemo(() => (real ? columnChain(path) : []), [path, real]);
  useEffect(() => {
    if (view !== "columns") return;
    let live = true;
    chain.slice(0, -1).forEach((p) => {
      if (cols[p]) return;
      invoke<Listing | null>("finder_list", { path: p }).then((l) => live && l && setCols((c) => ({ ...c, [p]: l.entries })));
    });
    return () => {
      live = false;
    };
  }, [view, chain, cols]);

  const shown = useMemo(() => {
    if (found) return arrange(found, sort, showHidden, "");
    return arrange(listing?.entries ?? [], sort, showHidden, query);
  }, [listing, found, sort, showHidden, query]);
  const order = useMemo(() => shown.map((e) => e.path), [shown]);
  const chosen = useMemo(() => shown.filter((e) => sel.includes(e.path)), [shown, sel]);

  // ---------------------------------------------------------------- azioni

  const go = useCallback((p: string) => setHist((h) => (h.stack[h.i] === p ? h : { stack: [...h.stack.slice(0, h.i + 1), p], i: h.i + 1 })), []);
  const back = useCallback(() => setHist((h) => (h.i > 0 ? { ...h, i: h.i - 1 } : h)), []);
  const forward = useCallback(() => setHist((h) => (h.i < h.stack.length - 1 ? { ...h, i: h.i + 1 } : h)), []);
  const up = useCallback(() => listing?.parent && go(listing.parent), [listing, go]);

  const fail = (err: unknown) => setNote(typeof err === "string" ? err : "Operazione non riuscita.");

  const open = useCallback(
    (e: Entry) => {
      if (inTrash) return setNote("Per aprirlo, prima ripristinalo.");
      if (e.dir) go(e.path);
      else void invoke("finder_open", { path: e.path });
    },
    [go, inTrash],
  );

  const restore = useCallback(() => {
    invoke<number>("finder_trash_restore", { ids: chosen.map((e) => e.path) })
      .then((n) => setNote(`${n} ${n === 1 ? "elemento ripristinato" : "elementi ripristinati"}.`))
      .catch(fail)
      .finally(() => load());
  }, [chosen, load]);

  const trash = useCallback(() => {
    if (!chosen.length) return;
    if (inTrash) {
      setSheet({
        kind: "confirm",
        text: `Eliminare per sempre ${chosen.length === 1 ? `«${chosen[0].name}»` : `${chosen.length} elementi`}? Non si potrà annullare.`,
        ok: "Elimina",
        run: () =>
          void invoke("finder_trash_delete", { ids: chosen.map((e) => e.path) })
            .catch(fail)
            .finally(() => load()),
      });
      return;
    }
    if (!real && !tagColor) return;
    invoke("finder_trash", { paths: chosen.map((e) => e.path) })
      .catch(fail)
      .finally(() => load());
  }, [chosen, load, inTrash, real, tagColor]);

  const emptyTrash = useCallback(
    () =>
      setSheet({
        kind: "confirm",
        text: "Svuotare il Cestino? Gli elementi saranno eliminati per sempre.",
        ok: "Svuota",
        run: () =>
          void invoke("finder_trash_empty")
            .catch(fail)
            .finally(() => load()),
      }),
    [load],
  );

  const copy = useCallback(
    (cut: boolean) => {
      if (!chosen.length || inTrash) return;
      invoke<number>("finder_clip", { paths: chosen.map((e) => e.path), cut })
        .then((n) => {
          setClip(n);
          setNote(`${n} ${n === 1 ? "elemento" : "elementi"} ${cut ? "da spostare" : "copiati"}: incolla qui, in un'altra cartella o in Esplora file.`);
        })
        .catch(fail);
    },
    [chosen, inTrash],
  );
  const paste = useCallback(() => {
    if (!real) return;
    invoke("finder_paste", { dest: path })
      .catch(fail)
      .finally(() => {
        load();
        invoke<number>("finder_clip_count").then(setClip).catch(() => {});
      });
  }, [path, load, real]);
  const newFolder = useCallback(() => {
    if (!real) return;
    invoke<string>("finder_new_folder", { dir: path })
      .then((made) => {
        load([made]);
        setRenaming(made);
      })
      .catch(fail);
  }, [path, load, real]);
  const rename = useCallback(
    (e: Entry, name: string) => {
      setRenaming(null);
      if (!name.trim() || name === e.name) return;
      invoke<string>("finder_rename", { path: e.path, name })
        .then((np) => {
          load([np]);
          invoke<Record<string, string[]>>("finder_tags").then(setTags).catch(() => {});
        })
        .catch(fail);
    },
    [load],
  );
  const tag = useCallback(
    (color: string) => {
      if (!chosen.length || inTrash) return;
      const on = !chosen.every((e) => tags[e.path]?.includes(color));
      invoke<Record<string, string[]>>("finder_tag", { paths: chosen.map((e) => e.path), color, on }).then((t) => {
        setTags(t);
        if (tagColor) load(sel);
      });
    },
    [chosen, tags, tagColor, load, sel, inTrash],
  );
  const showInfo = useCallback((e: Entry) => {
    setSheet({ kind: "info", entry: e, info: null });
    invoke<Info | null>("finder_info", { path: e.path }).then((info) => setSheet((s) => (s?.kind === "info" && s.entry.path === e.path ? { ...s, info } : s)));
  }, []);
  const togglePeek = useCallback(() => {
    if (peek) return setPeek(null);
    const e = chosen[0];
    if (!e || e.dir) return;
    setPeek({ entry: e, data: null });
    invoke<Peek>("finder_peek", { path: e.path }).then((data) => setPeek((p) => (p?.entry.path === e.path ? { ...p, data } : p)));
  }, [peek, chosen]);
  // Anteprima aperta: segue la selezione (frecce).
  useEffect(() => {
    const e = chosen[0];
    if (!peek || !e || e.path === peek.entry.path) return;
    if (e.dir) return setPeek(null);
    setPeek({ entry: e, data: null });
    invoke<Peek>("finder_peek", { path: e.path }).then((data) => setPeek((p) => (p?.entry.path === e.path ? { ...p, data } : p)));
  }, [chosen, peek]);

  // ---------------------------------------------------------------- trascinamento dei file

  const dragFrom = useRef<{ x: number; y: number; path: string } | null>(null);
  const startDrag = (ev: React.PointerEvent, e: Entry) => {
    if (ev.button !== 0 || renaming || inTrash) return;
    dragFrom.current = { x: ev.clientX, y: ev.clientY, path: e.path };
  };
  useEffect(() => {
    const move = (ev: PointerEvent) => {
      const d = dragFrom.current;
      if (!d || !(ev.buttons & 1) || Math.hypot(ev.clientX - d.x, ev.clientY - d.y) < 6) return;
      dragFrom.current = null;
      const paths = sel.includes(d.path) ? order.filter((p) => sel.includes(p)) : [d.path];
      if (!sel.includes(d.path)) setSel([d.path]);
      void invoke("finder_drag", { paths });
    };
    const upp = () => {
      dragFrom.current = null;
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", upp);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", upp);
    };
  }, [sel, order]);

  // Destinazione: la cartella sotto il cursore (voce, barra laterale, percorso) o la cartella aperta.
  const targetAt = useCallback(
    (x: number, y: number): string | null => {
      const el = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-drop]");
      if (el?.dataset.drop) return el.dataset.drop;
      return real && !found ? path : null;
    },
    [path, real, found],
  );
  useEffect(() => {
    let off: (() => void) | undefined;
    let gone = false;
    getCurrentWebview()
      .onDragDropEvent((e) => {
        const p = e.payload;
        const dpr = window.devicePixelRatio || 1;
        if (p.type === "over" || p.type === "enter") {
          setDropOn(targetAt(p.position.x / dpr, p.position.y / dpr));
        } else if (p.type === "drop") {
          const dest = targetAt(p.position.x / dpr, p.position.y / dpr);
          setDropOn(null);
          if (dest && p.paths.length) {
            invoke("finder_drop", { paths: p.paths, dest })
              .catch(fail)
              .finally(() => load());
          }
        } else {
          setDropOn(null);
        }
      })
      .then((fn) => (gone ? fn() : (off = fn)))
      .catch(() => {});
    return () => {
      gone = true;
      off?.();
    };
  }, [targetAt, load]);

  // ---------------------------------------------------------------- tastiera

  const columnsPerRow = () => {
    const grid = body.current?.querySelector<HTMLElement>(".fd-grid");
    const item = grid?.querySelector<HTMLElement>(".fd-item");
    return grid && item ? Math.max(1, Math.floor(grid.clientWidth / item.offsetWidth)) : 1;
  };

  useEffect(() => {
    const key = (ev: KeyboardEvent) => {
      const inField = (ev.target as HTMLElement | null)?.tagName === "INPUT";
      const ctrl = ev.ctrlKey || ev.metaKey;
      const k = ev.key;
      const stop = () => ev.preventDefault();
      if (k === "Escape") {
        if (peek) setPeek(null);
        else if (sheet) setSheet(null);
        else if (menu) setMenu(null);
        else if (inField) (ev.target as HTMLInputElement).blur();
        else setSel([]);
        return;
      }
      if (sheet) return;
      if (ctrl && k.toLowerCase() === "f") return stop(), search.current?.focus();
      if (ctrl && k.toLowerCase() === "w") return stop(), win("close");
      if (ctrl && ev.shiftKey && k.toLowerCase() === "g") return stop(), setSheet({ kind: "goto" });
      if (inField) return;
      if (ctrl && ev.shiftKey && k.toLowerCase() === "n") return stop(), newFolder();
      if (ctrl && k.toLowerCase() === "n") return stop(), void invoke("finder_window", { path: real ? path : null });
      if (ctrl && k.toLowerCase() === "a") return stop(), setSel(order);
      if (ctrl && k.toLowerCase() === "c") return stop(), copy(false);
      if (ctrl && k.toLowerCase() === "x") return stop(), copy(true);
      if (ctrl && k.toLowerCase() === "v") return stop(), paste();
      if (ctrl && k.toLowerCase() === "i" && chosen[0]) return stop(), showInfo(chosen[0]);
      if (ctrl && ["1", "2", "3", "4"].includes(k)) return stop(), setView(VIEWS[Number(k) - 1]);
      if (ctrl && ev.shiftKey && (k === "." || k === ":")) return stop(), setShowHidden((v) => !v);
      if (k === " ") return stop(), togglePeek();
      if ((ctrl || ev.altKey) && k === "ArrowUp") return stop(), void up();
      if (ctrl && (k === "ArrowDown" || k.toLowerCase() === "o") && chosen.length) return stop(), chosen.forEach(open);
      if ((ev.altKey && k === "ArrowLeft") || k === "BrowserBack") return stop(), back();
      if ((ev.altKey && k === "ArrowRight") || k === "BrowserForward") return stop(), forward();
      if (k === "Delete" || (ctrl && k === "Backspace")) return stop(), trash();
      if (k === "Backspace") return stop(), back();
      if ((k === "Enter" || k === "F2") && chosen.length === 1 && !inTrash) return stop(), setRenaming(chosen[0].path);
      if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(k) && order.length) {
        stop();
        const perRow = view === "icons" ? columnsPerRow() : view === "gallery" ? 1 : 1;
        const horiz = view === "gallery";
        const cur = anchor.current ? order.indexOf(anchor.current) : -1;
        const step = k === "ArrowLeft" ? -1 : k === "ArrowRight" ? 1 : k === "ArrowUp" ? (horiz ? -1 : -perRow) : horiz ? 1 : perRow;
        const next = cur < 0 ? 0 : Math.min(order.length - 1, Math.max(0, cur + step));
        const r = clickSelect(order, sel, anchor.current, order[next], false, ev.shiftKey);
        if (!ev.shiftKey) anchor.current = r.anchor;
        setSel(r.selected);
        body.current?.querySelector(`[data-path="${CSS.escape(order[next])}"]`)?.scrollIntoView({ block: "nearest", inline: "nearest" });
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [menu, sheet, peek, order, sel, chosen, view, path, real, inTrash, win, open, back, forward, up, trash, copy, paste, newFolder, showInfo, togglePeek]);

  // ---------------------------------------------------------------- selezione e menu

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
    setMenu({ x: Math.min(ev.clientX, window.innerWidth - 240), y: Math.min(ev.clientY, window.innerHeight - 380), target: e });
  };

  const placeName = (p: string) => places.find((pl) => pl.path.replace(/\\$/, "").toLowerCase() === p.replace(/\\$/, "").toLowerCase())?.name;
  const title = found ? `Ricerca di «${query.trim()}»` : (placeName(path) ?? listing?.name ?? "");
  const trail = real ? crumbs(path) : [];
  const sortBy = (key: SortKey) => setSort((s) => (s.key === key ? { key, asc: !s.asc } : { key, asc: key === "name" || key === "kind" }));
  const head = (key: SortKey, label: string) => (
    <button type="button" className="fd-th" data-key={key} data-on={sort.key === key || undefined} onClick={() => sortBy(key)}>
      {label}
      {sort.key === key && <span className="fd-arrow">{sort.asc ? "▲" : "▼"}</span>}
    </button>
  );
  const dropAttr = (e: Entry) => (e.dir && !inTrash ? e.path : undefined);

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
        onPointerDown={(ev) => ev.stopPropagation()}
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
      <span className="fd-name">
        {e.name}
        <TagDots colors={tags[e.path]} />
      </span>
    );

  const itemProps = (e: Entry) => ({
    "data-path": e.path,
    "data-drop": dropAttr(e),
    "data-on": sel.includes(e.path) || undefined,
    "data-target": (dropOn !== null && dropOn === e.path) || undefined,
    "data-hidden": e.hidden || undefined,
    onClick: (ev: React.MouseEvent) => pick(ev, e),
    onDoubleClick: () => open(e),
    onContextMenu: (ev: React.MouseEvent) => context(ev, e),
    onPointerDown: (ev: React.PointerEvent) => startDrag(ev, e),
  });

  const badge = (e: Entry) => (e.dir ? badgeFor(e.path, places) : undefined);
  const focusEntry = chosen[0] ?? shown[0];

  // ---------------------------------------------------------------- viste

  const iconsView = (
    <div className="fd-grid">
      {shown.map((e) => (
        <div key={e.path} className="fd-item" {...itemProps(e)}>
          <EntryIcon entry={e} theme={theme} badge={badge(e)} size={128} />
          {nameCell(e)}
        </div>
      ))}
    </div>
  );

  const listView = (
    <div className="fd-list">
      <div className="fd-head">
        {head("name", "Nome")}
        {head("modified", inTrash ? "Eliminato il" : "Data di modifica")}
        {head("size", "Dimensioni")}
        {head("kind", inTrash ? "Posizione originale" : found ? "Cartella" : "Tipo")}
      </div>
      {shown.map((e) => (
        <div key={e.path} className="fd-row" {...itemProps(e)}>
          <span className="fd-cell fd-cell-name">
            <EntryIcon entry={e} theme={theme} badge={badge(e)} size={32} />
            {nameCell(e)}
          </span>
          <span className="fd-cell">{when(e.modified)}</span>
          <span className="fd-cell fd-num">{e.dir ? "--" : sizeLabel(e.size)}</span>
          <span className="fd-cell">{inTrash ? (trashItems.find((t) => t.id === e.path)?.origin ?? "") : found ? e.path.slice(0, e.path.lastIndexOf("\\")) : kindLabel(e)}</span>
        </div>
      ))}
    </div>
  );

  const preview = (e: Entry | undefined) =>
    e && (
      <div className="fd-preview">
        <PreviewImage entry={e} theme={theme} badge={badge(e)} />
        <div className="fd-preview-name">{e.name}</div>
        <div className="fd-preview-kind">
          {kindLabel(e)}
          {!e.dir && ` · ${sizeLabel(e.size)}`}
        </div>
        <dl className="fd-preview-meta">
          <dt>Modificato</dt>
          <dd>{when(e.modified)}</dd>
        </dl>
      </div>
    );

  const columnsView = (
    <div className="fd-columns">
      {chain.slice(0, -1).map((p, i) => {
        const next = chain[i + 1];
        const list = arrange(cols[p] ?? [], { key: "name", asc: true }, showHidden, "");
        return (
          <div key={p} className="fd-col" data-drop={p}>
            {list.map((e) => (
              <div key={e.path} className="fd-col-row" data-drop={dropAttr(e)} data-open={e.path.toLowerCase() === next?.toLowerCase() || undefined} onClick={() => (e.dir ? go(e.path) : void invoke("finder_open", { path: e.path }))}>
                <EntryIcon entry={e} theme={theme} badge={badge(e)} size={32} />
                <span className="fd-name">{e.name}</span>
                {e.dir && <span className="fd-chev">›</span>}
              </div>
            ))}
          </div>
        );
      })}
      <div className="fd-col" data-drop={real ? path : undefined}>
        {shown.map((e) => (
          <div key={e.path} className="fd-col-row" {...itemProps(e)} onDoubleClick={() => open(e)}>
            <EntryIcon entry={e} theme={theme} badge={badge(e)} size={32} />
            {nameCell(e)}
            {e.dir && <span className="fd-chev">›</span>}
          </div>
        ))}
      </div>
      {chosen.length === 1 && !chosen[0].dir && <div className="fd-col fd-col-preview">{preview(chosen[0])}</div>}
    </div>
  );

  const galleryView = (
    <div className="fd-gallery">
      <div className="fd-gallery-stage">{preview(focusEntry)}</div>
      <div className="fd-strip">
        {shown.map((e) => (
          <div key={e.path} className="fd-strip-item" {...itemProps(e)} title={e.name}>
            <EntryIcon entry={e} theme={theme} badge={badge(e)} size={128} />
          </div>
        ))}
      </div>
    </div>
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

  const viewIcon: Record<ViewMode, React.ReactNode> = {
    icons: <path d="M2.5 2.5h4.5v4.5H2.5zM9 2.5h4.5v4.5H9zM2.5 9h4.5v4.5H2.5zM9 9h4.5v4.5H9z" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinejoin="round" />,
    list: <path d="M2.5 4h11M2.5 8h11M2.5 12h11" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />,
    columns: <path d="M2.5 3h11v10h-11zM6.2 3v10M9.8 3v10" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinejoin="round" />,
    gallery: <path d="M2.5 2.5h11v7h-11zM2.5 12h2.5M6.75 12h2.5M11 12h2.5" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />,
  };
  const viewName: Record<ViewMode, string> = { icons: "Icone", list: "Elenco", columns: "Colonne", gallery: "Galleria" };

  return (
    <div className="fd" data-theme={theme} data-active={active || undefined} onClick={() => setMenu(null)} onContextMenu={(ev) => ev.preventDefault()}>
      <aside className="fd-side">
        <div className="fd-head-lights" data-tauri-drag-region>
          {left && <Lights left active={active} onAction={win} />}
        </div>
        <nav className="fd-places">
          <div className="fd-group">Preferiti</div>
          {places
            .filter((p) => p.kind !== "drive" && p.kind !== "trash")
            .map((p) => (
              <button type="button" key={p.path} className="fd-place" data-drop={p.path} data-target={dropOn === p.path || undefined} data-on={p.path === path || undefined} onClick={() => go(p.path)}>
                <PlaceGlyph kind={p.kind} />
                <span>{p.name}</span>
              </button>
            ))}
          <div className="fd-group">Posizioni</div>
          {places
            .filter((p) => p.kind === "drive" || p.kind === "trash")
            .map((p) => (
              <button type="button" key={p.path} className="fd-place" data-drop={p.kind === "drive" ? p.path : undefined} data-target={dropOn === p.path || undefined} data-on={p.path === path || undefined} onClick={() => go(p.path)}>
                <PlaceGlyph kind={p.kind} />
                <span>{p.name}</span>
              </button>
            ))}
          <div className="fd-group">Etichette</div>
          {TAGS.map((t) => (
            <button type="button" key={t.id} className="fd-place" data-on={path === TAG_PREFIX + t.id || undefined} onClick={() => go(TAG_PREFIX + t.id)}>
              <i className="fd-tag-dot" style={{ background: t.color }} />
              <span>{t.name}</span>
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
              {VIEWS.map((v) => (
                <button key={v} type="button" className="fd-btn" data-on={view === v || undefined} aria-label={viewName[v]} title={viewName[v]} onClick={() => setView(v)}>
                  <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden>
                    {viewIcon[v]}
                  </svg>
                </button>
              ))}
            </div>
            {inTrash ? (
              <button type="button" className="fd-pill" disabled={!shown.length} onClick={emptyTrash}>
                Svuota
              </button>
            ) : (
              <button type="button" className="fd-btn fd-round" aria-label="Nuova cartella" title="Nuova cartella" disabled={!real} onClick={newFolder}>
                <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden>
                  <path d="M8 3.5v9M3.5 8h9" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
                </svg>
              </button>
            )}
            <label className="fd-search">
              <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden>
                <circle cx="7" cy="7" r="4.3" fill="none" stroke="currentColor" strokeWidth="1.6" />
                <path d="M10.3 10.3l3.2 3.2" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
              </svg>
              <input ref={search} value={query} placeholder="Cerca" spellCheck={false} onChange={(ev) => setQuery(ev.target.value)} />
            </label>
            {!left && <Lights left={false} active={active} onAction={win} />}
          </div>
        </header>

        {query.trim() && real && (
          <div className="fd-scope">
            Cerca in:
            <button type="button" data-on={!deep || undefined} onClick={() => setDeep(false)}>
              «{listing?.name}»
            </button>
            <button type="button" data-on={deep || undefined} onClick={() => setDeep(true)}>
              «{listing?.name}» e sottocartelle
            </button>
            {searching && <span className="fd-spin" aria-label="Ricerca in corso" />}
          </div>
        )}

        <div className="fd-body" ref={body} data-view={view} data-target={(dropOn !== null && dropOn === path) || undefined} onClick={() => setSel([])} onContextMenu={(ev) => context(ev, null)}>
          {listing?.error ? (
            <div className="fd-empty">{listing.error}</div>
          ) : listing && shown.length === 0 && !searching ? (
            <div className="fd-empty">{query ? "Nessun elemento corrisponde alla ricerca." : inTrash ? "Il Cestino è vuoto." : tagColor ? "Nessun elemento con questa etichetta." : "Questa cartella è vuota."}</div>
          ) : view === "list" ? (
            listView
          ) : view === "columns" && real && !found ? (
            columnsView
          ) : view === "gallery" ? (
            galleryView
          ) : (
            iconsView
          )}
        </div>

        <footer className="fd-foot">
          <div className="fd-path">
            {real ? (
              trail.map((c, i) => (
                <React.Fragment key={c.path}>
                  {i > 0 && <span className="fd-sep">›</span>}
                  <button type="button" className="fd-crumb" data-drop={c.path} data-target={dropOn === c.path || undefined} onClick={() => go(c.path)}>
                    <span className="fd-crumb-icon">
                      <Art>{i === 0 ? driveArt(theme) : folderArt(theme, badgeFor(c.path, places))}</Art>
                    </span>
                    {i === 0 ? (placeName(c.path) ?? c.name) : c.name}
                  </button>
                </React.Fragment>
              ))
            ) : (
              <span className="fd-crumb">{title}</span>
            )}
          </div>
          <div className="fd-count">{chosen.length ? `${chosen.length} di ${shown.length} selezionati` : `${shown.length} ${shown.length === 1 ? "elemento" : "elementi"}`}</div>
        </footer>
      </main>

      {menu && (
        <div className="fd-menu" style={{ left: menu.x, top: menu.y }} onClick={(ev) => ev.stopPropagation()}>
          {inTrash ? (
            menu.target ? (
              <>
                {item("Ripristina", restore)}
                {item("Elimina subito", trash, false, "Canc")}
              </>
            ) : (
              item("Svuota il Cestino", emptyTrash, !shown.length)
            )
          ) : menu.target ? (
            <>
              {item("Apri", () => chosen.forEach(open))}
              {menu.target.dir && item("Apri in una nuova finestra", () => void invoke("finder_window", { path: menu.target?.path }))}
              {!menu.target.dir && item("Anteprima rapida", togglePeek, false, "Spazio")}
              <hr />
              {item("Ottieni informazioni", () => menu.target && showInfo(menu.target), false, "Ctrl I")}
              {item("Rinomina", () => menu.target && setRenaming(menu.target.path), chosen.length !== 1, "Invio")}
              {item("Copia", () => copy(false), false, "Ctrl C")}
              {item("Taglia", () => copy(true), false, "Ctrl X")}
              {item("Sposta nel Cestino", trash, false, "Canc")}
              <hr />
              <div className="fd-menu-tags">
                {TAGS.map((t) => (
                  <button
                    key={t.id}
                    type="button"
                    title={t.name}
                    aria-label={`Etichetta ${t.name}`}
                    data-on={chosen.every((e) => tags[e.path]?.includes(t.id)) || undefined}
                    style={{ background: t.color }}
                    onClick={() => {
                      setMenu(null);
                      tag(t.id);
                    }}
                  />
                ))}
              </div>
              <hr />
              {found && item("Mostra nella cartella", () => menu.target && go(menu.target.path.slice(0, menu.target.path.lastIndexOf("\\")) || menu.target.path))}
              {item("Mostra in Esplora file", () => menu.target && void invoke("finder_reveal", { path: menu.target.path }))}
            </>
          ) : (
            <>
              {item("Nuova cartella", newFolder, !real, "Ctrl ⇧ N")}
              {item("Incolla", paste, clip === 0 || !real, "Ctrl V")}
              <hr />
              {VIEWS.map((v, i) => item(`Vista ${viewName[v].toLowerCase()}${view === v ? "  ✓" : ""}`, () => setView(v), false, `Ctrl ${i + 1}`))}
              {item(showHidden ? "Nascondi gli elementi nascosti" : "Mostra gli elementi nascosti", () => setShowHidden((v) => !v), false, "Ctrl ⇧ .")}
              <hr />
              {item("Vai alla cartella…", () => setSheet({ kind: "goto" }), false, "Ctrl ⇧ G")}
              {item("Nuova finestra", () => void invoke("finder_window", { path: real ? path : null }), false, "Ctrl N")}
              {real && item("Ottieni informazioni", () => listing && showInfo({ name: title, path, dir: true, size: 0, modified: 0, ext: "", hidden: false }))}
              {real && item("Mostra in Esplora file", () => void invoke("finder_reveal", { path }))}
            </>
          )}
        </div>
      )}

      {sheet && (
        <div className="fd-sheet-back" onClick={() => setSheet(null)}>
          <div className="fd-sheet" onClick={(ev) => ev.stopPropagation()}>
            {sheet.kind === "goto" && (
              <form
                onSubmit={(ev) => {
                  ev.preventDefault();
                  const v = (new FormData(ev.currentTarget).get("p") as string).trim().replace(/^"|"$/g, "");
                  if (v) go(v.replace(/%([A-Za-z]+)%/g, (_m, n: string) => (n.toLowerCase() === "userprofile" ? (places.find((p) => p.kind === "home")?.path ?? _m) : _m)));
                  setSheet(null);
                }}
              >
                <div className="fd-sheet-title">Vai alla cartella</div>
                <input name="p" className="fd-sheet-input" autoFocus spellCheck={false} placeholder="C:\Cartella o \\server\condivisione" defaultValue={real ? path : ""} />
                <div className="fd-sheet-btns">
                  <button type="button" className="fd-pill" onClick={() => setSheet(null)}>
                    Annulla
                  </button>
                  <button type="submit" className="fd-pill" data-primary>
                    Vai
                  </button>
                </div>
              </form>
            )}
            {sheet.kind === "confirm" && (
              <>
                <div className="fd-sheet-text">{sheet.text}</div>
                <div className="fd-sheet-btns">
                  <button type="button" className="fd-pill" onClick={() => setSheet(null)}>
                    Annulla
                  </button>
                  <button
                    type="button"
                    className="fd-pill"
                    data-primary
                    data-danger
                    autoFocus
                    onClick={() => {
                      sheet.run();
                      setSheet(null);
                    }}
                  >
                    {sheet.ok}
                  </button>
                </div>
              </>
            )}
            {sheet.kind === "info" && (
              <div className="fd-info">
                <div className="fd-info-top">
                  <span className="fd-info-icon">
                    <EntryIcon entry={sheet.entry} theme={theme} badge={badge(sheet.entry)} size={128} />
                  </span>
                  <div>
                    <div className="fd-info-name">{sheet.entry.name}</div>
                    <div className="fd-preview-kind">{kindLabel(sheet.entry)}</div>
                  </div>
                </div>
                <dl className="fd-preview-meta">
                  <dt>Dimensioni</dt>
                  <dd>{sheet.info ? `${sizeLabel(sheet.info.size)}${sheet.entry.dir ? ` · ${sheet.info.items} elementi${sheet.info.partial ? " (conteggio parziale)" : ""}` : ""}` : "Calcolo…"}</dd>
                  <dt>Posizione</dt>
                  <dd className="fd-info-path">{sheet.entry.path}</dd>
                  <dt>Creato</dt>
                  <dd>{sheet.info ? when(sheet.info.created) : "…"}</dd>
                  <dt>Modificato</dt>
                  <dd>{sheet.info ? when(sheet.info.modified) : "…"}</dd>
                  <dt>Ultima apertura</dt>
                  <dd>{sheet.info ? when(sheet.info.accessed) : "…"}</dd>
                  {sheet.info && (sheet.info.readonly || sheet.info.hidden) && (
                    <>
                      <dt>Attributi</dt>
                      <dd>{[sheet.info.readonly && "Sola lettura", sheet.info.hidden && "Nascosto"].filter(Boolean).join(", ")}</dd>
                    </>
                  )}
                  {tags[sheet.entry.path]?.length ? (
                    <>
                      <dt>Etichette</dt>
                      <dd>
                        <TagDots colors={tags[sheet.entry.path]} />
                      </dd>
                    </>
                  ) : null}
                </dl>
                <div className="fd-sheet-btns">
                  <button type="button" className="fd-pill" data-primary autoFocus onClick={() => setSheet(null)}>
                    Fine
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {peek && (
        <div className="fd-peek-back" onClick={() => setPeek(null)}>
          <div className="fd-peek" onClick={(ev) => ev.stopPropagation()}>
            <div className="fd-peek-bar">
              <button type="button" className="fd-peek-close" aria-label="Chiudi l'anteprima" onClick={() => setPeek(null)}>
                ✕
              </button>
              <span className="fd-peek-name">{peek.entry.name}</span>
              <button type="button" className="fd-pill" onClick={() => void invoke("finder_open", { path: peek.entry.path })}>
                Apri
              </button>
            </div>
            <div className="fd-peek-body">
              {!peek.data ? (
                <span className="fd-spin" />
              ) : peek.data.text !== null ? (
                <pre className="fd-peek-text">{peek.data.text}</pre>
              ) : peek.data.image ? (
                <img src={peek.data.image} alt="" draggable={false} />
              ) : (
                <span className="fd-peek-icon">
                  <Art>{FILE_ART[artFor(peek.entry.ext)](theme)}</Art>
                </span>
              )}
            </div>
          </div>
        </div>
      )}
      {note && <div className="fd-note">{note}</div>}
    </div>
  );
}

/** L'immagine grande della galleria e delle colonne: anteprima vera se c'e', altrimenti l'icona. */
function PreviewImage({ entry, theme, badge }: { entry: Entry; theme: Theme; badge?: FolderBadge }) {
  const [img, setImg] = useState<string | null>(null);
  useEffect(() => {
    setImg(null);
    if (entry.dir || (!hasThumb(entry.ext) && !usesShellIcon(entry.ext))) return;
    let live = true;
    void shellImage(`p|${entry.path}|${entry.modified}`, entry.path, 256, hasThumb(entry.ext)).then((v) => live && setImg(v));
    return () => {
      live = false;
    };
  }, [entry]);
  return (
    <span className="fd-preview-img" data-thumb={img && hasThumb(entry.ext) ? "1" : undefined}>
      {entry.dir ? <Art>{folderArt(theme, badge)}</Art> : img ? <img src={img} alt="" draggable={false} /> : <Art>{FILE_ART[artFor(entry.ext)](theme)}</Art>}
    </span>
  );
}
