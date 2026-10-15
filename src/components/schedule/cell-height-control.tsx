import { useState } from "react";
import { MoveVertical } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Hint } from "@/components/ui/hint";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { useT } from "@/lib/i18n";
import {
  CELL_EXTRA_DEFAULT,
  CELL_EXTRA_MAX,
  CELL_EXTRA_PRESETS,
  CELL_EXTRA_STEP,
} from "@/lib/schedule/cell-height";

/**
 * Day-cell height of the calendar (T-0822): S / M / L presets plus a slider
 * for finer steps. The value is the extra height added to every surface's own
 * minimum, so the screen, the HTML export and the image copy all grow by the
 * same amount and 0 leaves them as they were.
 */

interface Props {
  extra: number;
  onChange: (extra: number) => void;
  disabled?: boolean;
}

export function CellHeightControl({ extra, onChange, disabled }: Props) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const changed = extra !== CELL_EXTRA_DEFAULT;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <Hint label={t("schedule.cellHeight.hint")} disabled={disabled}>
        <PopoverTrigger asChild>
          <Button
            size="sm"
            variant={changed ? "secondary" : "ghost"}
            className="h-7 text-xs"
            disabled={disabled}
          >
            <MoveVertical className="mr-1 size-3" />
            {changed
              ? t("schedule.cellHeight.buttonActive", { px: extra })
              : t("schedule.cellHeight.button")}
          </Button>
        </PopoverTrigger>
      </Hint>
      <PopoverContent className="w-64 space-y-3 p-3 text-xs" align="start">
        <div className="text-muted-foreground">{t("schedule.cellHeight.label")}</div>
        <div className="flex items-center gap-1">
          {CELL_EXTRA_PRESETS.map((preset) => (
            <Button
              key={preset.id}
              size="sm"
              variant={extra === preset.extra ? "secondary" : "ghost"}
              className="h-7 px-3 text-xs"
              onClick={() => onChange(preset.extra)}
            >
              {preset.id}
            </Button>
          ))}
        </div>
        <div className="flex items-center gap-2">
          <input
            type="range"
            className="min-w-0 flex-1"
            min={0}
            max={CELL_EXTRA_MAX}
            step={CELL_EXTRA_STEP}
            value={extra}
            aria-label={t("schedule.cellHeight.label")}
            onChange={(e) => onChange(Number(e.target.value))}
          />
          <span className="w-12 text-right tabular-nums">+{extra}px</span>
        </div>
        <Button
          size="sm"
          variant="ghost"
          className="h-7 text-xs"
          onClick={() => onChange(CELL_EXTRA_DEFAULT)}
          disabled={!changed}
        >
          {t("schedule.cellHeight.reset")}
        </Button>
      </PopoverContent>
    </Popover>
  );
}
