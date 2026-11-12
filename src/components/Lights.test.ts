import { describe, expect, it } from "vitest";
import { lightsOrder } from "./Lights";

describe("semafori di EosOS", () => {
  it("hanno lo stesso ordine dei cerchi sulle finestre delle app", () => {
    expect(lightsOrder(true)).toEqual(["close", "min", "zoom"]);
    expect(lightsOrder(false)).toEqual(["min", "zoom", "close"]);
  });
});
