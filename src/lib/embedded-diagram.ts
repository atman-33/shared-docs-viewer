/**
 * What the Diagrams tab hands an editor view (every kind) when it hosts
 * it (T-0680).
 *
 * Standing alone, those views own their project picker, note picker and the
 * create/rename controls. Embedded, the Diagrams tab owns all of that - one
 * list for every kind of diagram - and the view is told which note to edit.
 * The view then keeps only what is specific to editing that kind of note.
 */
export interface EmbeddedDiagram {
  /** Slug of the project the open note belongs to. */
  project: string;
  /** Absolute path of the note to edit; empty when none is open. */
  path: string;
  /** The note's title, for confirmation dialogs. */
  title: string;
  /** The view asks for another note (or none, after a delete or a rename). */
  onPathChange: (path: string) => void;
  /** True while an AI edit holds the file (T-0685): the editor must not write,
   * and should stop answering gestures, keys and toolbar buttons. */
  locked: boolean;
  /** True in a read-only host (the Docs tab, T-1000): the host also sets
   * `locked`, so the editor cannot write — this only tells the views that no
   * agent holds the file, so they hide the "AI edit running" hint. */
  readOnly?: boolean;
  /**
   * The editor hands the host a function that writes its pending (debounced)
   * save now, or `null` when it goes away. The host calls it before it starts
   * an AI edit, so the file the agent reads has what is on screen. The host's
   * callbacks are new on every render: call `registerFlush` through a ref.
   */
  registerFlush: (flush: (() => Promise<void>) | null) => void;
  /** Bumped by the host after an AI edit ends or is undone: reload the note
   * from disk even if the file watcher's event was missed or the mtime did
   * not change. */
  reloadToken: number;
  /** Whether this kind's right-hand panel is shown (T-0686). The host owns it
   * because it is stored in the vault settings, per kind. */
  sidePanelOpen: boolean;
  onSidePanelOpenChange: (open: boolean) => void;
  /** Projects panes only (T-0998): copy the `diagram-edit` prompt for pasting
   * into a terminal / desktop agent session. Absent in the Diagrams tab,
   * whose toolbar owns its own copy button. */
  onCopyPrompt?: () => Promise<unknown> | void;
  /** Projects panes only (T-1005): jump to the Diagrams tab with this note.
   * Absent in the Diagrams tab, where the view already lives there. */
  onOpenInDiagrams?: () => void;
  /** Projects panes only (T-1005): open this note in Obsidian. */
  onOpenInObsidian?: () => void;
}
