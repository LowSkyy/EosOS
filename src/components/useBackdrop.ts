import { useEffect, useRef, type RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseFrame, sourceRect, type Frame } from "../lib/backdrop";

const POLL_MS = 40;

/**
 * Sfondo sfocato dietro un pannello: chiede a Rust (`command`) i fotogrammi dello schermo dietro
 * la finestra e li disegna nel canvas, che il CSS sfoca e ritaglia (vedi `.launcher-blur`).
 * `resetKey` cambia a ogni apertura: si butta il vecchio fotogramma e si toglie `data-blur` finché
 * non ne arriva uno nuovo, così non si vede mai lo sfondo di prima. Con la finestra senza focus
 * non si chiede nulla (nascosta). Il Dock ha una sua copia di questa logica in Dock.tsx.
 */
export function useBackdrop(
  command: string,
  panel: RefObject<HTMLElement | null>,
  canvas: RefObject<HTMLCanvasElement | null>,
  margin: number,
  resetKey: number,
) {
  const frame = useRef<Frame | null>(null);
  const source = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const root = document.documentElement;
    if (!source.current) source.current = document.createElement("canvas");
    frame.current = null;
    root.removeAttribute("data-blur");
    let seq = 0;
    let busy = false;
    let stopped = false;

    const paint = () => {
      const f = frame.current;
      const src = source.current;
      const dst = canvas.current;
      const el = panel.current;
      if (!f || !src || !dst || !el) return;
      const r = el.getBoundingClientRect();
      const k = window.devicePixelRatio / f.factor;
      const w = Math.max(1, Math.round((r.width + 2 * margin) * k));
      const h = Math.max(1, Math.round((r.height + 2 * margin) * k));
      if (dst.width !== w) dst.width = w;
      if (dst.height !== h) dst.height = h;
      const s = sourceRect(r, margin, window.devicePixelRatio, f);
      const ctx = dst.getContext("2d");
      if (!ctx) return;
      ctx.clearRect(0, 0, w, h);
      ctx.drawImage(src, s.sx, s.sy, s.sw, s.sh, 0, 0, w, h);
    };

    const t = setInterval(() => {
      if (busy || !document.hasFocus()) return;
      busy = true;
      invoke<ArrayBuffer>(command, { since: seq })
        .then((buf) => {
          const f = stopped ? null : parseFrame(buf);
          if (!f || !source.current) return;
          seq = f.seq;
          const s = source.current;
          s.width = f.w;
          s.height = f.h;
          s.getContext("2d")?.putImageData(new ImageData(f.data, f.w, f.h), 0, 0);
          frame.current = f;
          root.setAttribute("data-blur", "on");
          paint();
        })
        .catch(() => {})
        .finally(() => (busy = false));
    }, POLL_MS);

    // Il pannello cambia altezza con i risultati: lo sfondo va ridisegnato.
    const ro = new ResizeObserver(paint);
    if (panel.current) ro.observe(panel.current);
    return () => {
      stopped = true;
      clearInterval(t);
      ro.disconnect();
    };
  }, [command, panel, canvas, margin, resetKey]);
}
