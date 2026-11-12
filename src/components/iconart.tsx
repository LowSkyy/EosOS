import type React from "react";

/**
 * Disegni del pacchetto di icone in stile macOS (viewBox 100). Servono a due cose: le tessere del Dock, del Launcher e del Launchpad
 * (`MacIcon.tsx`) e le icone di cartelle e file di Esplora file, generate in `.ico` da `scripts/build-icons.mjs`.
 *
 * Tema: ogni disegno ha la versione chiara e quella scura (come le icone di macOS con l'aspetto scuro).
 */

export type Theme = "light" | "dark";

export type PackId =
  | "finder"
  | "settings"
  | "notes"
  | "calculator"
  | "paint"
  | "terminal"
  | "monitor"
  | "snip"
  | "store"
  | "photos"
  | "camera"
  | "clock";

export type FileId = "folder" | "file" | "text" | "image" | "audio" | "video" | "zip" | "pdf" | "word" | "excel" | "powerpoint" | "code";

type Art = {
  bg: [string, string];
  dark: [string, string];
  glyph: (t: Theme) => React.ReactNode;
};

/** Le tessere (icone di app): sfondo sfumato e simbolo. */
export const ART: Record<PackId, Art> = {
  finder: {
    // Il Finder di macOS: volto diviso a meta', lato sinistro azzurro chiaro e destro blu.
    bg: ["#8fd8ff", "#2b8cf6"],
    dark: ["#3b7fc4", "#14407f"],
    glyph: (t) => (
      <g>
        <defs>
          <linearGradient id={`fr-${t}`} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor={t === "dark" ? "#1c63c2" : "#2a95ff"} />
            <stop offset="1" stopColor={t === "dark" ? "#0b3a8a" : "#0a58d6"} />
          </linearGradient>
        </defs>
        <path d="M51 0h49v100H51c-2-26 2-52 0-100z" fill={`url(#fr-${t})`} />
        <path d="M51 0c3 28-3 54 0 100" fill="none" stroke="#fff" strokeOpacity={t === "dark" ? 0.55 : 0.9} strokeWidth="2.4" />
        <rect x="27" y="26" width="6.5" height="17" rx="3.2" fill="#0d2d5e" />
        <rect x="66" y="26" width="6.5" height="17" rx="3.2" fill="#0d2d5e" />
        <path d="M58 34c-3 14-6 22-10 27" fill="none" stroke="#0d2d5e" strokeWidth="2.6" strokeLinecap="round" />
        <path d="M22 66c14 15 41 15 56 0" fill="none" stroke="#0d2d5e" strokeWidth="4.6" strokeLinecap="round" />
      </g>
    ),
  },
  settings: {
    bg: ["#cfd3da", "#6e737e"],
    dark: ["#5b5f69", "#2b2d33"],
    glyph: (t) => (
      <g>
        <circle cx="50" cy="50" r="26" fill="none" stroke={t === "dark" ? "#9da1ab" : "#3a3d45"} strokeWidth="13" strokeDasharray="6.8 6.8" />
        <circle cx="50" cy="50" r="22" fill={t === "dark" ? "#b6bac4" : "#4a4e58"} />
        <circle cx="50" cy="50" r="10" fill={t === "dark" ? "#33363d" : "#d9dce2"} />
      </g>
    ),
  },
  notes: {
    bg: ["#fff7c2", "#ffd54a"],
    dark: ["#4a4a50", "#232326"],
    glyph: (t) => (
      <g>
        <rect x="0" y="0" width="100" height="24" fill={t === "dark" ? "#c98f00" : "#f0b400"} opacity="0.95" />
        {[40, 54, 68, 82].map((y) => (
          <path key={y} d={`M16 ${y}h68`} stroke={t === "dark" ? "#f2e08a" : "#b28a00"} strokeOpacity={t === "dark" ? 0.5 : 0.55} strokeWidth="3" strokeLinecap="round" />
        ))}
      </g>
    ),
  },
  calculator: {
    bg: ["#4a4a50", "#1c1c1f"],
    dark: ["#3a3a3f", "#0f0f11"],
    glyph: () => (
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
    dark: ["#46464c", "#222226"],
    glyph: (t) => (
      <g>
        <path d="M50 18c-22 0-36 14-36 32 0 16 12 30 28 30 9 0 8-8 3-12-4-4 0-10 6-10h10c10 0 16-6 16-16 0-14-14-24-27-24z" fill={t === "dark" ? "#2d2d31" : "#f3f3f5"} stroke={t === "dark" ? "#7b7d86" : "#b9bcc6"} strokeWidth="2.5" />
        <circle cx="34" cy="40" r="6" fill="#ff453a" />
        <circle cx="52" cy="32" r="6" fill="#ffd60a" />
        <circle cx="68" cy="44" r="6" fill="#30d158" />
        <circle cx="36" cy="60" r="6" fill="#0a84ff" />
      </g>
    ),
  },
  terminal: {
    bg: ["#3a3a3f", "#0d0d0f"],
    dark: ["#2c2c30", "#050506"],
    glyph: () => (
      <g fill="none" stroke="#f5f5f7" strokeWidth="6" strokeLinecap="round" strokeLinejoin="round">
        <path d="M26 36l18 14-18 14" />
        <path d="M52 66h24" />
      </g>
    ),
  },
  monitor: {
    bg: ["#3a3a3f", "#0d0d0f"],
    dark: ["#2c2c30", "#050506"],
    glyph: () => (
      <g fill="none" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 56h18l8-22 14 38 10-26h26" stroke="#30d158" strokeWidth="5" />
        <path d="M12 82h76" stroke="#ffffff" strokeOpacity="0.25" strokeWidth="3" />
      </g>
    ),
  },
  snip: {
    bg: ["#8fd3ff", "#2a82ee"],
    dark: ["#2f6fb8", "#123a73"],
    glyph: () => (
      <g fill="none" stroke="#fff" strokeWidth="5" strokeLinecap="round">
        <path d="M30 22v12M30 22h12M70 22v12M70 22H58M30 78V66M30 78h12M70 78V66M70 78H58" />
        <circle cx="50" cy="50" r="6" fill="#fff" stroke="none" />
      </g>
    ),
  },
  store: {
    bg: ["#6cc4ff", "#1472f0"],
    dark: ["#2c6fb4", "#0f3a85"],
    glyph: () => (
      <g fill="none" stroke="#fff" strokeWidth="7" strokeLinecap="round" strokeLinejoin="round">
        <path d="M30 70L50 26l20 44M37 56h26" />
      </g>
    ),
  },
  photos: {
    bg: ["#ffffff", "#eceef3"],
    dark: ["#46464c", "#222226"],
    glyph: (t) => (
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
          <ellipse key={a} cx="50" cy="31" rx="10" ry="19" fill={c} opacity="0.8" transform={`rotate(${a} 50 50)`} style={{ mixBlendMode: t === "dark" ? "screen" : "multiply" }} />
        ))}
      </g>
    ),
  },
  camera: {
    bg: ["#505057", "#1b1b1e"],
    dark: ["#3a3a3f", "#0c0c0e"],
    glyph: () => (
      <g>
        <rect x="14" y="30" width="72" height="46" rx="9" fill="#2a2a2e" stroke="#9a9aa2" strokeWidth="3" />
        <circle cx="50" cy="53" r="15" fill="#101114" stroke="#9a9aa2" strokeWidth="3" />
        <circle cx="50" cy="53" r="7" fill="#2d6cdf" />
      </g>
    ),
  },
  clock: {
    bg: ["#ffffff", "#e9ebf0"],
    dark: ["#46464c", "#222226"],
    glyph: (t) => (
      <g>
        <circle cx="50" cy="50" r="38" fill={t === "dark" ? "#f2f2f5" : "#111"} />
        <path d="M50 24v28l18 10" fill="none" stroke={t === "dark" ? "#111" : "#fff"} strokeWidth="4.5" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="50" cy="50" r="3" fill="#ff9f0a" />
      </g>
    ),
  },
};

/** Icone di cartelle e file (Esplora file): forme intere, senza tessera. */
/** Le cartelle speciali hanno un simbolo impresso sul davanti, come in macOS. */
export type FolderBadge = "home" | "desktop" | "documents" | "downloads" | "pictures" | "music" | "videos" | "drive";

export const FOLDER_GLYPHS: Record<Exclude<FolderBadge, "drive">, React.ReactNode> = {
  home: <path d="M50 46l-13 11v13h9v-9h8v9h9V57z" />,
  desktop: (
    <g>
      <rect x="35" y="47" width="30" height="18" rx="2.5" />
      <rect x="44" y="67" width="12" height="2.6" rx="1.3" />
    </g>
  ),
  documents: <path d="M40 45h13l7 7v19a2 2 0 0 1-2 2H40a2 2 0 0 1-2-2V47a2 2 0 0 1 2-2zm12 1.5V53h6.5z" fillRule="evenodd" />,
  downloads: (
    <g>
      <path d="M50 45v15m-7-6 7 7 7-7" fill="none" strokeWidth="3.4" strokeLinecap="round" strokeLinejoin="round" />
      <rect x="38" y="69" width="24" height="3.2" rx="1.6" />
    </g>
  ),
  pictures: (
    <g>
      <rect x="35" y="46" width="30" height="24" rx="3" fill="none" strokeWidth="3" />
      <circle cx="43" cy="54" r="2.6" />
      <path d="M37 68l9-9 6 6 4-4 7 7z" />
    </g>
  ),
  music: <path d="M45 67.5a4.5 4.5 0 1 1-3-4.2V48l17-3.5v19.7a4.5 4.5 0 1 1-3-4.2V51l-11 2.3z" />,
  videos: (
    <g>
      <rect x="35" y="47" width="30" height="22" rx="3" fill="none" strokeWidth="3" />
      <path d="M46.5 52.5v11l9-5.5z" />
    </g>
  ),
};

/** La cartella di macOS: dietro piu' scuro con la linguetta a sinistra, davanti piu' chiaro con il bordo luminoso. */
export function folderArt(t: Theme, badge?: FolderBadge): React.ReactNode {
  const dark = t === "dark";
  const ink = dark ? "#0f3f86" : "#1672d6";
  return (
    <g>
      <defs>
        <linearGradient id={`fb-${t}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor={dark ? "#2a7fe0" : "#4fb4fb"} />
          <stop offset="1" stopColor={dark ? "#1859b8" : "#2590ee"} />
        </linearGradient>
        <linearGradient id={`ff-${t}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor={dark ? "#52a8f6" : "#93d8ff"} />
          <stop offset="0.5" stopColor={dark ? "#3a8eea" : "#62bdfc"} />
          <stop offset="1" stopColor={dark ? "#2a74d8" : "#3ea2f6"} />
        </linearGradient>
      </defs>
      <path d="M6 27q0-8 8-8h19q4.2 0 6.8 3.2l2.4 2.9q2.3 2.9 6 2.9H86q8 0 8 8v40q0 8-8 8H14q-8 0-8-8z" fill={`url(#fb-${t})`} />
      <path d="M6 42q0-8 8-8h72q8 0 8 8v34q0 8-8 8H14q-8 0-8-8z" fill={`url(#ff-${t})`} />
      <path d="M7 42q0-7 7-7h72q7 0 7 7" fill="none" stroke="#fff" strokeOpacity={dark ? 0.35 : 0.7} strokeWidth="1.3" />
      <path d="M7 76q0 7 7 7h72q7 0 7-7" fill="none" stroke={dark ? "#0b2f6b" : "#1667c4"} strokeOpacity="0.35" strokeWidth="1.3" />
      {badge && badge !== "drive" && (
        <g fill={ink} stroke={ink} strokeWidth="0" opacity={dark ? 0.75 : 0.5}>
          {FOLDER_GLYPHS[badge]}
        </g>
      )}
    </g>
  );
}

/** Un disco (unita'): scatola metallica con la spia, come i dischi di macOS. */
export function driveArt(t: Theme): React.ReactNode {
  const dark = t === "dark";
  return (
    <g>
      <defs>
        <linearGradient id={`dv-${t}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor={dark ? "#8a8d96" : "#f4f5f8"} />
          <stop offset="1" stopColor={dark ? "#55585f" : "#b9bcc6"} />
        </linearGradient>
      </defs>
      <path d="M22 24h56l12 38v10a8 8 0 0 1-8 8H18a8 8 0 0 1-8-8V62z" fill={`url(#dv-${t})`} stroke={dark ? "#3c3e44" : "#9a9da8"} strokeWidth="1.5" />
      <path d="M10.5 62h79" stroke={dark ? "#3c3e44" : "#9a9da8"} strokeWidth="1.5" />
      <circle cx="78" cy="71" r="2.6" fill="#30d158" />
    </g>
  );
}

export const FILE_ART: Record<FileId, (t: Theme) => React.ReactNode> = {
  folder: (t) => folderArt(t),
  file: (t) => page(t, null),
  text: (t) =>
    page(
      t,
      <g stroke={t === "dark" ? "#8d909a" : "#9ea2ad"} strokeWidth="3" strokeLinecap="round">
        <path d="M30 44h40M30 54h40M30 64h40M30 74h26" />
      </g>,
    ),
  image: (t) =>
    page(
      t,
      <g>
        <circle cx="42" cy="52" r="5" fill="#ffd60a" />
        <path d="M26 78l16-18 12 12 8-8 14 14z" fill="#30d158" />
      </g>,
    ),
  audio: (t) => page(t, <path d="M42 70V44l22-5v26" fill="none" stroke="#ff375f" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round" />, "#ff375f"),
  video: (t) => page(t, <path d="M40 46l22 12-22 12z" fill="#bf5af2" />, "#bf5af2"),
  zip: (t) => page(t, <path d="M50 24v50M44 30h6M50 36h6M44 42h6M50 48h6M44 54h6" stroke="#8e8e93" strokeWidth="3" strokeLinecap="round" fill="none" />, "#8e8e93"),
  pdf: (t) => page(t, <text x="50" y="66" fontSize="20" fontWeight="700" textAnchor="middle" fill="#ff453a" fontFamily="sans-serif">PDF</text>, "#ff453a"),
  word: (t) => page(t, <text x="50" y="68" fontSize="30" fontWeight="700" textAnchor="middle" fill="#2b6fe6" fontFamily="sans-serif">W</text>, "#2b6fe6"),
  excel: (t) => page(t, <text x="50" y="68" fontSize="30" fontWeight="700" textAnchor="middle" fill="#1f9d55" fontFamily="sans-serif">X</text>, "#1f9d55"),
  powerpoint: (t) => page(t, <text x="50" y="68" fontSize="30" fontWeight="700" textAnchor="middle" fill="#e8590c" fontFamily="sans-serif">P</text>, "#e8590c"),
  code: (t) => page(t, <path d="M44 48l-10 10 10 10M58 48l10 10-10 10" fill="none" stroke="#0a84ff" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round" />, "#0a84ff"),
};

/** Una pagina con l'angolo piegato, il contenuto e (se c'e') una striscia colorata in basso. */
function page(t: Theme, content: React.ReactNode, accent?: string) {
  const fill = t === "dark" ? ["#4a4b52", "#34353b"] : ["#ffffff", "#e9ebf0"];
  return (
    <g>
      <defs>
        <linearGradient id={`pg-${t}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor={fill[0]} />
          <stop offset="1" stopColor={fill[1]} />
        </linearGradient>
      </defs>
      <path d="M20 6h40l22 22v60a6 6 0 0 1-6 6H20a6 6 0 0 1-6-6V12a6 6 0 0 1 6-6z" fill={`url(#pg-${t})`} stroke={t === "dark" ? "#6a6c75" : "#c9ccd6"} strokeWidth="1.5" />
      <path d="M60 6v16a6 6 0 0 0 6 6h16z" fill={t === "dark" ? "#6a6c75" : "#d5d8e0"} />
      {accent && <rect x="14" y="82" width="68" height="6" rx="3" fill={accent} opacity="0.9" />}
      {content}
    </g>
  );
}
