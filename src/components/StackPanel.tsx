import { forwardRef, useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isLatest, shortName, stackAria, stackSummary, type StackReply } from "../lib/stacks";

/**
 * Il pannello di una pila del Dock (dentro la finestra "panel", sopra l'icona): i file più recenti della cartella in
 * griglia, ognuno con la sua icona; un clic lo apre (i programmi e gli script si mostrano solo in Esplora risorse, lo
 * decide Rust). «Apri cartella» apre la cartella intera. La pagina manda solo l'indice della pila e l'id di un file che
 * Rust le ha appena mostrato.
 */
const StackPanel = forwardRef<HTMLDivElement, { index: number; opens: number }>(function StackPanel({ index, opens }, ref) {
  const [reply, setReply] = useState<StackReply | null>(null);
  const seq = useRef(0);

  const load = useCallback(async () => {
    const mine = ++seq.current;
    try {
      const r = await invoke<StackReply>("stack_list", { index });
      if (isLatest(mine, seq.current)) setReply(r);
    } catch {
      if (isLatest(mine, seq.current)) setReply({ title: "", state: "missing", entries: [], total: 0 });
    }
  }, [index]);

  useEffect(() => {
    setReply(null);
    void load();
  }, [load, opens]);

  const open = async (id: string) => {
    const ok = await invoke<boolean>("stack_open", { id }).catch(() => false);
    if (ok) await invoke("panel_hide").catch(() => undefined);
    else await load();
  };
  const openFolder = async () => {
    const ok = await invoke<boolean>("stack_open_folder", { index }).catch(() => false);
    if (ok) await invoke("panel_hide").catch(() => undefined);
  };

  return (
    <div className="stack" ref={ref} role="group" aria-label={stackAria(reply)} data-phase={opens % 2 ? "a" : "b"}>
      <div className="stack-head">
        <h2 className="stack-title">{reply?.title || "Pila"}</h2>
        {reply && reply.state !== "missing" && (
          <button type="button" className="stack-folder" onClick={() => void openFolder()}>
            Apri cartella
          </button>
        )}
      </div>
      {reply?.state === "missing" && (
        <p className="stack-empty" role="alert">
          La cartella non c'è o non si può leggere.
        </p>
      )}
      {reply?.state === "empty" && <p className="stack-empty">La cartella è vuota.</p>}
      {reply?.state === "ok" && (
        <>
          <div className="stack-grid" role="list">
            {reply.entries.map((e) => (
              <button key={e.id} type="button" className="stack-file" role="listitem" title={e.name} aria-label={`${e.is_dir ? "Cartella" : "Apri"} ${e.name}`} onClick={() => void open(e.id)}>
                {e.icon ? <img className="stack-icon" src={e.icon} alt="" draggable={false} /> : <span className="stack-icon stack-icon-fallback">{e.is_dir ? "▤" : "▫"}</span>}
                <span className={e.name.includes(" ") ? "stack-name" : "stack-name stack-name-one"}>{shortName(e.name)}</span>
              </button>
            ))}
          </div>
          {stackSummary(reply) && <div className="stack-more">{stackSummary(reply)}</div>}
        </>
      )}
    </div>
  );
});

export default StackPanel;
