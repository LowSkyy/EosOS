import { invoke } from "@tauri-apps/api/core";
import { indicatorItems, type IndicatorKey } from "../lib/indicators";
import { Glyph } from "./IndicatorGlyph";
import { useIndicators } from "./useIndicators";

/**
 * Indicatori di sistema nella barra: batteria (con la percentuale sempre visibile), rete e audio.
 * Un clic apre (o chiude) il pannello sotto l'indicatore: la pagina dice a Rust quale indicatore e
 * dov'è sullo schermo (rettangolo in px CSS), e Rust apre la finestra del pannello.
 */
export default function Indicators() {
  const state = useIndicators();
  const items = indicatorItems(state);
  if (items.length === 0) return null;

  const toggle = (key: IndicatorKey, el: HTMLElement) => {
    const r = el.getBoundingClientRect();
    void invoke("panel_toggle", { key, left: r.left, top: r.top, width: r.width, height: r.height });
  };

  return (
    <div className="indicators" role="group" aria-label="Indicatori di sistema">
      {items.map((it) => (
        <button
          key={it.key}
          type="button"
          className="indicator"
          data-key={it.key}
          data-alert={it.alert}
          title={it.title}
          aria-label={it.title}
          aria-haspopup="dialog"
          onClick={(e) => toggle(it.key, e.currentTarget)}
        >
          <Glyph item={it} percent={state?.battery?.percent ?? 0} />
          {it.label && <span className="indicator-label">{it.label}</span>}
        </button>
      ))}
    </div>
  );
}
