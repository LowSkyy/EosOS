import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { formatClock, msToNextMinute } from "../lib/clock";

export default function Clock() {
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    let timer: number;
    const tick = () => {
      const d = new Date();
      setNow(d);
      // Si riallinea all'inizio di ogni minuto: niente drift.
      timer = window.setTimeout(tick, msToNextMinute(d) + 20);
    };
    timer = window.setTimeout(tick, msToNextMinute(new Date()) + 20);
    return () => window.clearTimeout(timer);
  }, []);

  // Un clic apre (o chiude) il calendario sotto l'orologio: la pagina dice a Rust dov'è il pulsante sullo schermo.
  const toggle = (el: HTMLElement) => {
    const r = el.getBoundingClientRect();
    void invoke("panel_toggle", { key: "calendar", left: r.left, top: r.top, width: r.width, height: r.height });
  };

  return (
    <button type="button" className="clock" title="Calendario" aria-haspopup="dialog" onClick={(e) => toggle(e.currentTarget)}>
      <time dateTime={now.toISOString()}>{formatClock(now)}</time>
    </button>
  );
}
