import {
  batteryFill,
  volumeWaves,
  wifiWaves,
  type IndicatorItem,
} from "../lib/indicators";

/** Icona di un indicatore: SVG 16x16 che prende il colore della barra (`currentColor`). */
export function Glyph({ item, percent }: { item: IndicatorItem; percent: number }) {
  const icon = item.icon;
  const common = { width: 16, height: 16, viewBox: "0 0 16 16", "aria-hidden": true, className: "indicator-glyph" } as const;

  if (item.key === "battery") {
    const fill = batteryFill(percent);
    return (
      <svg {...common} viewBox="0 0 24 16" width={24}>
        <rect x="1" y="3" width="19" height="10" rx="2.5" fill="none" stroke="currentColor" strokeWidth="1.2" opacity="0.8" />
        <rect x="21" y="6" width="2" height="4" rx="1" fill="currentColor" opacity="0.8" />
        <rect x="2.6" y="4.6" width={Math.max(fill * 15.8, fill > 0 ? 1 : 0)} height="6.8" rx="1.3" fill="currentColor" className="indicator-fill" />
        {icon === "battery-charging" && (
          <path d="M11.5 2.5 7 9h3.2l-.7 4.5L14 7h-3.3z" fill="var(--bar-bg-solid)" stroke="currentColor" strokeWidth="0.6" strokeLinejoin="round" />
        )}
        {icon === "battery-plugged" && (
          <path d="M9 4v2.2M13 4v2.2M8 6.2h6v1.6a3 3 0 0 1-6 0zM11 10.8V12.5" fill="none" stroke="var(--bar-bg-solid)" strokeWidth="1.1" strokeLinecap="round" />
        )}
      </svg>
    );
  }

  if (item.key === "network") {
    const w = wifiWaves(icon);
    if (w !== null) {
      // Tre archi concentrici e un punto: gli archi oltre il livello sono sbiaditi.
      const arc = (r: number, on: boolean) => (
        <path d={`M${8 - r} ${12 - r * 0.62}a${r * 1.6} ${r * 1.6} 0 0 1 ${r * 2} 0`} fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" opacity={on ? 1 : 0.28} />
      );
      return (
        <svg {...common}>
          {arc(6.2, w >= 3 && icon !== "wifi-alert")}
          {arc(4.2, w >= 2 && icon !== "wifi-alert")}
          {arc(2.2, w >= 1 && icon !== "wifi-alert")}
          <circle cx="8" cy="12.6" r="1.1" fill="currentColor" opacity={icon === "wifi-alert" ? 0.28 : 1} />
          {icon === "wifi-alert" && <path d="M12.5 2.5v4M12.5 8.6v.2" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />}
        </svg>
      );
    }
    if (icon === "ethernet" || icon === "ethernet-alert") {
      return (
        <svg {...common}>
          <rect x="5.5" y="2" width="5" height="4" rx="0.8" fill="currentColor" />
          <path d="M8 6v3M3 9h10M3 9v2.5M8 9v2.5M13 9v2.5" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          <rect x="1.8" y="11" width="2.8" height="2.6" rx="0.5" fill="currentColor" />
          <rect x="6.6" y="11" width="2.8" height="2.6" rx="0.5" fill="currentColor" />
          <rect x="11.4" y="11" width="2.8" height="2.6" rx="0.5" fill="currentColor" />
          {icon === "ethernet-alert" && <circle cx="13.6" cy="3.2" r="2" fill="currentColor" />}
        </svg>
      );
    }
    if (icon === "cellular") {
      return (
        <svg {...common}>
          {[3, 6, 9, 12].map((h, i) => (
            <rect key={h} x={2 + i * 3.4} y={14 - h} width="2.2" height={h} rx="0.6" fill="currentColor" />
          ))}
        </svg>
      );
    }
    // net-off: Wi-Fi sbiadito con una barra obliqua.
    return (
      <svg {...common}>
        <path d="M1.8 7a9 9 0 0 1 12.4 0M4 9.6a5.8 5.8 0 0 1 8 0M6.2 12.2a2.6 2.6 0 0 1 3.6 0" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" opacity="0.4" />
        <path d="M2.5 2.5l11 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    );
  }

  // Altoparlante con 0-3 onde; spento = croce.
  const waves = volumeWaves(icon);
  return (
    <svg {...common}>
      <path d="M2 6h2.6L8 3v10L4.6 10H2z" fill="currentColor" />
      {icon === "volume-muted" ? (
        <path d="M10.4 6l4 4M14.4 6l-4 4" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
      ) : (
        [1, 2, 3].map((n) => (
          <path key={n} d={`M${9.6 + n * 1.5} ${8 - n * 1.6}a${n * 1.9} ${n * 1.9} 0 0 1 0 ${n * 3.2}`} fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" opacity={waves >= n ? 1 : 0.25} />
        ))
      )}
    </svg>
  );
}
