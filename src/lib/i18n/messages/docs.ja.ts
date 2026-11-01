import type { MessageKey } from "./en";

/** Japanese strings for the Docs tab and its viewer window (T-0423). */
export const docsJa: Partial<Record<MessageKey, string>> = {
  "docs.tree.loading": "読み込み中…",
  "docs.tree.noMatches": "一致するものがありません。",
  "docs.tree.folderEmpty": "このフォルダは空です。",
  "docs.tree.collapse": "折りたたむ",
  "docs.tree.expand": "展開",
  "docs.tree.ariaLabel": "ドキュメント",

  "docs.entry.openWithDefaultApp": "既定のアプリで開く",
  "docs.entry.showInExplorer": "エクスプローラーで表示",
  "docs.entry.addToShortcuts": "ショートカットに追加",
  "docs.entry.renameDiagram": "図解の名前を変更",
  "docs.entry.deleteDiagram": "図解をごみ箱に移動",
  "docs.entry.copyPath": "パスをコピー",
  "docs.entry.copyPaths": "{count} 件のパスをコピー",
  "docs.entry.opensOutsideHint": "{name} \u2014 \u4ed6\u306e\u30a2\u30d7\u30ea\u3067\u958b\u304d\u307e\u3059",

  "docs.view.filterPlaceholder": "名前でフィルター",
  "docs.view.collapseAllHint": "すべてのフォルダを折りたたむ",
  "docs.view.collapseAll": "すべて折りたたむ",
  "docs.view.refreshHint": "フォルダを再読み込み（ツリーは開いたまま）",
  "docs.view.refresh": "再読み込み",
  "docs.view.openPathHint": "貼り付けたパスを開く",
  "docs.view.openPathAria": "貼り付けたパスを開く",
  "docs.view.openPathPlaceholder": "開くパスを貼り付け…",
  "docs.view.openPathAriaLabel": "開くパスを貼り付け",
  "docs.view.foundBySearch":
    "フォルダ内を検索して見つかりました — フォルダ構成が貼り付けたパスと異なります: {path}",
  "docs.view.openedByTail": "末尾一致で開きました: {path}",
  "docs.view.matchesFoundByName": "この名前のファイルが {count} 件見つかりました — 1つ選んでください。",
  "docs.view.matchesFoundByTail": "このパスの末尾に一致するファイルが {count} 件あります — 1つ選んでください。",
  "docs.view.noRootsMessage": "\u30d5\u30a9\u30eb\u30c0\u304c\u307e\u3060\u767b\u9332\u3055\u308c\u3066\u3044\u307e\u305b\u3093\u3002\u30c1\u30fc\u30e0\u306e Markdown \u3092\u7f6e\u3044\u3066\u3044\u308b\u5171\u6709\u30d5\u30a9\u30eb\u30c0\uff08\u305f\u3068\u3048\u3070 Google Drive \u306e\u30cd\u30c3\u30c8\u30ef\u30fc\u30af\u30c9\u30e9\u30a4\u30d6\uff09\u3092\u8ffd\u52a0\u3059\u308b\u3068\u3001\u3053\u3053\u3067\u30c9\u30ad\u30e5\u30e1\u30f3\u30c8\u3092\u8aad\u3081\u307e\u3059\u3002\u66f8\u304d\u8fbc\u307e\u308c\u308b\u306e\u306f\u7de8\u96c6\u3057\u305f\u56f3\u89e3\u30e1\u30e2\u3060\u3051\u3067\u3059\u3002",
  "docs.view.rootUnreachable":
    "{path} はこの PC から到達できません。ドライブがマウントされているか確認し、場所が変わっている場合は上の鉛筆ボタンでパスを修正してください。",
  "docs.view.tryAgain": "再試行",
  "docs.rename.title": "図解の名前変更",
  "docs.rename.nameLabel": "名前",
  "docs.rename.namePlaceholder": "図解の名前…",
  "docs.delete.title": "図解をごみ箱に移動",
  "docs.delete.confirmDescription":
    "「{name}」をごみ箱に移動します。間違いだった場合はごみ箱から元に戻せます。",

  "docs.sidebar.dragToReorder": "ドラッグして並べ替え",
  "docs.sidebar.revealInTree": "ツリーで表示",
  "docs.sidebar.removeFromShortcuts": "ショートカットから削除",
  "docs.sidebar.shortcuts": "ショートカット",
  "docs.sidebar.shortcutsEmptyElsewhere":
    "このフォルダにはスターがありません。他のフォルダに {count} 件あります。",
  "docs.sidebar.shortcutsEmpty":
    "ツリーの右クリックメニューからフォルダやドキュメントをスターすると、ここに表示されます。",
  "docs.sidebar.recentFiles": "最近使ったファイル",
  "docs.sidebar.clearListHint": "リストを消去",
  "docs.sidebar.clearRecentFiles": "最近使ったファイルを消去",
  "docs.sidebar.recentEmpty": "ここで開いたドキュメントが順に表示されます。",
  "docs.sidebar.removeFromList": "リストから削除",
  "docs.sidebar.notes": "ノート",
  "docs.sidebar.copyNotesPrompt": "ノートをプロンプトとしてコピー",
  "docs.sidebar.deleteAllNotesHint": "このドキュメントのすべてのノートを削除",
  "docs.sidebar.clearNotesAria": "ノートを消去",
  "docs.sidebar.notesEmpty": "ドキュメント内のテキストを選択して右クリックすると、変更内容をノートに残せます。",
  "docs.sidebar.notesStale": "このドキュメントは、いくつかのノートを取った後に変更されています。",
  "docs.sidebar.deleteThisNote": "このノートを削除",

  "docs.fileList.noFolderSelected": "フォルダが選択されていません",
  "docs.fileList.filesAria": "選択したフォルダ内のファイル",
  "docs.fileList.pickFolderPrompt": "ツリーでフォルダを選ぶと、中身が一覧表示されます。",
  "docs.fileList.noFiles": "このフォルダにファイルはありません。",

  "docs.preview.pickPrompt": "左側でドキュメントを選んで読んでください。",
  "docs.preview.zoomOutHint": "縮小（Ctrl+ホイール）",
  "docs.preview.zoomOut": "縮小",
  "docs.preview.resetZoomHint": "ズームをリセット",
  "docs.preview.zoomInHint": "拡大（Ctrl+ホイール）",
  "docs.preview.zoomIn": "拡大",
  "docs.preview.readingWidth": "読みやすい幅",
  "docs.preview.useFullWidth": "幅いっぱいに表示",
  "docs.preview.toggleFullWidth": "幅いっぱいに表示を切り替え",
  "docs.preview.openNewWindowHint": "新しいウィンドウで開く",
  "docs.preview.openNewWindow": "新しいウィンドウで開く",
  "docs.preview.showFileHint": "このファイルをエクスプローラーで表示",
  "docs.preview.reading": "読み込み中…",
  "docs.preview.empty": "このドキュメントは空です。",
  "docs.preview.wikiNotFound": "[[{target}]] \u306b\u4e00\u81f4\u3059\u308b\u30d5\u30a1\u30a4\u30eb\u306f\u3053\u306e\u30d5\u30a9\u30eb\u30c0\u306b\u3042\u308a\u307e\u305b\u3093\u3002",
  "docs.preview.wikiAmbiguous":
    "[[{target}]] に一致するファイルが {count} 件あるため、プレビューでは開けません。",

  "docs.rootsBar.addFolderTitle": "ドキュメントフォルダを追加",
  "docs.rootsBar.pickFolder": "フォルダを選択",
  "docs.rootsBar.noFoldersRegistered": "フォルダが登録されていません",
  "docs.rootsBar.notOnThisPc": "（この PC にありません）",
  "docs.rootsBar.addHint": "参照するフォルダを登録",
  "docs.rootsBar.addAria": "フォルダを追加",
  "docs.rootsBar.editHint": "このフォルダの名前とパスを編集",
  "docs.rootsBar.editAria": "フォルダを編集",
  "docs.rootsBar.removeHint": "このフォルダの登録を解除（共有先には何も行いません）",
  "docs.rootsBar.removeAria": "フォルダを削除",
  "docs.rootsBar.settingsHint": "Docs 設定（PlantUML サーバー）",
  "docs.rootsBar.settingsAria": "Docs 設定",
  "docs.rootsBar.removeConfirmTitle": "このフォルダを削除しますか？",
  "docs.rootsBar.removeConfirmDescription":
    "「{name}」がリストから削除されます。フォルダ自体とその中身はそのまま残ります。",

  "docs.rootDialog.title": "フォルダを編集",
  "docs.rootDialog.description": "\u30d5\u30a9\u30eb\u30c0\u306e\u4e00\u89a7\u306f\u3053\u306e\u7aef\u672b\u306b\u4fdd\u5b58\u3055\u308c\u308b\u305f\u3081\u3001\u5225\u306e PC \u306b\u306f\u5f15\u304d\u7d99\u304c\u308c\u307e\u305b\u3093\u3002",
  "docs.rootDialog.name": "名前",
  "docs.rootDialog.namePlaceholder": "チーム共有",
  "docs.rootDialog.nameHint": "ピッカーに表示するラベルです。空にするとパスが表示されます。",
  "docs.rootDialog.folder": "フォルダ",
  "docs.rootDialog.folderPlaceholder": "G:/shared drives/team/docs",
  "docs.rootDialog.pickFolderHint": "フォルダを選択",
  "docs.rootDialog.browseAria": "フォルダを参照",
  "docs.rootDialog.pickFolderDialogTitle": "フォルダを選択",

  "docs.settings.title": "Docs 設定",
  "docs.settings.description": "\u30d5\u30a9\u30eb\u30c0\u306e\u4e00\u89a7\u3068\u540c\u69d8\u306b\u3053\u306e\u7aef\u672b\u306b\u4fdd\u5b58\u3055\u308c\u307e\u3059\u3002",
  "docs.settings.listPaneLabel": "ツリーの横にファイル一覧を表示",
  "docs.settings.listPaneDescription":
    "Obsidian の Notebook Navigator と同じようにサイドバーを分割します: 左にフォルダ、右に選んだフォルダのファイル。オフにすると、サイドバーは1つのツリーにまとまります。",
  "docs.settings.showHiddenLabel": "ドット付きフォルダ・ファイルを表示",
  "docs.settings.showHiddenDescription":
    ".backup のようなドットで始まる名前（.git や .obsidian も含む）を一覧表示します。オフの場合は非表示です。desktop.ini はどちらでも非表示です。",
  "docs.settings.remoteImagesLabel": "https: の URL から画像を読み込む",
  "docs.settings.remoteImagesDescription":
    "オフの場合、Web を指す画像は読み込めないものとして表示されます。オンにすると取得しますが、配信元に読み込みが伝わるため、信頼できるドキュメントにのみオンにしてください。http: はどちらでも読み込まれません。",
  "docs.settings.plantumlLabel": "PlantUML サーバー",
  "docs.settings.plantumlDescription":
    "ブロックはこのサーバーで描画されます: 各図の元データがサーバーに送られ、画像が返されます。空にするとコードのまま表示され、何も送信しません。パブリックサーバー（{server}）は動作しますが第三者のサーバーです — チームのドキュメントには自前のサーバーを使うことを推奨します。",

  "docs.figure.zoomOutHint": "縮小（ホイール）",
  "docs.figure.zoomInHint": "拡大（ホイール）",
  "docs.figure.fitHint": "ウィンドウに合わせる",
  "docs.figure.fit": "ウィンドウに合わせる",
  "docs.figure.actualSizeHint": "実際のサイズ（100%）",
  "docs.figure.actualSize": "実際のサイズ",

  "docs.html.failedStyles":
    "このページが参照する {count} 件のスタイルシートを読み込めなかったため、記載どおりに見えない場合があります。",
  "docs.html.failedImages":
    "このページが参照する {count} 件の画像を読み込めなかったため、記載どおりに見えない場合があります。",
  "docs.html.failedBoth":
    "このページが参照する {styles} 件のスタイルシートと {images} 件の画像を読み込めなかったため、記載どおりに見えない場合があります。",
  "docs.html.scriptsDisabled":
    "このページのスクリプトはプレビューでは無効です。実行するには、上のツールバーから既定のアプリで開いてください。",
  "docs.html.preparingPage": "ページを準備中…",

  "docs.annotation.placeholder": "ここをどう変更しますか？",

  "docs.viewer.nothingToShow": "表示するものがありません — このウィンドウは開かれた対象を失いました。",
  "docs.viewer.windowTitleFallback": "shared-docs-viewer",
};
