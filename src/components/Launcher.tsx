import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  clampSelection,
  fallbackLetter,
  knownIds,
  mergeIcons,
  moveSelection,
  windowHeight,
  type IconCache,
  type LauncherItem,
} from "../lib/launcher";

/**
 * Launcher in stile Spotlight (finestra "launcher", vista ?view=launcher). Compare con la
 * scorciatoia globale (Rust, launcherwin.rs); qui ci sono il campo, i risultati e la tastiera.
 * La pagina manda a Rust solo l'id di un risultato che Rust le ha dato.
 */
export default function Launcher() {
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<LauncherItem[]>([]);
  const [sel, setSel] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const cache = useRef<IconCache>(new Map());
  // Numero della ricerca più recente: una risposta in ritardo non deve sovrascrivere la nuova.
  const epoch = useRef(0);

  const search = useCallback(async (q: string) => {
    const mine = ++epoch.current;
    try {
      const found = await invoke<LauncherItem[]>("launcher_search", { query: q, have: knownIds(cache.current) });
      if (mine !== epoch.current) return;
      const merged = mergeIcons(found, cache.current);
      setItems(merged);
      setSel((s) => clampSelection(s, merged.length));
    } catch {
      if (mine === epoch.current) setItems([]);
    }
  }, []);

  useEffect(() => {
    void search(query);
  }, [query, search]);

  // All'apertura: campo vuoto e a fuoco, elenco delle app più usate.
  useEffect(() => {
    const un = listen("launcher-open", () => {
      setQuery("");
      setSel(0);
      void search("");
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

  const launch = async (item: LauncherItem | undefined) => {
    if (!item) return;
    hide();
    await invoke("launcher_launch", { id: item.id });
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    // Durante la composizione (IME) Invio e frecce appartengono all'editor di metodo di input.
    if (e.nativeEvent.isComposing) return;
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSel((s) => moveSelection(s, 1, items.length));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSel((s) => moveSelection(s, -1, items.length));
        break;
      case "Enter":
        e.preventDefault();
        void launch(items[sel]);
        break;
      case "Escape":
        e.preventDefault();
        hide();
        break;
    }
  };

  const hasQuery = query.trim() !== "";
  return (
    <div className="launcher" onKeyDown={onKeyDown}>
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
          placeholder="Cerca app"
          spellCheck={false}
          autoComplete="off"
          autoFocus
          role="combobox"
          aria-expanded={items.length > 0}
          aria-controls="launcher-list"
          aria-activedescendant={items[sel] ? `launcher-item-${sel}` : undefined}
        />
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
              onMouseMove={() => i !== sel && setSel(i)}
              onClick={() => void launch(it)}
            >
              {it.icon ? (
                <img className="launcher-icon" src={it.icon} alt="" draggable={false} />
              ) : (
                <span className="launcher-icon launcher-icon-fallback">{fallbackLetter(it.name)}</span>
              )}
              <span className="launcher-name">{it.name}</span>
              <span className="launcher-kind">App</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
