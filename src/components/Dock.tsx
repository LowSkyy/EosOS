import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import type React from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  DRAG_THRESHOLD,
  applyMove,
  clampGlass,
  dotState,
  dropTarget,
  magnifyScale,
  previewOffsets,
  pruneBouncing,
  sameItems,
  splitDock,
  type DockItem,
  type DockSettings,
} from "../lib/dock";

/** Ogni quanto si rilegge l'elenco delle app aperte. */
const POLL_MS = 2000;
/** Un rimbalzo non dura più di così, anche se l'app non compare mai. */
const BOUNCE_MAX_MS = 8000;

function Icon({
  item,
  bouncing,
  offset,
  dragging,
  onClick,
  onPointerDown,
}: {
  item: DockItem;
  bouncing: boolean;
  /** Spostamento orizzontale durante un trascinamento (px). */
  offset: number;
  dragging: boolean;
  onClick: (item: DockItem) => void;
  onPointerDown: (e: React.PointerEvent, item: DockItem) => void;
}) {
  // Il frontend manda solo l'id: il percorso da eseguire lo ricava Rust dall'elenco del Dock.
  const menu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation(); // il menu dello sfondo è un'altra cosa
    void invoke("dock_menu", { id: item.id });
  };
  return (
    <div
      className="dock-item"
      title={item.name}
      role="button"
      data-bounce={bouncing}
      data-dragging={dragging}
      style={{ "--dx": `${offset}px` } as React.CSSProperties}
      onClick={() => onClick(item)}
      onPointerDown={(e) => onPointerDown(e, item)}
      onContextMenu={menu}
    >
      {item.icon ? (
        <img className="dock-icon" src={item.icon} alt={item.name} draggable={false} />
      ) : (
        <div className="dock-icon dock-icon-fallback" aria-label={item.name}>
          {item.name.charAt(0).toUpperCase()}
        </div>
      )}
      <span className="dock-dot" data-state={dotState(item)} />
    </div>
  );
}

/** Stesso aspetto della capsula a riposo, invisibile: se ne misura la dimensione. */
function Ghost({ items, ref }: { items: DockItem[]; ref: React.Ref<HTMLDivElement> }) {
  const { pinned, others } = splitDock(items);
  const cell = (i: DockItem) => (
    <div className="dock-item" key={i.id}>
      <div className="dock-icon" />
      <span className="dock-dot" />
    </div>
  );
  return (
    <div className="dock dock-ghost" aria-hidden ref={ref}>
      {pinned.map(cell)}
      {pinned.length > 0 && others.length > 0 && <div className="dock-separator" />}
      {others.map(cell)}
    </div>
  );
}

const cssPx = (name: string) =>
  parseFloat(getComputedStyle(document.documentElement).getPropertyValue(name)) || 0;

export default function Dock() {
  const [items, setItems] = useState<DockItem[]>([]);
  const [bouncing, setBouncing] = useState<Set<string>>(new Set());
  const zone = useRef<HTMLDivElement>(null);
  const pill = useRef<HTMLDivElement>(null);
  const ghost = useRef<HTMLDivElement>(null);
  const alive = useRef(true);

  const load = useCallback(
    () =>
      invoke<DockItem[]>("dock_items")
        .then((next) => {
          if (!alive.current) return;
          setItems((cur) => (sameItems(cur, next) ? cur : next));
          setBouncing((cur) => {
            const pruned = pruneBouncing(cur, next);
            return pruned.size === cur.size ? cur : pruned;
          });
        })
        .catch(() => {}),
    [],
  );

  // Vetro: la trasparenza scelta dal menu arriva da Rust, all'avvio e a ogni cambio.
  useEffect(() => {
    const apply = (st: Partial<DockSettings> | null) => {
      if (st) document.documentElement.style.setProperty("--glass", String(clampGlass(st.glass)));
    };
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen<DockSettings>("dock-settings-changed", (e) => apply(e.payload)).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    invoke<DockSettings>("dock_settings").then(apply).catch(() => {});
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

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

  const refreshSoon = useCallback(() => {
    void load();
    setTimeout(load, 700);
  }, [load]);

  // Clic: avvia / porta davanti / minimizza. Un'app chiusa rimbalza finché non compare.
  const click = useCallback(
    (item: DockItem) => {
      if (!item.running) {
        setBouncing((cur) => new Set(cur).add(item.id));
        setTimeout(
          () =>
            setBouncing((cur) => {
              if (!cur.has(item.id)) return cur;
              const next = new Set(cur);
              next.delete(item.id);
              return next;
            }),
          BOUNCE_MAX_MS,
        );
      }
      void invoke("dock_activate", { id: item.id }).finally(refreshSoon);
    },
    [refreshSoon],
  );

  // La finestra nativa è fissa: più grande della capsula a riposo, con margine
  // sopra e ai lati per le icone ingrandite. Il CSS decide tutte le misure.
  useLayoutEffect(() => {
    const el = ghost.current;
    if (!el) return;
    const fit = () => {
      const r = el.getBoundingClientRect();
      const side = cssPx("--dock-side");
      const head = cssPx("--dock-head");
      const radius = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 0;
      void invoke("dock_fit", {
        fit: {
          zoneW: r.width + 2 * side,
          zoneH: r.height + head,
          capX: side,
          capY: head,
          capW: r.width,
          capH: r.height,
          radius,
        },
      }).catch(() => {});
    };
    fit();
    const ro = new ResizeObserver(fit);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // Trascinamento per riordinare. Si usano gli eventi del puntatore (non il
  // drag-and-drop HTML, inaffidabile in una finestra trasparente e ritagliata).
  // Le posizioni si misurano sulla copia invisibile a riposo: le icone vere
  // possono essere ingrandite e spostate.
  type Pending = {
    id: string;
    pointerId: number;
    startX: number;
    active: boolean;
    /** Centri a riposo delle altre fissate, in ordine. */
    centers: number[];
    /** Bordo sinistro del separatore: oltre questo punto non si fissa. */
    separatorX: number;
    wasPinned: boolean;
  };
  const pending = useRef<Pending | null>(null);
  const suppressClick = useRef(false);
  const [drag, setDrag] = useState<{ id: string; dx: number; target: number | null } | null>(null);

  const pointerDown = useCallback(
    (e: React.PointerEvent, item: DockItem) => {
      if (e.button !== 0 || !ghost.current) return;
      const cells = [...ghost.current.querySelectorAll<HTMLElement>(".dock-item")];
      const pinnedList = items.filter((i) => i.pinned);
      const centers = pinnedList
        .map((p, k) => ({ id: p.id, r: cells[k]?.getBoundingClientRect() }))
        .filter((c) => c.id !== item.id && c.r)
        .map((c) => (c.r as DOMRect).left + (c.r as DOMRect).width / 2);
      const sep = ghost.current.querySelector<HTMLElement>(".dock-separator");
      pending.current = {
        id: item.id,
        pointerId: e.pointerId,
        startX: e.clientX,
        active: false,
        centers,
        separatorX: sep ? sep.getBoundingClientRect().left : Infinity,
        wasPinned: item.pinned,
      };
    },
    [items],
  );

  const targetFor = (p: Pending, x: number): number | null => {
    // Una app non fissata si può fissare solo rilasciandola tra le fissate.
    if (!p.wasPinned && x >= p.separatorX) return null;
    return dropTarget(p.centers, x);
  };

  const pointerMove = (e: React.PointerEvent) => {
    const p = pending.current;
    if (!p) return;
    const dx = e.clientX - p.startX;
    if (!p.active) {
      if (Math.abs(dx) < DRAG_THRESHOLD) return;
      p.active = true;
      zone.current?.setPointerCapture(p.pointerId);
      rest(); // niente ingrandimento mentre si trascina
    }
    setDrag({ id: p.id, dx, target: targetFor(p, e.clientX) });
  };

  const pointerEnd = (e: React.PointerEvent, commit: boolean) => {
    const p = pending.current;
    pending.current = null;
    if (!p) return;
    if (p.active) {
      suppressClick.current = true; // il clic che segue il rilascio non è un avvio
      setTimeout(() => (suppressClick.current = false), 0);
      zone.current?.releasePointerCapture(p.pointerId);
      const target = commit ? targetFor(p, e.clientX) : null;
      if (target !== null) {
        setItems((cur) => applyMove(cur, p.id, target)); // subito, senza aspettare Rust
        void invoke("dock_reorder", { id: p.id, index: target }).finally(refreshSoon);
      }
    }
    setDrag(null);
  };

  // Ingrandimento "a onda": la scala di ogni icona dipende dalla distanza del
  // cursore dal suo centro attuale. Si scrive direttamente nel DOM (una variabile
  // CSS per icona): niente rendering React a ogni movimento del mouse.
  const magnify = useCallback((e: React.MouseEvent) => {
    if (pending.current) return; // si sta trascinando
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const cells = pill.current?.querySelectorAll<HTMLElement>(".dock-item");
    if (!cells || cells.length === 0) return;
    const pitch = cssPx("--dock-icon") + cssPx("--dock-gap");
    cells.forEach((c) => {
      const r = c.getBoundingClientRect();
      c.style.setProperty("--s", String(magnifyScale(e.clientX - (r.left + r.width / 2), pitch)));
    });
  }, []);

  const rest = useCallback(() => {
    pill.current?.querySelectorAll<HTMLElement>(".dock-item").forEach((c) => {
      c.style.removeProperty("--s");
    });
  }, []);

  // Clic destro sullo sfondo della capsula: impostazioni. (Sulle icone c'è il loro menu.)
  const background = (e: React.MouseEvent) => {
    e.preventDefault();
    const onCapsule = e.target === pill.current || (e.target as HTMLElement).closest(".dock-separator");
    if (onCapsule) void invoke("dock_settings_menu");
  };

  const { pinned, others } = splitDock(items);
  const onIconClick = (item: DockItem) => {
    if (suppressClick.current) return;
    click(item);
  };
  const pitch = cssPx("--dock-icon") + cssPx("--dock-gap");
  const offsets =
    drag && drag.target !== null
      ? previewOffsets(
          pinned.map((p) => p.id),
          drag.id,
          drag.target,
          pitch,
        )
      : {};
  const icon = (i: DockItem) => (
    <Icon
      key={i.id}
      item={i}
      bouncing={bouncing.has(i.id)}
      offset={drag?.id === i.id ? drag.dx : (offsets[i.id] ?? 0)}
      dragging={drag?.id === i.id}
      onClick={onIconClick}
      onPointerDown={pointerDown}
    />
  );
  return (
    <div
      className="dock-zone"
      ref={zone}
      onMouseMove={magnify}
      onMouseLeave={rest}
      onPointerMove={pointerMove}
      onPointerUp={(e) => pointerEnd(e, true)}
      onPointerCancel={(e) => pointerEnd(e, false)}
      onContextMenu={background}
    >
      <div className="dock" ref={pill}>
        {pinned.map(icon)}
        {pinned.length > 0 && others.length > 0 && <div className="dock-separator" />}
        {others.map(icon)}
      </div>
      <Ghost items={items} ref={ghost} />
    </div>
  );
}
