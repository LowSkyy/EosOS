import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { filterPages, groupPages, type WinPage } from "../lib/winsettings";
import WinSystem from "./WinSystem";

/**
 * Le Impostazioni di Windows dentro quelle di EosOS: l'elenco delle pagine, raggruppato come in Windows 11 e con una
 * ricerca; un clic apre la pagina vera di Windows (le pagine di Windows non si possono incorporare). La pagina manda solo
 * l'id di una voce: l'indirizzo lo sceglie Rust da una tabella fissa.
 */
export default function WinSettings() {
  const [pages, setPages] = useState<WinPage[]>([]);
  const [query, setQuery] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    invoke<WinPage[]>("winsettings_list")
      .then(setPages)
      .catch(() => setError("Non riesco a leggere l'elenco."));
  }, []);

  const groups = useMemo(() => groupPages(filterPages(pages, query)), [pages, query]);

  const open = async (id: string) => {
    setError("");
    const ok = await invoke<boolean>("winsettings_open", { id }).catch(() => false);
    if (!ok) setError("Non riesco ad aprire questa pagina di Windows.");
  };

  return (
    <>
      <WinSystem />
      <section className="winset" aria-label="Altre pagine delle Impostazioni di Windows">
      <h2 className="winset-title winset-more">Altre impostazioni di Windows</h2>
      <p className="winset-note">Queste pagine non si possono rifare dentro EosOS: si aprono nella finestra di Windows.</p>
      <input className="settings-input winset-search" type="search" placeholder="Cerca nelle Impostazioni di Windows" aria-label="Cerca nelle Impostazioni di Windows" value={query} onChange={(e) => setQuery(e.target.value)} />
      {error && (
        <p className="settings-error" role="alert">
          {error}
        </p>
      )}
      {groups.length === 0 && pages.length > 0 && <p className="settings-empty">Nessuna pagina corrisponde.</p>}
      {groups.map((g) => (
        <div key={g.group} className="winset-group" role="group" aria-label={g.group}>
          <h2 className="winset-title">{g.group}</h2>
          <div className="winset-grid">
            {g.pages.map((p) => (
              <button key={p.id} type="button" className="winset-tile" onClick={() => void open(p.id)} aria-label={`${p.label}: ${p.hint}`}>
                <span className="winset-label">{p.label}</span>
                <span className="winset-hint">{p.hint}</span>
              </button>
            ))}
          </div>
        </div>
      ))}
      </section>
    </>
  );
}
