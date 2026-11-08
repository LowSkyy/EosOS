import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { dateLabel, todayOf } from "../lib/calendar";
import { hhmm, isLatest, isOverdue, remindersTitle, weatherNotice, weatherSpoken, type Reminder, type Weather } from "../lib/oggi";

/**
 * Il blocco «Oggi» in cima al pannello sotto l'orologio: la data per esteso, il meteo (solo se lo hai acceso nelle
 * Impostazioni e hai scelto la città) e i tuoi promemoria (testo e ora facoltativa, salvati solo sul PC; nessun allarme).
 * La pagina non decide niente: ordine, limiti e controlli sono di Rust.
 */
export default function OggiBox({ opens }: { opens: number }) {
  const [today, setToday] = useState(() => todayOf(new Date()));
  const [now, setNow] = useState(() => hhmm(new Date()));
  const [list, setList] = useState<Reminder[]>([]);
  const [weather, setWeather] = useState<Weather | null>(null);
  const [text, setText] = useState("");
  const [time, setTime] = useState("");
  const [error, setError] = useState("");
  const seq = useRef(0);

  // A ogni apertura si rilegge tutto (data, ora, promemoria, meteo).
  useEffect(() => {
    setToday(todayOf(new Date()));
    setNow(hhmm(new Date()));
    setError("");
    invoke<Reminder[]>("oggi_reminders")
      .then(setList)
      .catch(() => undefined);
    const mine = ++seq.current;
    invoke<Weather>("oggi_weather")
      .then((w) => isLatest(mine, seq.current) && setWeather(w))
      .catch(() => undefined);
  }, [opens]);

  // L'ora corrente si aggiorna ogni minuto (per segnare ciò che è passato).
  useEffect(() => {
    const t = window.setInterval(() => setNow(hhmm(new Date())), 60_000);
    return () => window.clearInterval(t);
  }, []);

  const act = useCallback(async (cmd: string, args: Record<string, unknown>) => {
    const next = await invoke<Reminder[]>(cmd, args).catch(() => null);
    if (next) setList(next);
  }, []);

  const add = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    try {
      const next = await invoke<Reminder[]>("oggi_add", { text, time: time || null });
      setList(next);
      setText("");
      setTime("");
    } catch (err) {
      setError(typeof err === "string" ? err : "Non riesco ad aggiungerlo.");
    }
  };

  const notice = weatherNotice(weather);
  const hasDone = list.some((r) => r.done);
  return (
    <section className="oggi" aria-label="Oggi">
      <h2 className="oggi-date">{dateLabel(today)}</h2>
      {weather?.state === "ok" && (
        <p className="oggi-weather" aria-label={weatherSpoken(weather)}>
          <span className="oggi-glyph" aria-hidden>
            {weather.glyph}
          </span>
          <span className="oggi-temp" aria-hidden>
            {weather.temp}°
          </span>
          <span className="oggi-desc" aria-hidden>
            {weather.place} · {weather.label} · {weather.min}° / {weather.max}°
          </span>
        </p>
      )}
      {notice && <p className="oggi-notice">{notice}</p>}
      <div className="oggi-head">
        <h3 className="oggi-title">{remindersTitle(list)}</h3>
        {hasDone && (
          <button type="button" className="oggi-clear" onClick={() => void act("oggi_clear_done", {})}>
            Svuota completati
          </button>
        )}
      </div>
      {list.length === 0 && <p className="oggi-empty">Nessun promemoria.</p>}
      {list.length > 0 && (
        <ul className="oggi-list">
          {list.map((r) => (
            <li key={r.id} className={`oggi-item${r.done ? " oggi-done" : ""}${isOverdue(r, now) ? " oggi-overdue" : ""}`}>
              <button type="button" className="oggi-check" role="checkbox" aria-checked={r.done} aria-label={`${r.text}${r.time ? `, alle ${r.time}` : ""}`} onClick={() => void act("oggi_toggle", { id: r.id })}>
                {r.done ? "✓" : ""}
              </button>
              <span className="oggi-text">{r.text}</span>
              {r.time && <span className="oggi-time">{r.time}</span>}
              <button type="button" className="oggi-x" aria-label={`Togli il promemoria «${r.text}»`} onClick={() => void act("oggi_remove", { id: r.id })}>
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
      <form className="oggi-form" onSubmit={(e) => void add(e)}>
        <input className="oggi-input" type="text" value={text} maxLength={120} placeholder="Nuovo promemoria" aria-label="Nuovo promemoria" onChange={(e) => setText(e.target.value)} />
        <input className="oggi-hour" type="time" value={time} aria-label="Ora (facoltativa)" onChange={(e) => setTime(e.target.value)} />
        <button type="submit" className="oggi-add" disabled={text.trim() === ""}>
          Aggiungi
        </button>
      </form>
      {error && (
        <p className="oggi-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
