import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import type React from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { nextPoll, POLL_MAX_MS } from "../lib/backdrop";
import { badgeAria, badgeMap, sameBadges, type Badge } from "../lib/badge";
import {
  DRAG_THRESHOLD,
  applyMove,
  clampGlass,
  dockVars,
  dotState,
  dropTarget,
  knownIconIds,
  magnifyScale,
  previewOffsets,
  pruneBouncing,
  sameItems,
  splitDock,
  updateIconCache,
  withCachedIcons,
  type DockItem,
  type IconCache,
  type DockSettings,
} from "../lib/dock";
import { parseFrame, sourceRect, type Frame } from "../lib/backdrop";
import { MacTile } from "./MacIcon";
import { packFor } from "../lib/iconpack";
import { sameStacks, type StackItem } from "../lib/stacks";

/** Contorno (px CSS) di sfondo disegnato oltre la capsula: la sfocatura ne ha bisogno ai bordi. */
const BLUR_MARGIN = 28;
/** Ogni quanto si chiede un nuovo fotogramma dello sfondo. */
const BACKDROP_MS = 40;

/** Ogni quanto si rilegge l'elenco delle app aperte. */
const POLL_MS = 1000;
/** Ogni quanto si rilegge il conteggio delle notifiche per i badge. */
const BADGE_MS = 4000;
/** Un rimbalzo non dura più di così, anche se l'app non compare mai. */
const BOUNCE_MAX_MS = 8000;
/** Ogni quanto si scartano le icone già ricevute per ricaricarle. */
const ICON_REFRESH_MS = 60 * 60 * 1000;

function Icon({
  item,
  badge,
  bouncing,
  offset,
  dragging,
  mac,
  onClick,
  onPointerDown,
}: {
  item: DockItem;
  /** Il testo del badge delle notifiche (assente = nessuna notifica). */
  badge?: string;
  bouncing: boolean;
  /** Spostamento orizzontale durante un trascinamento (px). */
  offset: number;
  dragging: boolean;
  /** Icone nello stile di macOS. */
  mac: boolean;
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
      data-label={item.name}
      role="button"
      aria-label={badgeAria(item.name, badge)}
      data-bounce={bouncing}
      data-dragging={dragging}
      style={{ "--dx": `${offset}px` } as React.CSSProperties}
      onClick={() => onClick(item)}
      onPointerDown={(e) => onPointerDown(e, item)}
      onContextMenu={menu}
    >
      {mac ? (
        <MacTile pack={packFor(item)} src={item.icon} alt={item.name} />
      ) : item.icon ? (
        <img className="dock-icon" src={item.icon} alt={item.name} draggable={false} />
      ) : (
        <div className="dock-icon dock-icon-fallback" aria-label={item.name}>
          {item.name.charAt(0).toUpperCase()}
        </div>
      )}
      {badge && (
        <span className="dock-badge" aria-hidden>
          {badge}
        </span>
      )}
      <span className="dock-dot" data-state={dotState(item)} />
    </div>
  );
}

/** Una pila: la cartella nel Dock. Un clic apre il pannello con i file più recenti sopra l'icona. */
function StackIcon({ stack }: { stack: StackItem }) {
  const open = (e: React.MouseEvent<HTMLDivElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    void invoke("panel_toggle", { key: `stack:${stack.index}`, left: r.left, top: r.top, width: r.width, height: r.height });
  };
  return (
    <div className="dock-item dock-stack" data-label={stack.title} role="button" aria-label={`Pila ${stack.title}`} aria-haspopup="dialog" onClick={open}>
      {stack.icon ? (
        <img className="dock-icon" src={stack.icon} alt="" draggable={false} />
      ) : (
        <div className="dock-icon dock-icon-fallback" aria-hidden>
          {stack.title.charAt(0).toUpperCase()}
        </div>
      )}
      <span className="dock-dot" />
    </div>
  );
}

const LAUNCH_COLORS = ["#ff453a", "#ff9f0a", "#ffd60a", "#30d158", "#64d2ff", "#0a84ff", "#bf5af2", "#ff375f", "#e5e5ea"];

/** L'icona del Launchpad (tutte le app) in testa al Dock, come in macOS. */
function LaunchpadIcon({ mac }: { mac: boolean }) {
  return (
    <div className="dock-item dock-launchpad" data-label="Launchpad" role="button" aria-label="Launchpad" onClick={() => void invoke("dock_launchpad")}>
      <div className="dock-icon dock-launchpad-icon" data-mac={mac} aria-hidden>
        <svg viewBox="0 0 24 24" width="62%" height="62%">
          {[0, 1, 2].flatMap((r) => [0, 1, 2].map((c) => <rect key={`${r}${c}`} x={3 + c * 7} y={3 + r * 7} width="4.6" height="4.6" rx="1.2" fill={mac ? LAUNCH_COLORS[r * 3 + c] : "currentColor"} />))}
        </svg>
      </div>
      <span className="dock-dot" />
    </div>
  );
}

/** Stesso aspetto della capsula a riposo, invisibile: se ne misura la dimensione. */
function Ghost({ items, stacks, ref }: { items: DockItem[]; stacks: StackItem[]; ref: React.Ref<HTMLDivElement> }) {
  const { pinned, others } = splitDock(items);
  const cell = (i: DockItem) => (
    <div className="dock-item" key={i.id}>
      <div className="dock-icon" />
      <span className="dock-dot" />
    </div>
  );
  return (
    <div className="dock dock-ghost" aria-hidden ref={ref}>
      {cell({ id: "launchpad-cell" } as DockItem)}
      {pinned.map(cell)}
      {pinned.length > 0 && others.length > 0 && <div className="dock-separator" />}
      {others.map(cell)}
      {stacks.length > 0 && (pinned.length > 0 || others.length > 0) && <div className="dock-separator dock-separator-stacks" />}
      {stacks.map((s) => (
        <div className="dock-item" key={`stack${s.index}`}>
          <div className="dock-icon" />
          <span className="dock-dot" />
        </div>
      ))}
    </div>
  );
}

const cssPx = (name: string) =>
  parseFloat(getComputedStyle(document.documentElement).getPropertyValue(name)) || 0;

export default function Dock() {
  const [items, setItems] = useState<DockItem[]>([]);
  // L'ingrandimento al passaggio del mouse è acceso (Impostazioni → Dock).
  const magnifyOn = useRef(true);
  // Badge delle notifiche per icona (id → testo). Rust risponde vuoto se la lettura delle notifiche è spenta.
  const [badges, setBadges] = useState<Record<string, string>>({});
  // Quante app girano in background (area di notifica): l'icona compare solo se almeno una.
  const [stacks, setStacks] = useState<StackItem[]>([]);
  const [bouncing, setBouncing] = useState<Set<string>>(new Set());
  const zone = useRef<HTMLDivElement>(null);
  const pill = useRef<HTMLDivElement>(null);
  const ghost = useRef<HTMLDivElement>(null);
  const alive = useRef(true);
  const blurCanvas = useRef<HTMLCanvasElement>(null);
  const source = useRef<HTMLCanvasElement | null>(null);
  const frame = useRef<Frame | null>(null);
  const [blur, setBlur] = useState(false);
  const [mac, setMac] = useState(true);

  // Generazione dell'elenco: sale a ogni modifica fatta qui (riordino). Un sondaggio partito
  // prima non deve poter sovrascrivere con dati vecchi il risultato appena mostrato.
  const epoch = useRef(0);
  // Le icone sono grandi (PNG in base64): Rust le manda una volta sola, qui si tengono.
  const icons = useRef<IconCache>(new Map());
  const load = useCallback(
    () => {
      const started = epoch.current;
      return invoke<DockItem[]>("dock_items", { have: knownIconIds(icons.current) })
        .then((received) => {
          if (!alive.current || started !== epoch.current) return;
          updateIconCache(icons.current, received);
          const next = withCachedIcons(received, icons.current);
          setItems((cur) => (sameItems(cur, next) ? cur : next));
          setBouncing((cur) => {
            const pruned = pruneBouncing(cur, next);
            return pruned.size === cur.size ? cur : pruned;
          });
        })
        .catch(() => {});
    },
    [],
  );

  // Pile: all'avvio, a ogni cambio di impostazioni e ogni minuto (Rust tiene le icone in cache).
  const loadStacks = useCallback(() => {
    invoke<StackItem[]>("dock_stacks")
      .then((next) => alive.current && setStacks((cur) => (sameStacks(cur, next) ? cur : next)))
      .catch(() => {});
  }, []);
  useEffect(() => {
    loadStacks();
    const t = setInterval(loadStacks, 60_000);
    return () => clearInterval(t);
  }, [loadStacks]);

  // Vetro: la trasparenza scelta dal menu arriva da Rust, all'avvio e a ogni cambio.
  useEffect(() => {
    const apply = (st: Partial<DockSettings> | null) => {
      if (!st) return;
      document.documentElement.style.setProperty("--glass", String(clampGlass(st.glass)));
      for (const [k, v] of Object.entries(dockVars(st.size ?? 1))) document.documentElement.style.setProperty(k, v);
      setBlur(st.blur === true);
      setMac(st.icons !== false);
      document.documentElement.setAttribute("data-dock-anim", st.anim ?? "bounce");
      magnifyOn.current = st.magnify !== false;
      if (!magnifyOn.current) rest();
    };
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen<DockSettings>("dock-settings-changed", (e) => {
      apply(e.payload);
      loadStacks();
      // Un'app fissata o tolta dal menu (o dalle Impostazioni) deve comparire o sparire SUBITO: senza questo l'elenco
      // si rinfrescava solo al giro successivo (fino a 2 secondi dopo) e il Dock sembrava bloccato.
      epoch.current += 1;
      void load();
      setTimeout(() => void load(), 600);
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    invoke<DockSettings>("dock_settings").then(apply).catch(() => {});
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [loadStacks, load]);

  // Sfondo sfocato: disegna nel canvas della capsula la parte di schermo che sta
  // dietro di lei. Il CSS lo sfoca e lo ritaglia con la forma della capsula.
  const paint = useCallback(() => {
    const f = frame.current;
    const src = source.current;
    const dst = blurCanvas.current;
    const el = pill.current;
    if (!f || !src || !dst || !el) return;
    const r = el.getBoundingClientRect();
    // La tela ha sempre la misura della capsula a riposo (la copia invisibile): con l'ingrandimento la capsula cambia misura a ogni
    // movimento del mouse e ricreare la tela ogni volta (e svuotarla) pesava. Lo sfondo e' sfocato: stirarlo di poco non si nota.
    const rest = ghost.current?.getBoundingClientRect() ?? r;
    const k = window.devicePixelRatio / f.factor;
    const w = Math.max(1, Math.round((rest.width + 2 * BLUR_MARGIN) * k));
    const h = Math.max(1, Math.round((rest.height + 2 * BLUR_MARGIN) * k));
    if (dst.width !== w) dst.width = w;
    if (dst.height !== h) dst.height = h;
    const s = sourceRect(r, BLUR_MARGIN, window.devicePixelRatio, f);
    const ctx = dst.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, w, h);
    ctx.drawImage(src, s.sx, s.sy, s.sw, s.sh, 0, 0, w, h);
  }, []);

  useEffect(() => {
    const root = document.documentElement;
    if (!blur) {
      root.removeAttribute("data-blur");
      frame.current = null;
      return;
    }
    if (!source.current) source.current = document.createElement("canvas");
    let seq = 0;
    let stopped = false;
    let delay = BACKDROP_MS;
    let timer: number | undefined;
    // Un giro alla volta: con lo sfondo fermo Rust risponde «niente di nuovo» e si rallenta (vedi `nextPoll`).
    const tick = () => {
      invoke<ArrayBuffer>("dock_backdrop", { since: seq })
        .then((buf) => {
          const f = stopped ? null : parseFrame(buf);
          delay = nextPoll(delay, !!f, BACKDROP_MS, POLL_MAX_MS);
          if (!f || !source.current) return;
          seq = f.seq;
          const s = source.current;
          // Solo se cambia misura: assegnare width/height ricrea la tela a ogni fotogramma.
          if (s.width !== f.w) s.width = f.w;
          if (s.height !== f.h) s.height = f.h;
          s.getContext("2d")?.putImageData(new ImageData(f.data, f.w, f.h), 0, 0);
          frame.current = f;
          root.setAttribute("data-blur", "on");
          paint();
        })
        .catch(() => {
          delay = nextPoll(delay, false, BACKDROP_MS, POLL_MAX_MS);
        })
        .finally(() => {
          if (!stopped) timer = window.setTimeout(tick, delay);
        });
    };
    timer = window.setTimeout(tick, delay);
    return () => {
      stopped = true;
      window.clearTimeout(timer);
    };
  }, [blur, paint]);

  // La capsula cambia misura con l'ingrandimento: lo sfondo va ridisegnato.
  useEffect(() => {
    const el = pill.current;
    if (!el) return;
    const ro = new ResizeObserver(() => paint());
    ro.observe(el);
    return () => ro.disconnect();
  }, [paint]);

  // Ogni 10 minuti si scartano le icone ricevute: se un'app ha cambiato icona si ricarica.
  useEffect(() => {
    const t = setInterval(() => icons.current.clear(), ICON_REFRESH_MS);
    return () => clearInterval(t);
  }, []);

  // Badge delle notifiche: la pagina manda le sue icone e riceve solo i conteggi; si rilegge ogni pochi secondi.
  const itemsKey = items.map((i) => `${i.id}|${i.name}|${i.path}`).join("\n");
  useEffect(() => {
    if (items.length === 0) return;
    let gone = false;
    const ask = () =>
      invoke<Badge[]>("dock_badges", { items: items.map((i) => ({ id: i.id, name: i.name, path: i.path })) })
        .then((list) => {
          if (gone) return;
          const next = badgeMap(list);
          setBadges((cur) => (sameBadges(cur, next) ? cur : next));
        })
        .catch(() => undefined);
    void ask();
    const t = setInterval(ask, BADGE_MS);
    return () => {
      gone = true;
      clearInterval(t);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [itemsKey]);

  // Legge l'elenco subito e poi a intervalli; aggiorna lo stato solo se cambia.
  useEffect(() => {
    alive.current = true;
    void load();
    const t = setInterval(load, POLL_MS);
    // Quando cambia l'app in primo piano (se ne apre una, se ne chiude una, si passa a un'altra) si rilegge subito, senza
    // aspettare il giro periodico: il Dock segue le app aperte senza ritardo.
    let pending: number | undefined;
    let unlisten: (() => void) | undefined;
    let gone = false;
    listen("foreground-changed", () => {
      window.clearTimeout(pending);
      pending = window.setTimeout(() => void load(), 120);
    }).then((fn) => (gone ? fn() : (unlisten = fn)));
    return () => {
      alive.current = false;
      clearInterval(t);
      gone = true;
      window.clearTimeout(pending);
      unlisten?.();
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
    // Nessun tasto premuto: il rilascio è avvenuto fuori dalla zona cliccabile (che si
    // restringe alla capsula quando il cursore esce) e non è mai arrivato. Senza questo il
    // movimento successivo avvierebbe un trascinamento fantasma.
    if ((e.buttons & 1) === 0) {
      pending.current = null;
      setDrag(null);
      return;
    }
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
        epoch.current += 1; // i sondaggi già partiti non valgono più
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
    if (!magnifyOn.current) return; // ingrandimento spento dalle Impostazioni
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const cells = pill.current?.querySelectorAll<HTMLElement>(".dock-item");
    if (!cells || cells.length === 0) return;
    const pitch = cssPx("--dock-icon") + cssPx("--dock-gap");
    // Prima tutte le letture del layout, poi tutte le scritture: alternarle costringerebbe il
    // browser a ricalcolare il layout una volta per icona a ogni movimento del mouse.
    const rects = Array.from(cells, (c) => c.getBoundingClientRect());
    cells.forEach((c, i) => {
      const r = rects[i];
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
      badge={badges[i.id]}
      bouncing={bouncing.has(i.id)}
      offset={drag?.id === i.id ? drag.dx : (offsets[i.id] ?? 0)}
      dragging={drag?.id === i.id}
      mac={mac}
      onClick={onIconClick}
      onPointerDown={pointerDown}
    />
  );
  return (
    <div
      className="dock-zone"
      ref={zone}
      onMouseMove={magnify}
      onMouseLeave={() => {
        rest();
        // Una pressione che non è ancora un trascinamento non deve restare in sospeso.
        if (pending.current && !pending.current.active) pending.current = null;
      }}
      onPointerMove={pointerMove}
      onPointerUp={(e) => pointerEnd(e, true)}
      onPointerCancel={(e) => pointerEnd(e, false)}
      onContextMenu={background}
    >
      <div className="dock" ref={pill}>
        <canvas className="dock-blur" ref={blurCanvas} aria-hidden />
        <LaunchpadIcon mac={mac} />
        {pinned.map(icon)}
        {pinned.length > 0 && others.length > 0 && <div className="dock-separator" />}
        {others.map(icon)}
        {stacks.length > 0 && (pinned.length > 0 || others.length > 0) && <div className="dock-separator dock-separator-stacks" />}
        {stacks.map((s) => (
          <StackIcon key={`stack${s.index}`} stack={s} />
        ))}
      </div>
      <Ghost items={items} stacks={stacks} ref={ghost} />
    </div>
  );
}
