import { useEffect, useState } from "react";
import { FolderOpen, RotateCcw, Save, Trash2, Upload } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import { Button, KeyCapture, Row, Section, Segmented, Slider, Stepper, TextField, Toggle } from "../components/primitives";

export default function SettingsPage() {
  const s = useStore((st) => st.settings);
  const update = useStore((st) => st.update);
  const setSettings = useStore((st) => st.setSettings);
  const version = useStore((st) => st.version);
  const [open, setOpen] = useState<string | null>(null);
  const [presets, setPresets] = useState<string[]>([]);
  const [presetName, setPresetName] = useState("");
  const [updateMsg, setUpdateMsg] = useState<string | null>(null);
  const [dataDir, setDataDir] = useState("");

  useEffect(() => {
    api.presetList().then(setPresets);
    api.dataDir().then(setDataDir);
  }, []);

  if (!s) return null;
  const toggle = (k: string) => setOpen((o) => (o === k ? null : k));

  const savePreset = async () => {
    if (!presetName.trim()) return;
    await api.presetSave(presetName.trim());
    setPresets(await api.presetList());
    setPresetName("");
  };

  const importLegacy = async () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async () => {
      const f = input.files?.[0];
      if (!f) return;
      try {
        setSettings(await api.legacyImport(await f.text()));
      } catch (e) {
        setUpdateMsg(String(e));
      }
    };
    input.click();
  };

  return (
    <div className="pb-4 pt-2">
      <Section title="Tracking">
        <Row title="Tracking" sub="Physics mode models the catch zone's acceleration and input latency. Lookahead is the fallback." open={open === "ctl"} onToggle={() => toggle("ctl")}>
          <Field label="Mode">
            <Segmented
              value={s.fishing.control.mode}
              options={[
                { value: "lookahead", label: "Lookahead" },
                { value: "physics", label: "Physics" },
              ]}
              onChange={(v) => update((x) => void (x.fishing.control.mode = v))}
            />
          </Field>
          {s.fishing.control.mode === "physics" && (
            <div className="text-[11px] text-fg-dim font-mono py-1">
              hold {s.fishing.control.physics.accel_hold.toFixed(2)} · release {s.fishing.control.physics.accel_release.toFixed(2)} · max {s.fishing.control.physics.max_speed.toFixed(2)} · latency {Math.round(s.fishing.control.physics.latency_ms)} ms
              {s.fishing.control.physics.calibrated_at === 0 && <span className="text-warn"> · not calibrated</span>}
            </div>
          )}
          <Field label="Lookahead">
            <Slider value={s.fishing.control.lookahead_ms} min={0} max={400} step={10} format={(v) => `${v} ms`} onChange={(v) => update((x) => void (x.fishing.control.lookahead_ms = v))} />
          </Field>
          <Field label="Dead band">
            <Slider value={s.fishing.control.hysteresis} min={0} max={0.08} step={0.005} format={(v) => v.toFixed(3)} onChange={(v) => update((x) => void (x.fishing.control.hysteresis = v))} />
          </Field>
          <Field label="Velocity smoothing">
            <Slider value={s.fishing.control.velocity_smoothing} min={0} max={0.9} step={0.05} format={(v) => v.toFixed(2)} onChange={(v) => update((x) => void (x.fishing.control.velocity_smoothing = v))} />
          </Field>
          <Field label="Invert hold">
            <Toggle value={s.fishing.control.invert} onChange={(v) => update((x) => void (x.fishing.control.invert = v))} />
          </Field>
          <Field label="Tracking rate">
            <Segmented
              value={String(s.fishing.track_hz)}
              options={[
                { value: "30", label: "30 Hz" },
                { value: "60", label: "60 Hz" },
                { value: "120", label: "120 Hz" },
              ]}
              onChange={(v) => update((x) => void (x.fishing.track_hz = Number(v)))}
            />
          </Field>
        </Row>
        <Row
          title="Record reel logs"
          sub="Every reel writes a small logs/traces/reel-*.json with per-frame numbers and metrics. No screenshots."
          right={<Toggle value={s.fishing.trace} onChange={(v) => update((x) => void (x.fishing.trace = v))} />}
        />
        <Row title="Bite detection" sub="Frames the bar must be seen before reeling starts, and frames it must vanish before the catch counts." open={open === "bite"} onToggle={() => toggle("bite")}>
          <Field label="Confirm frames">
            <Stepper value={s.fishing.bite_confirm_frames} min={1} max={10} onChange={(v) => update((x) => void (x.fishing.bite_confirm_frames = v))} />
          </Field>
          <Field label="Lost frames">
            <Stepper value={s.fishing.lost_frames} min={1} max={20} onChange={(v) => update((x) => void (x.fishing.lost_frames = v))} />
          </Field>
          <Field label="Min reel time">
            <Slider value={s.fishing.min_track_s} min={0} max={5} step={0.1} format={(v) => `${v.toFixed(1)} s`} onChange={(v) => update((x) => void (x.fishing.min_track_s = v))} />
          </Field>
        </Row>
        <Row title="Timeouts" sub="When to give up and recast." open={open === "timeouts"} onToggle={() => toggle("timeouts")}>
          <Field label="Wait for bite">
            <Slider value={s.fishing.scan_timeout_s} min={5} max={60} step={1} format={(v) => `${v} s`} onChange={(v) => update((x) => void (x.fishing.scan_timeout_s = v))} />
          </Field>
          <Field label="Reel timeout">
            <Slider value={s.fishing.track_timeout_s} min={10} max={120} step={5} format={(v) => `${v} s`} onChange={(v) => update((x) => void (x.fishing.track_timeout_s = v))} />
          </Field>
          <Field label="After catch">
            <Slider value={s.fishing.wait_after_catch_s} min={0} max={5} step={0.1} format={(v) => `${v.toFixed(1)} s`} onChange={(v) => update((x) => void (x.fishing.wait_after_catch_s = v))} />
          </Field>
          <Field label="Cast hold">
            <Slider value={s.fishing.cast_hold_ms} min={200} max={3000} step={50} format={(v) => `${v} ms`} onChange={(v) => update((x) => void (x.fishing.cast_hold_ms = v))} />
          </Field>
        </Row>
        <Row title="Bar recognition" sub="Color tolerance and the shape a blue patch must have to count as the fishing bar." open={open === "tol"} onToggle={() => toggle("tol")}>
          <Field label="Color tolerance">
            <Slider value={s.fishing.palette.tolerance} min={0} max={40} step={1} onChange={(v) => update((x) => void (x.fishing.palette.tolerance = v))} />
          </Field>
          <Field label="Min bar height">
            <Slider value={s.fishing.palette.min_bar_height_px} min={10} max={200} step={5} format={(v) => `${v} px`} onChange={(v) => update((x) => void (x.fishing.palette.min_bar_height_px = v))} />
          </Field>
          <Field label="Min tall ratio">
            <Slider value={s.fishing.palette.min_bar_aspect} min={1} max={6} step={0.1} format={(v) => `${v.toFixed(1)}×`} onChange={(v) => update((x) => void (x.fishing.palette.min_bar_aspect = v))} />
          </Field>
          <Field label="Min blue fill">
            <Slider value={s.fishing.palette.min_bar_fill} min={0.1} max={0.9} step={0.05} format={(v) => `${Math.round(v * 100)}%`} onChange={(v) => update((x) => void (x.fishing.palette.min_bar_fill = v))} />
          </Field>
        </Row>
        <Row
          title="Watchdog"
          sub="Stops automation if it stops making progress. Restart manually after reviewing the game."
          right={<Toggle value={s.watchdog.enabled} onChange={(v) => { update((x) => void (x.watchdog.enabled = v)); if (v) setOpen("wd"); }} />}
          open={open === "wd"}
          onToggle={() => toggle("wd")}
        >
          <Field label="Heartbeat timeout">
            <Slider value={s.watchdog.heartbeat_timeout_s} min={10} max={120} step={5} format={(v) => `${v} s`} onChange={(v) => update((x) => void (x.watchdog.heartbeat_timeout_s = v))} />
          </Field>
        </Row>
      </Section>

      <Section title="Hotkeys">
        <Row title="Emergency stop" sub="Ctrl+Alt+F12 · always available, cannot be reassigned." />
        <HotkeyRow label="Start / pause" value={s.hotkeys.toggle} onChange={(v) => update((x) => void (x.hotkeys.toggle = v))} />
        <HotkeyRow label="Edit areas" value={s.hotkeys.overlay} onChange={(v) => update((x) => void (x.hotkeys.overlay = v))} />
        <HotkeyRow label="Hide HUD" value={s.hotkeys.hide_hud} onChange={(v) => update((x) => void (x.hotkeys.hide_hud = v))} />
        <HotkeyRow label="Quit" value={s.hotkeys.quit} onChange={(v) => update((x) => void (x.hotkeys.quit = v))} />
      </Section>

      <Section title="Presets">
        <Row title="Save current" sub="Snapshot every setting under a name.">
          <div className="flex gap-2">
            <TextField value={presetName} onChange={setPresetName} placeholder="e.g. main account" />
            <Button onClick={savePreset} disabled={!presetName.trim()} icon={<Save size={13} />}>
              Save
            </Button>
          </div>
        </Row>
        {presets.map((p) => (
          <Row
            key={p}
            title={p}
            right={
              <>
                <Button size="sm" onClick={async () => setSettings(await api.presetLoad(p))} icon={<Upload size={13} />}>
                  Load
                </Button>
                <Button
                  size="sm"
                  kind="ghost"
                  onClick={async () => {
                    await api.presetDelete(p);
                    setPresets(await api.presetList());
                  }}
                  icon={<Trash2 size={13} />}
                />
              </>
            }
          />
        ))}
        <Row
          title="Import v3 settings"
          sub="Converts an old default_settings.json. Roblox must be open at the same size it was."
          right={
            <Button size="sm" onClick={importLegacy} icon={<FolderOpen size={13} />}>
              Choose file
            </Button>
          }
        />
        <Row
          title="Reset everything"
          right={
            <Button size="sm" kind="danger" onClick={async () => setSettings(await api.settingsReset())} icon={<RotateCcw size={13} />}>
              Reset
            </Button>
          }
        />
      </Section>

      <Section title="App">
        {updateMsg && <div role="alert" className="text-warn px-3 py-2">{updateMsg}</div>}
        <Row title="Updates disabled" sub={`Version ${version} · Install only releases you have verified.`} />
        <Row
          title="Data folder"
          sub={<span className="font-mono text-[11px] select-text">{dataDir}</span>}
          right={
            <Button size="sm" kind="ghost" onClick={() => api.openDataDir()} icon={<FolderOpen size={13} />} />
          }
        />
      </Section>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center min-h-10 py-1">
      <div className="text-fg-dim w-32 shrink-0">{label}</div>
      <div className="ml-auto">{children}</div>
    </div>
  );
}

function HotkeyRow({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return <Row title={label} right={<KeyCapture value={value} onChange={onChange} />} />;
}
