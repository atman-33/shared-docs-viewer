import { describe, expect, it } from "vitest";
import { wrapTitle } from "./text";

describe("wrapTitle", () => {
  it("breaks at an explicit newline and wraps each line on its own", () => {
    expect(wrapTitle("one\ntwo", 400, 14)).toEqual(["one", "two"]);
    expect(wrapTitle("a b\nccccccccccccccccccccccccccccccccccccccc", 80, 14).length).toBeGreaterThan(2);
  });

  it("leaves a title without a newline as it was", () => {
    expect(wrapTitle("one two", 400, 14)).toEqual(["one two"]);
  });
});
