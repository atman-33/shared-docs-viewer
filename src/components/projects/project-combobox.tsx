import * as React from "react";
import { CheckIcon, ChevronsUpDownIcon } from "lucide-react";

import { cn } from "@/lib/utils";
import { optionSearchValue } from "@/lib/combobox-options";
import { useT } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import type { VaultProject } from "@/types";

interface ProjectComboboxProps {
  /** Active (non-archived) projects, in display order. */
  active: VaultProject[];
  /** Archived projects, in display order. Empty when the Archived toggle is off. */
  archived: VaultProject[];
  selected: string;
  onSelect: (slug: string) => void;
  className?: string;
}

/**
 * The Projects tab's top picker (T-0997).
 *
 * The main selection route after the sidebar went away: the trigger shows the
 * current project, the dropdown lists every project for a typed filter to cut
 * through. Built on the same Command/Popover pair as `BranchCombobox`, not on
 * the generic `Combobox`, because the trigger shows the project's *name* while
 * the committed value is its slug, and the list is grouped (pinned, rest,
 * archived) rather than flat.
 *
 * Each item's search text carries the name and summary as well as the slug
 * (via `optionSearchValue`), so typing "garden" finds the project whose README
 * paragraph mentions it — but `onSelect` commits the slug the row closed over,
 * never anything parsed back out of that text.
 */
export function ProjectCombobox({
  active,
  archived,
  selected,
  onSelect,
  className,
}: ProjectComboboxProps) {
  const t = useT();
  const [open, setOpen] = React.useState(false);

  const pinned = active.filter((p) => p.pinned);
  const plain = active.filter((p) => !p.pinned);
  const current =
    [...active, ...archived].find((p) => p.slug === selected) ?? null;

  const commit = (slug: string) => {
    setOpen(false);
    if (slug !== selected) onSelect(slug);
  };

  const renderItem = (p: VaultProject) => (
    <CommandItem
      key={p.slug}
      value={optionSearchValue(p.slug, { label: p.name, meta: p.summary })}
      onSelect={() => commit(p.slug)}
    >
      <CheckIcon
        className={cn(
          "size-3.5",
          p.slug === selected ? "opacity-100" : "opacity-0",
        )}
      />
      <span className="truncate">{p.name}</span>
      <span className="truncate font-mono text-[11px] text-muted-foreground">
        {p.slug}
      </span>
      {p.archived && (
        <span className="shrink-0 rounded bg-muted-foreground/15 px-1 text-[10px] uppercase text-muted-foreground">
          {t("projects.list.archivedBadge")}
        </span>
      )}
    </CommandItem>
  );

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn(
            "h-8 max-w-64 justify-between px-2 text-xs font-normal",
            !current && "text-muted-foreground",
            className,
          )}
        >
          <span className="truncate">
            {current ? current.name : t("projects.view.projectPickerPlaceholder")}
          </span>
          <ChevronsUpDownIcon className="size-3.5 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-(--radix-popover-trigger-width) p-0" align="start">
        <Command
          filter={(value, search) =>
            value.toLowerCase().includes(search.toLowerCase().trim()) ? 1 : 0
          }
        >
          <CommandInput
            placeholder={t("projects.view.searchPlaceholder")}
            className="h-8 text-xs"
          />
          <CommandList>
            <CommandEmpty>{t("projects.view.projectPickerEmpty")}</CommandEmpty>
            {pinned.length > 0 && (
              <CommandGroup heading={t("projects.list.pinned")}>
                {pinned.map(renderItem)}
              </CommandGroup>
            )}
            <CommandGroup>{plain.map(renderItem)}</CommandGroup>
            {archived.length > 0 && (
              <CommandGroup heading={t("projects.list.archived")}>
                {archived.map(renderItem)}
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
