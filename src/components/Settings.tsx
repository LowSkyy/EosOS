import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  addName,
  asList,
  byKeys,
  choiceLabel,
  inSection,
  pickSection,
  removeName,
  sectionsOf,
  type SetReply,
  type SettingEntry,
  type SettingValue,
} from "../lib/settings";

/**
 * Impostazioni di EosOS (finestra "settings", vista ?view=settings): barra laterale con le sezioni e a destra le
 * impostazioni, che agiscono subito. La pagina non decide niente: manda a Rust chiave e valore e mostra la
 * risposta; se Rust rifiuta, mostra il messaggio e rilegge lo stato vero.
 */
export default function Settings() {
  const [entries, setEntries] = useState<SettingEntry[]>([]);
  const [section, setSection] = useState("");
  const [query, setQuery] = useState("");
  const [found, setFound] = useState<string[] | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [notes, setNotes] = useState<Record<string, string>>({});
  const wanted = useRef<string | null>(null);
  const search = useRef<HTMLInputElement>(null);

  const reload = useCallback(async () => {
    try {
      const all = await invoke<SettingEntry[]>("settings_all");
      setEntries(all);
      setSection((cur) => pickSection(all, cur || wanted.current));
    } catch {
      /* resta lo stato di prima */
    }
  }, []);

  useEffect(() => {
    void reload();
    // Una modifica da qualunque parte (questa finestra, un menu contestuale, il Launcher) la si rilegge;
    // anche quando la finestra torna in primo piano.
    const subs = [
      listen("settings-changed", () => void reload()),
      listen("dock-settings-changed", () => void reload()),
      listen<string | null>("settings-open", (e) => {
        wanted.current = e.payload;
        if (e.payload) {
          setQuery("");
          setSection(e.payload);
        }
        void reload();
        search.current?.focus();
      }),
    ];
    const onFocus = () => void reload();
    window.addEventListener("focus", onFocus);
    return () => {
      window.removeEventListener("focus", onFocus);
      for (const s of subs) void s.then((f) => f());
    };
  }, [reload]);

  // Ricerca: Rust dà le chiavi, qui si mostrano in quell'ordine.
  useEffect(() => {
    if (!query.trim()) return setFound(null);
    let cancelled = false;
    invoke<string[]>("settings_search", { query }).then((k) => !cancelled && setFound(k));
    return () => {
      cancelled = true;
    };
  }, [query]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.preventDefault();
      if (query) setQuery("");
      else void invoke("settings_close");
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [query]);

  const set = async (key: string, value: SettingValue) => {
    setErrors((e) => ({ ...e, [key]: "" }));
    const r = await invoke<SetReply>("settings_set", { key, value }).catch(() => ({ ok: false, live: false, error: "Non riesco a salvare." }) as SetReply);
    setNotes((n) => ({ ...n, [key]: r.ok && !r.live ? "Vale dal prossimo avvio di EosOS." : "" }));
    if (!r.ok) setErrors((e) => ({ ...e, [key]: r.error ?? "Non riesco a salvare." }));
    await reload(); // lo stato vero, anche dopo un rifiuto
  };

  const sections = sectionsOf(entries);
  const shown = found !== null ? byKeys(entries, found) : inSection(entries, section);
  const title = found !== null ? `Risultati per “${query.trim()}”` : (sections.find((s) => s.id === section)?.label ?? "");

  return (
    <div className="settings" role="dialog" aria-label="Impostazioni di EosOS">
      <div className="settings-titlebar" data-tauri-drag-region>
        <span className="settings-title" data-tauri-drag-region>
          Impostazioni di EosOS
        </span>
        <button type="button" className="settings-close" aria-label="Chiudi" onClick={() => void invoke("settings_close")}>
          ×
        </button>
      </div>
      <aside className="settings-side">
        <input
          ref={search}
          className="settings-search"
          type="search"
          placeholder="Cerca"
          aria-label="Cerca nelle impostazioni"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <nav aria-label="Sezioni">
          {sections.map((s) => (
            <button
              key={s.id}
              type="button"
              className={`settings-nav${found === null && s.id === section ? " settings-nav-on" : ""}`}
              aria-current={found === null && s.id === section ? "page" : undefined}
              onClick={() => {
                setQuery("");
                setSection(s.id);
              }}
            >
              {s.label}
            </button>
          ))}
        </nav>
      </aside>
      <main className="settings-main">
        <h1 className="settings-h">{title}</h1>
        {shown.length === 0 && <p className="settings-empty">{found !== null ? "Nessuna impostazione corrisponde." : ""}</p>}
        {shown.map((e) => (
          <Row key={e.key} entry={e} error={errors[e.key]} note={notes[e.key]} onSet={(v) => void set(e.key, v)} showSection={found !== null} />
        ))}
      </main>
    </div>
  );
}

function Row({ entry, error, note, onSet, showSection }: { entry: SettingEntry; error?: string; note?: string; onSet: (v: SettingValue) => void; showSection: boolean }) {
  const { kind, value } = entry;
  const id = `s-${entry.key}`;
  return (
    <section className="settings-row" aria-labelledby={`${id}-l`}>
      <div className="settings-text">
        <label id={`${id}-l`} htmlFor={id} className="settings-label">
          {entry.label}
          {showSection && <span className="settings-where"> · {entry.section_label}</span>}
        </label>
        <p className="settings-help">{entry.help}</p>
        {note && <p className="settings-note">{note}</p>}
        {error && (
          <p className="settings-error" role="alert">
            {error}
          </p>
        )}
      </div>
      <div className="settings-control">
        {kind.type === "bool" && (
          <button id={id} type="button" role="switch" aria-checked={value === true} className={`switch${value === true ? " switch-on" : ""}`} onClick={() => onSet(value !== true)}>
            <span className="switch-knob" />
          </button>
        )}
        {kind.type === "choice" && (
          <select id={id} className="settings-select" value={typeof value === "string" ? value : ""} onChange={(ev) => onSet(ev.target.value)} aria-label={entry.label}>
            {kind.options.map(([oid, olabel]) => (
              <option key={oid} value={oid}>
                {olabel}
              </option>
            ))}
          </select>
        )}
        {kind.type === "names" && <NameList id={id} items={asList(value)} max={kind.max} onChange={onSet} />}
        {(kind.type === "hotkey" || kind.type === "folders") && (
          <span id={id} className="settings-static">
            {kind.type === "hotkey" ? String(value ?? "") : `${asList(value).length} cartelle`}
          </span>
        )}
        {kind.type === "choice" && <span className="settings-sr">{choiceLabel(kind, value)}</span>}
      </div>
    </section>
  );
}

function NameList({ id, items, max, onChange }: { id: string; items: string[]; max: number; onChange: (v: string[]) => void }) {
  const [text, setText] = useState("");
  const add = () => {
    const next = addName(items, text);
    setText("");
    if (next !== items) onChange(next);
  };
  return (
    <div className="names" id={id}>
      <ul className="names-list" aria-label="Elenco">
        {items.length === 0 && <li className="names-empty">Nessuna</li>}
        {items.map((n) => (
          <li key={n} className="names-item">
            <span>{n}</span>
            <button type="button" aria-label={`Rimuovi ${n}`} onClick={() => onChange(removeName(items, n))}>
              ×
            </button>
          </li>
        ))}
      </ul>
      <div className="names-add">
        <input
          className="settings-input"
          type="text"
          placeholder="Nome dell'app"
          aria-label="Aggiungi un'app"
          value={text}
          maxLength={60}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && (e.preventDefault(), add())}
        />
        <button type="button" className="settings-button" onClick={add} disabled={!text.trim() || items.length >= max}>
          Aggiungi
        </button>
      </div>
    </div>
  );
}
