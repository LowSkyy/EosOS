import { invoke } from "@tauri-apps/api/core";
import AppMenus from "./components/AppMenus";
import Clock from "./components/Clock";
import Indicators from "./components/Indicators";

export default function App() {
  return (
    <header
      className="menubar"
      onContextMenu={(e) => {
        // Clic destro sulla barra: quali indicatori mostrare e se la batteria mostra la percentuale.
        e.preventDefault();
        void invoke("indicators_menu");
      }}
    >
      <AppMenus />
      <div className="menubar-right">
        <Indicators />
        <Clock />
      </div>
    </header>
  );
}
