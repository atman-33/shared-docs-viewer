import { useEffect, useState } from "react";
import { FigureViewer } from "@/components/docs/figure-viewer";
import { api } from "@/lib/api";
import { useT } from "@/lib/i18n";

/**
 * A standalone image file on the pan/zoom canvas the Docs tab uses for
 * figures (T-0279).
 *
 * The bytes come through the backend (`docs_read_asset`) rather than the
 * webview: the file may sit on a share the webview cannot reach, and the
 * backend is where the containment guard lives. The pan/zoom canvas itself is
 * `FigureViewer` with the data URI as its content, so there is no second zoom
 * implementation to drift from the first.
 *
 * Shared by the Projects tab's Backlog and Documents panes: both list image
 * files a task left behind, and both read them through the same `docs_*`
 * commands as the Docs tab.
 */
export function ImagePreview({
  path,
  title,
  refreshToken,
  onError,
}: {
  /** Absolute path of the image file; "" when nothing is selected. */
  path: string;
  /** What the canvas shows above the image. */
  title: string;
  /** Bumped by the toolbar's refresh button to re-read the file. */
  refreshToken: number;
  onError: (message: string) => void;
}) {
  const t = useT();
  const [dataUri, setDataUri] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!path) {
      setDataUri("");
      return;
    }
    let live = true;
    setLoading(true);
    setDataUri("");
    void api
      .docsReadAsset(path)
      .then((uri) => {
        if (live) setDataUri(uri);
      })
      .catch((e) => {
        if (live) onError(String(e));
      })
      .finally(() => {
        if (live) setLoading(false);
      });
    return () => {
      live = false;
    };
    // `onError` is the parent's `setError`, stable across renders.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [path, refreshToken]);

  if (!path) return null;
  if (loading && !dataUri) {
    return <p className="p-6 text-xs text-muted-foreground">{t("docs.preview.reading")}</p>;
  }
  if (!dataUri) return null;
  return (
    <FigureViewer
      key={`${path}|${refreshToken}`}
      figure={{ type: "image", content: dataUri, title }}
    />
  );
}
