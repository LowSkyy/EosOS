import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  batteryDetails,
  batteryFill,
  INPUT_HOLD_MS,
  makeLatestSender,
  networkDetails,
  panelTitle,
  reconcileVolume,
  settingsLabel,
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
  volume: { percent: 40, muted: false, device: "Altoparlanti", icon: "volume-2", title: "Volume 40%" },
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
  it("shows no label when the percentage is turned off, but keeps the tooltip", () => {
    const noPct = { ...full, battery: { ...full.battery!, label: "" } };
    const b = indicatorItems(noPct)[0];
    expect(b.label).toBeNull();
    expect(b.title).toBe("Batteria 85%");
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

describe("reconcileVolume", () => {
  const local = { percent: 70, muted: false };
  const incoming = { percent: 40, muted: false };
  it("keeps the user's value right after a gesture, then follows the system", () => {
    expect(reconcileVolume(local, incoming, 0)).toEqual(local);
    expect(reconcileVolume(local, incoming, INPUT_HOLD_MS - 1)).toEqual(local);
    expect(reconcileVolume(local, incoming, INPUT_HOLD_MS)).toEqual(incoming);
  });
  it("uses whatever exists when one side is missing", () => {
    expect(reconcileVolume(null, incoming, 0)).toEqual(incoming);
    expect(reconcileVolume(local, null, 99999)).toEqual(local);
    expect(reconcileVolume(null, null, 0)).toBeNull();
  });
});

describe("makeLatestSender", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(1_000_000);
  });
  afterEach(() => vi.useRealTimers());

  it("sends the first value immediately", () => {
    const sent: number[] = [];
    const s = makeLatestSender((v: number) => sent.push(v), 50);
    s(1);
    expect(sent).toEqual([1]);
  });
  it("limits the rate while dragging but never loses the last value", () => {
    const sent: number[] = [];
    const s = makeLatestSender((v: number) => sent.push(v), 50);
    s(1);
    for (const v of [2, 3, 4, 5]) {
      vi.advanceTimersByTime(10);
      s(v);
    }
    expect(sent).toEqual([1]); // 40 ms: ancora nella finestra
    vi.advanceTimersByTime(20);
    expect(sent).toEqual([1, 5]); // arriva l'ultimo, non i valori intermedi
    vi.advanceTimersByTime(500);
    expect(sent).toEqual([1, 5]);
  });
  it("sends again right away once the interval has passed", () => {
    const sent: number[] = [];
    const s = makeLatestSender((v: number) => sent.push(v), 50);
    s(1);
    vi.advanceTimersByTime(80);
    s(2);
    expect(sent).toEqual([1, 2]);
  });
});

describe("panel details", () => {
  it("describes the network", () => {
    const wifi = { kind: "wifi", online: "internet", name: "Casa", bars: 4, icon: "wifi-3", title: "" } as const;
    expect(networkDetails(wifi)).toEqual([
      { label: "Rete", value: "Casa" },
      { label: "Tipo", value: "Wi-Fi" },
      { label: "Stato", value: "Connesso a Internet" },
      { label: "Segnale", value: "4 su 5" },
    ]);
    const cable = { kind: "ethernet", online: "limited", name: null, bars: null, icon: "ethernet-alert", title: "" } as const;
    expect(networkDetails(cable).map((r) => r.value)).toEqual(["Cavo di rete", "Cavo di rete", "Connesso, senza Internet"]);
    const none = { kind: "none", online: "none", name: null, bars: null, icon: "net-off", title: "" } as const;
    expect(networkDetails(none)).toEqual([
      { label: "Tipo", value: "Nessuna" },
      { label: "Stato", value: "Non connesso" },
    ]);
    expect(networkDetails({ ...wifi, name: "  " })[0].value).toBe("Wi-Fi");
  });
  it("describes the battery", () => {
    const base = { percent: 85, charging: false, plugged: false, minutes_left: 135, icon: "battery-high", label: "85%", title: "" };
    expect(batteryDetails(base)).toEqual([
      { label: "Stato", value: "A batteria" },
      { label: "Tempo rimanente", value: "2 h 15 min" },
    ]);
    expect(batteryDetails({ ...base, minutes_left: 45 })[1].value).toBe("45 min");
    expect(batteryDetails({ ...base, minutes_left: 120 })[1].value).toBe("2 h");
    expect(batteryDetails({ ...base, minutes_left: null })).toEqual([{ label: "Stato", value: "A batteria" }]);
    expect(batteryDetails({ ...base, charging: true, plugged: true, minutes_left: null })).toEqual([{ label: "Stato", value: "In carica" }]);
    expect(batteryDetails({ ...base, plugged: true, minutes_left: 90 })).toEqual([{ label: "Stato", value: "Collegata alla rete" }]);
  });
  it("titles and settings buttons", () => {
    expect(panelTitle("volume")).toBe("Audio");
    expect(panelTitle("network")).toBe("Rete");
    expect(panelTitle("battery")).toBe("Batteria");
    expect(settingsLabel("volume")).toContain("audio");
  });
});
