import { useState } from "react";
import { BookOpen, Check, Circle, PencilRuler, ScanText } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import type { OcrTest } from "../lib/types";
import { Button, Kbd, KeyCapture, Pill, Row, Section, Segmented } from "../components/primitives";
import { RegionField } from "../components/RegionField";
import { PointField } from "../components/PointField";

export default function Setup() {
  const s = useStore((st) => st.settings);
  const roblox = useStore((st) => st.roblox);
  const ocrAvailable = useStore((st) => st.ocrAvailable);
  const update = useStore((st) => st.update);
  const [open, setOpen] = useState<string | null>(null);
  const [ocr, setOcr] = useState<OcrTest | null>(null);
  const [ocrErr, setOcrErr] = useState<string | null>(null);
  if (!s) return null;

  const toggle = (k: string) => setOpen((o) => (o === k ? null : k));

  const runOcr = async () => {
    try {
      setOcr(await api.ocrTest());
      setOcrErr(null);
    } catch (e) {
      setOcrErr(String(e));
    }
  };

  return (
    <div className="pb-4 pt-2">
      <Section title="Getting started">
        <Row
          title={<span className="inline-flex items-center gap-2"><BookOpen size={14} className="text-accent" />How to set up</span>}
          sub="Six animated steps from an empty hotbar to your first catch."
          right={
            <Button size="sm" kind="primary" onClick={() => api.guideOpen()}>
              Open guide
            </Button>
          }
        />
      </Section>

      <Section title="Required">
        <Row
          title={<Step done={!!roblox} label="Roblox window" />}
          sub={roblox ? `${roblox.client.w} × ${roblox.client.h} · ${roblox.is_foreground ? "focused" : "background"}` : "Open Roblox and join GPO"}
          right={<Pill tone={roblox ? "ok" : "warn"}>{roblox ? "detected" : "not found"}</Pill>}
        />
        <Row
          title={<span className="inline-flex items-center gap-2"><PencilRuler size={14} className="text-accent" />Edit areas on screen</span>}
          sub={<>Drag both areas directly over the game. Press <Kbd>{s.hotkeys.overlay}</Kbd> any time, <Kbd>Tab</Kbd> switches, <Kbd>Enter</Kbd> saves.</>}
          right={
            <Button size="sm" kind="primary" disabled={!roblox} onClick={() => api.overlayOpenRegions()}>
              Open editor
            </Button>
          }
        />
        <Row
          title={<Step done label="Fishing bar area" />}
          sub="Where the blue minigame bar appears. Cast once, then use Auto-detect."
          open={open === "bar"}
          onToggle={() => toggle("bar")}
        >
          <RegionField target="bar_region" value={s.regions.bar} />
        </Row>
        <Row
          title={<Step done label="Cast point" />}
          sub="Where the rod is thrown. Leave on auto to use the screen center."
          open={open === "cast"}
          onToggle={() => toggle("cast")}
          right={
            <Pill tone={s.features.auto_mouse_position ? "accent" : "mute"}>{s.features.auto_mouse_position ? "auto" : "custom"}</Pill>
          }
        >
          <div className="flex items-center gap-3 flex-wrap">
            <Segmented
              value={s.features.auto_mouse_position ? "auto" : "custom"}
              options={[
                { value: "auto", label: "Auto (center)" },
                { value: "custom", label: "Custom" },
              ]}
              onChange={(v) => update((x) => void (x.features.auto_mouse_position = v === "auto"))}
            />
            {!s.features.auto_mouse_position && <PointField target="fishing_point" value={s.points.fishing} />}
          </div>
        </Row>
        <Row
          title={<Step done label="Rod key" />}
          sub="Put the rod in slot 1 and empty the rest of your inventory. Fruit and shop keys live with their features."
          open={open === "keys"}
          onToggle={() => toggle("keys")}
        >
          <KeyRow label="Rod" value={s.keys.rod} onChange={(v) => update((x) => void (x.keys.rod = v))} />
        </Row>
      </Section>

      <Section title="Devil fruits">
        <Row
          title={<Step done={ocrAvailable} label="Text recognition" />}
          sub={ocrAvailable ? "Windows OCR ready" : "Windows OCR language pack missing. Install English in Settings › Language."}
          right={<Pill tone={ocrAvailable ? "ok" : "bad"}>{ocrAvailable ? "ready" : "unavailable"}</Pill>}
        />
        <Row
          title={<Step done label="Drop message area" />}
          sub="The popup at the top middle of the screen: 'New Item <Fruit>' after a catch, 'A Fruit has spawned at Place' for world spawns."
          open={open === "drop"}
          onToggle={() => toggle("drop")}
        >
          <RegionField target="drop_region" value={s.regions.drop} />
          <div className="mt-3 flex items-start gap-3">
            <Button size="sm" onClick={runOcr} disabled={!roblox || !ocrAvailable} icon={<ScanText size={13} />}>
              Read now
            </Button>
            <div className="flex-1 min-w-0 text-[11px]">
              {ocrErr && <div className="text-warn">{ocrErr}</div>}
              {ocr && (
                <div className="font-mono text-fg-dim break-words select-text">
                  {ocr.text.trim() || "(nothing read)"}
                  <div className="mt-1 flex gap-1.5">
                    {ocr.drop && <Pill tone="fruit">{ocr.drop.is_legendary ? "legendary drop (pity 0)" : "fruit drop"}</Pill>}
                    {ocr.spawn && <Pill tone="accent">spawn: {ocr.spawn.name ?? "unknown fruit"}{ocr.spawn.location ? ` at ${ocr.spawn.location}` : ""}</Pill>}
                  </div>
                </div>
              )}
            </div>
          </div>
        </Row>
      </Section>
    </div>
  );
}

function Step({ done, label }: { done: boolean; label: string }) {
  return (
    <span className="inline-flex items-center gap-2">
      {done ? <Check size={14} className="text-ok" /> : <Circle size={14} className="text-fg-mute" />}
      {label}
    </span>
  );
}

function KeyRow({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div className="flex items-center">
      <div className="text-fg-dim w-28">{label}</div>
      <KeyCapture single value={value} onChange={onChange} />
    </div>
  );
}
