// @vitest-environment happy-dom
import { act, useRef } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useScrollCursor } from "./use-scroll-cursor";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

function List({ cursor, paths }: { cursor: string; paths: string[] }) {
  const ref = useRef<HTMLDivElement>(null);
  // A fresh array each render, as the tree's `rows` is.
  useScrollCursor(ref, cursor, [...paths]);
  return (
    <div ref={ref}>
      {paths.map((p) => (
        <div key={p} data-path={p} />
      ))}
    </div>
  );
}

describe("useScrollCursor", () => {
  let host: HTMLDivElement;
  let scrolled: string[];

  beforeEach(() => {
    host = document.createElement("div");
    document.body.append(host);
    scrolled = [];
    vi.spyOn(HTMLElement.prototype, "scrollIntoView").mockImplementation(function (
      this: HTMLElement,
    ) {
      scrolled.push(this.dataset.path ?? "");
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
    host.remove();
  });

  const render = (cursor: string, paths: string[]) => {
    const root = createRoot(host);
    act(() => root.render(<List cursor={cursor} paths={paths} />));
    return (c: string, p: string[]) => act(() => root.render(<List cursor={c} paths={p} />));
  };

  it("scrolls to the cursor when it moves", () => {
    const update = render("a", ["a", "b"]);
    update("b", ["a", "b"]);
    expect(scrolled).toEqual(["a", "b"]);
  });

  it("stays put when only the rows change", () => {
    const update = render("a", ["a"]);
    update("a", ["a", "dir", "dir/x"]);
    update("a", ["a", "dir"]);
    expect(scrolled).toEqual(["a"]);
  });

  it("waits for a cursor row that is not there yet", () => {
    const update = render("dir/x", ["dir"]);
    expect(scrolled).toEqual([]);
    update("dir/x", ["dir", "dir/x"]);
    update("dir/x", ["dir", "dir/x", "dir/y"]);
    expect(scrolled).toEqual(["dir/x"]);
  });

  it("scrolls again when the cursor returns after being cleared", () => {
    const update = render("a", ["a"]);
    update("", ["a"]);
    update("a", ["a"]);
    expect(scrolled).toEqual(["a", "a"]);
  });
});
