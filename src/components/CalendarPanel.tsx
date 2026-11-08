import { forwardRef, useEffect, useState } from "react";
import NotifList from "./NotifList";
import { dateLabel, monthGrid, monthTitle, msUntilMidnight, shiftMonth, todayOf, WEEKDAYS_SHORT, type YearMonth } from "../lib/calendar";

/**
 * Calendario del mese sotto l'orologio (dentro la finestra "panel"): giorni da lunedì, numeri di settimana ISO, il
 * giorno di oggi evidenziato, mese precedente e successivo, «Oggi». Frecce sinistra/destra e PagUp/PagGiù cambiano
 * mese, Home torna a oggi; Esc lo chiude (lo fa la finestra). Si riallinea da solo a mezzanotte.
 */
const CalendarPanel = forwardRef<HTMLDivElement, { opens: number }>(function CalendarPanel({ opens }, ref) {
  const [today, setToday] = useState(() => todayOf(new Date()));
  const [view, setView] = useState<YearMonth>(() => ({ y: today.y, m0: today.m0 }));

  // A ogni apertura si riparte dal mese di oggi.
  useEffect(() => {
    const t = todayOf(new Date());
    setToday(t);
    setView({ y: t.y, m0: t.m0 });
  }, [opens]);

  // A mezzanotte «oggi» cambia: si rilegge e si rimette il timer per la mezzanotte dopo.
  useEffect(() => {
    let timer: number;
    const arm = () => {
      timer = window.setTimeout(() => {
        setToday(todayOf(new Date()));
        arm();
      }, msUntilMidnight(new Date()));
    };
    arm();
    return () => window.clearTimeout(timer);
  }, []);

  const go = (delta: number) => setView((v) => shiftMonth(v, delta));
  const goToday = () => setView({ y: today.y, m0: today.m0 });

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowLeft" || e.key === "PageUp") go(-1);
      else if (e.key === "ArrowRight" || e.key === "PageDown") go(1);
      else if (e.key === "Home") goToday();
      else return;
      e.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const rows = monthGrid(view, today);
  const atToday = view.y === today.y && view.m0 === today.m0;

  return (
    <div className="calendar" ref={ref} role="group" aria-label="Notifiche e calendario">
      <NotifList />
      <div className="cal-head">
        <h2 className="cal-title" aria-live="polite">
          {monthTitle(view)}
        </h2>
        <div className="cal-nav">
          <button type="button" className="cal-btn" aria-label="Mese precedente" onClick={() => go(-1)}>
            ‹
          </button>
          <button type="button" className="cal-btn cal-today-btn" onClick={goToday} disabled={atToday}>
            Oggi
          </button>
          <button type="button" className="cal-btn" aria-label="Mese successivo" onClick={() => go(1)}>
            ›
          </button>
        </div>
      </div>
      <table className="cal-grid" aria-label={monthTitle(view)}>
        <thead>
          <tr>
            <th className="cal-week" scope="col" aria-label="Settimana">

            </th>
            {WEEKDAYS_SHORT.map((w, i) => (
              <th key={w} scope="col" className={i >= 5 ? "cal-weekend" : undefined}>
                {w}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={`${r.cells[0].y}-${r.cells[0].m0}-${r.cells[0].d}`}>
              <th className="cal-week" scope="row" aria-label={`Settimana ${r.week}`}>
                {r.week}
              </th>
              {r.cells.map((c) => (
                <td
                  key={`${c.m0}-${c.d}`}
                  className={`cal-day${c.inMonth ? "" : " cal-out"}${c.weekend ? " cal-weekend" : ""}${c.isToday ? " cal-today" : ""}`}
                  aria-label={dateLabel(c)}
                  aria-current={c.isToday ? "date" : undefined}
                >
                  {c.d}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
});

export default CalendarPanel;
