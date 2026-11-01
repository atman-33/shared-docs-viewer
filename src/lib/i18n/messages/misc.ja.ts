import type { MessageKey } from "./en";

/** Japanese strings for miscellaneous dialogs/panels and shared UI (T-0423). */
export const miscJa: Partial<Record<MessageKey, string>> = {

  "misc.diagnosticLog.title": "診断ログ",
  "misc.diagnosticLog.openFolder": "フォルダを開く",
  "misc.diagnosticLog.description":
    "アプリが実行中に記録した内容です。パッケージ版にはコンソールがないため、この画面とその元になるログファイルが、問題を報告する際の唯一の手がかりになります。記録されるのはアプリの動作(エラー、時刻、ウィンドウ配置)のみで、入力・発話・コピーした内容は含まれません。",
  "misc.diagnosticLog.empty": "まだ何も記録されていません。",
  "misc.diagnosticLog.autoRefresh": "自動更新",

  "misc.copyPromptButton.copying": "コピー中…",
  "misc.copyPromptButton.default": "プロンプトをコピー",

  "misc.errorBoundary.title": "{label} の表示中にエラーが発生しました。",
  "misc.errorBoundary.description":
    "アプリの他の部分はそのまま動作しています。この内容は診断ログ(設定 → 診断)に書き込まれているため、再起動後も確認できます。",
  "misc.errorBoundary.tryAgain": "再試行",
  "misc.errorBoundary.copyDetails": "詳細をコピー",

  "misc.markdown.copyCode": "コードをコピー",
  "misc.markdown.openFigure": "ウィンドウで開く",
  "misc.markdown.openFigureHint": "ウィンドウで開く（ダブルクリックでも可）",

};
