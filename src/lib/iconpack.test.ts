import { describe, expect, it } from "vitest";
import { exeStem, packFor } from "./iconpack";

describe("pacchetto di icone macOS", () => {
  it("ricava il nome del file", () => {
    expect(exeStem("C:\\Windows\\explorer.exe")).toBe("explorer");
    expect(exeStem("C:\\x\\Foo.LNK")).toBe("foo");
    expect(exeStem("")).toBe("");
  });
  it("riconosce le app di sistema dall'eseguibile", () => {
    expect(packFor({ name: "x", path: "C:\\Windows\\explorer.exe" })).toBe("finder");
    expect(packFor({ name: "x", path: "C:\\Windows\\System32\\notepad.exe" })).toBe("notes");
    expect(packFor({ name: "x", path: "C:\\Windows\\System32\\cmd.exe" })).toBe("terminal");
  });
  it("riconosce le app dello Store dal nome", () => {
    expect(packFor({ name: "Impostazioni", path: "shell:AppsFolder\\x" })).toBe("settings");
    expect(packFor({ name: "Calcolatrice", path: "shell:AppsFolder\\y" })).toBe("calculator");
  });
  it("lascia le altre app con la loro icona", () => {
    expect(packFor({ name: "Google Chrome", path: "C:\\Chrome\\chrome.exe" })).toBeNull();
  });
});
