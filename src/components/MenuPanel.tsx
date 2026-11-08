import { forwardRef, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { firstSelectable, flatRows, moveSelection, pathKey, selectable, type MbMenu } from "../lib/menubar";

/**
 * Tendina di un menu della barra (dentro la finestra "panel"): voci con scorciatoia a destra, separatori,
 * voci disattivate in grigio, spunte e sottomenu che si aprono sul posto (rientrati). Frecce su/giù, Invio e
 * Spazio scelgono, destra/sinistra aprono e chiudono i sottomenu; Esc lo gestisce la finestra.
 * La pagina manda a Rust solo l'indice del menu e il percorso della voce: cosa eseguire lo decide Rust.
 */
const MenuPanel = forwardRef<HTMLDivElement, { index: number; opens: number }>(function MenuPanel({ index, opens }, ref) {
  const [menu, setMenu] = useState<MbMenu | null>(null);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  const [sel, setSel] = useState(-1);

  // Le voci le dà Rust (se il menu è "pigro" le legge adesso: può servire un attimo).
  useEffect(() => {
    let cancelled = false;
    setMenu(null);
    setLoading(true);
    setOpen(new Set());
    setSel(-1);
    invoke<MbMenu | null>("menubar_menu", { index })
      .then((m) => {
        if (cancelled) return;
        setMenu(m);
        setLoading(false);
      })
      .catch(() => !cancelled && setLoading(false));
    return () => {
      cancelled = true;
    };
  }, [index, opens]);

  const rows = useMemo(() => (menu ? flatRows(menu.items, open) : []), [menu, open]);

  const toggleSub = (key: string, force?: boolean) =>
    setOpen((cur) => {
      const next = new Set(cur);
      const want = force ?? !next.has(key);
      if (want) next.add(key);
      else next.delete(key);
      return next;
    });

  const choose = (i: number) => {
    const r = rows[i];
    if (!r || !selectable(r)) return;
    if (r.item.children.length > 0) return toggleSub(pathKey(r.path));
    void invoke("menubar_choose", { index, path: r.path });
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") setSel((s) => (s < 0 ? firstSelectable(rows) : moveSelection(rows, s, 1)));
      else if (e.key === "ArrowUp") setSel((s) => (s < 0 ? firstSelectable(rows) : moveSelection(rows, s, -1)));
      else if (e.key === "Enter" || e.key === " ") choose(sel);
      else if (e.key === "ArrowRight" && rows[sel]?.item.children.length) toggleSub(pathKey(rows[sel].path), true);
      else if (e.key === "ArrowLeft" && rows[sel]) {
        const r = rows[sel];
        if (open.has(pathKey(r.path))) toggleSub(pathKey(r.path), false);
        else if (r.path.length > 1) {
          // Da una voce di sottomenu: chiude il sottomenu e torna sulla voce che lo apre.
          const parent = r.path.slice(0, -1);
          toggleSub(pathKey(parent), false);
          setSel(rows.findIndex((x) => pathKey(x.path) === pathKey(parent)));
        }
      } else return;
      e.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  return (
    <div className="menu-panel" ref={ref} role="menu" aria-label={menu?.title}>
      {loading && <div className="menu-empty">Lettura del menu…</div>}
      {!loading && rows.length === 0 && <div className="menu-empty">Nessuna voce</div>}
      {rows.map((r, i) =>
        r.item.separator ? (
          <hr key={pathKey(r.path)} className="menu-sep" />
        ) : (
          <button
            key={pathKey(r.path)}
            type="button"
            role="menuitem"
            className={`menu-item${i === sel ? " menu-item-sel" : ""}${r.item.enabled ? "" : " menu-item-off"}`}
            style={{ paddingLeft: 12 + r.depth * 16 }}
            disabled={!r.item.enabled}
            aria-checked={r.item.checked || undefined}
            aria-haspopup={r.item.children.length > 0 || undefined}
            aria-expanded={r.item.children.length > 0 ? open.has(pathKey(r.path)) : undefined}
            onMouseEnter={() => selectable(r) && setSel(i)}
            onClick={() => choose(i)}
          >
            <span className="menu-check">{r.item.checked ? "✓" : ""}</span>
            <span className="menu-title">{r.item.title}</span>
            <span className="menu-short">{r.item.children.length > 0 ? (open.has(pathKey(r.path)) ? "▾" : "▸") : (r.item.shortcut ?? "")}</span>
          </button>
        ),
      )}
    </div>
  );
});

export default MenuPanel;
