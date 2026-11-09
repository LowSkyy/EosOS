import { describe, expect, it } from "vitest";
import { commandName, displayLine, sortStartup, timeoutLabel, TIMEOUTS } from "./winsys";

describe("Windows system settings", () => {
  it("names the timeouts", () => {
    expect(timeoutLabel(0)).toBe("Mai");
    expect(timeoutLabel(1)).toBe("1 minuto");
    expect(timeoutLabel(15)).toBe("15 minuti");
    expect(timeoutLabel(60)).toBe("1 ora");
    expect(timeoutLabel(120)).toBe("2 ore");
    expect(TIMEOUTS[0]).toBe(0);
    expect([...TIMEOUTS]).toEqual([...TIMEOUTS].sort((a, b) => a - b));
  });
  it("describes a display", () => {
    expect(displayLine({ name: "A", primary: true, width: 1920, height: 1080, scale: 1.25, x: 0, y: 0 })).toBe("1920 × 1080 · 125% · principale");
    expect(displayLine({ name: "B", primary: false, width: 2560, height: 1440, scale: 1, x: -2560, y: 0 })).toBe("2560 × 1440 · 100%");
  });
  it("shows only the file name of a startup command", () => {
    expect(commandName('"C:\\Program Files\\LGHUB\\system_tray\\lghub_system_tray.exe" --minimized')).toBe("lghub_system_tray.exe");
    expect(commandName("C:\\a b\\Medal.exe --hidden")).toBe("Medal.exe");
    expect(commandName("C:\\x\\Docker Desktop.exe")).toBe("Docker Desktop.exe");
    expect(commandName("  ")).toBe("  ");
  });
  it("lists enabled startup apps first, then by name", () => {
    const a = (name: string, enabled: boolean) => ({ name, command: "", enabled });
    expect(sortStartup([a("Zeta", true), a("beta", false), a("Alfa", true), a("Gamma", false)]).map((x) => x.name)).toEqual(["Alfa", "Zeta", "beta", "Gamma"]);
  });
});
