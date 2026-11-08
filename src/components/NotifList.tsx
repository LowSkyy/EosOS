import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { dismissLabel, headingLabel, hiddenLabel, isLatest, REFRESH_MS, withoutApp, withoutNotif, type NotifReply } from "../lib/notifications";

/**
 * Le notifiche di Windows sopra il calendario (dentro la finestra "panel"): gruppi per app, ogni notifica con titolo,
 * corpo e «quanto fa», «×» per togliere una notifica o tutto il gruppo, un clic apre l'app. Si rilegge all'apertura e
 * ogni pochi secondi finché il pannello è aperto. Se la lettura è spenta (predefinito) lo dice e offre di accenderla.
 * La pagina non decide niente: tutto passa da Rust, che controlla che gli id esistano.
 */
export default function NotifList() {
  const [reply, setReply] = useState<NotifReply | null>(null);
  const seq = useRef(0);

  const load = useCallback(async () => {
    const mine = ++seq.current;
    try {
      const r = await invoke<NotifReply>("notif_list");
      if (isLatest(mine, seq.current)) setReply(r);
    } catch {
      if (isLatest(mine, seq.current)) setReply({ state: "error", groups: [], total: 0, message: "Non riesco a leggere le notifiche." });
    }
  }, []);

  useEffect(() => {
    void load();
    const t = window.setInterval(() => void load(), REFRESH_MS);
    return () => window.clearInterval(t);
  }, [load]);

  const enable = async () => {
    await invoke("settings_set", { key: "notif.read", value: true }).catch(() => undefined);
    await load();
  };

  const setDnd = async (on: boolean) => {
    await invoke("settings_set", { key: "notif.dnd", value: on }).catch(() => undefined);
    await load();
  };

  // Tolta subito dall'elenco mostrato; la rilettura che segue conferma lo stato vero.
  const dismiss = async (id: number) => {
    setReply((r) => (r ? { ...r, groups: withoutNotif(r.groups, id), total: Math.max(0, r.total - 1) } : r));
    await invoke("notif_dismiss", { id }).catch(() => false);
    await load();
  };
  const clearApp = async (appId: string) => {
    setReply((r) => (r ? { ...r, groups: withoutApp(r.groups, appId) } : r));
    await invoke("notif_clear_app", { appId }).catch(() => 0);
    await load();
  };
  const open = async (id: number) => {
    const ok = await invoke<boolean>("notif_open", { id }).catch(() => false);
    if (ok) await invoke("panel_hide").catch(() => undefined);
    else await load();
  };

  if (!reply) return null;
  return (
    <section className="notifs" aria-label="Notifiche">
      <div className="notifs-head">
        <h2 className="notifs-title">{headingLabel(reply.state === "ok" ? reply.total : 0)}</h2>
        {reply.state !== "off" && (
          <button type="button" className="notifs-btn" aria-pressed={reply.state === "paused"} onClick={() => void setDnd(reply.state !== "paused")}>
            {reply.state === "paused" ? "Riprendi" : "Non disturbare"}
          </button>
        )}
      </div>
      {reply.state === "paused" && (
        <p className="notifs-empty" role="status">
          Non disturbare è acceso: EosOS non legge né mostra le notifiche. Restano nel Centro notifiche di Windows.
        </p>
      )}
      {reply.state === "off" && (
        <div className="notifs-off">
          <p>La lettura delle notifiche è spenta. Accendendola, EosOS mostra qui le notifiche di Windows (restano solo in memoria).</p>
          <button type="button" className="notifs-btn" onClick={() => void enable()}>
            Mostra le notifiche
          </button>
        </div>
      )}
      {reply.state === "error" && (
        <p className="notifs-empty" role="alert">
          {reply.message ?? "Non riesco a leggere le notifiche."}
        </p>
      )}
      {reply.state === "ok" && reply.groups.length === 0 && <p className="notifs-empty">Nessuna notifica.</p>}
      {reply.state === "ok" && reply.groups.length > 0 && (
        <div className="notifs-list">
          {reply.groups.map((g) => (
            <div key={g.app_id} className="notifs-group" role="group" aria-label={g.app_name}>
              <div className="notifs-app">
                <span>{g.app_name}</span>
                <button type="button" className="notifs-x" aria-label={`Rimuovi tutte le notifiche di ${g.app_name}`} onClick={() => void clearApp(g.app_id)}>
                  ×
                </button>
              </div>
              {g.items.map((n) => (
                <div key={n.id} className="notifs-item">
                  <button type="button" className="notifs-open" onClick={() => void open(n.id)} aria-label={`Apri ${g.app_name}: ${n.title || n.body}`}>
                    <span className="notifs-ttl">{n.title || n.body}</span>
                    {n.title && n.body && <span className="notifs-body">{n.body}</span>}
                    <span className="notifs-ago">{n.ago}</span>
                  </button>
                  <button type="button" className="notifs-x" aria-label={dismissLabel(g, n)} onClick={() => void dismiss(n.id)}>
                    ×
                  </button>
                </div>
              ))}
              {hiddenLabel(g) && <div className="notifs-more">{hiddenLabel(g)}</div>}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
