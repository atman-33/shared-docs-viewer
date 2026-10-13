/**
 * A request from one tab to another: "select this, and do it now".
 *
 * The Projects tab is the only source today — its Tasks / Schedule / Mindmap /
 * Repo buttons hand the receiving view the project slug (or repository path)
 * to focus on. The counter is what makes the request repeatable: the same
 * value asked for twice is still two requests, and a receiving view that has
 * since been navigated elsewhere has to honour the second one.
 */
export interface TabFocus {
  /** Project slug, or repository path for the Repos tab. */
  value: string;
  /** Incremented per request; the receiving effect keys on it. */
  n: number;
  /**
   * One note inside the tab to select — a diagram note's absolute path for
   * the Diagrams tab (T-0715). Absent, the tab keeps its project-level
   * behaviour (select the project, keep or clear the open note).
   */
  path?: string;
}

/**
 * The note a pending focus request should select now, or `null`.
 *
 * A request is applied once: `handledN` is the counter of the last request
 * applied, so a later listing reload (every write-back of an open note fires
 * `diagrams-changed`) does not drag the selection back to the requested note
 * after the owner picked another one. The request waits until the listing
 * holds the note, since it loads after the project is set.
 */
export function focusPathToApply(args: {
  n: number;
  path: string;
  handledN: number;
  filesLoaded: boolean;
  files: readonly { path: string }[];
}): string | null {
  const { n, path, handledN, filesLoaded, files } = args;
  if (n <= 0 || !path || n === handledN || !filesLoaded) return null;
  return files.some((f) => f.path === path) ? path : null;
}
