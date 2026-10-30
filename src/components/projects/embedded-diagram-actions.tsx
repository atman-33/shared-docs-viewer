import { BookOpen, Shapes } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CopyPromptButton } from "@/components/copy-prompt-button";
import { useT } from "@/lib/i18n";
import type { EmbeddedDiagram } from "@/lib/embedded-diagram";

/**
 * The "open elsewhere" buttons for an embedded editor toolbar (T-1005),
 * plus the AI prompt copy (T-0998).
 *
 * Every diagram view renders these at the head of its toolbar row when the
 * host passes the callbacks (the Projects panes). The Diagrams tab passes
 * none, so its toolbars render exactly as before — its own copy button lives
 * in the tab toolbar. One component so all nine kinds read the same.
 */
export function EmbeddedDiagramActions({ embedded }: { embedded?: EmbeddedDiagram }) {
  const t = useT();
  if (!embedded?.onCopyPrompt && !embedded?.onOpenInDiagrams && !embedded?.onOpenInObsidian) return null;
  return (
    <>
      {embedded.onCopyPrompt && (
        <CopyPromptButton
          label={t("diagram.aiPanel.copyPrompt")}
          onCopy={async () => {
            await embedded.onCopyPrompt?.();
          }}
        />
      )}
      {embedded.onOpenInDiagrams && (
        <Button
          size="sm"
          variant="ghost"
          className="h-6 shrink-0 gap-1.5 text-xs"
          onClick={embedded.onOpenInDiagrams}
        >
          <Shapes className="size-3.5" />
          {t("projects.view.backlog.openInDiagrams")}
        </Button>
      )}
      {embedded.onOpenInObsidian && (
        <Button
          size="sm"
          variant="ghost"
          className="h-6 shrink-0 gap-1.5 text-xs"
          onClick={embedded.onOpenInObsidian}
        >
          <BookOpen className="size-3.5" />
          {t("projects.view.docs.openInObsidian")}
        </Button>
      )}
    </>
  );
}
