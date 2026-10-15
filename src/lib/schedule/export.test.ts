import { describe, expect, it } from "vitest";
import { exportScheduleHtml, type ExportOptions } from "./export";
import type { ScheduleDocModel } from "./parse";

const DOC: ScheduleDocModel = {
  title: "Plan",
  range: "2026-10-05..2026-10-18",
  nonWorking: { weekly: [0, 6], ranges: [] },
  items: [],
  rawItems: [],
  rawNonWorking: [],
};

const BASE: ExportOptions = {
  start: "2026-10-05",
  end: "2026-10-18",
  today: "2026-10-11",
  locale: "en",
};

const cellHeight = (html: string) => /td\.daycell \{[^}]*?height: (\d+)px/.exec(html)?.[1];

describe("exportScheduleHtml cellExtra", () => {
  it("leaves the output untouched when omitted or 0", () => {
    const plain = exportScheduleHtml(DOC, BASE);
    expect(exportScheduleHtml(DOC, { ...BASE, cellExtra: 0 })).toBe(plain);
    expect(cellHeight(plain)).toBe("96");
  });

  it("adds the extra to the day cell's minimum height", () => {
    expect(cellHeight(exportScheduleHtml(DOC, { ...BASE, cellExtra: 64 }))).toBe("160");
  });

  it("clamps an out-of-range extra", () => {
    expect(cellHeight(exportScheduleHtml(DOC, { ...BASE, cellExtra: 9999 }))).toBe("336");
  });

  it("does not affect the timeline", () => {
    const a = exportScheduleHtml(DOC, { ...BASE, mode: "timeline" });
    const b = exportScheduleHtml(DOC, { ...BASE, mode: "timeline", cellExtra: 64 });
    expect(b).toBe(a);
  });
});
