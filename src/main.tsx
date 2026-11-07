import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import "./styles.css";
import App from "./App";
import Dock from "./components/Dock";
import Launcher from "./components/Launcher";
import Panel from "./components/Panel";
import { initGlass } from "./lib/glass";
import { initTheme } from "./styles/theme";

initTheme();
void initGlass();

// La stessa pagina serve la barra (predefinita) e il Dock (?view=dock), il Launcher (?view=launcher) e il pannello degli indicatori (?view=panel).
const view = new URLSearchParams(window.location.search).get("view");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {view === "dock" ? <Dock /> : view === "launcher" ? <Launcher /> : view === "panel" ? <Panel /> : <App />}
  </React.StrictMode>,
);
