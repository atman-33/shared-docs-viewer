import { useEffect, useRef } from "react";

/** Inline rename. Enter commits, Escape abandons, blur commits — the same
 * bargain the task board's inline fields make.
 *
 * The value lives in the canvas so the box can size itself to the text; this
 * component only owns focus.
 *
 * With `multiline`, the field is a textarea and Shift+Enter inserts a line
 * break (Enter still commits), so the break is visible while typing.
 */
export function NodeInput({
  value,
  onChange,
  onCommit,
  onCancel,
  multiline = false,
}: {
  value: string;
  onChange: (value: string) => void;
  onCommit: (value: string) => void;
  onCancel: () => void;
  multiline?: boolean;
}) {
  const ref = useRef<HTMLInputElement & HTMLTextAreaElement>(null);
  useEffect(() => {
    ref.current?.focus();
    ref.current?.select();
  }, []);
  const props = {
    ref,
    value,
    onChange: (e: React.ChangeEvent<HTMLInputElement & HTMLTextAreaElement>) => onChange(e.target.value),
    onBlur: () => onCommit(value),
    onKeyDown: (e: React.KeyboardEvent) => {
      e.stopPropagation();
      if (e.key === "Enter" && !(multiline && e.shiftKey)) {
        e.preventDefault();
        onCommit(value);
      } else if (e.key === "Escape") {
        e.preventDefault();
        onCancel();
      }
    },
    // Its own background and text colour rather than the node's: the root is
    // drawn in the foreground colour, so an inherited-colour field on it was
    // white text on white.
    className:
      "size-full rounded border border-ring bg-background px-2 text-center text-sm text-foreground outline-none",
  };
  if (multiline) {
    return <textarea {...props} className={`${props.className} resize-none py-1 leading-[1.45]`} />;
  }
  return <input {...props} />;
}
