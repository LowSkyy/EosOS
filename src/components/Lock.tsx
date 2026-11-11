import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { msToNextMinute } from "../lib/clock";
import { canSubmit, errorText, IDLE_BACK_MS, lockClock, waitText, type LockAttempt, type LockInfo } from "../lib/lock";

/**
 * La schermata di blocco di EosOS (una finestra per monitor, vista `?view=lock`): sfondo del desktop sfocato, orologio e data grandi; muovendo il mouse o
 * premendo un tasto compaiono l'avatar, il nome e il campo di accesso. La password va a Rust, che la fa verificare a Windows (`LogonUser`): qui non
 * si salva, non si registra e si cancella dopo ogni tentativo. Con un account senza password basta un clic; con Windows Hello/PIN c'e' il pulsante dedicato.
 * Dopo qualche errore Rust impone un'attesa (la pagina la mostra, ma non e' lei a farla rispettare). Esc cancella e torna al solo orologio.
 */
export default function Lock() {
  const [info, setInfo] = useState<LockInfo | null>(null);
  const [now, setNow] = useState(() => new Date());
  const [wall, setWall] = useState<string | null>(null);
  const [awake, setAwake] = useState(false);
  const [pw, setPw] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [wait, setWait] = useState(0);
  const [shake, setShake] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const root = useRef<HTMLDivElement>(null);
  const idleTimer = useRef<number>(0);
  const lastMove = useRef({ x: -1, y: -1 });

  // La radice tiene il focus (riceve i tasti anche prima che compaia il campo).
  useEffect(() => {
    const take = () => {
      if (document.activeElement === document.body || document.activeElement === null) root.current?.focus();
    };
    take();
    window.addEventListener("focus", take);
    return () => window.removeEventListener("focus", take);
  }, []);

  // Informazioni dell'account (qualche tentativo: la finestra puo' aprirsi un attimo prima che Rust abbia finito di prepararle).
  useEffect(() => {
    let gone = false;
    let tries = 0;
    const load = () =>
      invoke<LockInfo | null>("lock_info")
        .then((i) => {
          if (gone) return;
          if (i) {
            setInfo(i);
            setWait(i.wait_secs);
          } else if (tries++ < 20) window.setTimeout(load, 150);
        })
        .catch(() => !gone && tries++ < 20 && window.setTimeout(load, 150));
    load();
    return () => {
      gone = true;
    };
  }, []);

  // Sfondo del desktop: i byte arrivano da Rust e diventano un indirizzo locale (blob:), revocato alla chiusura.
  useEffect(() => {
    let url: string | null = null;
    let gone = false;
    invoke<ArrayBuffer>("lock_wallpaper")
      .then((buf) => {
        if (gone || !buf || buf.byteLength === 0) return;
        url = URL.createObjectURL(new Blob([buf]));
        setWall(url);
      })
      .catch(() => undefined);
    return () => {
      gone = true;
      if (url) URL.revokeObjectURL(url);
    };
  }, []);

  // L'orologio si riallinea all'inizio di ogni minuto.
  useEffect(() => {
    let timer: number;
    const tick = () => {
      const d = new Date();
      setNow(d);
      timer = window.setTimeout(tick, msToNextMinute(d) + 20);
    };
    timer = window.setTimeout(tick, msToNextMinute(new Date()) + 20);
    return () => window.clearTimeout(timer);
  }, []);

  // Conto alla rovescia dell'attesa imposta da Rust.
  useEffect(() => {
    if (wait <= 0) return;
    const t = window.setInterval(() => setWait((w) => Math.max(0, w - 1)), 1000);
    return () => window.clearInterval(t);
  }, [wait > 0]); // eslint-disable-line react-hooks/exhaustive-deps

  // Mouse o tastiera: compare l'accesso; dopo un po' di quiete si torna al solo orologio.
  const wake = useCallback(() => {
    setAwake(true);
    window.clearTimeout(idleTimer.current);
    idleTimer.current = window.setTimeout(() => {
      setAwake(false);
      setPw("");
      setError("");
    }, IDLE_BACK_MS);
  }, []);
  useEffect(() => () => window.clearTimeout(idleTimer.current), []);

  // Il campo tiene il focus finche' l'accesso e' visibile.
  useEffect(() => {
    if (!awake || info?.mode !== "password") return;
    const t = window.setTimeout(() => input.current?.focus(), 30);
    return () => window.clearTimeout(t);
  }, [awake, info?.mode]);

  const failed = (a: LockAttempt) => {
    setPw("");
    setWait(a.wait_secs);
    setError(errorText(a.wait_secs));
    setShake((n) => n + 1);
  };

  const submit = async () => {
    if (!canSubmit(pw, busy, wait)) return;
    setBusy(true);
    setError("");
    const a = await invoke<LockAttempt>("lock_try_password", { password: pw }).catch(() => ({ ok: false, wait_secs: 0 }) as LockAttempt);
    setBusy(false);
    // Con la riuscita Rust chiude questa finestra: qui non resta niente da fare.
    if (!a.ok) failed(a);
  };

  const enterBlank = async () => {
    if (busy) return;
    setBusy(true);
    const ok = await invoke<boolean>("lock_try_blank").catch(() => false);
    setBusy(false);
    if (!ok) failed({ ok: false, wait_secs: 0 });
  };

  const hello = async () => {
    if (busy) return;
    setBusy(true);
    setError("");
    const ok = await invoke<boolean>("lock_try_hello").catch(() => false);
    setBusy(false);
    if (!ok) setError("Non sono riuscito a verificarti");
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    wake();
    if (e.key === "Escape") {
      e.preventDefault();
      window.clearTimeout(idleTimer.current);
      setPw("");
      setError("");
      setAwake(false);
    } else if (e.key === "Enter" && info?.mode === "blank") {
      e.preventDefault();
      void enterBlank();
    }
  };

  const onMouseMove = (e: React.MouseEvent) => {
    // Un movimento vero (qualche pixel), non il tremolio di un cursore fermo.
    const d = Math.abs(e.clientX - lastMove.current.x) + Math.abs(e.clientY - lastMove.current.y);
    lastMove.current = { x: e.clientX, y: e.clientY };
    if (d > 4) wake();
  };

  const clock = lockClock(now);
  const showLogin = awake && info !== null;
  const waiting = waitText(wait);
  return (
    <div
      ref={root}
      className="lock"
      data-awake={showLogin}
      onKeyDown={onKeyDown}
      onMouseMove={onMouseMove}
      onMouseDown={wake}
      onContextMenu={(e) => e.preventDefault()}
      tabIndex={-1}
      role="dialog"
      aria-label="Schermo bloccato"
    >
      <div className="lock-bg" style={wall ? { backgroundImage: `url(${wall})` } : undefined} data-fallback={!wall} aria-hidden />
      <div className="lock-shade" aria-hidden />
      <div className="lock-top">
        <div className="lock-day">{clock.day}</div>
        <div className="lock-time">{clock.time}</div>
      </div>
      <div className="lock-hint" aria-hidden>
        Muovi il mouse o premi un tasto
      </div>
      {info && (
        <div className="lock-login" aria-hidden={!showLogin}>
          <div className="lock-avatar" aria-hidden>
            {info.initial}
          </div>
          <div className="lock-name">{info.name}</div>
          {info.mode === "password" ? (
            <form
              key={shake}
              className="lock-form"
              data-shake={shake > 0}
              onSubmit={(e) => {
                e.preventDefault();
                void submit();
              }}
            >
              <input
                ref={input}
                className="lock-field"
                type="password"
                value={pw}
                autoComplete="off"
                spellCheck={false}
                placeholder="Inserisci la password"
                aria-label="Password"
                disabled={wait > 0}
                onChange={(e) => {
                  setPw(e.target.value);
                  setError("");
                }}
              />
              <button type="submit" className="lock-go" aria-label="Sblocca" disabled={!canSubmit(pw, busy, wait)} data-visible={pw.length > 0}>
                <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden>
                  <path d="M2.5 7h9M7.5 3l4 4-4 4" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </button>
            </form>
          ) : (
            <button type="button" className="lock-enter" onClick={() => void enterBlank()} disabled={busy}>
              Entra
            </button>
          )}
          <div className="lock-msg" role="alert" aria-live="polite">
            {waiting && wait > 0 ? waiting : error}
          </div>
          {info.hello && (
            <button type="button" className="lock-alt" onClick={() => void hello()} disabled={busy || wait > 0}>
              Usa PIN o Windows Hello
            </button>
          )}
        </div>
      )}
    </div>
  );
}
