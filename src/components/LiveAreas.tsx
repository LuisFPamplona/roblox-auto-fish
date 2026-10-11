import { useEffect, useState } from "react";
import { PencilRuler } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore, isActive } from "../lib/store";
import type { RegionPreview } from "../lib/types";
import { Button, Kbd, cx } from "./primitives";

export function LiveAreas() {
  const settings = useStore((s) => s.settings);
  const roblox = useStore((s) => s.roblox);
  const state = useStore((s) => s.state);
  const reading = useStore((s) => s.reading);
  const [bar, setBar] = useState<RegionPreview | null>(null);
  const [drop, setDrop] = useState<RegionPreview | null>(null);
  const active = isActive(state);
  const regions = settings?.regions;
  const hk = settings?.hotkeys;

  useEffect(() => {
    if (!roblox || !regions) return;
    let alive = true;
    const tick = async () => {
      try {
        const [b, d] = await Promise.all([api.regionPreview(regions.bar, 200), api.regionPreview(regions.drop, 320)]);
        if (alive) {
          setBar(b);
          setDrop(d);
        }
      } catch {
        if (alive) { setBar(null); setDrop(null); }
      }
    };
    tick();
    const t = setInterval(tick, active ? 700 : 2000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, [roblox?.client.w, roblox?.client.h, !!roblox, active, regions?.bar.x, regions?.bar.y, regions?.bar.w, regions?.bar.h, regions?.drop.x, regions?.drop.y, regions?.drop.w, regions?.drop.h]);

  const score = bar?.confidence.score ?? 0;
  const barTone = reading ? "ok" : score >= 0.6 ? "ok" : score > 0 ? "warn" : "mute";

  return (
    <div className="px-4 py-3 flex flex-col gap-3 @container">
      <div className="grid gap-3 grid-cols-1 @[440px]:grid-cols-[minmax(160px,2fr)_3fr]">
        <Tile
          label="Fishing bar"
          color="#4f8cff"
          preview={bar}
          fallbackRatio={0.5}
          status={reading ? (reading.hold ? "reeling · hold" : "reeling · release") : score >= 0.6 ? "bar visible" : active ? "waiting" : "idle"}
          tone={barTone}
          overlay={
            reading && bar ? (
              <>
                <div
                  className="absolute inset-x-0 bg-white/30"
                  style={{ top: `${((reading.bar.y0 + reading.fish.start) / bar.height) * 100}%`, height: `${((reading.fish.end - reading.fish.start + 1) / bar.height) * 100}%` }}
                />
                <div
                  className={cx("absolute inset-x-0", reading.hold ? "bg-ok" : "bg-accent")}
                  style={{ top: `${((reading.bar.y0 + reading.marker.start) / bar.height) * 100}%`, height: `max(2px, ${((reading.marker.end - reading.marker.start + 1) / bar.height) * 100}%)` }}
                />
                <div
                  className="absolute inset-x-0 border-t border-dashed border-warn"
                  style={{ top: `${((reading.bar.y0 + (reading.marker_center + reading.predicted_error) * (reading.bar.y1 - reading.bar.y0 + 1)) / bar.height) * 100}%` }}
                />
              </>
            ) : null
          }
        />
        <Tile label="Drop message" color="#22c55e" preview={drop} fallbackRatio={3} status={drop ? "watching" : "no capture"} tone={drop ? "accent" : "mute"} />
      </div>
      <div className="flex items-center gap-x-3 gap-y-1.5 flex-wrap text-[11px] text-fg-dim">
        <Button size="sm" onClick={() => api.overlayOpenRegions()} disabled={!roblox} icon={<PencilRuler size={13} />}>
          Edit areas
        </Button>
        <span className="inline-flex items-center gap-1">
          <Kbd>{hk?.overlay ?? "F2"}</Kbd> edit areas
        </span>
        <span className="inline-flex items-center gap-1">
          <Kbd>{hk?.toggle ?? "F1"}</Kbd> start / pause
        </span>
        <span className="inline-flex items-center gap-1">
          <Kbd>{hk?.hide_hud ?? "F4"}</Kbd> hide HUD
        </span>
      </div>
    </div>
  );
}

const PILL = {
  ok: "bg-ok-soft text-ok",
  warn: "bg-warn-soft text-warn",
  accent: "bg-accent-soft text-accent",
  mute: "bg-white/[0.06] text-fg-mute",
} as const;

function Tile({
  label,
  color,
  preview,
  fallbackRatio,
  status,
  tone,
  overlay,
}: {
  label: string;
  color: string;
  preview: RegionPreview | null;
  fallbackRatio: number;
  status: string;
  tone: keyof typeof PILL;
  overlay?: React.ReactNode;
}) {
  const ratio = preview ? preview.width / preview.height : fallbackRatio;
  return (
    <div className="min-w-0 rounded-xl border border-line bg-bg-elev/70 overflow-hidden flex flex-col">
      <div className="flex items-center gap-2 px-2.5 py-1.5 text-[11px] min-w-0">
        <span className="w-1.5 h-1.5 rounded-full shrink-0" style={{ background: color, boxShadow: `0 0 6px ${color}` }} />
        <span className="font-medium truncate">{label}</span>
        <span className={cx("ml-auto shrink-0 rounded-full px-1.5 py-px font-mono text-[10px] whitespace-nowrap", PILL[tone])}>{status}</span>
      </div>
      <div className="relative h-28 border-t border-line bg-black/40">
        <div className="relative h-full max-w-full mx-auto overflow-hidden" style={{ aspectRatio: ratio }}>
          {preview ? (
            <img src={`data:image/png;base64,${preview.png_base64}`} alt="" className="block w-full h-full" style={{ imageRendering: preview.width < 200 ? "pixelated" : "auto" }} />
          ) : (
            <div className="w-full h-full grid place-items-center text-[11px] text-fg-mute text-center px-1 leading-tight">open Roblox</div>
          )}
          {overlay}
        </div>
      </div>
    </div>
  );
}
