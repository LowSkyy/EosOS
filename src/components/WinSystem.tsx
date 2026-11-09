import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  commandName,
  displayLine,
  sortStartup,
  timeoutLabel,
  TIMEOUTS,
  type Display,
  type Drive,
  type Power,
  type StartupApp,
  type SysInfo,
  type Theme,
} from "../lib/winsys";

function Switch({
  on,
  label,
  onChange,
}: {
  on: boolean;
  label: string;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      className={`switch${on ? " switch-on" : ""}`}
      onClick={() => onChange(!on)}
    >
      <span className="switch-knob" />
    </button>
  );
}

function Timeout({
  value,
  label,
  onChange,
}: {
  value: number;
  label: string;
  onChange: (m: number) => void;
}) {
  return (
    <select
      className="settings-select"
      value={value}
      aria-label={label}
      onChange={(e) => onChange(Number(e.target.value))}
    >
      {[
        ...(TIMEOUTS.includes(value as never)
          ? TIMEOUTS
          : [...TIMEOUTS, value].sort((a, b) => a - b)),
      ].map((m) => (
        <option key={m} value={m}>
          {timeoutLabel(m)}
        </option>
      ))}
    </select>
  );
}

/**
 * Le impostazioni di Windows fatte da EosOS, senza aprire l'app Impostazioni: informazioni sul sistema, aspetto (tema chiaro/scuro e trasparenza),
 * alimentazione, app all'avvio, archiviazione e schermi. La pagina manda solo id fissi e valori; Rust scrive sul sistema solo ciò che conosce.
 */
export default function WinSystem() {
  const [info, setInfo] = useState<SysInfo | null>(null);
  const [theme, setTheme] = useState<Theme | null>(null);
  const [power, setPower] = useState<Power | null>(null);
  const [startup, setStartup] = useState<StartupApp[]>([]);
  const [drives, setDrives] = useState<Drive[]>([]);
  const [displays, setDisplays] = useState<Display[]>([]);
  const [error, setError] = useState("");

  const load = useCallback(() => {
    void invoke<SysInfo | null>("winsys_info")
      .then(setInfo)
      .catch(() => undefined);
    void invoke<Theme | null>("winsys_theme")
      .then(setTheme)
      .catch(() => undefined);
    void invoke<Power | null>("winsys_power")
      .then(setPower)
      .catch(() => undefined);
    void invoke<StartupApp[]>("winsys_startup")
      .then((a) => setStartup(sortStartup(a)))
      .catch(() => undefined);
    void invoke<Drive[]>("winsys_drives")
      .then(setDrives)
      .catch(() => undefined);
    void invoke<Display[]>("winsys_displays")
      .then(setDisplays)
      .catch(() => undefined);
  }, []);
  useEffect(load, [load]);

  const fail = (what: string) =>
    setError(`Windows non ha accettato la modifica (${what}).`);

  const setThemeValue = async (
    which: "apps" | "system" | "transparency",
    on: boolean,
  ) => {
    setError("");
    if (!theme) return;
    const before = theme;
    setTheme({
      ...theme,
      ...(which === "apps"
        ? { apps_light: on }
        : which === "system"
          ? { system_light: on }
          : { transparency: on }),
    });
    const ok = await invoke<boolean>("winsys_theme_set", { which, on }).catch(
      () => false,
    );
    if (!ok) {
      setTheme(before);
      fail("tema");
    }
  };

  const setPowerValue = async (
    which: "screen-ac" | "screen-dc" | "sleep-ac" | "sleep-dc",
    minutes: number,
  ) => {
    setError("");
    const ok = await invoke<boolean>("winsys_power_set", {
      which,
      minutes,
    }).catch(() => false);
    if (!ok) fail("alimentazione");
    void invoke<Power | null>("winsys_power")
      .then(setPower)
      .catch(() => undefined);
  };

  const setStartupValue = async (name: string, on: boolean) => {
    setError("");
    setStartup((cur) =>
      cur.map((a) => (a.name === name ? { ...a, enabled: on } : a)),
    );
    const ok = await invoke<boolean>("winsys_startup_set", { name, on }).catch(
      () => false,
    );
    if (!ok) {
      fail("app all'avvio");
      void invoke<StartupApp[]>("winsys_startup")
        .then((a) => setStartup(sortStartup(a)))
        .catch(() => undefined);
    }
  };

  return (
    <section className="winsys" aria-label="Impostazioni di Windows">
      {error && (
        <p className="settings-error" role="alert">
          {error}
        </p>
      )}

      <div
        className="winsys-card"
        role="group"
        aria-label="Informazioni sul sistema"
      >
        <h2 className="winset-title">Informazioni sul sistema</h2>
        {info ? (
          <dl className="winsys-info">
            {(
              [
                ["Dispositivo", info.device],
                ["Utente", info.user],
                ["Windows", info.os],
                ["Processore", info.cpu],
                ["Memoria", info.memory],
                ["Acceso da", info.uptime],
              ] as const
            )
              .filter(([, v]) => v)
              .map(([k, v]) => (
                <div key={k} className="winsys-info-row">
                  <dt>{k}</dt>
                  <dd>{v}</dd>
                </div>
              ))}
          </dl>
        ) : (
          <p className="settings-empty">Leggo…</p>
        )}
      </div>

      <div className="winsys-card" role="group" aria-label="Aspetto di Windows">
        <h2 className="winset-title">Aspetto di Windows</h2>
        {theme ? (
          <>
            <div className="winsys-row">
              <span>
                <strong>Tema chiaro per le app</strong>
                <small>Spento: le app di Windows usano il tema scuro.</small>
              </span>
              <Switch
                on={theme.apps_light}
                label="Tema chiaro per le app"
                onChange={(v) => void setThemeValue("apps", v)}
              />
            </div>
            <div className="winsys-row">
              <span>
                <strong>Tema chiaro per Windows</strong>
                <small>Barra delle applicazioni e menu di sistema.</small>
              </span>
              <Switch
                on={theme.system_light}
                label="Tema chiaro per Windows"
                onChange={(v) => void setThemeValue("system", v)}
              />
            </div>
            <div className="winsys-row">
              <span>
                <strong>Effetti di trasparenza</strong>
                <small>Finestre e menu leggermente trasparenti.</small>
              </span>
              <Switch
                on={theme.transparency}
                label="Effetti di trasparenza"
                onChange={(v) => void setThemeValue("transparency", v)}
              />
            </div>
          </>
        ) : (
          <p className="settings-empty">Leggo…</p>
        )}
      </div>

      <div className="winsys-card" role="group" aria-label="Alimentazione">
        <h2 className="winset-title">Alimentazione</h2>
        {power ? (
          <>
            <div className="winsys-row">
              <span>
                <strong>Spegni lo schermo dopo</strong>
                <small>
                  {power.has_battery
                    ? "Con la corrente"
                    : "Quando il PC è inattivo"}
                </small>
              </span>
              <Timeout
                value={power.screen_ac}
                label="Spegni lo schermo dopo (con la corrente)"
                onChange={(m) => void setPowerValue("screen-ac", m)}
              />
            </div>
            <div className="winsys-row">
              <span>
                <strong>Sospendi dopo</strong>
                <small>
                  {power.has_battery
                    ? "Con la corrente"
                    : "Quando il PC è inattivo"}
                </small>
              </span>
              <Timeout
                value={power.sleep_ac}
                label="Sospendi dopo (con la corrente)"
                onChange={(m) => void setPowerValue("sleep-ac", m)}
              />
            </div>
            {power.has_battery && (
              <>
                <div className="winsys-row">
                  <span>
                    <strong>Spegni lo schermo dopo</strong>
                    <small>A batteria</small>
                  </span>
                  <Timeout
                    value={power.screen_dc}
                    label="Spegni lo schermo dopo (a batteria)"
                    onChange={(m) => void setPowerValue("screen-dc", m)}
                  />
                </div>
                <div className="winsys-row">
                  <span>
                    <strong>Sospendi dopo</strong>
                    <small>A batteria</small>
                  </span>
                  <Timeout
                    value={power.sleep_dc}
                    label="Sospendi dopo (a batteria)"
                    onChange={(m) => void setPowerValue("sleep-dc", m)}
                  />
                </div>
              </>
            )}
          </>
        ) : (
          <p className="settings-empty">Leggo…</p>
        )}
      </div>

      <div className="winsys-card" role="group" aria-label="App all'avvio">
        <h2 className="winset-title">App all'avvio</h2>
        {startup.length === 0 && (
          <p className="settings-empty">Nessuna app parte con Windows.</p>
        )}
        {startup.map((a) => (
          <div key={a.name} className="winsys-row">
            <span>
              <strong>{a.name}</strong>
              <small>{commandName(a.command)}</small>
            </span>
            <Switch
              on={a.enabled}
              label={`Avvia ${a.name} con Windows`}
              onChange={(v) => void setStartupValue(a.name, v)}
            />
          </div>
        ))}
      </div>

      <div className="winsys-card" role="group" aria-label="Archiviazione">
        <h2 className="winset-title">Archiviazione</h2>
        {drives.map((d) => (
          <div key={d.letter} className="winsys-drive">
            <div className="winsys-drive-head">
              <strong>{d.label ? `${d.label} (${d.letter})` : d.letter}</strong>
              <small>
                {d.free} liberi su {d.total}
              </small>
            </div>
            <div
              className="winsys-bar"
              role="progressbar"
              aria-valuenow={d.used_percent}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-label={`${d.letter} usato al ${d.used_percent}%`}
            >
              <div
                className="winsys-bar-fill"
                style={{ width: `${d.used_percent}%` }}
                data-high={d.used_percent >= 90}
              />
            </div>
          </div>
        ))}
      </div>

      <div className="winsys-card" role="group" aria-label="Schermi">
        <h2 className="winset-title">Schermi</h2>
        {displays.map((d, i) => (
          <div key={`${d.x},${d.y}`} className="winsys-row">
            <span>
              <strong>{d.name || `Schermo ${i + 1}`}</strong>
              <small>{displayLine(d)}</small>
            </span>
          </div>
        ))}
      </div>
    </section>
  );
}
