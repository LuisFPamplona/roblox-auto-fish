import { useEffect, useState } from "react";
import { Crosshair, RefreshCw, ScanSearch } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import type { OverlayTarget, RegionPreview, RelRect } from "../lib/types";
import { Button, Pill, cx } from "./primitives";

export function RegionField({ target, value }: { target: OverlayTarget; value: RelRect }) {
  const roblox = useStore((s) => s.roblox);
  const update = useStore((s) => s.update);
  const [preview, setPreview] = useState<RegionPreview | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = async () => {
    if (!roblox) return;
    setBusy(true);
    try {
      setPreview(await api.regionPreview(value, 240));
      setErr(null);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value.x, value.y, value.w, value.h, roblox?.client.w, roblox?.client.h]);

  const autoDetect = async () => {
    setBusy(true);
    try {
      const r = await api.detectBarRegion();
      await update((s) => {
        s.regions.bar = r;
      });
      setErr(null);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const score = preview?.confidence.score ?? 0;
  const tone = target !== "bar_region" ? "mute" : score >= 0.6 ? "ok" : score > 0 ? "warn" : "mute";

  return (
    <div className="flex gap-3">
      <div
        className={cx(
          "relative rounded-lg overflow-hidden border bg-black/40 shrink-0",
          tone === "ok" ? "border-ok/50" : "border-line-strong",
        )}
        style={{ width: 120, height: 120 }}
      >
        {preview ? (
          <img
            src={`data:image/png;base64,${preview.png_base64}`}
            alt=""
            className="w-full h-full object-contain"
            style={{ imageRendering: preview.width < 120 ? "pixelated" : "auto" }}
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-fg-mute text-[11px] text-center px-2">
            {roblox ? "no preview" : "open Roblox"}
          </div>
        )}
        {preview?.reading && (
          <div className="absolute bottom-1 left-1">
            <Pill tone="ok">bar seen</Pill>
          </div>
        )}
      </div>
      <div className="flex-1 min-w-0 flex flex-col gap-2">
        <div className="font-mono text-[11px] text-fg-dim tabular-nums">
          {(value.x * 100).toFixed(1)}%, {(value.y * 100).toFixed(1)}% · {(value.w * 100).toFixed(1)}% × {(value.h * 100).toFixed(1)}%
        </div>
        {target === "bar_region" && preview && (
          <div className="flex items-center gap-2 text-[11px]">
            <Pill tone={tone}>match {Math.round(score * 100)}%</Pill>
            <span className="text-fg-mute">blue {Math.round(preview.confidence.bar * 100)}% · dark {Math.round(preview.confidence.fish * 100)}%</span>
          </div>
        )}
        {err && <div className="text-[11px] text-warn leading-snug">{err}</div>}
        <div className="flex gap-2 mt-auto flex-wrap">
          <Button size="sm" disabled={!roblox} onClick={() => api.overlayOpen(target)} icon={<Crosshair size={13} />}>
            Draw
          </Button>
          {target === "bar_region" && (
            <Button size="sm" disabled={!roblox || busy} onClick={autoDetect} icon={<ScanSearch size={13} />}>
              Auto-detect
            </Button>
          )}
          <Button size="sm" kind="ghost" disabled={!roblox || busy} onClick={refresh} icon={<RefreshCw size={13} className={cx(busy && "animate-spin")} />} />
        </div>
      </div>
    </div>
  );
}
