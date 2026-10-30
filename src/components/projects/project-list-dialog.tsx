import { useState } from "react";
import { CircleAlert } from "lucide-react";
import { useT } from "@/lib/i18n";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ProjectList } from "@/components/projects/project-list";
import type {
  ProjectOrderWrite,
  TaskCounts,
  UnknownProject,
} from "@/lib/vault-project";
import type { VaultProject } from "@/types";

interface ProjectListDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Active (non-archived) projects, in display order. */
  active: VaultProject[];
  /** Archived projects, in display order. Empty when the Archived toggle is off. */
  archived: VaultProject[];
  selected: string;
  counts: Map<string, TaskCounts>;
  /** False while the list is filtered or the sort mode has no positions —
   *  a drop position computed from a filtered list would record an order the
   *  user never asked for. */
  reorderable: boolean;
  orphans: UnknownProject[];
  onSelect: (slug: string) => void;
  onTogglePin: (project: VaultProject) => void;
  /** The writes one drop implies; usually a single project. */
  onMove: (writes: ProjectOrderWrite[]) => void;
}

/**
 * The Projects tab's secondary picker (T-0997).
 *
 * The sidebar went away, so the full list — pins, drag-reorder, archived
 * section and the orphan report — lives here, behind the toolbar button that
 * sits where the sidebar toggle used to be. The combobox up top is the main
 * route; this dialog is where a project is managed, not just chosen.
 *
 * The filter input narrows this list only. It never touches the selection:
 * typing must not switch the detail pane out from under the current project.
 */
export function ProjectListDialog({
  open,
  onOpenChange,
  active,
  archived,
  selected,
  counts,
  reorderable,
  orphans,
  onSelect,
  onTogglePin,
  onMove,
}: ProjectListDialogProps) {
  const t = useT();
  const [query, setQuery] = useState("");

  const close = (next: boolean) => {
    if (!next) setQuery("");
    onOpenChange(next);
  };

  const needle = query.trim().toLowerCase();
  const match = (p: VaultProject) =>
    !needle ||
    p.slug.toLowerCase().includes(needle) ||
    p.name.toLowerCase().includes(needle) ||
    p.summary.toLowerCase().includes(needle);
  const matchedActive = active.filter(match);
  const matchedArchived = archived.filter(match);
  const empty =
    active.length === 0
      ? t("projects.view.emptyList")
      : t("projects.view.projectPickerEmpty");

  const select = (slug: string) => {
    onSelect(slug);
    close(false);
  };

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent className="flex max-h-[85vh] flex-col gap-4 sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("projects.view.projectList")}</DialogTitle>
        </DialogHeader>
        <Input
          value={query}
          placeholder={t("projects.view.searchPlaceholder")}
          className="h-8 text-sm"
          onChange={(e) => setQuery(e.target.value)}
        />
        <div className="min-h-0 flex-1 overflow-y-auto">
          {matchedActive.length === 0 && matchedArchived.length === 0 ? (
            <p className="p-4 text-sm text-muted-foreground">{empty}</p>
          ) : (
            <ProjectList
              active={matchedActive}
              archived={matchedArchived}
              selected={selected}
              counts={counts}
              reorderable={reorderable && needle === ""}
              onSelect={select}
              onTogglePin={onTogglePin}
              onMove={onMove}
            />
          )}
          {orphans.length > 0 && (
            <div className="border-b bg-muted/30 px-3 py-2">
              <p className="flex items-center gap-1.5 text-[11px] font-medium text-muted-foreground">
                <CircleAlert className="size-3" />
                {t("projects.view.orphansHeading")}
              </p>
              <ul className="mt-1 space-y-0.5">
                {orphans.map((o) => (
                  <li key={o.project} className="text-[11px] text-muted-foreground">
                    <span className="font-mono">{o.project}</span>{" "}
                    {t(
                      o.counts.total === 1
                        ? "projects.view.orphanCountOne"
                        : "projects.view.orphanCountOther",
                      { count: o.counts.total },
                    )}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
