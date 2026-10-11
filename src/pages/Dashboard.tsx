import { useEffect, useState } from "react";
import { Pause, Play, Square } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore, isActive } from "../lib/store";
import { STATE_LABEL } from "../lib/types";
import { Button, Pill, Section, cx, fmtRuntime } from "../components/primitives";
import { StateBadge } from "../components/StateIcon";
import { LogList } from "../components/LogList";
import { LiveAreas } from "../components/LiveAreas";

export default function Dashboard() {
  const state = useStore((s) => s.state);
  const detail = useStore((s) => s.detail);
  const stats = useStore((s) => s.stats);
  const roblox = useStore((s) => s.roblox);
  const settings = useStore((s) => s.settings);
  const [runtime, setRuntime] = useState(stats.runtime_s);
  const active = isActive(state);
  const total = stats.total;
  const totalRuntime = total.runtime_s + (runtime - stats.runtime_s);

  useEffect(() => {
    setRuntime(stats.runtime_s);
    if (!active) return;
    const t = setInterval(() => setRuntime((r) => r + 1), 1000);
    return () => clearInterval(t);
  }, [stats.runtime_s, active]);

  const sessionNote = (n: number) => (active || state === "paused" ? `this run ${n}` : undefined);

  const hk = settings?.hotkeys.toggle ?? "F1";

  return (
    <div className="pb-4">
      <div className="px-4 pt-4 pb-3 flex items-center gap-3">
        <StateBadge state={state} />
        <div className="flex-1 min-w-0">
          <div className="text-[15px] font-semibold leading-tight">{STATE_LABEL[state]}</div>
          <div className="text-[12px] text-fg-dim truncate">
            {detail ?? (roblox ? (roblox.is_foreground ? "Roblox focused" : "Roblox in background") : "Roblox not detected")}
          </div>
        </div>
        <div className="flex gap-1.5">
          <Button kind={active ? "default" : "primary"} onClick={() => api.botToggle()} icon={active ? <Pause size={14} /> : <Play size={14} />}>
            {active ? "Pause" : state === "paused" ? "Resume" : "Start"}
          </Button>
          {(active || state === "paused") && <Button kind="danger" onClick={() => api.botStop()} icon={<Square size={13} />} />}
        </div>
      </div>
      <div className="px-4 pb-3 text-[11px] text-fg-mute">
        Press <span className="font-mono text-fg-dim">{hk}</span> anywhere to start or pause.
      </div>

      <Section title="All time">
        <Stat label="Fish caught" value={total.fish} sub={sessionNote(stats.fish)} />
        <Stat label="Failed reels" value={total.failed} sub={sessionNote(stats.failed)} />
        <Stat
          label="Devil fruits"
          value={total.fruits}
          sub={sessionNote(stats.fruits)}
          extra={total.last_fruit ? <Pill tone="fruit">{total.last_fruit.slice(0, 40)}</Pill> : undefined}
        />
        <Stat label="Bait bought" value={total.bait_purchased} sub={sessionNote(stats.bait_purchased)} />
        <Stat label="Time fishing" value={fmtRuntime(totalRuntime)} sub={active || state === "paused" ? `this run ${fmtRuntime(runtime)}` : undefined} />
        <Stat label="Sessions" value={total.sessions} />
        <Stat label="Bite rate" value={`${Math.round(stats.success_rate * 100)}%`} extra={<Bar pct={stats.success_rate} />} />
        {stats.restarts > 0 && <Stat label="Recoveries" value={stats.restarts} />}
        {total.last_spawn && <Stat label="Last spawn" value={total.last_spawn} />}
      </Section>

      <Section title="Live">
        <LiveAreas />
      </Section>

      <Section title="Activity">
        <LogList height={220} />
      </Section>
    </div>
  );
}

function Stat({ label, value, extra, sub }: { label: string; value: React.ReactNode; extra?: React.ReactNode; sub?: string }) {
  return (
    <div className="flex items-center px-4 h-11 border-b border-line">
      <div className="text-fg-dim">
        {label}
        {sub && <span className="ml-2 text-[11px] text-fg-mute font-mono">{sub}</span>}
      </div>
      <div className="ml-auto flex items-center gap-3">
        {extra}
        <div className="font-mono tabular-nums text-[14px] font-medium">{value}</div>
      </div>
    </div>
  );
}

function Bar({ pct }: { pct: number }) {
  return (
    <div className="w-24 h-1.5 rounded-full bg-white/10 overflow-hidden">
      <div className={cx("h-full rounded-full", pct > 0.7 ? "bg-ok" : pct > 0.4 ? "bg-warn" : "bg-bad")} style={{ width: `${pct * 100}%` }} />
    </div>
  );
}
