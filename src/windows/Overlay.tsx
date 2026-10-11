import { useEffect, useRef, useState } from "react";
import { api, on } from "../lib/ipc";
import type { OverlaySession, Reading, RegionPreview, RelPoint, RelRect } from "../lib/types";
import { TARGET_LABEL } from "../lib/types";
import { cx } from "../components/primitives";

type Mode =
  | { kind: "single"; session: OverlaySession }
  | { kind: "regions"; roblox: { w: number; h: number } }
  | { kind: "idle" };
type PxBox = { x: number; y: number; w: number; h: number };
type Drag = { type: "move" | "draw" | "resize"; edge?: string; sx: number; sy: number; start: RelRect };
type RegionKey = "bar" | "drop";

const HANDLE = 8;
const COLORS: Record<RegionKey, string> = { bar: "#4f8cff", drop: "#22c55e" };
const LABELS: Record<RegionKey, string> = { bar: "Fishing bar", drop: "Drop message" };

export default function Overlay() {
  const [mode, setMode] = useState<Mode>({ kind: "idle" });
  const [region, setRegion] = useState<RelRect | null>(null);
  const [point, setPoint] = useState<RelPoint | null>(null);
  const [regions, setRegions] = useState<{ bar: RelRect; drop: RelRect } | null>(null);
  const [active, setActive] = useState<RegionKey>("bar");
  const [reading, setReading] = useState<Reading | null>(null);
  const [barMatch, setBarMatch] = useState<RegionPreview | null>(null);
  const [size, setSize] = useState({ w: window.innerWidth, h: window.innerHeight });
  const drag = useRef<Drag | null>(null);

  useEffect(() => {
    const openSingle = (s: OverlaySession) => {
      setRegion(s.region);
      setPoint(s.point);
      setMode({ kind: "single", session: s });
    };
    const openRegions = (s: { roblox: { w: number; h: number }; bar: RelRect; drop: RelRect }) => {
      setRegions({ bar: s.bar, drop: s.drop });
      setActive("bar");
      setMode({ kind: "regions", roblox: { w: s.roblox.w, h: s.roblox.h } });
    };
    const sync = async () => {
      try {
        const p = await api.overlayPending();
        if (!p) setMode({ kind: "idle" });
        else if (p.kind === "single") openSingle(p.session);
        else openRegions(p.session);
      } catch {
        /* backend not ready */
      }
    };
    const subs = [
      on("overlay:session", openSingle),
      on("overlay:regions", openRegions),
      on("overlay:close", () => setMode({ kind: "idle" })),
      on("bot:reading", setReading),
    ];
    const onResize = () => setSize({ w: window.innerWidth, h: window.innerHeight });
    const onVisible = () => {
      if (document.visibilityState === "visible") sync();
    };
    window.addEventListener("resize", onResize);
    window.addEventListener("focus", sync);
    document.addEventListener("visibilitychange", onVisible);
    sync();
    return () => {
      subs.forEach((p) => p.then((u) => u()));
      window.removeEventListener("resize", onResize);
      window.removeEventListener("focus", sync);
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, []);

  useEffect(() => {
    if (mode.kind !== "regions" || !regions) return;
    let alive = true;
    const tick = async () => {
      try {
        const p = await api.regionPreview(regions.bar, 64);
        if (alive) setBarMatch(p);
      } catch {
        if (alive) setBarMatch(null);
      }
    };
    tick();
    const t = setInterval(tick, 700);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, [mode.kind, regions?.bar.x, regions?.bar.y, regions?.bar.w, regions?.bar.h]);

  const cancel = () => {
    setMode({ kind: "idle" });
    api.overlayCancel();
  };

  const commit = async () => {
    if (mode.kind === "single") {
      const t = mode.session.target;
      const isRegion = t === "bar_region" || t === "drop_region";
      if (isRegion ? !region || region.w < 0.005 : !point) return;
      await api.overlayCommit({ target: t, region: isRegion ? region : null, point: isRegion ? null : point });
    } else if (mode.kind === "regions" && regions) {
      await api.overlayCommitRegions(regions);
    }
    setMode({ kind: "idle" });
  };

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (mode.kind === "idle") return;
      if (e.key === "Escape") cancel();
      else if (e.key === "Enter") commit();
      else if (mode.kind === "regions") {
        if (e.key === "Tab") {
          e.preventDefault();
          setActive((a) => (a === "bar" ? "drop" : "bar"));
        } else if (e.key === "1") setActive("bar");
        else if (e.key === "2") setActive("drop");
        else if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.key) && regions) {
          e.preventDefault();
          const step = (e.shiftKey ? 10 : 1) / (e.key === "ArrowUp" || e.key === "ArrowDown" ? size.h : size.w);
          const r = { ...regions[active] };
          if (e.key === "ArrowUp") r.y -= step;
          if (e.key === "ArrowDown") r.y += step;
          if (e.key === "ArrowLeft") r.x -= step;
          if (e.key === "ArrowRight") r.x += step;
          setRegions({ ...regions, [active]: r });
        }
      }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  });

  const { w: W, h: H } = size;
  const toPx = (r: RelRect): PxBox => ({ x: r.x * W, y: r.y * H, w: r.w * W, h: r.h * H });

  if (mode.kind === "idle") return null;

  if (mode.kind === "regions" && regions) {
    const startDrag = (e: React.PointerEvent) => {
      if (e.button !== 0) return;
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      const order: RegionKey[] = [active, active === "bar" ? "drop" : "bar"];
      for (const k of order) {
        const edge = hitEdge(toPx(regions[k]), e.clientX, e.clientY);
        if (!edge) continue;
        setActive(k);
        drag.current = {
          type: edge === "inside" ? "move" : "resize",
          edge: edge === "inside" ? undefined : edge,
          sx: e.clientX,
          sy: e.clientY,
          start: regions[k],
        };
        return;
      }
    };
    const move = (e: React.PointerEvent) => {
      const d = drag.current;
      if (!d) return;
      setRegions({ ...regions, [active]: applyDrag(d, e.clientX, e.clientY, W, H) });
    };
    const cursorFor = (e: React.MouseEvent) => {
      for (const k of [active, active === "bar" ? "drop" : "bar"] as RegionKey[]) {
        const edge = hitEdge(toPx(regions[k]), e.clientX, e.clientY);
        if (edge) return edge === "inside" ? "move" : `${edge}-resize`;
      }
      return "default";
    };
    const score = barMatch?.confidence.score ?? 0;
    return (
      <div
        className="w-full h-full relative select-none"
        onPointerDown={startDrag}
        onPointerMove={move}
        onPointerUp={() => (drag.current = null)}
        onMouseMove={(e) => ((e.currentTarget as HTMLElement).style.cursor = cursorFor(e))}
        style={{ background: "rgba(5,7,12,0.28)" }}
      >
        {(["drop", "bar"] as RegionKey[]).map((k) => (
          <Box
            key={k}
            r={toPx(regions[k])}
            color={COLORS[k]}
            label={`${k === "bar" ? "1" : "2"} · ${LABELS[k]}`}
            handles={active === k}
            dim={active !== k}
            badge={
              k === "bar" && barMatch ? (
                <span
                  className="text-[10px] font-mono px-1.5 rounded-sm"
                  style={{ background: score >= 0.6 ? "#22c55e" : score > 0 ? "#f59e0b" : "#5f6675", color: "#000" }}
                >
                  match {Math.round(score * 100)}%
                </span>
              ) : undefined
            }
          />
        ))}
        {reading && <LiveBar reading={reading} origin={toPx(regions.bar)} />}
        <Toolbar
          title="Edit areas"
          hint="Drag to move · edges to resize · arrows nudge (Shift ×10) · Tab / 1 / 2 switch · Enter save · Esc cancel"
          chips={[
            { key: "1", label: LABELS.bar, active: active === "bar", color: COLORS.bar, onClick: () => setActive("bar") },
            { key: "2", label: LABELS.drop, active: active === "drop", color: COLORS.drop, onClick: () => setActive("drop") },
          ]}
          onSave={commit}
          onCancel={cancel}
          canSave
        />
      </div>
    );
  }

  if (mode.kind !== "single") return null;

  const target = mode.session.target;
  const isRegion = target === "bar_region" || target === "drop_region";
  const px = region ? toPx(region) : null;

  const onDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    const p = { x: clamp01(e.clientX / W), y: clamp01(e.clientY / H) };
    if (!isRegion) {
      setPoint(p);
      return;
    }
    const edge = px ? hitEdge(px, e.clientX, e.clientY) : null;
    if (region && edge === "inside") {
      drag.current = { type: "move", sx: e.clientX, sy: e.clientY, start: region };
    } else if (region && edge) {
      drag.current = { type: "resize", edge, sx: e.clientX, sy: e.clientY, start: region };
    } else {
      const start = { x: p.x, y: p.y, w: 0, h: 0 };
      setRegion(start);
      drag.current = { type: "draw", sx: e.clientX, sy: e.clientY, start };
    }
  };

  const onMove = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    setRegion(applyDrag(d, e.clientX, e.clientY, W, H));
  };

  const cursorFor = (e: React.MouseEvent) => {
    if (!isRegion || !px) return "crosshair";
    const edge = hitEdge(px, e.clientX, e.clientY);
    return edge === "inside" ? "move" : edge ? `${edge}-resize` : "crosshair";
  };

  return (
    <div
      className="w-full h-full relative select-none"
      onPointerDown={onDown}
      onPointerMove={onMove}
      onPointerUp={() => (drag.current = null)}
      onMouseMove={(e) => ((e.currentTarget as HTMLElement).style.cursor = cursorFor(e))}
      style={{ background: "rgba(5,7,12,0.42)" }}
    >
      {isRegion && px && (
        <>
          <div className="absolute inset-0 pointer-events-none" style={maskStyle(px, W, H)} />
          <Box r={px} color={target === "bar_region" ? COLORS.bar : COLORS.drop} handles />
        </>
      )}
      {!isRegion && point && <Cross x={point.x * W} y={point.y * H} />}
      <Toolbar
        title={TARGET_LABEL[target]}
        hint={isRegion ? "Drag to draw · edges to resize · Enter save · Esc cancel" : "Click to place · Enter save · Esc cancel"}
        onSave={commit}
        onCancel={cancel}
        canSave={isRegion ? !!region && region.w >= 0.005 : !!point}
      />
    </div>
  );
}

function applyDrag(d: Drag, cx0: number, cy0: number, W: number, H: number): RelRect {
  const dx = (cx0 - d.sx) / W;
  const dy = (cy0 - d.sy) / H;
  if (d.type === "move") {
    return { ...d.start, x: clamp01(d.start.x + dx, 1 - d.start.w), y: clamp01(d.start.y + dy, 1 - d.start.h) };
  }
  if (d.type === "draw") {
    return {
      x: clamp01(Math.min(d.start.x, d.start.x + dx)),
      y: clamp01(Math.min(d.start.y, d.start.y + dy)),
      w: Math.abs(dx),
      h: Math.abs(dy),
    };
  }
  let { x, y, w, h } = d.start;
  const edge = d.edge ?? "";
  if (edge.includes("w")) {
    x += dx;
    w -= dx;
  }
  if (edge.includes("e")) w += dx;
  if (edge.includes("n")) {
    y += dy;
    h -= dy;
  }
  if (edge.includes("s")) h += dy;
  return { x, y, w: Math.max(0.005, w), h: Math.max(0.005, h) };
}

function clamp01(v: number, max = 1) {
  return Math.max(0, Math.min(max, v));
}

function hitEdge(r: PxBox, x: number, y: number): string | null {
  const inX = x >= r.x - HANDLE && x <= r.x + r.w + HANDLE;
  const inY = y >= r.y - HANDLE && y <= r.y + r.h + HANDLE;
  if (!inX || !inY) return null;
  const v = Math.abs(y - r.y) <= HANDLE ? "n" : Math.abs(y - (r.y + r.h)) <= HANDLE ? "s" : "";
  const h = Math.abs(x - r.x) <= HANDLE ? "w" : Math.abs(x - (r.x + r.w)) <= HANDLE ? "e" : "";
  return v || h ? `${v}${h}` : "inside";
}

function maskStyle(r: PxBox, W: number, H: number): React.CSSProperties {
  const path = `polygon(0 0, ${W}px 0, ${W}px ${H}px, 0 ${H}px, 0 0, ${r.x}px ${r.y}px, ${r.x}px ${r.y + r.h}px, ${r.x + r.w}px ${r.y + r.h}px, ${r.x + r.w}px ${r.y}px, ${r.x}px ${r.y}px)`;
  return { background: "rgba(0,0,0,0.35)", clipPath: path };
}

function Box({
  r,
  color,
  label,
  handles,
  dim,
  badge,
}: {
  r: PxBox;
  color: string;
  label?: string;
  handles?: boolean;
  dim?: boolean;
  badge?: React.ReactNode;
}) {
  return (
    <div
      className="absolute pointer-events-none transition-opacity"
      style={{
        left: r.x,
        top: r.y,
        width: r.w,
        height: r.h,
        opacity: dim ? 0.55 : 1,
        boxShadow: `0 0 0 ${handles ? 2 : 1.5}px ${color}, 0 0 ${handles ? 22 : 12}px ${color}66`,
        background: `${color}0f`,
      }}
    >
      {label && (
        <div className="absolute -top-5 left-0 flex gap-1 items-center">
          <span className="text-[10px] font-mono px-1.5 rounded-sm whitespace-nowrap" style={{ background: color, color: "#000" }}>
            {label}
          </span>
          {badge}
        </div>
      )}
      {handles &&
        ["nw", "ne", "sw", "se"].map((k) => (
          <div
            key={k}
            className="absolute w-2.5 h-2.5 rounded-sm bg-white"
            style={{
              left: k.includes("w") ? -5 : undefined,
              right: k.includes("e") ? -5 : undefined,
              top: k.includes("n") ? -5 : undefined,
              bottom: k.includes("s") ? -5 : undefined,
              boxShadow: `0 0 0 1.5px ${color}`,
            }}
          />
        ))}
    </div>
  );
}

function Cross({ x, y }: { x: number; y: number }) {
  return (
    <div className="absolute pointer-events-none" style={{ left: x, top: y }}>
      <div className="absolute -left-4 -top-px w-8 h-0.5 bg-accent" />
      <div className="absolute -top-4 -left-px h-8 w-0.5 bg-accent" />
      <div className="absolute -left-3 -top-3 w-6 h-6 rounded-full border-2 border-accent" />
    </div>
  );
}

function LiveBar({ reading, origin }: { reading: Reading; origin: PxBox }) {
  const bx = origin.x + reading.origin_dx + reading.bar.x0;
  const by = origin.y + reading.bar.y0;
  const bw = reading.bar.x1 - reading.bar.x0 + 1;
  return (
    <>
      <div className="absolute bg-white/25 pointer-events-none" style={{ left: bx, top: by + reading.fish.start, width: bw, height: reading.fish.end - reading.fish.start + 1 }} />
      <div className={cx("absolute pointer-events-none", reading.hold ? "bg-ok/70" : "bg-accent/70")} style={{ left: bx, top: by + reading.marker.start, width: bw, height: Math.max(2, reading.marker.end - reading.marker.start + 1) }} />
    </>
  );
}

function Toolbar({
  title,
  hint,
  chips,
  onSave,
  onCancel,
  canSave,
}: {
  title: string;
  hint: string;
  chips?: { key: string; label: string; active: boolean; color: string; onClick: () => void }[];
  onSave: () => void;
  onCancel: () => void;
  canSave: boolean;
}) {
  return (
    <div
      className="absolute left-1/2 -translate-x-1/2 bottom-6 glass rounded-2xl px-4 py-3 flex flex-wrap items-center justify-center gap-x-4 gap-y-2 shadow-[0_10px_40px_rgba(0,0,0,0.5)] max-w-[min(92vw,900px)]"
      onPointerDown={(e) => e.stopPropagation()}
    >
      <div className="min-w-0 basis-full sm:basis-auto sm:flex-1 text-center sm:text-left">
        <div className="font-semibold text-[13px]">{title}</div>
        <div className="text-[11px] text-fg-dim leading-snug">{hint}</div>
      </div>
      {chips && (
        <div className="flex gap-1.5">
          {chips.map((c) => (
            <button
              key={c.key}
              onClick={c.onClick}
              className={cx("h-8 px-2.5 rounded-lg text-[12px] font-medium inline-flex items-center gap-1.5 border transition-colors whitespace-nowrap", c.active ? "bg-white/[0.12] border-white/20" : "bg-white/[0.04] border-line hover:bg-white/[0.08]")}
            >
              <kbd className="font-mono text-[10px] px-1 rounded bg-black/40 text-fg-dim">{c.key}</kbd>
              <span className="w-2 h-2 rounded-full" style={{ background: c.color }} />
              {c.label}
            </button>
          ))}
        </div>
      )}
      <div className="flex gap-2 shrink-0">
        <button onClick={onCancel} className="h-8 px-3 rounded-lg bg-white/[0.06] hover:bg-white/[0.12] text-[12px] whitespace-nowrap">
          Cancel <kbd className="font-mono text-[10px] text-fg-mute ml-1">Esc</kbd>
        </button>
        <button
          onClick={onSave}
          disabled={!canSave}
          className={cx("h-8 px-3 rounded-lg text-[12px] font-medium whitespace-nowrap", canSave ? "bg-accent text-white hover:brightness-110" : "bg-white/[0.08] text-fg-mute")}
        >
          Save <kbd className="font-mono text-[10px] opacity-70 ml-1">Enter</kbd>
        </button>
      </div>
    </div>
  );
}
