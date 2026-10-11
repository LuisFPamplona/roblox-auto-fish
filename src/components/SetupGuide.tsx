import type { ReactNode } from "react";
import { Apple, ArrowRight, Crosshair, Fish, Pause, Play, ShoppingCart } from "lucide-react";
import type { Settings } from "../lib/types";
import { Kbd, Toggle, cx } from "./primitives";

const SLOT_KEYS = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

export type GuideStep = { title: string; body: (s: Settings) => ReactNode; scene: (s: Settings) => ReactNode };

function Frame({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cx("relative w-full h-[230px] rounded-xl bg-black/40 border border-line-strong overflow-hidden", className)}>{children}</div>;
}

function Label({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cx("text-[10.5px] uppercase tracking-[0.1em] text-fg-mute font-semibold", className)}>{children}</div>;
}

function Callout({ children, className, tone = "accent" }: { children: ReactNode; className?: string; tone?: "accent" | "ok" | "fruit" }) {
  const t = { accent: "bg-accent-soft text-accent", ok: "bg-ok-soft text-ok", fruit: "bg-fruit/15 text-fruit" }[tone];
  return <div className={cx("absolute inline-flex items-center h-6 px-2 rounded-md text-[11px] font-medium whitespace-nowrap", t, className)}>{children}</div>;
}

function RodIcon({ size = 22 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 20 18 4" />
      <path d="M18 4c1 3 .5 6-1.5 8.5" />
      <path d="M16.5 12.5v3.2" />
      <circle cx={16.5} cy={16.9} r={1.3} />
    </svg>
  );
}

function Junk({ i }: { i: number }) {
  const shapes = [
    <path key="a" d="M5 9h14l-1.5 11h-11z M8 9V6h8v3" />,
    <path key="b" d="M12 3l3 6 6 1-4.5 4 1 6-5.5-3-5.5 3 1-6L3 10l6-1z" />,
    <path key="c" d="M4 6h16v12H4z M4 10h16" />,
    <path key="d" d="M7 3h10l2 5H5z M6 8h12v13H6z" />,
  ];
  return (
    <svg width={18} height={18} viewBox="0 0 24 24" fill="currentColor" opacity={0.55}>
      {shapes[i % shapes.length]}
    </svg>
  );
}

function Hotbar({ s, items, size = 36 }: { s: Settings; items: Array<"rod" | "junk" | null>; size?: number }) {
  const fruits = [s.keys.fruit_slot_1, s.keys.fruit_slot_2];
  return (
    <div className="inline-flex gap-1 p-1.5 rounded-lg bg-black/50 border border-line-strong">
      {SLOT_KEYS.map((k, i) => {
        const v = items[i];
        const isRod = k === s.keys.rod;
        const isFruit = fruits.includes(k);
        return (
          <div
            key={k}
            style={{ width: size, height: size }}
            className={cx(
              "relative rounded-md grid place-items-center border",
              v === "rod" && "bg-accent-soft text-accent border-accent",
              v === "junk" && "text-fg-dim bg-white/[0.05] border-line",
              v == null && isRod && "border-dashed border-accent/60",
              v == null && isFruit && "border-dashed border-fruit/50",
              v == null && !isRod && !isFruit && "border-line bg-white/[0.02]",
            )}
          >
            <span className="absolute top-0.5 left-1 text-[8.5px] font-mono text-fg-mute leading-none">{k}</span>
            {v === "rod" && <RodIcon size={size * 0.55} />}
            {v === "junk" && <Junk i={i} />}
            {v == null && isFruit && <Apple size={size * 0.36} className="text-fruit/45" />}
          </div>
        );
      })}
    </div>
  );
}

function HotbarScene(s: Settings) {
  const before: Array<"rod" | "junk" | null> = SLOT_KEYS.map((k, i) => (k === s.keys.rod ? "junk" : i === 3 ? "rod" : [1, 5, 7].includes(i) ? "junk" : null));
  const after: Array<"rod" | "junk" | null> = SLOT_KEYS.map((k) => (k === s.keys.rod ? "rod" : null));
  const fruits = [s.keys.fruit_slot_1, s.keys.fruit_slot_2];
  return (
    <Frame className="p-4 flex flex-col justify-center gap-4">
      <div>
        <Label className="mb-1.5">Before · what most inventories look like</Label>
        <Hotbar s={s} items={before} />
      </div>
      <div className="flex items-center gap-2 text-fg-mute text-[11px]">
        <ArrowRight size={14} /> drop everything, move the rod
      </div>
      <div>
        <Label className="mb-1.5">After · what the bot needs</Label>
        <div className="flex items-center gap-4">
          <Hotbar s={s} items={after} />
          <div className="text-[11px] text-fg-dim leading-snug">
            <div className="flex items-center gap-1.5"><span className="w-2.5 h-2.5 rounded-sm bg-accent" />Rod in <Kbd>{s.keys.rod.toUpperCase()}</Kbd></div>
            <div className="flex items-center gap-1.5 mt-1"><Apple size={11} className="text-fruit/70" />Slots <Kbd>{fruits[0].toUpperCase()}</Kbd><Kbd>{fruits[1].toUpperCase()}</Kbd> empty for fruits</div>
            <div className="flex items-center gap-1.5 mt-1"><span className="w-2.5 h-2.5 rounded-sm border border-line-strong" />Everything else empty</div>
          </div>
        </div>
      </div>
    </Frame>
  );
}

function SpotScene(s: Settings) {
  return (
    <Frame>
      <svg className="absolute inset-0 w-full h-full" viewBox="0 0 420 230" fill="none">
        <defs>
          <linearGradient id="g-water" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="#1e40af" stopOpacity="0.6" />
            <stop offset="1" stopColor="#0b0d12" stopOpacity="0.95" />
          </linearGradient>
        </defs>
        <rect x="0" y="140" width="420" height="90" fill="url(#g-water)" />
        <g stroke="#60a5fa" strokeOpacity="0.35" strokeWidth="1.5" strokeLinecap="round">
          <path d="M150 154q16-5 32 0t32 0 32 0 32 0 32 0 32 0 32 0 32 0" />
          <path d="M130 178q16-5 32 0t32 0 32 0 32 0 32 0 32 0 32 0 32 0 32 0" />
        </g>
        <path d="M0 134h160v12H0z" fill="#3f3f46" />
        <g fill="#cbd5e1">
          <circle cx="96" cy="92" r="9" />
          <rect x="88" y="102" width="16" height="32" rx="4" />
        </g>
        <path d="M104 112 Q 118 96 134 72" stroke="#e8eaf0" strokeWidth="2.5" strokeLinecap="round" />
        <path d="M134 72 Q 200 20 262 160" stroke="#e8eaf0" strokeWidth="1.2" />
        <circle cx="262" cy="160" r="4" fill="#ef4444" />
        <g transform="translate(22 96)">
          <ellipse cx="18" cy="38" rx="18" ry="6" fill="#2a1a10" />
          <path d="M0 10v26q18 8 36 0V10z" fill="#4a2e1c" />
          <path d="M0 18q18 6 36 0M0 28q18 6 36 0" stroke="#2a1a10" strokeWidth="2" fill="none" />
          <ellipse cx="18" cy="10" rx="18" ry="6" fill="#6b4630" />
          <path d="M10 8c3-4 8-3 10 0s6 1 8-2" stroke="#c46a8a" strokeWidth="2.5" strokeLinecap="round" fill="none" />
          <path d="M8 12c4 1 9-1 12 1s7 1 9-1" stroke="#a8556f" strokeWidth="2" strokeLinecap="round" fill="none" />
        </g>
        <rect x="14" y="70" width="52" height="14" rx="4" fill="#0b0d12" fillOpacity="0.9" stroke="#f59e0b" strokeOpacity="0.6" />
        <text x="40" y="80" textAnchor="middle" fontSize="8" fill="#f59e0b" fontFamily="Inter, system-ui" fontWeight="600">
          Hold {s.keys.shop.toUpperCase()} to buy
        </text>
        <rect x="350" y="26" width="22" height="130" rx="4" fill="#1d4ed8" fillOpacity="0.85" />
        <rect x="352" y="60" width="18" height="34" rx="2" fill="#0b0d12" />
        <rect x="352" y="110" width="18" height="3" fill="#fff" />
      </svg>
      <Callout className="left-[62px] top-[36px]">you, facing the water</Callout>
      <Callout className="left-3 bottom-3" tone="ok">bait barrel on the dock, needed for auto buy</Callout>
      <Callout className="right-3 top-[164px]">the bar the bot tracks</Callout>
    </Frame>
  );
}

function BarScene() {
  return (
    <Frame>
      <svg className="absolute inset-0 w-full h-full" viewBox="0 0 420 230" fill="none">
        <rect x="0" y="0" width="420" height="230" fill="#0f172a" fillOpacity="0.5" />
        <rect x="300" y="30" width="22" height="160" rx="4" fill="#1d4ed8" fillOpacity="0.9" />
        <rect x="302" y="70" width="18" height="34" rx="2" fill="#0b0d12" />
        <rect x="302" y="130" width="18" height="3" fill="#fff" />
        <rect x="288" y="20" width="46" height="180" rx="5" fill="#22c55e" fillOpacity="0.1" stroke="#22c55e" strokeWidth="1.5" />
        <rect x="40" y="40" width="112" height="30" rx="8" fill="#4f8cff" />
        <text x="96" y="59" textAnchor="middle" fontSize="12" fontFamily="Inter, system-ui" fontWeight="600" fill="#fff">
          Auto-detect
        </text>
        <path d="M152 55 H 270" stroke="#4f8cff" strokeWidth="1.2" strokeDasharray="4 4" />
        <path d="M264 50l8 5-8 5" stroke="#4f8cff" strokeWidth="1.2" fill="none" />
      </svg>
      <Callout className="left-[40px] top-[84px]" tone="ok">Bar matched 96%</Callout>
      <Callout className="left-[40px] bottom-3">tight box, just around the blue bar</Callout>
    </Frame>
  );
}

function DropScene() {
  return (
    <Frame>
      <svg className="absolute inset-0 w-full h-full" viewBox="0 0 420 230" fill="none">
        <defs>
          <linearGradient id="g-sky" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="#1e3a5f" />
            <stop offset="1" stopColor="#0b0d12" />
          </linearGradient>
        </defs>
        <rect x="0" y="0" width="420" height="230" fill="url(#g-sky)" />
        <rect x="0" y="166" width="420" height="64" fill="#1e40af" fillOpacity="0.35" />
        <g fill="#cbd5e1">
          <circle cx="210" cy="130" r="7" />
          <rect x="204" y="138" width="12" height="24" rx="3" />
        </g>
        {SLOT_KEYS.slice(0, 7).map((k, i) => (
          <rect key={k} x={140 + i * 21} y="204" width="18" height="18" rx="3" fill="#ffffff" fillOpacity={i === 0 ? 0.22 : 0.08} />
        ))}
        <rect x="118" y="16" width="184" height="34" rx="8" fill="#0b0d12" fillOpacity="0.92" stroke="#a78bfa" strokeOpacity="0.5" />
        <circle cx="136" cy="33" r="7" fill="#a78bfa" />
        <text x="152" y="37" fontSize="11" fontFamily="Inter, system-ui" fontWeight="600" fill="#e8eaf0">
          New Item {"<Buddha>"}
        </text>
        <rect x="108" y="8" width="204" height="50" rx="6" fill="#a78bfa" fillOpacity="0.1" stroke="#a78bfa" strokeWidth="1.5" strokeDasharray="5 4" />
      </svg>
      <Callout className="left-1/2 -translate-x-1/2 top-[66px]" tone="fruit">draw the area over this popup</Callout>
      <Callout className="left-3 bottom-3">top middle of the screen, shows after a fruit catch</Callout>
    </Frame>
  );
}

function FeaturesScene() {
  const rows = [
    { icon: <Fish size={13} />, label: "Auto bait" },
    { icon: <ShoppingCart size={13} />, label: "Auto buy bait" },
    { icon: <Apple size={13} />, label: "Store fruits" },
  ];
  return (
    <Frame className="p-3 flex gap-3">
      <div className="w-[200px] shrink-0 flex flex-col gap-1.5">
        <Label className="mb-0.5">Features page</Label>
        {rows.map((r) => (
          <div key={r.label} className="flex items-center h-9 px-3 rounded-lg bg-white/[0.04] border border-line">
            <span className="text-fg-dim mr-2">{r.icon}</span>
            <span className="text-[12px]">{r.label}</span>
            <span className="ml-auto pointer-events-none">
              <Toggle value onChange={() => undefined} />
            </span>
          </div>
        ))}
        <div className="mt-1 flex items-center gap-2">
          <span className="text-[11px] text-fg-dim">Confirm button</span>
          <span className="inline-flex items-center gap-1.5 h-8 px-3 rounded-lg bg-accent text-white text-[12px] font-medium">
            <Crosshair size={12} /> Pick
          </span>
        </div>
      </div>
      <div className="relative flex-1 rounded-lg bg-[#0f172a]/70 border border-line grid place-items-center overflow-hidden">
        <Label className="absolute top-2 left-2">Roblox · bait shop</Label>
        <div className="flex flex-col items-center gap-2">
          <div className="h-7 w-24 rounded bg-white/10 grid place-items-center text-[11px] text-fg-dim">90</div>
          <div className="flex gap-2">
            <div className="relative h-8 px-4 rounded-md bg-emerald-600 text-white text-[11px] font-semibold grid place-items-center ring-2 ring-accent">
              Confirm
              <Crosshair size={22} className="absolute -right-3 -top-3 text-accent" />
            </div>
            <div className="h-8 px-4 rounded-md bg-zinc-600 text-white text-[11px] font-semibold grid place-items-center">Cancel</div>
          </div>
        </div>
        <Callout className="left-1/2 -translate-x-1/2 bottom-2">click the exact button in game</Callout>
      </div>
    </Frame>
  );
}

function StartScene(s: Settings) {
  return (
    <Frame className="grid place-items-center">
      <div className="flex flex-col items-center gap-5">
        <div className="flex items-center gap-4">
          <div className="w-16 h-16 rounded-xl bg-white/[0.08] border border-line-strong shadow-[0_5px_0_rgba(255,255,255,0.08)] grid place-items-center font-mono text-[18px] font-semibold">
            {s.hotkeys.toggle}
          </div>
          <div className="text-[11px] text-fg-dim leading-snug">
            <div>press once to start</div>
            <div>press again to pause</div>
          </div>
        </div>
        <div className="relative w-[320px] h-[44px] rounded-full glass flex items-center gap-2 pl-3 pr-1.5">
          <div className="w-7 h-7 rounded-full bg-ok-soft text-ok grid place-items-center">
            <Fish size={14} />
          </div>
          <div className="min-w-0 flex-1 leading-tight">
            <div className="text-[12px] font-semibold">Tracking</div>
            <div className="text-[10.5px] text-fg-dim font-mono">12 fish · 00:14:05</div>
          </div>
          <div className="h-8 w-8 rounded-full bg-warn-soft text-warn grid place-items-center">
            <Pause size={13} />
          </div>
          <div className="h-8 w-8 rounded-full bg-white/[0.06] text-fg-dim grid place-items-center">
            <Play size={13} />
          </div>
        </div>
        <div className="text-[11px] text-fg-dim">The HUD sits on top of Roblox and shows what the bot is doing.</div>
      </div>
    </Frame>
  );
}

export const GUIDE_STEPS: GuideStep[] = [
  {
    title: "Prepare your hotbar",
    body: (s) => (
      <>
        The bot switches slots by number, so the rod must be in slot <Kbd>{s.keys.rod.toUpperCase()}</Kbd> and every other slot empty. Two slots stay free for fruits.
      </>
    ),
    scene: HotbarScene,
  },
  {
    title: "Stand at your fishing spot",
    body: () => "Face the water and keep Roblox on screen, windowed or borderless. If you use auto buy, stand next to the bait barrel on the dock so the shop key opens it.",
    scene: SpotScene,
  },
  {
    title: "Mark the fishing bar",
    body: () => (
      <>
        Cast once by hand so the bar is on screen, then open <b>Fishing bar area</b> in Setup and press <b>Auto-detect</b>. You can also draw the box yourself, keep it tight.
      </>
    ),
    scene: BarScene,
  },
  {
    title: "Mark the drop message",
    body: () => (
      <>
        Only needed for fruit features. GPO shows a popup at the top middle of the screen: "New Item &lt;Fruit&gt;" after a catch and "A Fruit has spawned at Place" for world spawns. Draw the <b>Drop message area</b> over that spot so the OCR can read it.
      </>
    ),
    scene: DropScene,
  },
  {
    title: "Turn on features",
    body: () => "Each feature on the Features page lists its own steps. Buttons in the game are picked with a crosshair: press Pick, then click the exact button in Roblox.",
    scene: FeaturesScene,
  },
  {
    title: "Start fishing",
    body: (s) => (
      <>
        <Kbd>{s.hotkeys.toggle}</Kbd> starts and pauses from anywhere. <Kbd>{s.hotkeys.overlay}</Kbd> reopens the area editor, <Kbd>{s.hotkeys.hide_hud}</Kbd> hides the HUD, <Kbd>{s.hotkeys.quit}</Kbd> quits.
      </>
    ),
    scene: StartScene,
  },
];
