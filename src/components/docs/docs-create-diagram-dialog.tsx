import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { api } from "@/lib/api";
import {
  CREATABLE_KINDS,
  KIND_LABEL_KEY,
  SIMPLE_KINDS,
  type DiagramKind,
} from "@/lib/diagram-kinds";
import { useT } from "@/lib/i18n";

/**
 * Creates a diagram note inside the open document folder (T-1001).
 *
 * The viewer's own creation path: the Diagrams tab's dialog needs a vault
 * project, which the viewer does not have. A kind and a name are enough —
 * the note lands beside the folder being read, with the kind's starting
 * text. Renaming and deleting stay out of the shell.
 */
export function DocsCreateDiagramDialog({
  open,
  onOpenChange,
  dir,
  defaultKind,
  onCreated,
  onError,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The folder the note is created in; "" disables creating. */
  dir: string;
  defaultKind: DiagramKind;
  /** Called with the new note's path. */
  onCreated: (path: string) => void;
  onError: (message: string) => void;
}) {
  const t = useT();
  const [kind, setKind] = useState<DiagramKind>(defaultKind);
  const [title, setTitle] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open) return;
    setKind(defaultKind);
    setTitle("");
    setError("");
  }, [open, defaultKind]);

  async function submit() {
    if (!dir || !title.trim() || busy) return;
    setBusy(true);
    setError("");
    try {
      const path = await api.docsCreateDiagram(dir, kind, title.trim());
      onOpenChange(false);
      onCreated(path);
    } catch (e) {
      const message = t("diagram.create.failed", { error: String(e) });
      setError(message);
      onError(message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("diagram.create.title")}</DialogTitle>
        </DialogHeader>
        <div className="space-y-3 text-xs">
          <label className="block space-y-1">
            <span className="text-muted-foreground">{t("diagram.create.kind")}</span>
            <Select value={kind} onValueChange={(v) => setKind(v as DiagramKind)}>
              <SelectTrigger className="h-8 w-full text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CREATABLE_KINDS.map((value) => (
                  <SelectItem key={value} value={value}>
                    {t(KIND_LABEL_KEY[value])}
                    {SIMPLE_KINDS.includes(value) && ` (${t("diagram.create.simple")})`}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </label>
          <label className="block space-y-1">
            <span className="text-muted-foreground">{t("diagram.create.name")}</span>
            <Input
              value={title}
              autoFocus
              placeholder={t("diagram.create.namePlaceholder")}
              className="h-8 text-xs"
              onChange={(e) => setTitle(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") void submit();
              }}
            />
          </label>
          {error && <p className="text-destructive">{error}</p>}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button disabled={busy || !dir || !title.trim()} onClick={() => void submit()}>
            {t("common.create")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
