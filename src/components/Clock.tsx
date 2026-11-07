import { useEffect, useState } from "react";
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

  return <time className="clock" dateTime={now.toISOString()}>{formatClock(now)}</time>;
}
