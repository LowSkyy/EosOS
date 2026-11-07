import ActiveAppName from "./components/ActiveApp";
import Clock from "./components/Clock";
import Indicators from "./components/Indicators";

export default function App() {
  return (
    <header className="menubar">
      <ActiveAppName />
      <div className="menubar-right">
        <Indicators />
        <Clock />
      </div>
    </header>
  );
}
