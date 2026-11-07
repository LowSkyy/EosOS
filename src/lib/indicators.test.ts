import { describe, expect, it } from "vitest";
import {
  batteryFill,
  indicatorItems,
  KNOWN_ICONS,
  openId,
  volumeWaves,
  wifiWaves,
  type IndicatorsState,
} from "./indicators";

const full: IndicatorsState = {
  battery: { percent: 85, charging: false, plugged: false, minutes_left: 135, icon: "battery-high", label: "85%", title: "Batteria 85%" },
  network: { kind: "wifi", online: "internet", name: "Casa", bars: 4, icon: "wifi-3", title: "Wi-Fi: Casa" },
  volume: { percent: 40, muted: false, icon: "volume-2", title: "Volume 40%" },
};

describe("indicatorItems", () => {
  it("lists battery, network and sound in the macOS order, battery with its percentage", () => {
    const items = indicatorItems(full);
    expect(items.map((i) => i.key)).toEqual(["battery", "network", "volume"]);
    expect(items[0].label).toBe("85%");
    expect(items[1].label).toBeNull();
    expect(items[2].label).toBeNull();
  });
  it("omits indicators the machine does not have", () => {
    expect(indicatorItems({ ...full, battery: null }).map((i) => i.key)).toEqual(["network", "volume"]);
    expect(indicatorItems({ battery: null, network: full.network, volume: null }).map((i) => i.key)).toEqual(["network"]);
    expect(indicatorItems({ battery: null, network: null, volume: null })).toEqual([]);
    expect(indicatorItems(null)).toEqual([]);
  });
  it("flags only what needs attention", () => {
    expect(indicatorItems(full).some((i) => i.alert)).toBe(false);
    const low = { ...full, battery: { ...full.battery!, percent: 5, icon: "battery-critical", label: "5%" } };
    expect(indicatorItems(low)[0].alert).toBe(true);
    const lowButCharging = { ...full, battery: { ...full.battery!, percent: 5, icon: "battery-charging" } };
    expect(indicatorItems(lowButCharging)[0].alert).toBe(false);
    const offline = { ...full, network: { ...full.network!, kind: "none" as const, online: "none" as const, icon: "net-off" } };
    expect(indicatorItems(offline)[1].alert).toBe(true);
    const limited = { ...full, network: { ...full.network!, online: "limited" as const, icon: "wifi-alert" } };
    expect(indicatorItems(limited)[1].alert).toBe(true);
    const muted = { ...full, volume: { ...full.volume!, muted: true, icon: "volume-muted" } };
    expect(indicatorItems(muted)[2].alert).toBe(false);
  });
  it("passes the title through for the tooltip", () => {
    expect(indicatorItems(full).map((i) => i.title)).toEqual(["Batteria 85%", "Wi-Fi: Casa", "Volume 40%"]);
  });
});

describe("openId", () => {
  it("maps each indicator to the fixed id Rust expects", () => {
    expect(openId("battery")).toBe("battery");
    expect(openId("network")).toBe("network");
    expect(openId("volume")).toBe("sound");
  });
});

describe("batteryFill", () => {
  it("clamps and survives garbage", () => {
    expect(batteryFill(0)).toBe(0);
    expect(batteryFill(50)).toBe(0.5);
    expect(batteryFill(100)).toBe(1);
    expect(batteryFill(180)).toBe(1);
    expect(batteryFill(-4)).toBe(0);
    expect(batteryFill(Number.NaN)).toBe(0);
    expect(batteryFill(Number.POSITIVE_INFINITY)).toBe(0);
  });
});

describe("icon helpers", () => {
  it("counts wifi waves", () => {
    expect(wifiWaves("wifi-1")).toBe(1);
    expect(wifiWaves("wifi-2")).toBe(2);
    expect(wifiWaves("wifi-3")).toBe(3);
    expect(wifiWaves("wifi-alert")).toBe(0);
    expect(wifiWaves("ethernet")).toBeNull();
    expect(wifiWaves("net-off")).toBeNull();
  });
  it("counts speaker waves", () => {
    expect(volumeWaves("volume-0")).toBe(0);
    expect(volumeWaves("volume-1")).toBe(1);
    expect(volumeWaves("volume-3")).toBe(3);
    expect(volumeWaves("volume-muted")).toBe(0);
  });
  it("knows every icon Rust can send, without duplicates", () => {
    expect(new Set(KNOWN_ICONS).size).toBe(KNOWN_ICONS.length);
    expect(KNOWN_ICONS).toHaveLength(20);
  });
});
