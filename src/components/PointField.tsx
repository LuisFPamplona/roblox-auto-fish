import { Crosshair, X } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import type { OverlayTarget, RelPoint } from "../lib/types";
import { cx } from "./primitives";

const CELL =
  "inline-flex items-center gap-1.5 h-8 px-2.5 text-[12px] transition-colors outline-none focus-visible:bg-white/[0.1] hover:bg-white/[0.08] disabled:opacity-40 disabled:pointer-events-none";

export function PointField({
  target,
  value,
  clearable,
  onCleared,
}: {
  target: OverlayTarget;
  value: RelPoint | null;
  clearable?: boolean;
  onCleared?: () => void;
}) {
  const roblox = useStore((s) => s.roblox);
  const pick = async () => {
    try {
      await api.overlayOpen(target);
    } catch {
      /* roblox missing; the Roblox row shows the status */
    }
  };
  return (
    <div className="inline-flex items-stretch rounded-lg border border-line-strong bg-white/[0.04] overflow-hidden shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]">
      <button type="button" onClick={pick} disabled={!roblox} title="Pick on screen" className={cx(CELL, "font-mono tabular-nums", value ? "text-fg-dim hover:text-fg" : "text-fg-mute")}>
        <Crosshair size={12} className={value ? "text-accent" : undefined} />
        {value ? `${(value.x * 100).toFixed(1)}%, ${(value.y * 100).toFixed(1)}%` : "not set"}
      </button>
      <button type="button" onClick={pick} disabled={!roblox} className={cx(CELL, "border-l border-line-strong font-medium")}>
        {value ? "Re-pick" : "Pick"}
      </button>
      {clearable && value && (
        <button type="button" onClick={onCleared} title="Clear" className={cx(CELL, "border-l border-line-strong text-fg-mute hover:text-fg px-2")}>
          <X size={13} />
        </button>
      )}
    </div>
  );
}
