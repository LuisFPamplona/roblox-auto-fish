import { useEffect, useState } from "react";
import { ArrowLeft, ArrowRight, Check, X } from "lucide-react";
import { api, on } from "../lib/ipc";
import { useStore } from "../lib/store";
import { Button, cx } from "../components/primitives";
import { GUIDE_STEPS } from "../components/SetupGuide";
import logo from "../assets/logo.png";

const close = () => api.guideHide();

export default function Guide() {
  const init = useStore((s) => s.init);
  const s = useStore((st) => st.settings);
  const [i, setI] = useState(0);

  useEffect(() => {
    init();
    const un = on("guide:open", () => setI(0));
    return () => {
      un.then((u) => u());
    };
  }, [init]);

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight") setI((n) => Math.min(GUIDE_STEPS.length - 1, n + 1));
      if (e.key === "ArrowLeft") setI((n) => Math.max(0, n - 1));
      if (e.key === "Escape") close();
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, []);

  const step = GUIDE_STEPS[i];
  const last = i === GUIDE_STEPS.length - 1;

  return (
    <div className="h-full w-full p-1.5">
      <div className="glass rounded-2xl h-full w-full flex flex-col overflow-hidden shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
        <header className="h-11 flex items-center gap-2.5 px-4 border-b border-line drag shrink-0">
          <img src={logo} alt="" draggable={false} className="w-6 h-6 rounded-md" />
          <div className="font-semibold">How to set up</div>
          <div className="ml-auto flex items-center gap-1 no-drag">
            {GUIDE_STEPS.map((st, n) => (
              <button
                key={st.title}
                onClick={() => setI(n)}
                title={st.title}
                className={cx(
                  "h-6 min-w-6 px-1.5 rounded-md grid place-items-center text-[11px] font-mono transition-colors",
                  n === i ? "bg-accent text-white" : n < i ? "bg-accent-soft text-accent" : "bg-white/[0.06] text-fg-mute hover:text-fg",
                )}
              >
                {n + 1}
              </button>
            ))}
            <button className="ml-2 w-8 h-8 rounded-lg grid place-items-center text-fg-mute hover:bg-bad-soft hover:text-bad" onClick={close}>
              <X size={15} />
            </button>
          </div>
        </header>

        <main className="flex-1 min-h-0 flex flex-col px-6 pt-4 pb-3">
          {s ? (
            <div key={i} className="rise flex-1 min-h-0 flex flex-col">
              <div className="text-[11px] uppercase tracking-[0.12em] text-fg-mute font-semibold">
                Step {i + 1} of {GUIDE_STEPS.length}
              </div>
              <h1 className="text-[18px] font-semibold mt-1 leading-tight">{step.title}</h1>
              <p className="text-fg-dim text-[12.5px] leading-relaxed mt-1.5 mb-3">{step.body(s)}</p>
              <div className="mt-auto">{step.scene(s)}</div>
            </div>
          ) : (
            <div className="flex-1 grid place-items-center text-fg-dim">Loading…</div>
          )}
        </main>

        <footer className="h-14 flex items-center px-6 border-t border-line shrink-0 gap-2">
          <Button kind="ghost" size="sm" disabled={i === 0} onClick={() => setI(i - 1)} icon={<ArrowLeft size={14} />}>
            Back
          </Button>
          <div className="ml-auto" />
          {last ? (
            <Button kind="primary" size="sm" onClick={close} icon={<Check size={14} />}>
              Finish
            </Button>
          ) : (
            <Button kind="primary" size="sm" onClick={() => setI(i + 1)} icon={<ArrowRight size={14} />}>
              Next
            </Button>
          )}
        </footer>
      </div>
    </div>
  );
}
