// Interroga la pagina del Dock tramite la porta di debug di WebView2.
// EosOS va avviato con  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222
//
//   node scripts/cdp-dock.mjs eval "<espressione JS>"      stampa il risultato (JSON)
//   node scripts/cdp-dock.mjs shot <file.png> ["<JS prima dello scatto>"]
//                                                            screenshot della pagina (stampa "larghezza interna")
//   node scripts/cdp-dock.mjs menu                          simula il clic destro sullo sfondo della capsula
//
// Serve alle prove automatiche: la finestra del Dock con la sfocatura attiva e' esclusa dalle
// catture dello schermo, ma lo screenshot della pagina ne mostra il contenuto.
import fs from "node:fs";

const port = process.env.CDP_PORT || 9222;
const list = await (await fetch(`http://localhost:${port}/json`)).json();
const view = process.env.CDP_VIEW || "dock";
// "bar" = le pagine della barra in alto (senza ?view=); CDP_INDEX sceglie quale (una per monitor).
const pages = view === "bar"
  ? list.filter((x) => x.type === "page" && !x.url.includes("view=") && !x.url.startsWith("devtools"))
  : list.filter((x) => x.url.includes(`view=${view}`));
const target = pages[Number(process.env.CDP_INDEX || 0)];
if (!target) {
  console.error(`pagina ${view} non trovata`);
  process.exit(2);
}
const ws = new WebSocket(target.webSocketDebuggerUrl);
let id = 0;
const pending = new Map();
const send = (method, params = {}) =>
  new Promise((res) => {
    const i = ++id;
    pending.set(i, res);
    ws.send(JSON.stringify({ id: i, method, params }));
  });
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id && pending.has(d.id)) {
    pending.get(d.id)(d);
    pending.delete(d.id);
  }
};
await new Promise((r) => (ws.onopen = r));
const evaluate = async (expression) =>
  (await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true })).result.result?.value;

const [cmd, a, b] = process.argv.slice(2);
if (cmd === "eval") {
  console.log(JSON.stringify(await evaluate(a)));
} else if (cmd === "shot") {
  if (b) await evaluate(b);
  await new Promise((r) => setTimeout(r, 400));
  const r = await send("Page.captureScreenshot", { format: "png" });
  fs.writeFileSync(a, Buffer.from(r.result.data, "base64"));
  console.log(await evaluate("window.innerWidth"));
} else if (cmd === "menu") {
  // Non si attende la risposta: il comando e' bloccante finche' il menu e' aperto.
  send("Runtime.evaluate", {
    expression:
      "document.querySelector('.dock').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true,clientX:5,clientY:5}))",
  });
  await new Promise((r) => setTimeout(r, 300));
} else {
  console.error("comando sconosciuto: " + cmd);
  process.exit(1);
}
process.exit(0);
