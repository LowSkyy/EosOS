import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import "./styles.css";
import App from "./App";
import Dock from "./components/Dock";
import { initGlass } from "./lib/glass";
import { initTheme } from "./styles/theme";

initTheme();
void initGlass();

// La stessa pagina serve la barra (predefinita) e il Dock (?view=dock).
const isDock = new URLSearchParams(window.location.search).get("view") === "dock";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isDock ? <Dock /> : <App />}
  </React.StrictMode>,
);
