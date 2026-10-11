import { Fish, Hourglass, MonitorOff, Package, Pause, Power, RefreshCw, Send, Settings2, ShoppingCart, Sparkles } from "lucide-react";
import type { BotState } from "../lib/types";
import { cx } from "./primitives";

export type StateTone = "ok" | "warn" | "mute" | "fruit";

const MAP: Record<BotState, { Icon: typeof Fish; tone: StateTone; spin?: boolean }> = {
  stopped: { Icon: Power, tone: "mute" },
  waiting_for_roblox: { Icon: MonitorOff, tone: "warn" },
  initial_setup: { Icon: Settings2, tone: "ok", spin: true },
  casting: { Icon: Send, tone: "ok" },
  waiting_for_bite: { Icon: Hourglass, tone: "ok" },
  tracking: { Icon: Fish, tone: "ok" },
  post_catch: { Icon: Sparkles, tone: "ok" },
  purchasing: { Icon: ShoppingCart, tone: "ok" },
  storing_fruit: { Icon: Package, tone: "fruit" },
  recovering: { Icon: RefreshCw, tone: "warn", spin: true },
  paused: { Icon: Pause, tone: "warn" },
};

const TEXT: Record<StateTone, string> = { ok: "text-ok", warn: "text-warn", mute: "text-fg-mute", fruit: "text-fruit" };
const BG: Record<StateTone, string> = { ok: "bg-ok-soft", warn: "bg-warn-soft", mute: "bg-white/[0.06]", fruit: "bg-fruit/15" };

export function stateTone(state: BotState): StateTone {
  return MAP[state].tone;
}

export function StateIcon({ state, size = 16, className }: { state: BotState; size?: number; className?: string }) {
  const { Icon, tone, spin } = MAP[state];
  return <Icon size={size} className={cx(TEXT[tone], spin && "animate-spin [animation-duration:2.4s]", className)} />;
}

export function StateBadge({ state, size = 44, icon = 20, className }: { state: BotState; size?: number; icon?: number; className?: string }) {
  const tone = stateTone(state);
  return (
    <div className={cx("rounded-full grid place-items-center shrink-0", BG[tone], className)} style={{ width: size, height: size }}>
      <StateIcon state={state} size={icon} />
    </div>
  );
}
