import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Gauge, ListChecks, Minus, Settings2, SlidersHorizontal, X } from "lucide-react";
import { api, on } from "../lib/ipc";
import { useStore } from "../lib/store";
import { cx, Dot } from "../components/primitives";
import { ConnectGate } from "../components/ConnectGate";
import Dashboard from "../pages/Dashboard";
import Setup from "../pages/Setup";
import Features from "../pages/Features";
import SettingsPage from "../pages/Settings";

type Tab = "dashboard" | "setup" | "features" | "settings";

const TABS: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: "dashboard", label: "Visão geral", icon: <Gauge size={14} /> },
  { id: "setup", label: "Calibrar", icon: <ListChecks size={14} /> },
  { id: "features", label: "Recursos", icon: <SlidersHorizontal size={14} /> },
  { id: "settings", label: "Ajustes", icon: <Settings2 size={14} /> },
];

export default function Panel() {
  const init = useStore((s) => s.init);
  const ready = useStore((s) => s.ready);
  const error = useStore((s) => s.error);
  const roblox = useStore((s) => s.roblox);
  const version = useStore((s) => s.version);
  const [tab, setTab] = useState<Tab>("dashboard");
  const [skipGate, setSkipGate] = useState(false);
  const [appear, setAppear] = useState(0);
  const visibleRef = useRef(true);

  useEffect(() => {
    const un = on("ui:visibility", (v) => {
      if (v && !visibleRef.current) setAppear((n) => n + 1);
      visibleRef.current = v;
    });
    return () => {
      un.then((u) => u());
    };
  }, []);

  useEffect(() => {
    init().finally(() => api.panelShow());
  }, [init]);

  useEffect(() => {
    if (roblox) setSkipGate(false);
  }, [roblox]);

  useEffect(() => {
    let t: number | undefined;
    const schedule = () => {
      window.clearTimeout(t);
      t = window.setTimeout(() => api.panelPlacementChanged(), 400);
    };
    const w = getCurrentWindow();
    const subs = [w.onResized(schedule), w.onMoved(schedule)];
    return () => {
      subs.forEach((p) => p.then((u) => u()));
    };
  }, []);

  const gated = ready && !roblox && !skipGate;

  return (
    <div className="h-full w-full p-1.5">
      <div key={appear} className="glass panel-shell h-full w-full flex overflow-hidden rise">
        <div className="flex-1 min-w-0 flex flex-col">
          <header className="flex items-center px-4 py-3 border-b border-line drag shrink-0">
            <div>
              <div className="brand-name">AutoFish <span>Local</span></div>
              <div className="flex items-center gap-1.5 text-[10px] text-fg-dim mt-1">
                <Dot tone={roblox ? (roblox.is_foreground ? "ok" : "accent") : "mute"} />
                <span>{roblox ? "Roblox conectado" : "Aguardando Roblox"}</span>
                <span className="ml-1 font-mono" title={version}>v{version.split("-")[0]}</span>
              </div>
            </div>
            <div className="ml-auto flex items-center gap-1 no-drag">
              <button aria-label="Minimizar" className="w-8 h-8 rounded-lg grid place-items-center text-fg-mute hover:bg-white/[0.06] hover:text-fg" onClick={() => getCurrentWindow().minimize()}>
                <Minus size={15} />
              </button>
              <button aria-label="Ocultar painel" className="w-8 h-8 rounded-lg grid place-items-center text-fg-mute hover:bg-bad-soft hover:text-bad" onClick={() => api.panelHide()}>
                <X size={15} />
              </button>
            </div>
          </header>
          <nav aria-label="Navegação principal" className={cx("panel-tabs shrink-0", gated && "opacity-40 pointer-events-none")}>
            {TABS.map((t) => (
              <button key={t.id} onClick={() => setTab(t.id)} aria-current={tab === t.id ? "page" : undefined} className={cx("panel-tab", tab === t.id && "selected")}>
                {t.icon}<span>{t.label}</span>
              </button>
            ))}
          </nav>
          <main className="flex-1 overflow-y-auto">
            {ready && error && <div role="alert" className="m-3 p-2 rounded-lg bg-bad-soft text-bad text-[12px] break-words">{error}</div>}
            {!ready && <Booting error={error} />}
            {gated && <ConnectGate onSkip={() => setSkipGate(true)} />}
            {ready && !gated && (
              <>
                {tab === "dashboard" && <Dashboard />}
                {tab === "setup" && <Setup />}
                {tab === "features" && <Features />}
                {tab === "settings" && <SettingsPage />}
              </>
            )}
          </main>
        </div>
      </div>
    </div>
  );
}

function Booting({ error }: { error: string | null }) {
  return (
    <div className="h-full flex flex-col items-center justify-center text-center px-8">
      <div className="w-8 h-8 rounded-full border-2 border-white/10 border-t-accent animate-spin mb-4" />
      <div className="text-fg-dim">Starting…</div>
      {error && <div className="mt-3 text-[11px] text-warn font-mono break-all select-text max-w-[320px]">{error}</div>}
    </div>
  );
}
