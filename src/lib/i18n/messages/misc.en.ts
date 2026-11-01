/** English strings for miscellaneous dialogs/panels and shared UI (T-0423). */
export const miscEn = {

  "misc.diagnosticLog.title": "Diagnostic log",
  "misc.diagnosticLog.openFolder": "Open folder",
  "misc.diagnosticLog.description":
    "What the app recorded while it ran. The packaged build has no console window, so this — and the log file behind it — is where a problem you want to report leaves a trace. It holds what the app did (errors, timings, window placement), never what you typed, dictated, or copied.",
  "misc.diagnosticLog.empty": "Nothing recorded yet.",
  "misc.diagnosticLog.autoRefresh": "Auto-refresh",

  "misc.copyPromptButton.copying": "Copying…",
  "misc.copyPromptButton.default": "Copy prompt",

  "misc.errorBoundary.title": "The {label} view stopped with an error.",
  "misc.errorBoundary.description":
    "The rest of the app is still running. This was written to the diagnostic log (Settings → Diagnostics), so it is still readable after a restart.",
  "misc.errorBoundary.tryAgain": "Try again",
  "misc.errorBoundary.copyDetails": "Copy details",

  "misc.markdown.copyCode": "Copy code",
  "misc.markdown.openFigure": "Open in a window",
  "misc.markdown.openFigureHint": "Open in a window (or double-click)",

} as const;
