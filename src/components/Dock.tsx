import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import type React from "react";
import { invoke } from "@tauri-apps/api/core";
import { sameItems, splitDock, type DockItem } from "../lib/dock";

/** Ogni quanto si rilegge l'elenco delle app aperte (D3 lo sostituirà con gli eventi). */
const POLL_MS = 2000;

function Icon({ item, onAction }: { item: DockItem; onAction: () => void }) {
  // Il frontend manda solo l'id: il percorso da eseguire lo ricava Rust dall'elenco del Dock.
  const click = () => {
    void invoke("dock_activate", { id: item.id }).finally(onAction);
  };
  const menu = (e: React.MouseEvent) => {
    e.preventDefault();
    void invoke("dock_menu", { id: item.id }).finally(onAction);
  };
  return (
    <div className="dock-item" title={item.name} role="button" onClick={click} onContextMenu={menu}>
      {item.icon ? (
        <img className="dock-icon" src={item.icon} alt={item.name} draggable={false} />
      ) : (
        <div className="dock-icon dock-icon-fallback" aria-label={item.name}>
          {item.name.charAt(0).toUpperCase()}
        </div>
      )}
      <span className="dock-dot" data-running={item.running} />
    </div>
  );
}

export default function Dock() {
  const [items, setItems] = useState<DockItem[]>([]);
  const pill = useRef<HTMLDivElement>(null);

  const alive = useRef(true);
  const load = useCallback(
    () =>
      invoke<DockItem[]>("dock_items")
        .then((next) => {
          if (alive.current) setItems((cur) => (sameItems(cur, next) ? cur : next));
        })
        .catch(() => {}),
    [],
  );

  // Legge l'elenco subito e poi a intervalli; aggiorna lo stato solo se cambia.
  useEffect(() => {
    alive.current = true;
    void load();
    const t = setInterval(load, POLL_MS);
    return () => {
      alive.current = false;
      clearInterval(t);
    };
  }, [load]);

  // Dopo un'azione l'app impiega un attimo ad aprirsi, chiudersi o essere fissata.
  const refreshSoon = useCallback(() => {
    void load();
    setTimeout(load, 700);
  }, [load]);

  // Il CSS è l'unica fonte delle misure: la finestra nativa si adatta alla pillola.
  useLayoutEffect(() => {
    const el = pill.current;
    if (!el) return;
    const fit = () => {
      const r = el.getBoundingClientRect();
      const radius = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 0;
      void invoke("dock_fit", { width: r.width, height: r.height, radius }).catch(() => {});
    };
    fit();
    const ro = new ResizeObserver(fit);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const { pinned, others } = splitDock(items);
  return (
    <div className="dock" ref={pill}>
      {pinned.map((i) => (
        <Icon key={i.id} item={i} onAction={refreshSoon} />
      ))}
      {pinned.length > 0 && others.length > 0 && <div className="dock-separator" />}
      {others.map((i) => (
        <Icon key={i.id} item={i} onAction={refreshSoon} />
      ))}
    </div>
  );
}
