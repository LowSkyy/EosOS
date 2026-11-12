import type React from "react";
import type { PackId } from "../lib/iconpack";

/** Le icone disegnate apposta (viewBox 100): sfondo sfumato e simbolo, come le app di sistema di macOS. */
const ART: Record<PackId, { bg: [string, string]; glyph: React.ReactNode }> = {
  finder: {
    bg: ["#7fd0ff", "#1d7cf2"],
    glyph: (
      <g>
        <path d="M50 0h50v100H50z" fill="#0b5fd6" opacity="0.55" />
        <path d="M50 8c-4 12-4 30 0 46 3 12 3 26-2 38" fill="none" stroke="#fff" strokeOpacity="0.9" strokeWidth="3" strokeLinecap="round" />
        <rect x="26" y="28" width="6" height="16" rx="3" fill="#102a54" />
        <rect x="68" y="28" width="6" height="16" rx="3" fill="#102a54" />
        <path d="M24 66c14 14 38 14 52 0" fill="none" stroke="#102a54" strokeWidth="4.5" strokeLinecap="round" />
      </g>
    ),
  },
  settings: {
    bg: ["#c8ccd4", "#6a6f7a"],
    glyph: (
      <g>
        <circle cx="50" cy="50" r="26" fill="none" stroke="#3a3d45" strokeWidth="13" strokeDasharray="6.8 6.8" />
        <circle cx="50" cy="50" r="22" fill="#4a4e58" />
        <circle cx="50" cy="50" r="10" fill="#d9dce2" />
      </g>
    ),
  },
  notes: {
    bg: ["#fff7c2", "#ffd54a"],
    glyph: (
      <g>
        <rect x="0" y="0" width="100" height="24" fill="#f0b400" opacity="0.9" />
        {[40, 54, 68, 82].map((y) => (
          <path key={y} d={`M16 ${y}h68`} stroke="#b28a00" strokeOpacity="0.55" strokeWidth="3" strokeLinecap="round" />
        ))}
      </g>
    ),
  },
  calculator: {
    bg: ["#4a4a50", "#1c1c1f"],
    glyph: (
      <g>
        <rect x="16" y="14" width="68" height="20" rx="5" fill="#2b2b2f" />
        <text x="78" y="30" fontSize="15" fill="#fff" textAnchor="end" fontFamily="sans-serif">
          0
        </text>
        {[0, 1, 2, 3].flatMap((r) =>
          [0, 1, 2, 3].map((c) => <rect key={`${r}${c}`} x={16 + c * 17.5} y={42 + r * 12.5} width="15" height="10" rx="3.5" fill={c === 3 ? "#ff9f0a" : "#6a6a70"} />),
        )}
      </g>
    ),
  },
  paint: {
    bg: ["#ffffff", "#e7e9ef"],
    glyph: (
      <g>
        <path d="M50 18c-22 0-36 14-36 32 0 16 12 30 28 30 9 0 8-8 3-12-4-4 0-10 6-10h10c10 0 16-6 16-16 0-14-14-24-27-24z" fill="#f3f3f5" stroke="#b9bcc6" strokeWidth="2.5" />
        <circle cx="34" cy="40" r="6" fill="#ff453a" />
        <circle cx="52" cy="32" r="6" fill="#ffd60a" />
        <circle cx="68" cy="44" r="6" fill="#30d158" />
        <circle cx="36" cy="60" r="6" fill="#0a84ff" />
      </g>
    ),
  },
  terminal: {
    bg: ["#3a3a3f", "#0d0d0f"],
    glyph: (
      <g fill="none" stroke="#f5f5f7" strokeWidth="6" strokeLinecap="round" strokeLinejoin="round">
        <path d="M26 36l18 14-18 14" />
        <path d="M52 66h24" />
      </g>
    ),
  },
  monitor: {
    bg: ["#3a3a3f", "#0d0d0f"],
    glyph: (
      <g fill="none" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 56h18l8-22 14 38 10-26h26" stroke="#30d158" strokeWidth="5" />
        <path d="M12 82h76" stroke="#ffffff" strokeOpacity="0.25" strokeWidth="3" />
      </g>
    ),
  },
  snip: {
    bg: ["#8fd3ff", "#2a82ee"],
    glyph: (
      <g fill="none" stroke="#fff" strokeWidth="5" strokeLinecap="round">
        <path d="M30 22v12M30 22h12M70 22v12M70 22H58M30 78V66M30 78h12M70 78V66M70 78H58" />
        <circle cx="50" cy="50" r="6" fill="#fff" stroke="none" />
      </g>
    ),
  },
  store: {
    bg: ["#6cc4ff", "#1472f0"],
    glyph: (
      <g fill="none" stroke="#fff" strokeWidth="7" strokeLinecap="round" strokeLinejoin="round">
        <path d="M30 70L50 26l20 44M37 56h26" />
      </g>
    ),
  },
  photos: {
    bg: ["#ffffff", "#eceef3"],
    glyph: (
      <g>
        {(
          [
            ["#ff453a", 0],
            ["#ff9f0a", 45],
            ["#ffd60a", 90],
            ["#30d158", 135],
            ["#0a84ff", 180],
            ["#bf5af2", 225],
            ["#ff375f", 270],
            ["#64d2ff", 315],
          ] as [string, number][]
        ).map(([c, a]) => (
          <ellipse key={a} cx="50" cy="31" rx="10" ry="19" fill={c} opacity="0.8" transform={`rotate(${a} 50 50)`} style={{ mixBlendMode: "multiply" }} />
        ))}
      </g>
    ),
  },
  camera: {
    bg: ["#505057", "#1b1b1e"],
    glyph: (
      <g>
        <rect x="14" y="30" width="72" height="46" rx="9" fill="#2a2a2e" stroke="#9a9aa2" strokeWidth="3" />
        <circle cx="50" cy="53" r="15" fill="#101114" stroke="#9a9aa2" strokeWidth="3" />
        <circle cx="50" cy="53" r="7" fill="#2d6cdf" />
      </g>
    ),
  },
  clock: {
    bg: ["#ffffff", "#e9ebf0"],
    glyph: (
      <g>
        <circle cx="50" cy="50" r="38" fill="#111" />
        <path d="M50 24v28l18 10" fill="none" stroke="#fff" strokeWidth="4.5" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="50" cy="50" r="3" fill="#ff9f0a" />
      </g>
    ),
  },
};

/** Una tessera nello stile di macOS: forma arrotondata, riflesso in alto, ombra. Con `pack` il disegno è dentro, altrimenti l'icona originale. */
export function MacTile({ pack, src, alt }: { pack: PackId | null; src: string | null; alt: string }) {
  const art = pack ? ART[pack] : null;
  return (
    <div className="dock-icon mac-tile" data-pack={pack ?? "app"} style={art ? ({ "--t1": art.bg[0], "--t2": art.bg[1] } as React.CSSProperties) : undefined} aria-label={alt}>
      {art ? (
        <svg viewBox="0 0 100 100" width="100%" height="100%" aria-hidden>
          {art.glyph}
        </svg>
      ) : src ? (
        <img className="mac-tile-img" src={src} alt={alt} draggable={false} />
      ) : (
        <span className="mac-tile-letter">{alt.charAt(0).toUpperCase()}</span>
      )}
    </div>
  );
}
