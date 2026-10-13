import { describe, expect, it } from "vitest";
import { focusPathToApply } from "./tab-focus";

const files = [{ path: "/v/a/040.md" }, { path: "/v/a/050.md" }];

describe("focusPathToApply", () => {
  it("waits for the listing, then selects the requested note", () => {
    const base = { n: 1, path: "/v/a/040.md", handledN: 0, files };
    expect(focusPathToApply({ ...base, filesLoaded: false })).toBeNull();
    expect(focusPathToApply({ ...base, files: [], filesLoaded: true })).toBeNull();
    expect(focusPathToApply({ ...base, filesLoaded: true })).toBe("/v/a/040.md");
  });

  it("does not re-apply a handled request when the listing reloads (collapse write-back)", () => {
    // Backlog opened 040 (n=1); the owner then picked 050 and collapsed a node.
    // The write-back reloads `files`; the old request must not pull 040 back.
    expect(
      focusPathToApply({ n: 1, path: "/v/a/040.md", handledN: 1, filesLoaded: true, files }),
    ).toBeNull();
  });

  it("applies a newer request for the same note", () => {
    expect(
      focusPathToApply({ n: 2, path: "/v/a/040.md", handledN: 1, filesLoaded: true, files }),
    ).toBe("/v/a/040.md");
  });

  it("ignores project-only requests", () => {
    expect(focusPathToApply({ n: 1, path: "", handledN: 0, filesLoaded: true, files })).toBeNull();
    expect(focusPathToApply({ n: 0, path: "/v/a/040.md", handledN: 0, filesLoaded: true, files })).toBeNull();
  });
});
