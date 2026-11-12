import type React from "react";
import { ART, type PackId, type Theme } from "./iconart";
import { tones, useDominant } from "./tint";

/** Una tessera nello stile di macOS: forma arrotondata, riflesso in alto, ombra. Con `pack` il disegno è dentro, altrimenti l'icona originale. */
export function MacTile({ pack, src, alt, theme = "light" }: { pack: PackId | null; src: string | null; alt: string; theme?: Theme }) {
  const art = pack ? ART[pack] : null;
  const dom = useDominant(!art ? src : null);
  // App senza disegno proprio: lo sfondo della tessera prende il colore dell'icona e il simbolo sta al centro, come le icone di terze parti su macOS.
  const bg = art ? (theme === "dark" ? art.dark : art.bg) : dom ? tones(dom, theme === "dark") : null;
  return (
    <div
      className="dock-icon mac-tile"
      data-pack={pack ?? "app"}
      data-theme={theme}
      data-tint={!art && dom ? "1" : undefined}
      style={bg ? ({ "--t1": bg[0], "--t2": bg[1] } as React.CSSProperties) : undefined}
      aria-label={alt}
    >
      {art ? (
        <svg viewBox="0 0 100 100" width="100%" height="100%" aria-hidden>
          {art.glyph(theme)}
        </svg>
      ) : src ? (
        <img className="mac-tile-img" src={src} alt={alt} draggable={false} />
      ) : (
        <span className="mac-tile-letter">{alt.charAt(0).toUpperCase()}</span>
      )}
    </div>
  );
}
