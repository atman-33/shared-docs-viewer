/**
 * Adjustable day-cell height of the calendar (T-0822).
 *
 * The setting is an *extra* height added to each surface's own minimum — the
 * screen grid, the HTML export and the image copy do not share an absolute
 * minimum (80px + 22px lanes on screen, 96px + 18px lanes in the export), so an
 * absolute value would change the default of one of them. An extra of 0 keeps
 * every surface exactly as it was, and any other value makes all three taller
 * by the same amount.
 *
 * It is machine-local UI state, like the rest of `view-state.ts`: the note's
 * format is untouched, so Obsidian and the `diagram-edit` skill never see it.
 */

/** Largest extra height. Keeps an exported week row (96 + 240) well inside one
 * A4-landscape page, so `break-inside: avoid` still has a page to avoid onto. */
export const CELL_EXTRA_MAX = 240;
export const CELL_EXTRA_STEP = 8;
export const CELL_EXTRA_DEFAULT = 0;

/** The S / M / L presets of the toolbar. */
export const CELL_EXTRA_PRESETS = [
  { id: "S", extra: 0 },
  { id: "M", extra: 64 },
  { id: "L", extra: 160 },
] as const;

const STORAGE_KEY = "schedule.cellExtra";

/** Snaps to the step and clamps to the range; anything non-numeric is the default. */
export function clampCellExtra(value: unknown): number {
  const n = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(n)) return CELL_EXTRA_DEFAULT;
  const snapped = Math.round(n / CELL_EXTRA_STEP) * CELL_EXTRA_STEP;
  return Math.min(CELL_EXTRA_MAX, Math.max(0, snapped));
}

export function readCellExtra(): number {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw === null ? CELL_EXTRA_DEFAULT : clampCellExtra(raw);
  } catch {
    // storage unavailable (private mode / quota) — remembering is a convenience
    return CELL_EXTRA_DEFAULT;
  }
}

export function writeCellExtra(extra: number): void {
  try {
    localStorage.setItem(STORAGE_KEY, String(clampCellExtra(extra)));
  } catch {
    // storage unavailable — the setting just lasts for this session
  }
}
