import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useBackdrop } from "./useBackdrop";
import {
  clampSelection,
  combine,
  filterLabel,
  filterForKey,
  glyphFor,
  kindLabel,
  knownIds,
  mergeIcons,
  moveSelection,
  pointerMoved,
  toggleFilter,
  wantsFiles,
  windowHeight,
  type Filter,
  type IconCache,
  type Point,
  type LauncherItem,
} from "../lib/launcher";

/** Contorno catturato attorno al pannello perché lo sfocato non sbavi ai bordi (px CSS). */
const BLUR_MARGIN = 28;

/**
 * Launcher in stile Spotlight (finestra "launcher", vista ?view=launcher). Compare con la
 * scorciatoia globale (Rust, launcherwin.rs); qui ci sono il campo, i risultati e la tastiera.
 * La pagina manda a Rust solo l'id di un risultato che Rust le ha dato.
 */
export default function Launcher() {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<Filter>(null);
  const [fast, setFast] = useState<LauncherItem[]>([]);
  const [fileItems, setFileItems] = useState<LauncherItem[]>([]);
  const items = combine(fast, fileItems, filter);
  const [sel, setSel] = useState(0);
  // Azione che aspetta la conferma (Riavvia, Spegni): la conferma vera la gestisce Rust.
  const [confirming, setConfirming] = useState<string | null>(null);
  const input = useRef<HTMLInputElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const blurCanvas = useRef<HTMLCanvasElement>(null);
  const [opens, setOpens] = useState(0);
  // Ultima posizione del mouse registrata: vedi `pointerMoved`.
  const pointer = useRef<Point | null>(null);
  useBackdrop("launcher_backdrop", panel, blurCanvas, BLUR_MARGIN, opens);
  const cache = useRef<IconCache>(new Map());
  // Numero della ricerca più recente: una risposta in ritardo non deve sovrascrivere la nuova.
  const epoch = useRef(0);

  const search = useCallback(async (q: string, f: Filter) => {
    const mine = ++epoch.current;
    setConfirming(null);
    try {
      const found = await invoke<LauncherItem[]>("launcher_search", {
        query: q,
        have: knownIds(cache.current),
        filter: f,
      });
      if (mine !== epoch.current) return;
      const merged = mergeIcons(found, cache.current);
      setFast(merged);
    } catch {
      if (mine === epoch.current) setFast([]);
    }
  }, []);

  useEffect(() => {
    void search(query, filter);
  }, [query, filter, search]);

  // I file arrivano a parte, dopo una breve pausa nella digitazione (la ricerca nell'indice costa di più).
  const fileEpoch = useRef(0);
  useEffect(() => {
    const mine = ++fileEpoch.current;
    setFileItems([]);
    if (!wantsFiles(query, filter)) return;
    const timer = setTimeout(async () => {
      try {
        const found = await invoke<LauncherItem[]>("launcher_files", { query, have: knownIds(cache.current) });
        if (mine === fileEpoch.current) setFileItems(mergeIcons(found, cache.current));
      } catch {
        /* nessun file: l'elenco resta com'è */
      }
    }, 150);
    return () => clearTimeout(timer);
  }, [query, filter]);

  // La selezione resta dentro l'elenco quando i risultati cambiano.
  useEffect(() => {
    setSel((s) => clampSelection(s, items.length));
  }, [items.length]);

  // All'apertura: campo vuoto e a fuoco, elenco delle app più usate.
  useEffect(() => {
    const un = listen("launcher-open", () => {
      setQuery("");
      setSel(0);
      setFilter(null);
      setConfirming(null);
      setOpens((n) => n + 1);
      pointer.current = null;
      void search("", null);
      input.current?.focus();
    });
    input.current?.focus();
    return () => {
      void un.then((f) => f());
    };
  }, [search]);

  // La finestra segue l'altezza che serve.
  useEffect(() => {
    void invoke("launcher_fit", { height: windowHeight(items.length, query.trim() !== "") });
  }, [items.length, query]);

  const hide = () => void invoke("launcher_hide");

  const launch = async (item: LauncherItem | undefined, reveal = false) => {
    if (!item) return;
    if (item.kind === "file") {
      hide();
      await invoke("launcher_open_file", { id: item.id, reveal });
    } else if (item.kind === "clip") {
      hide();
      await invoke("launcher_clip_use", { id: item.id });
    } else if (item.kind === "setting") {
      await invoke("launcher_clip_set", { on: true });
      void search(query, filter);
    } else if (item.kind === "calc") {
      // Copia il risultato: il Launcher è a fuoco, quindi la pagina può scrivere negli appunti.
      try {
        await navigator.clipboard.writeText(item.name);
      } catch {
        /* appunti non disponibili: si chiude comunque */
      }
      hide();
    } else if (item.kind === "action") {
      const r = await invoke<string>("launcher_action", { id: item.id });
      if (r === "confirm") setConfirming(item.id);
      else hide();
    } else {
      hide();
      await invoke("launcher_launch", { id: item.id });
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    // Durante la composizione (IME) Invio e frecce appartengono all'editor di metodo di input.
    if (e.nativeEvent.isComposing) return;
    if (e.ctrlKey && !e.altKey && !e.metaKey) {
      const wanted = filterForKey(e.key);
      if (wanted) {
        e.preventDefault();
        setFilter((f) => toggleFilter(f, wanted));
        setSel(0);
        return;
      }
    }
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setConfirming(null);
        setSel((s) => moveSelection(s, 1, items.length));
        break;
      case "ArrowUp":
        e.preventDefault();
        setConfirming(null);
        setSel((s) => moveSelection(s, -1, items.length));
        break;
      case "Enter":
        e.preventDefault();
        void launch(items[sel], e.ctrlKey);
        break;
      case "Delete": {
        // Toglie la voce selezionata dalla cronologia degli appunti.
        const it = items[sel];
        if (it?.kind === "clip") {
          e.preventDefault();
          void invoke("launcher_clip_delete", { id: it.id }).then(() => search(query, filter));
        }
        break;
      }
      case "Escape":
        e.preventDefault();
        hide();
        break;
    }
  };

  const hasQuery = query.trim() !== "";
  return (
    <div className="launcher" ref={panel} onKeyDown={onKeyDown}>
      <canvas className="launcher-blur" ref={blurCanvas} aria-hidden />
      <div className="launcher-field">
        <svg className="launcher-glass" viewBox="0 0 20 20" aria-hidden="true">
          <circle cx="8.5" cy="8.5" r="5.5" fill="none" stroke="currentColor" strokeWidth="1.8" />
          <path d="M13 13l4.5 4.5" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
        </svg>
        <input
          ref={input}
          className="launcher-input"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setSel(0);
          }}
          placeholder={filter ? `Cerca in ${filterLabel(filter)}` : "Cerca app, file, azioni, calcoli (Ctrl+4: appunti)"}
          spellCheck={false}
          autoComplete="off"
          autoFocus
          role="combobox"
          aria-expanded={items.length > 0}
          aria-controls="launcher-list"
          aria-activedescendant={items[sel] ? `launcher-item-${sel}` : undefined}
        />
        {filter && <span className="launcher-chip">{filterLabel(filter)}</span>}
      </div>
      {(items.length > 0 || hasQuery) && (
        <ul className="launcher-list" id="launcher-list" role="listbox">
          {items.length === 0 && <li className="launcher-empty">Nessun risultato</li>}
          {items.map((it, i) => (
            <li
              key={it.id}
              id={`launcher-item-${i}`}
              role="option"
              aria-selected={i === sel}
              className="launcher-item"
              data-selected={i === sel}
              data-kind={it.kind}
              data-confirming={confirming === it.id}
              onMouseMove={(e) => {
                const next = { x: e.screenX, y: e.screenY };
                const moved = pointerMoved(pointer.current, next);
                if (pointer.current === null || moved) pointer.current = next;
                if (moved && i !== sel) {
                  setSel(i);
                  setConfirming(null);
                }
              }}
              onClick={(e) => void launch(it, e.ctrlKey)}
            >
              {it.icon ? (
                <img className="launcher-icon" src={it.icon} alt="" draggable={false} />
              ) : (
                <span className="launcher-icon launcher-icon-fallback">{glyphFor(it)}</span>
              )}
              <span className="launcher-name">{it.name}</span>
              {it.detail && <span className="launcher-detail">{it.detail}</span>}
              <span className="launcher-kind">{kindLabel(it, confirming === it.id)}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
