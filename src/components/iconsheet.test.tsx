import { it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { writeFileSync } from "node:fs";
import { FILE_ART, type FileId, type Theme } from "./iconart";

/** Non e' una prova: genera la pagina con tutte le icone di cartelle e file (chiare e scure) per `scripts/build_icons.py`. Si salta senza la variabile. */
const out = process.env.EOSOS_ICON_SHEET;
(out ? it : it.skip)("foglio delle icone", () => {
  const ids = Object.keys(FILE_ART) as FileId[];
  const cells: string[] = [];
  for (const t of ["light", "dark"] as Theme[]) {
    for (const id of ids) {
      const svg = renderToStaticMarkup(<svg viewBox="0 0 100 100" width="512" height="512">{FILE_ART[id](t)}</svg>);
      cells.push(`<div class="c" data-n="${id}-${t}">${svg}</div>`);
    }
  }
  writeFileSync(out as string, `<!doctype html><style>html,body{margin:0;background:transparent}.g{display:grid;grid-template-columns:repeat(6,512px)}.c{width:512px;height:512px}</style><div class="g">${cells.join("")}</div>`);
  writeFileSync((out as string) + ".json", JSON.stringify(ids.flatMap((id) => ["light", "dark"].map((t) => `${id}-${t}`)).sort()));
});
