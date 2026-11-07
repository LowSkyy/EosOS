import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("./tokens.css", import.meta.url), "utf8");

type RGB = [number, number, number];
type RGBA = [number, number, number, number];

/** Variabili `--nome: valore;` di un blocco. */
function vars(body: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const m of body.matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) out[m[1]] = m[2].trim();
  return out;
}

/** Corpo del primo blocco `{...}` che segue il selettore indicato. */
function block(selectorStart: string): string {
  const i = css.indexOf(selectorStart);
  if (i < 0) throw new Error(`selettore non trovato: ${selectorStart}`);
  const open = css.indexOf("{", i);
  let depth = 0;
  for (let j = open; j < css.length; j++) {
    if (css[j] === "{") depth++;
    if (css[j] === "}" && --depth === 0) return css.slice(open + 1, j);
  }
  throw new Error("blocco non chiuso");
}

function parseColor(v: string): RGBA {
  const hex = v.match(/^#([0-9a-f]{6})$/i);
  if (hex) {
    const n = parseInt(hex[1], 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255, 1];
  }
  const m = v.match(/^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*([\d.]+))?\s*\)$/);
  if (!m) throw new Error(`colore non supportato: ${v}`);
  return [+m[1], +m[2], +m[3], m[4] === undefined ? 1 : +m[4]];
}

function over(fg: RGBA, bg: RGB): RGB {
  const a = fg[3];
  return [0, 1, 2].map((i) => fg[i] * a + bg[i] * (1 - a)) as RGB;
}

function luminance([r, g, b]: RGB): number {
  const f = (c: number) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}

function contrast(a: RGB, b: RGB): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const rgb = (c: RGBA): RGB => [c[0], c[1], c[2]];

const light = vars(block(':root[data-theme="light"]'));
const darkMedia = vars(block(':root:not([data-theme="light"])'));
const darkForced = vars(block(':root[data-theme="dark"]'));

const WHITE: RGB = [255, 255, 255];
const BLACK: RGB = [0, 0, 0];

describe("token: coerenza", () => {
  it("il tema scuro automatico e quello forzato hanno gli stessi valori", () => {
    const keys = Object.keys(darkForced);
    expect(keys.length).toBeGreaterThan(0);
    for (const k of keys) expect(darkMedia[k], k).toBe(darkForced[k]);
  });

  it("chiaro e scuro definiscono le stesse variabili", () => {
    expect(Object.keys(light).sort()).toEqual(Object.keys(darkForced).sort());
  });
});

describe.each([
  ["chiaro", light],
  ["scuro", darkForced],
])("contrasto WCAG AA, tema %s", (_name, t) => {
  const bg = parseColor(t["--bar-bg"]);
  const solid = parseColor(t["--bar-bg-solid"]);

  it.each(["--bar-fg", "--bar-fg-muted"])("%s leggibile sul vetro nel caso peggiore", (token) => {
    const fg = parseColor(t[token]);
    // Caso peggiore: lo sfondo dietro il vetro è tutto bianco oppure tutto nero.
    for (const backdrop of [WHITE, BLACK]) {
      expect(contrast(rgb(fg), over(bg, backdrop))).toBeGreaterThanOrEqual(4.5);
    }
  });

  it.each(["--bar-fg", "--bar-fg-muted"])("%s leggibile sul fondo opaco di ripiego", (token) => {
    const fg = parseColor(t[token]);
    expect(contrast(rgb(fg), rgb(solid))).toBeGreaterThanOrEqual(4.5);
  });
});
