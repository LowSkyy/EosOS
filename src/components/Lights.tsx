/** I semafori di tutte le finestre di EosOS (Finder, Impostazioni, Mission Control): stessi colori, simboli e ordine dei cerchi che
 *  EosOS mette sulle finestre delle app (`src-tauri/src/lights.rs`). A sinistra chiudi, riduci, ingrandisci (macOS); a destra
 *  riduci, ingrandisci, chiudi (al posto dei tasti di Windows). Il lato lo sceglie l'impostazione «Lato dei semafori». */

export type LightKind = "close" | "min" | "zoom";

/** I cerchi nell'ordine del lato scelto. Pura, testata. */
export function lightsOrder(left: boolean): LightKind[] {
  return left ? ["close", "min", "zoom"] : ["min", "zoom", "close"];
}

const LABELS: Record<LightKind, string> = { close: "Chiudi", min: "Riduci a icona", zoom: "Ingrandisci" };

export function Lights({
  left,
  active = true,
  enabled = { close: true, min: true, zoom: true },
  onAction,
  className = "",
}: {
  left: boolean;
  /** Finestra in primo piano: cerchi colorati (altrimenti grigi, come su macOS; passando il mouse tornano colorati). */
  active?: boolean;
  enabled?: Partial<Record<LightKind, boolean>>;
  onAction: (k: LightKind) => void;
  className?: string;
}) {
  return (
    <div className={`tl ${className}`} data-active={active || undefined} data-side={left ? "left" : "right"}>
      {lightsOrder(left).map((k) => (
        <button
          key={k}
          type="button"
          className="tl-dot"
          data-k={k}
          aria-label={LABELS[k]}
          disabled={enabled[k] === false}
          onClick={(e) => {
            e.stopPropagation();
            onAction(k);
          }}
        />
      ))}
    </div>
  );
}
