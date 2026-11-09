import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { barColors, meanLuminance, type Frame } from "./lib/backdrop";
import { invoke } from "@tauri-apps/api/core";
import AppMenus from "./components/AppMenus";
import Clock from "./components/Clock";
import EosButton from "./components/EosButton";
import ControlButton from "./components/ControlButton";
import Indicators from "./components/Indicators";
import { useBackdrop } from "./components/useBackdrop";

/** Contorno (px CSS) di sfondo disegnato oltre la barra: la sfocatura ne ha bisogno ai bordi. */
const BLUR_MARGIN = 12;

export default function App() {
  // Vetro «liquido»: lo schermo dietro la barra, sfocato e saturato come nel Dock, nel Launcher e nei pannelli.
  const bar = useRef<HTMLElement>(null);
  const blurCanvas = useRef<HTMLCanvasElement>(null);
  const lastKey = useRef("");
  const lastLum = useRef<number | null>(null);
  const apply = (lum: number) => {
    // Tinta e colore del testo si adattano a ciò che c'è dietro la barra (testo sempre leggibile, qualunque tema) e si
    // scrivono solo se cambiano.
    const root = document.documentElement;
    const theme = root.getAttribute("data-theme");
    const dark = theme === "dark" || (theme !== "light" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    const c = barColors(dark, lum);
    const key = `${c.tint}|${c.fg}`;
    if (key === lastKey.current) return;
    lastKey.current = key;
    root.style.setProperty("--bar-blur-bg", c.tint);
    root.style.setProperty("--bar-fg", c.fg);
    root.style.setProperty("--bar-fg-muted", c.fgMuted);
  };
  const adapt = (f: Frame) => {
    lastLum.current = meanLuminance(f.data);
    apply(lastLum.current);
  };
  // Se il tema del sistema cambia a barra aperta si ricalcola con l'ultimo sfondo noto.
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => lastLum.current !== null && apply(lastLum.current);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);
  useBackdrop("bar_backdrop", bar, blurCanvas, BLUR_MARGIN, 0, { needFocus: false, pollMs: 500, onFrame: adapt });
  // Con la sfocatura spenta (impostazione del Dock) non arrivano più fotogrammi: si toglie l'ultimo disegnato e torna
  // l'acrilico di Windows con la tinta di base.
  useEffect(() => {
    let off: (() => void) | undefined;
    let gone = false;
    listen<{ blur?: boolean }>("dock-settings-changed", (e) => {
      if (e.payload?.blur === false) {
        document.documentElement.removeAttribute("data-blur");
        for (const v of ["--bar-blur-bg", "--bar-fg", "--bar-fg-muted"]) document.documentElement.style.removeProperty(v);
        lastKey.current = "";
        lastLum.current = null;
      }
    }).then((fn) => (gone ? fn() : (off = fn)));
    return () => {
      gone = true;
      off?.();
    };
  }, []);
  return (
    <header
      ref={bar}
      className="menubar"
      onContextMenu={(e) => {
        // Clic destro sulla barra: quali indicatori mostrare e se la batteria mostra la percentuale.
        e.preventDefault();
        void invoke("indicators_menu");
      }}
    >
      <canvas className="menubar-blur" ref={blurCanvas} aria-hidden />
      <div className="menubar-left">
        <EosButton />
        <AppMenus />
      </div>
      <div className="menubar-right">
        <Indicators />
        <ControlButton />
        <Clock />
      </div>
    </header>
  );
}
