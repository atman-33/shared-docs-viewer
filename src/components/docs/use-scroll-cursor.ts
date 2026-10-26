import { type RefObject, useEffect, useRef } from "react";

/**
 * Keeps the keyboard cursor's row on screen — once per move (T-0946).
 *
 * The effect used to depend on the rows too, so every re-render of the list
 * (a folder opened or closed, a listing arriving, a filter keystroke) scrolled
 * back to the cursor. Clicking a folder does not move the cursor, so that
 * pulled the view away from the folder just clicked to the open document.
 *
 * `rows` is therefore only a trigger for retrying: the cursor can be set
 * before its row exists (a shortcut opens the tree down to a file whose
 * listing is still loading). The scroll is recorded once the row is found, and
 * a cursor already scrolled to is left alone.
 */
export function useScrollCursor(
  scroller: RefObject<HTMLElement | null>,
  cursor: string,
  rows: unknown,
) {
  const scrolledTo = useRef("");

  useEffect(() => {
    if (!cursor) {
      scrolledTo.current = "";
      return;
    }
    if (scrolledTo.current === cursor) return;
    const el = scroller.current?.querySelector<HTMLElement>(`[data-path="${CSS.escape(cursor)}"]`);
    if (!el) return;
    scrolledTo.current = cursor;
    el.scrollIntoView({ block: "nearest" });
  }, [scroller, cursor, rows]);
}
