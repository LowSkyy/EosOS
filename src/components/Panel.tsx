import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  batteryDetails,
  indicatorItems,
  makeLatestSender,
  networkDetails,
  openId,
  panelTitle,
  reconcileVolume,
  settingsLabel,
  type IndicatorKey,
  type VolumeValue,
} from "../lib/indicators";
import { Glyph } from "./IndicatorGlyph";
import StackPanel from "./StackPanel";
import EosMenuPanel from "./EosMenuPanel";
import ControlPanel from "./ControlPanel";
import TrayPanel from "./TrayPanel";
import CalendarPanel from "./CalendarPanel";
import MenuPanel from "./MenuPanel";
import { useBackdrop } from "./useBackdrop";
import { useIndicators } from "./useIndicators";

/** Contorno catturato attorno al pannello perché lo sfocato non sbavi ai bordi (px CSS). */
const BLUR_MARGIN = 28;
/** Ogni quanto si manda il valore del cursore mentre lo si trascina (ms). */
const SEND_MS = 40;

/**
 * Pannello sotto un indicatore (finestra "panel", vista ?view=panel): volume con cursore e silenzio,
 * dettagli di rete e di batteria. Lo apre Rust (`panel_toggle`) e lo chiude Esc, un clic fuori o un
 * secondo clic sull'indicatore. La pagina manda a Rust solo numeri e un id fisso per le Impostazioni.
 */
export default function Panel() {
  const state = useIndicators();
  const [key, setKey] = useState<IndicatorKey | null>(null);
  // Tendina del menu globale: indice del menu della barra aperto (null = il pannello è degli indicatori).
  const [menuIndex, setMenuIndex] = useState<number | null>(null);
  // Calendario sotto l'orologio.
  const [calendar, setCalendar] = useState(false);
  // Pila del Dock: indice della pila aperta (null = non è una pila).
  const [stackIndex, setStackIndex] = useState<number | null>(null);
  // Menu EosOS (il «menu mela» a sinistra della barra).
  const [eosMenu, setEosMenu] = useState(false);
  // Centro di controllo (interruttori rapidi, dal pulsante a destra nella barra).
  const [control, setControl] = useState(false);
  // App in background (icona nel Dock).
  const [tray, setTray] = useState(false);
  const [opens, setOpens] = useState(0);
  const [local, setLocal] = useState<VolumeValue | null>(null);
  const lastInput = useRef(0);
  const panel = useRef<HTMLDivElement>(null);
  const body = useRef<HTMLDivElement>(null);
  const blurCanvas = useRef<HTMLCanvasElement>(null);
  const slider = useRef<HTMLInputElement>(null);
  useBackdrop("panel_backdrop", panel, blurCanvas, BLUR_MARGIN, opens);

  const sendVolume = useRef(makeLatestSender((p: number) => void invoke("volume_set", { percent: p }), SEND_MS));

  useEffect(() => {
    const onOpen = listen<string>("panel-open", (e) => {
      setControl(e.payload === "control");
      setTray(e.payload === "tray");
      if (e.payload === "tray") {
        setKey(null);
        setMenuIndex(null);
        setCalendar(false);
        setStackIndex(null);
        setEosMenu(false);
        setOpens((n) => n + 1);
        return;
      }
      if (e.payload === "control") {
        setKey(null);
        setMenuIndex(null);
        setCalendar(false);
        setStackIndex(null);
        setEosMenu(false);
        setOpens((n) => n + 1);
        return;
      }
      if (e.payload === "eos") {
        setKey(null);
        setMenuIndex(null);
        setCalendar(false);
        setStackIndex(null);
        setEosMenu(true);
        setOpens((n) => n + 1);
        return;
      }
      setEosMenu(false);
      const st = /^stack:(\d+)$/.exec(e.payload);
      if (st) {
        setKey(null);
        setMenuIndex(null);
        setCalendar(false);
        setStackIndex(Number(st[1]));
        setOpens((n) => n + 1);
        return;
      }
      setStackIndex(null);
      if (e.payload === "calendar") {
        setKey(null);
        setMenuIndex(null);
        setCalendar(true);
        setOpens((n) => n + 1);
        return;
      }
      setCalendar(false);
      const m = /^menu:(\d+)$/.exec(e.payload);
      if (m) {
        setKey(null);
        setMenuIndex(Number(m[1]));
        setOpens((n) => n + 1);
        return;
      }
      setMenuIndex(null);
      if (e.payload === "battery" || e.payload === "network" || e.payload === "volume") {
        setKey(e.payload);
        setOpens((n) => n + 1);
        setLocal(null);
        lastInput.current = 0;
      }
    });
    const onClose = listen("panel-close", () => {
      setKey(null);
      setMenuIndex(null);
      setCalendar(false);
      setStackIndex(null);
      setEosMenu(false);
      setControl(false);
      setTray(false);
      setLocal(null);
    });
    return () => {
      void onOpen.then((f) => f());
      void onClose.then((f) => f());
    };
  }, []);

  // La finestra segue l'altezza del contenuto.
  useEffect(() => {
    const el = body.current;
    if (!el) return;
    const fit = () => void invoke("panel_fit", { height: Math.ceil(el.getBoundingClientRect().height) });
    fit();
    const ro = new ResizeObserver(fit);
    ro.observe(el);
    return () => ro.disconnect();
  }, [key, menuIndex, calendar, stackIndex, eosMenu, control, tray]);

  // Esc chiude; all'apertura il focus va al primo controllo.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        void invoke("panel_hide");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  useEffect(() => {
    if (key === "volume") slider.current?.focus();
  }, [key, opens]);

  const item = key ? indicatorItems(state).find((i) => i.key === key) : undefined;

  // Se l'indicatore sparisce mentre il pannello è aperto (cuffie scollegate, indicatore nascosto dalle
  // impostazioni) non resta una finestra vuota: si chiude.
  useEffect(() => {
    if (key && state && !item) void invoke("panel_hide");
  }, [key, state, item]);
  const incoming: VolumeValue | null = state?.volume ? { percent: state.volume.percent, muted: state.volume.muted } : null;
  const volume = reconcileVolume(local, incoming, Date.now() - lastInput.current);

  const setPercent = (p: number) => {
    lastInput.current = Date.now();
    const muted = p > 0 ? false : (volume?.muted ?? false);
    setLocal({ percent: p, muted });
    sendVolume.current(p);
    // Muovere il cursore con l'audio spento lo riaccende, come su macOS.
    if (volume?.muted && p > 0) void invoke("volume_mute", { on: false });
  };
  const toggleMute = () => {
    if (!volume) return;
    lastInput.current = Date.now();
    setLocal({ percent: volume.percent, muted: !volume.muted });
    void invoke("volume_mute", { on: !volume.muted });
  };
  const openSettings = (k: IndicatorKey) => {
    void invoke("indicator_open", { id: openId(k) });
    void invoke("panel_hide");
  };

  return (
    <div className="panel" ref={panel} role="dialog" aria-label={key ? panelTitle(key) : undefined}>
      <canvas className="panel-blur" ref={blurCanvas} aria-hidden />
      {menuIndex !== null && <MenuPanel ref={body} index={menuIndex} opens={opens} />}
      {calendar && <CalendarPanel ref={body} opens={opens} />}
      {stackIndex !== null && <StackPanel ref={body} index={stackIndex} opens={opens} />}
      {eosMenu && <EosMenuPanel ref={body} opens={opens} />}
      {control && <ControlPanel ref={body} opens={opens} />}
      {tray && <TrayPanel ref={body} opens={opens} />}
      {key && item && (
        <div className="panel-body" ref={body}>
          <h2 className="panel-title">{panelTitle(key)}</h2>

          {key === "volume" && volume && (
            <div className="panel-volume">
              <button type="button" className="panel-mute" onClick={toggleMute} aria-pressed={volume.muted} title={volume.muted ? "Riattiva l'audio" : "Disattiva l'audio"}>
                <Glyph item={{ ...item, icon: volume.muted ? "volume-muted" : item.icon }} percent={0} />
              </button>
              <input
                ref={slider}
                className="panel-slider"
                type="range"
                min={0}
                max={100}
                step={1}
                value={volume.percent}
                aria-label="Volume"
                aria-valuetext={volume.muted ? "Audio disattivato" : `${volume.percent}%`}
                style={{ ["--fill" as string]: `${volume.muted ? 0 : volume.percent}%` }}
                onChange={(e) => setPercent(Number(e.target.value))}
              />
              <span className="panel-percent">{volume.muted ? "—" : `${volume.percent}%`}</span>
            </div>
          )}
          {key === "volume" && state?.volume?.device && <div className="panel-device">{state.volume.device}</div>}

          {key === "network" && state?.network && (
            <dl className="panel-rows">
              {networkDetails(state.network).map((r) => (
                <div key={r.label} className="panel-row">
                  <dt>{r.label}</dt>
                  <dd>{r.value}</dd>
                </div>
              ))}
            </dl>
          )}

          {key === "battery" && state?.battery && (
            <>
              <div className="panel-battery">
                <Glyph item={item} percent={state.battery.percent} />
                <span className="panel-big">{state.battery.label}</span>
              </div>
              <dl className="panel-rows">
                {batteryDetails(state.battery).map((r) => (
                  <div key={r.label} className="panel-row">
                    <dt>{r.label}</dt>
                    <dd>{r.value}</dd>
                  </div>
                ))}
              </dl>
            </>
          )}

          <button type="button" className="panel-link" onClick={() => openSettings(key)}>
            {settingsLabel(key)}
          </button>
        </div>
      )}
    </div>
  );
}
