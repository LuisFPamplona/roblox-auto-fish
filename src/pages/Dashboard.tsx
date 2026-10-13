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
    <div className="pb-4 dashboard">
      <div className="session-card flex items-center gap-3">
        <StateBadge state={state} />
        <div className="flex-1 min-w-0">
          <div className="text-[15px] font-semibold leading-tight">{STATE_LABEL[state]}</div>
          <div className="text-[12px] text-fg-dim truncate">
            {detail ?? (roblox ? (roblox.is_foreground ? "Roblox focused" : "Roblox in background") : "Roblox not detected")}
          </div>
        </div>
        <div className="flex gap-1.5">
          <Button kind={active ? "default" : "primary"} onClick={() => api.botToggle()} icon={active ? <Pause size={14} /> : <Play size={14} />}>
            {active ? "Pausar" : state === "paused" ? "Retomar" : "Iniciar"}
          </Button>
          {(active || state === "paused") && <Button kind="danger" onClick={() => api.botStop()} icon={<Square size={13} />} />}
        </div>
      </div>
      <div className="px-4 pb-3 text-[11px] text-fg-mute">
        Use <span className="font-mono text-fg-dim">{hk}</span> para iniciar ou pausar.
      </div>

      <Section title="Seu histórico">
        <div className="stats-grid">
        <Stat label="Peixes registrados" value={total.fish} sub={sessionNote(stats.fish)} />
        <Stat label="Tentativas falhas" value={total.failed} sub={sessionNote(stats.failed)} />
        <Stat
          label="Frutas"
          value={total.fruits}
          sub={sessionNote(stats.fruits)}
          extra={total.last_fruit ? <Pill tone="fruit">{total.last_fruit.slice(0, 40)}</Pill> : undefined}
        />
        <Stat label="Iscas compradas" value={total.bait_purchased} sub={sessionNote(stats.bait_purchased)} />
        <Stat label="Tempo de pesca" value={fmtRuntime(totalRuntime)} sub={active || state === "paused" ? `this run ${fmtRuntime(runtime)}` : undefined} />
        <Stat label="Sessões" value={total.sessions} />
        <Stat label="Taxa de mordidas" value={`${Math.round(stats.success_rate * 100)}%`} extra={<Bar pct={stats.success_rate} />} />
        {stats.restarts > 0 && <Stat label="Recuperações" value={stats.restarts} />}
        {total.last_spawn && <Stat label="Última aparição" value={total.last_spawn} />}
        </div>
      </Section>

      <Section title="Captura ao vivo">
        <LiveAreas />
      </Section>

      <Section title="Atividade">
        <LogList height={220} />
      </Section>
    </div>
  );
}

function Stat({ label, value, extra, sub }: { label: string; value: React.ReactNode; extra?: React.ReactNode; sub?: string }) {
  return (
    <div className="stat-card">
      <div className="text-fg-dim">
        {label}
        {sub && <span className="ml-2 text-[11px] text-fg-mute font-mono">{sub}</span>}
      </div>
      <div className="stat-value flex items-center gap-2 flex-wrap">
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
