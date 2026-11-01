import type { MessageKey } from "./en";
import { projectsJa } from "./projects.ja";
import { miscJa } from "./misc.ja";
import { scheduleJa } from "./schedule.ja";
import { mindmapJa } from "./mindmap.ja";
import { diagramJa } from "./diagram.ja";
import { docsJa } from "./docs.ja";

/**
 * Japanese UI strings (T-0409). Typed against the English keys, so a key that
 * does not exist in `en.ts` is a type error; a key left out here is shown in
 * English, which is what lets the UI be translated one area at a time.
 */
export const ja: Partial<Record<MessageKey, string>> = {
  "app.windowTitle": "shared-docs-viewer",

  "common.cancel": "キャンセル",
  "common.save": "保存",
  "common.delete": "削除",
  "common.close": "閉じる",

  "common.add": "追加",
  "common.remove": "削除",
  "common.copy": "コピー",
  "common.copied": "コピーしました",

  "common.retry": "再試行",
  "common.refresh": "更新",

  "common.open": "開く",
  "common.today": "今日",
  "common.create": "作成",

  "nav.docs": "ドキュメント",

  "nav.zoomAria": "画面ズーム {percent}%、ズーム操作を開く",
  "nav.zoomOut": "縮小",
  "nav.zoomIn": "拡大",
  "nav.zoomSliderAria": "画面ズーム",
  "nav.zoomReset": "リセット",
  "nav.settingsAria": "設定",

  "banner.update.available": "新しいバージョン {tag} が利用できます（現在 v{version}）",
  "banner.update.action": "更新して再起動",
  "banner.update.downloading": "{tag} をダウンロード中…",
  "banner.update.installed": "✓ 更新をインストールしました",
  "banner.update.restartNow": "今すぐ再起動",
  "banner.update.failed": "更新に失敗しました: {error}",

  "task.status.todo": "未着手",
  "task.status.doing": "進行中",
  "task.status.review": "レビュー",
  "task.status.done": "完了",
  "task.status.inbox": "受信箱",

  "settings.title": "設定",
  "settings.description": "\u30a2\u30d7\u30ea\u306e\u52d5\u4f5c\u3068\u3001\u30a8\u30fc\u30b8\u30a7\u30f3\u30c8\u304c\u8fd4\u7b54\u3059\u308b\u8a00\u8a9e\u306e\u8a2d\u5b9a\u3067\u3059\u3002\u5358\u4e00\u306e\u30bf\u30d6\u304c\u6240\u6709\u3059\u308b\u8a2d\u5b9a\u306f\u305d\u306e\u30bf\u30d6\u306b\u3042\u308a\u307e\u3059\u3002",

  "settings.tab.general": "一般",
  "settings.tab.agents": "エージェント",

  "settings.general.theme.title": "テーマ",
  "settings.general.theme.light": "ライト",
  "settings.general.theme.dark": "ダーク",
  "settings.general.theme.system": "システムに合わせる",
  "settings.general.theme.description":
    "アプリの配色です。このマシンですべてのウィンドウに即時反映されます。vault には保存されません。",
  "settings.general.language.title": "表示言語",
  "settings.general.language.description":
    "アプリの画面が表示される言語です。スケジュールカレンダーとその HTML エクスポートを含みます。未翻訳の画面は英語のままです。表示のみに影響し、ノートに書き込まれる内容は翻訳されません。",
  "settings.general.startup.title": "起動",
  "settings.general.startup.autostart": "Windows \u30b5\u30a4\u30f3\u30a4\u30f3\u6642\u306b shared-docs-viewer \u3092\u8d77\u52d5\u3059\u308b",
  "settings.general.startup.autostartDescription": "\u6700\u5c0f\u5316\u3055\u308c\u305f\u72b6\u614b\u3067\u8d77\u52d5\u3057\u307e\u3059\u3002\u4f7f\u3046\u3068\u304d\u306f\u30bf\u30b9\u30af\u30d0\u30fc\u304b\u3089\u958b\u3044\u3066\u304f\u3060\u3055\u3044\u3002",
  "settings.general.startupChecks.title": "起動時のチェック",
  "settings.general.startupChecks.checkUpdates": "アプリの更新を確認する",

  "settings.general.appUpdate.title": "アプリの更新",
  "settings.general.appUpdate.currentVersion": "現在のバージョン: v{version}",
  "settings.general.appUpdate.checking": "確認中…",
  "settings.general.appUpdate.check": "更新を確認",
  "settings.general.appUpdate.installed": "更新をインストールしました",
  "settings.general.appUpdate.newVersion": "新しいバージョン {tag} が利用できます",
  "settings.general.appUpdate.downloading": "ダウンロード中…",
  "settings.general.appUpdate.downloadInstall": "ダウンロードしてインストール",
  "settings.general.appUpdate.upToDate": "最新の状態です。",
  "settings.general.appUpdate.failed": "更新に失敗しました: {error}",

  "settings.agents.language.label": "言語",
  "settings.agents.language.description": "AI \u30a8\u30fc\u30b8\u30a7\u30f3\u30c8\u304c\u8fd4\u7b54\u3059\u308b\u8a00\u8a9e\u3002",

  "settings.footer.saveFailed": "保存に失敗しました: {error}",
  "settings.footer.resetToDefaults": "既定値にリセット",
  "settings.footer.saving": "保存中…",

  // Areas kept in their own files (T-0423).
  ...projectsJa,
  ...miscJa,
  ...scheduleJa,
  ...mindmapJa,
  ...diagramJa,
  ...docsJa,
};
