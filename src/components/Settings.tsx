import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  addName,
  asList,
  comboFromEvent,
  folderName,
  moveItem,
  byKeys,
  choiceLabel,
  inSection,
  nextSection,
  pickSection,
  removeName,
  sectionsOf,
  withCurrentOption,
  type SetReply,
  type SettingEntry,
  type SettingValue,
} from "../lib/settings";
import UpdateBox from "./UpdateBox";
import WinSettings from "./WinSettings";

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

  const act = async (key: string) => {
    setErrors((e) => ({ ...e, [key]: "" }));
    const r = await invoke<SetReply>("settings_action", { key }).catch(() => ({ ok: false, live: false, error: "Non riesco a eseguirlo." }) as SetReply);
    if (!r.ok) setErrors((e) => ({ ...e, [key]: r.error ?? "Non riesco a eseguirlo." }));
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
        <nav
          aria-label="Sezioni"
          onKeyDown={(e) => {
            // Frecce su/giù: passa alla sezione vicina (e le dà il focus), come in ogni barra laterale.
            if (found !== null || (e.key !== "ArrowDown" && e.key !== "ArrowUp")) return;
            e.preventDefault();
            // Si parte dal pulsante che ha il focus (non per forza quello selezionato: ci si arriva anche con Tab).
            const from = (e.target as HTMLElement).closest<HTMLElement>("[data-section]")?.dataset.section ?? section;
            const next = nextSection(sections, from, e.key === "ArrowDown" ? 1 : -1);
            setSection(next);
            requestAnimationFrame(() => document.querySelector<HTMLButtonElement>(`[data-section="${next}"]`)?.focus());
          }}
        >
          {sections.map((s) => (
            <button
              key={s.id}
              type="button"
              className={`settings-nav${found === null && s.id === section ? " settings-nav-on" : ""}`}
              data-section={s.id}
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
        {found === null && section === "windows" && <WinSettings />}
        {!(found === null && section === "windows") && shown.map((e) => (
          <Row key={e.key} entry={e} error={errors[e.key]} note={notes[e.key]} onSet={(v) => void set(e.key, v)} onAction={() => void act(e.key)} showSection={found !== null} />
        ))}
        {found === null && section === "general" && <UpdateBox />}
      </main>
    </div>
  );
}

function Row({ entry, error, note, onSet, onAction, showSection }: { entry: SettingEntry; error?: string; note?: string; onSet: (v: SettingValue) => void; onAction: () => void; showSection: boolean }) {
  const { kind, value } = entry;
  const id = `s-${entry.key}`;
  return (
    <section className="settings-row" aria-labelledby={`${id}-l`}>
      <div className="settings-text">
        <label id={`${id}-l`} htmlFor={id} className="settings-label">
          {entry.label}
          {showSection && <span className="settings-where"> · {entry.section_label}</span>}
        </label>
        <p className="settings-help" id={`${id}-h`}>
          {entry.help}
        </p>
        {note && <p className="settings-note">{note}</p>}
        {error && (
          <p className="settings-error" role="alert">
            {error}
          </p>
        )}
      </div>
      <div className="settings-control">
        {kind.type === "bool" && (
          <button id={id} type="button" role="switch" aria-describedby={`${id}-h`} aria-checked={value === true} className={`switch${value === true ? " switch-on" : ""}`} onClick={() => onSet(value !== true)}>
            <span className="switch-knob" />
          </button>
        )}
        {kind.type === "choice" && (
          <select id={id} className="settings-select" value={typeof value === "string" ? value : ""} onChange={(ev) => onSet(ev.target.value)} aria-label={entry.label} aria-describedby={`${id}-h`}>
            {withCurrentOption(kind.options, value).map(([oid, olabel]) => (
              <option key={oid} value={oid}>
                {olabel}
              </option>
            ))}
          </select>
        )}
        {kind.type === "names" && <NameList id={id} items={asList(value)} max={kind.max} onChange={onSet} />}
        {kind.type === "text" && <TextField id={id} value={String(value ?? "")} max={kind.max} onSet={(v) => onSet(v)} />}
        {kind.type === "hotkey" && <HotkeyField id={id} value={String(value ?? "")} onSet={(v) => onSet(v)} />}
        {kind.type === "folders" && <FolderList id={id} items={asList(value)} max={kind.max} onChange={onSet} />}
        {kind.type === "pinned" && <PinnedList id={id} items={asList(value)} onChange={onSet} />}
        {kind.type === "action" && (
          <button id={id} type="button" className="settings-button" onClick={onAction}>
            {entry.key === "windows.home" ? "Apri" : "Mostra il rapporto"}
          </button>
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

/** Scorciatoia: si preme «Cambia», poi la nuova combinazione; Esc annulla. Rust la valida e la prova davvero. */
/** Un testo breve (per esempio la città del meteo): si salva con Invio o quando si esce dal campo. */
function TextField({ id, value, max, onSet }: { id: string; value: string; max: number; onSet: (v: string) => void }) {
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  const commit = () => {
    if (draft.trim() !== value) onSet(draft.trim());
  };
  return <input id={id} className="settings-input" type="text" value={draft} maxLength={max} onChange={(e) => setDraft(e.target.value)} onBlur={commit} onKeyDown={(e) => e.key === "Enter" && commit()} />;
}

function HotkeyField({ id, value, onSet }: { id: string; value: string; onSet: (v: string) => void }) {
  const [rec, setRec] = useState(false);
  useEffect(() => {
    if (!rec) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();
      if (e.key === "Escape") return setRec(false);
      const combo = comboFromEvent(e);
      if (combo) {
        setRec(false);
        onSet(combo);
      }
    };
    // In cattura: prima di Esc della finestra, così annulla la registrazione e non chiude le impostazioni.
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [rec, onSet]);
  return (
    <div className="hotkey" id={id}>
      <kbd className="hotkey-keys">{rec ? "Premi la nuova scorciatoia…" : value}</kbd>
      <button type="button" className="settings-button" onClick={() => setRec((r) => !r)}>
        {rec ? "Annulla" : "Cambia"}
      </button>
    </div>
  );
}

/** Cartelle di ricerca: elenco con «×» e «Aggiungi cartella…» (il selettore di Windows lo apre Rust). */
function FolderList({ id, items, max, onChange }: { id: string; items: string[]; max: number; onChange: (v: string[]) => void }) {
  const [busy, setBusy] = useState(false);
  const add = async () => {
    setBusy(true);
    try {
      const picked = await invoke<string | null>("settings_pick_folder");
      if (picked && !items.some((p) => p.toLowerCase() === picked.toLowerCase())) onChange([...items, picked]);
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="names" id={id}>
      <ul className="names-list" aria-label="Cartelle">
        {items.length === 0 && <li className="names-empty">Nessuna</li>}
        {items.map((p) => (
          <li key={p} className="names-item" title={p}>
            <span>{folderName(p)}</span>
            <button type="button" aria-label={`Rimuovi ${p}`} onClick={() => onChange(removeName(items, p))}>
              ×
            </button>
          </li>
        ))}
      </ul>
      <div className="names-add">
        <button type="button" className="settings-button" onClick={() => void add()} disabled={busy || items.length >= max}>
          {busy ? "Scegli la cartella…" : "Aggiungi cartella…"}
        </button>
      </div>
    </div>
  );
}

/** App fissate nel Dock: si riordinano con le frecce e si tolgono con «×»; non se ne aggiungono da qui. */
function PinnedList({ id, items, onChange }: { id: string; items: string[]; onChange: (v: string[]) => void }) {
  return (
    <ol className="pinned" id={id} aria-label="App fissate">
      {items.length === 0 && <li className="names-empty">Nessuna</li>}
      {items.map((n, i) => (
        <li key={`${i}-${n}`} className="pinned-item">
          <span className="pinned-name">{n}</span>
          <button type="button" aria-label={`Sposta su ${n}`} disabled={i === 0} onClick={() => onChange(moveItem(items, i, -1))}>
            ↑
          </button>
          <button type="button" aria-label={`Sposta giù ${n}`} disabled={i === items.length - 1} onClick={() => onChange(moveItem(items, i, 1))}>
            ↓
          </button>
          <button type="button" aria-label={`Togli ${n} dal Dock`} onClick={() => onChange(items.filter((_, j) => j !== i))}>
            ×
          </button>
        </li>
      ))}
    </ol>
  );
}
