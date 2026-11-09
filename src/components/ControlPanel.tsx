import { forwardRef, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { tiles, tileTitle, withToggled, type ControlState, type Tile } from "../lib/control";
import { makeLatestSender } from "../lib/indicators";
import { nextIndex } from "../lib/eosmenu";
import { useIndicators } from "./useIndicators";

type SettingRow = { key: string; value: unknown };

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
  const open = (id: string) => void invoke("eosmenu_run", { id }).then(() => invoke("panel_hide"));

  return (
    <div className="control" ref={ref} role="group" aria-label="Centro di controllo" onKeyDown={onKey}>
      <h2 className="panel-title">Centro di controllo</h2>
      <div ref={list}>
        <div className="control-tiles">
          {tiles(state, dnd).map((t) => (
            <button key={t.id} type="button" className="control-tile" data-id={t.id} data-on={t.on ? "true" : "false"} aria-pressed={t.on} title={tileTitle(t)} onClick={() => void flip(t)}>
              <span className="control-tile-label">{t.label}</span>
              <span className="control-tile-sub">{t.sub}</span>
            </button>
          ))}
        </div>
        {volume && (
          <div className="control-volume">
            <span className="control-volume-label">Volume</span>
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
        <div className="eosmenu-sep" role="separator" />
        <button type="button" className="control-link" onClick={() => open("settings")}>
          Impostazioni di EosOS…
        </button>
        <button type="button" className="control-link" onClick={() => open("winsettings")}>
          Impostazioni di Windows…
        </button>
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
