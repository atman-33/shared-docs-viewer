import { afterEach, describe, expect, it, vi } from "vitest";
import {
  CELL_EXTRA_MAX,
  clampCellExtra,
  readCellExtra,
  writeCellExtra,
} from "./cell-height";

describe("clampCellExtra", () => {
  it("clamps to the range and snaps to the step", () => {
    expect(clampCellExtra(-5)).toBe(0);
    expect(clampCellExtra(999)).toBe(CELL_EXTRA_MAX);
    expect(clampCellExtra(61)).toBe(64);
    expect(clampCellExtra("40")).toBe(40);
  });

  it("falls back to the default for anything non-numeric", () => {
    expect(clampCellExtra(undefined)).toBe(0);
    expect(clampCellExtra("abc")).toBe(0);
    expect(clampCellExtra(NaN)).toBe(0);
  });
});

/** Minimal in-memory localStorage — the default vitest env is node, which has none. */
function stubStorage() {
  const store = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => void store.set(k, v),
  });
  return store;
}

describe("stored cell extra", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("reads 0 when nothing is stored", () => {
    stubStorage();
    expect(readCellExtra()).toBe(0);
  });

  it("round-trips a value and sanitises a corrupt one", () => {
    const store = stubStorage();
    writeCellExtra(96);
    expect(readCellExtra()).toBe(96);
    store.set("schedule.cellExtra", "garbage");
    expect(readCellExtra()).toBe(0);
  });

  it("reads 0 when storage throws", () => {
    vi.stubGlobal("localStorage", {
      getItem: () => {
        throw new Error("denied");
      },
      setItem: () => {
        throw new Error("denied");
      },
    });
    expect(readCellExtra()).toBe(0);
    expect(() => writeCellExtra(64)).not.toThrow();
  });
});
