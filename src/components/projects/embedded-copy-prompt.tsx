import { CopyPromptButton } from "@/components/copy-prompt-button";
import { useT } from "@/lib/i18n";
import type { EmbeddedDiagram } from "@/lib/embedded-diagram";

/**
 * The `diagram-edit` prompt copy for the right end of an editor toolbar
 * (T-0998).
 *
 * Every diagram view renders this at the head of its `ml-auto` group, so the
 * button sits in the same place in the Diagrams tab and in the Projects
 * panes (Backlog and Documents) — the host decides whether it exists by
 * passing `onCopyPrompt`. One component so all nine kinds read the same.
 */
export function EmbeddedCopyPrompt({ embedded }: { embedded?: EmbeddedDiagram }) {
  const t = useT();
  if (!embedded?.onCopyPrompt) return null;
  return (
    <CopyPromptButton
      label={t("diagram.aiPanel.copyPrompt")}
      onCopy={async () => {
        await embedded.onCopyPrompt?.();
      }}
    />
  );
}
