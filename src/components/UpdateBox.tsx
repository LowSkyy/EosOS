import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** Lo stato degli aggiornamenti, come lo manda Rust (`updates::Status`). */
export type UpdateStatus = {
  state: "idle" | "none" | "available" | "installing" | "error";
  current: string;
  version: string | null;
  notes: string | null;
  message: string | null;
  text: string;
};

/**
 * Blocco «Aggiornamenti» della sezione Generale: la versione in uso, «Controlla ora» (una richiesta di rete) e, se c'è
 * una versione nuova, «Installa». Non parte niente da solo: cerca solo quando premi il pulsante (o all'avvio, se hai
 * acceso la ricerca automatica) e installa solo dal pulsante. La firma la verifica Rust.
 */
export default function UpdateBox() {
  const [st, setSt] = useState<UpdateStatus | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    invoke<UpdateStatus | null>("update_status")
      .then((s) => s && setSt(s))
      .catch(() => undefined);
  }, []);

  const check = async () => {
    setBusy(true);
    const s = await invoke<UpdateStatus | null>("update_check").catch(() => null);
    if (s) setSt(s);
    setBusy(false);
  };
  const install = async () => {
    setBusy(true);
    try {
      await invoke("update_install");
    } catch {
      const s = await invoke<UpdateStatus | null>("update_status").catch(() => null);
      if (s) setSt(s);
    }
    setBusy(false);
  };

  return (
    <section className="settings-row update-box" aria-labelledby="update-l">
      <div className="settings-text">
        <div id="update-l" className="settings-label">
          Aggiornamenti
        </div>
        <p className="settings-help">Cerca una versione nuova di EosOS. Gli aggiornamenti sono firmati: un file non firmato da noi viene rifiutato.</p>
      </div>
      <div className="settings-control update-control">
        <p className="update-text" role="status" aria-live="polite">
          {st?.text ?? ""}
        </p>
        {st?.state === "available" && st.notes && <p className="update-notes">{st.notes}</p>}
        <div className="update-buttons">
          <button type="button" className="settings-button" disabled={busy || st?.state === "installing"} onClick={() => void check()}>
            Controlla ora
          </button>
          {st?.state === "available" && (
            <button type="button" className="settings-button update-install" disabled={busy} onClick={() => void install()}>
              Installa la versione {st.version}
            </button>
          )}
        </div>
      </div>
    </section>
  );
}
