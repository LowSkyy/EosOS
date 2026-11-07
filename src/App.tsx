import ActiveAppName from "./components/ActiveApp";
import Clock from "./components/Clock";

export default function App() {
  return (
    <header className="menubar">
      <ActiveAppName />
      <Clock />
    </header>
  );
}
