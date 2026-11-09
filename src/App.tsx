import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { barTint, meanLuminance, type Frame } from "./lib/backdrop";
import { invoke } from "@tauri-apps/api/core";
import AppMenus from "./components/AppMenus";
import Clock from "./components/Clock";
import Indicators from "./components/Indicators";
import { useBackdrop } from "./components/useBackdrop";

/** Contorno (px CSS) di sfondo disegnato oltre la barra: la sfocatura ne ha bisogno ai bordi. */
const BLUR_MARGIN = 12;

export default function App() {
  // Vetro «liquido»: lo schermo dietro la barra, sfocato e saturato come nel Dock, nel Launcher e nei pannelli.
  const bar = useRef<HTMLElement>(null);
  const blurCanvas = useRef<HTMLCanvasElement>(null);
  const lastTint = useRef("");
  const adapt = (f: Frame) => {
    // La tinta si adatta alla luminosità dello sfondo (testo sempre leggibile) e si scrive solo se cambia.
    const root = document.documentElement;
    const theme = root.getAttribute("data-theme");
    const dark = theme === "dark" || (theme !== "light" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    const tint = barTint(dark, meanLuminance(f.data));
    if (tint !== lastTint.current) {
      lastTint.current = tint;
      root.style.setProperty("--bar-blur-bg", tint);
    }
  };
  useBackdrop("bar_backdrop", bar, blurCanvas, BLUR_MARGIN, 0, { needFocus: false, pollMs: 500, onFrame: adapt });
  // Con la sfocatura spenta (impostazione del Dock) non arrivano più fotogrammi: si toglie l'ultimo disegnato e torna
  // l'acrilico di Windows con la tinta di base.
  useEffect(() => {
    let off: (() => void) | undefined;
    let gone = false;
    listen<{ blur?: boolean }>("dock-settings-changed", (e) => {
      if (e.payload?.blur === false) {
        document.documentElement.removeAttribute("data-blur");
        document.documentElement.style.removeProperty("--bar-blur-bg");
        lastTint.current = "";
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
      <AppMenus />
      <div className="menubar-right">
        <Indicators />
        <Clock />
      </div>
    </header>
  );
}
