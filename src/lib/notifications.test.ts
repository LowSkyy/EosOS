import { describe, expect, it } from "vitest";
import { dismissLabel, headingLabel, hiddenLabel, isLatest, withoutApp, withoutNotif, type NotifGroup } from "./notifications";

const n = (id: number, title = `t${id}`) => ({ id, title, body: "", time_ms: 0, ago: "adesso" });
const g = (app: string, ids: number[], hidden = 0): NotifGroup => ({ app_id: app, app_name: app.toUpperCase(), items: ids.map((i) => n(i)), hidden });

describe("labels", () => {
  it("describes the dismiss button with title and app", () => {
    expect(dismissLabel(g("chat", [1]), n(1, "Marco"))).toBe("Rimuovi la notifica «Marco» di CHAT");
    expect(dismissLabel(g("chat", [1]), { ...n(1, ""), body: "ciao" })).toBe("Rimuovi la notifica «ciao» di CHAT");
  });
  it("writes the overflow and the total", () => {
    expect(hiddenLabel(g("a", [1], 0))).toBe("");
    expect(hiddenLabel(g("a", [1], 1))).toBe("+1 altra");
    expect(hiddenLabel(g("a", [1], 5))).toBe("+5 altre");
    expect(headingLabel(0)).toBe("Notifiche");
    expect(headingLabel(7)).toBe("Notifiche (7)");
  });
});

describe("local removal", () => {
  const groups = [g("a", [1, 2]), g("b", [3])];
  it("removes one notification and drops a group that becomes empty", () => {
    expect(withoutNotif(groups, 2).map((x) => x.items.map((i) => i.id))).toEqual([[1], [3]]);
    expect(withoutNotif(groups, 3).map((x) => x.app_id)).toEqual(["a"]);
    expect(withoutNotif(groups, 99)).toEqual(groups);
  });
  it("keeps a group whose shown items are gone but still has hidden ones", () => {
    expect(withoutNotif([g("a", [1], 4)], 1)).toHaveLength(1);
  });
  it("removes a whole app ignoring case", () => {
    expect(withoutApp(groups, "A").map((x) => x.app_id)).toEqual(["b"]);
    expect(withoutApp(groups, "zz")).toEqual(groups);
  });
});

describe("isLatest", () => {
  it("accepts only the last request", () => {
    expect(isLatest(5, 5)).toBe(true);
    expect(isLatest(4, 5)).toBe(false);
  });
});
