/** Cattura dello schermo: la logica pura della finestra di selezione. */

export type Point = { x: number; y: number };
export type Box = { x: number; y: number; w: number; h: number };

/** Il rettangolo fra il punto di partenza e quello attuale, anche trascinando all'indietro (larghezza e altezza mai negative). Pura, testata. */
export function selection(start: Point, cur: Point): Box {
  return { x: Math.min(start.x, cur.x), y: Math.min(start.y, cur.y), w: Math.abs(cur.x - start.x), h: Math.abs(cur.y - start.y) };
}

/** Il lato minimo (in pixel fisici) sotto il quale è un semplice clic e non una selezione: lo stesso di Rust. */
export const MIN_SIDE = 4;

/** È una vera selezione (e non un clic)? Si conta in pixel fisici, come fa Rust. Pura, testata. */
export function isSelection(b: Box, dpr: number): boolean {
  return b.w * dpr >= MIN_SIDE && b.h * dpr >= MIN_SIDE;
}

/** La misura in pixel fisici da mostrare accanto al rettangolo, come «640 × 480». Pura, testata. */
export function sizeLabel(b: Box, dpr: number): string {
  return `${Math.round(b.w * dpr)} × ${Math.round(b.h * dpr)}`;
}

/** Il rettangolo tenuto dentro la finestra (il cursore può uscire mentre si trascina). Pura, testata. */
export function clampPoint(p: Point, width: number, height: number): Point {
  return { x: Math.min(Math.max(p.x, 0), Math.max(width, 0)), y: Math.min(Math.max(p.y, 0), Math.max(height, 0)) };
}
