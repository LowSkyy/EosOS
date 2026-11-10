import { forwardRef, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { tiles, tileTitle, withToggled, type ControlState, type Tile } from "../lib/control";
import { makeLatestSender } from "../lib/indicators";
import { nextIndex } from "../lib/eosmenu";
import { useIndicators } from "./useIndicators";

type SettingRow = { key: string; value: unknown };

/** Le icone delle tessere (18 px, tratto): Wi-Fi, Bluetooth, Non disturbare (luna), Mantieni sveglio (sole). */
const ICONS: Record<string, React.ReactNode> = {
  wifi: (
    <svg width="18" height="18" viewBox="0 0 18 18" aria-hidden fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M2.2 7.1a10.5 10.5 0 0 1 13.6 0" />
      <path d="M4.9 10a6.6 6.6 0 0 1 8.2 0" />
      <path d="M7.4 12.8a2.8 2.8 0 0 1 3.2 0" />
      <circle cx="9" cy="15.2" r="0.6" fill="currentColor" />
    </svg>
  ),
  bluetooth: (
    <svg width="18" height="18" viewBox="0 0 18 18" aria-hidden fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M4.6 5.4 13 12.6 9 16.2V1.8l4 3.6-8.4 7.2" />
    </svg>
  ),
  dnd: (
    <svg width="18" height="18" viewBox="0 0 18 18" aria-hidden fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14.8 10.6A6.4 6.4 0 0 1 7.4 3.2a6.4 6.4 0 1 0 7.4 7.4z" />
    </svg>
  ),
  awake: (
    <svg width="18" height="18" viewBox="0 0 18 18" aria-hidden fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="9" cy="9" r="3.1" />
      <path d="M9 1.9v1.6M9 14.5v1.6M1.9 9h1.6M14.5 9h1.6M4 4l1.1 1.1M12.9 12.9 14 14M14 4l-1.1 1.1M5.1 12.9 4 14" />
    </svg>
  ),
};

/**
 * Il Centro di controllo (nella finestra "panel", dal pulsante a destra nella barra): Wi-Fi, Bluetooth, Non disturbare, Mantieni sveglio,
 * il volume e due scorciatoie alle impostazioni. Wi-Fi e Bluetooth compaiono solo se il PC li ha. La pagina manda a Rust solo id fissi e
 * veri/falsi: Rust decide se si può (Windows può rifiutare). Frecce per muoversi fra i controlli, Esc chiude.
 */
const ControlPanel = forwardRef<HTMLDivElement, { opens: number }>(function ControlPanel({ opens }, ref) {
  const indicators = useIndicators();
  const [state, setState] = useState<ControlState | null>(null);
  const [dnd, setDnd] = useState(false);
  const [error, setError] = useState("");
  const [vol, setVol] = useState<number | null>(null);
  const list = useRef<HTMLDivElement>(null);
  const sendVolume = useRef(makeLatestSender((p: number) => void invoke("volume_set", { percent: p }), 40));

  useEffect(() => {
    setError("");
    setVol(null);
    invoke<ControlState | null>("control_state")
      .then((s) => s && setState(s))
      .catch(() => undefined);
    invoke<SettingRow[]>("settings_all")
      .then((rows) => setDnd(rows.find((r) => r.key === "notif.dnd")?.value === true))
      .catch(() => undefined);
    window.setTimeout(() => list.current?.querySelector<HTMLButtonElement>("button")?.focus(), 80);
  }, [opens]);

  const flip = async (t: Tile) => {
    setError("");
    const on = !t.on;
    if (t.id === "dnd") {
      setDnd(on);
      const r = await invoke<{ ok: boolean }>("settings_set", { key: "notif.dnd", value: on }).catch(() => ({ ok: false }));
      if (!r.ok) {
        setDnd(!on);
        setError("Non riesco a cambiarlo.");
      }
      return;
    }
    if (!state) return;
    const before = state;
    setState(withToggled(state, t.id, on));
    const ok = await invoke<boolean>("control_set", { id: t.id, on }).catch(() => false);
    if (!ok) {
      setState(before);
      setError(t.id === "awake" ? "Non riesco a cambiarlo." : `Windows non permette di cambiare ${t.label}.`);
    }
  };

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp" && e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
    if ((e.target as HTMLElement).tagName === "INPUT") return;
    e.preventDefault();
    const buttons = Array.from(list.current?.querySelectorAll<HTMLButtonElement>("button") ?? []);
    const cur = buttons.findIndex((b) => b === document.activeElement);
    buttons[nextIndex(cur, buttons.length, e.key === "ArrowDown" || e.key === "ArrowRight" ? 1 : -1)]?.focus();
  };

  const volume = indicators?.volume ?? null;
  const shown = vol ?? volume?.percent ?? 0;
  const muted = !!volume?.muted && vol === null;
  // Cattura dello schermo: Rust la fa partire dopo un attimo, così il pannello si chiude e non finisce nell'immagine.
  const snap = (kind: "screen" | "region") => void invoke("capture_run", { kind }).then(() => invoke("panel_hide"));
  const open = (id: string) => void invoke("eosmenu_run", { id }).then(() => invoke("panel_hide"));

  return (
    <div className="control" ref={ref} role="group" aria-label="Centro di controllo" onKeyDown={onKey}>
      <h2 className="panel-title">Centro di controllo</h2>
      <div ref={list} className="control-body">
        <div className="control-tiles">
          {tiles(state, dnd).map((t) => (
            <button key={t.id} type="button" className="control-tile" data-id={t.id} data-on={t.on ? "true" : "false"} aria-pressed={t.on} title={tileTitle(t)} onClick={() => void flip(t)}>
              <span className="control-tile-icon" aria-hidden>
                {ICONS[t.id]}
              </span>
              <span className="control-tile-text">
                <span className="control-tile-label">{t.label}</span>
                <span className="control-tile-sub">{t.sub}</span>
              </span>
            </button>
          ))}
        </div>
        {volume && (
          <div className="control-volume">
            <div className="control-volume-head">
              <span className="control-volume-label">Volume</span>
              <span className="control-volume-value">{muted ? "Silenzioso" : `${shown}%`}</span>
            </div>
            <input
              className="panel-slider"
              type="range"
              min={0}
              max={100}
              step={1}
              value={shown}
              aria-label="Volume"
              aria-valuetext={muted ? "Audio disattivato" : `${shown}%`}
              style={{ ["--fill" as string]: `${muted ? 0 : shown}%` }}
              onChange={(e) => {
                const p = Number(e.target.value);
                setVol(p);
                sendVolume.current(p);
                if (volume.muted && p > 0) void invoke("volume_mute", { on: false });
              }}
            />
          </div>
        )}
        <div className="control-group" role="group" aria-label="Cattura schermo">
          <span className="control-section" aria-hidden>
            Cattura schermo
          </span>
          <div className="control-capture">
            <button type="button" className="control-cap" onClick={() => snap("screen")}>
              Schermo intero
            </button>
            <button type="button" className="control-cap" onClick={() => snap("region")}>
              Selezione…
            </button>
          </div>
        </div>
        <div className="control-links">
          <div className="eosmenu-sep" role="separator" />
          <button type="button" className="control-link" onClick={() => open("settings")}>
            Impostazioni di EosOS…
          </button>
          <button type="button" className="control-link" onClick={() => open("winsettings")}>
            Impostazioni di Windows…
          </button>
        </div>
      </div>
      {error && (
        <div className="control-error" role="alert">
          {error}
        </div>
      )}
    </div>
  );
});

export default ControlPanel;
