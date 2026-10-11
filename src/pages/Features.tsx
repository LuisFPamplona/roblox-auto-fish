import { useState } from "react";
import { Send } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import { PointField } from "../components/PointField";
import { Button, Kbd, KeyCapture, Pill, Row, Section, Slider, Step, Steps, Stepper, TextField, Toggle } from "../components/primitives";

export default function Features() {
  const s = useStore((st) => st.settings);
  const update = useStore((st) => st.update);
  const ocrAvailable = useStore((st) => st.ocrAvailable);
  const [open, setOpen] = useState<string | null>(null);
  const [whTest, setWhTest] = useState<"idle" | "ok" | string>("idle");
  if (!s) return null;
  const toggle = (k: string) => setOpen((o) => (o === k ? null : k));
  const baitReady = !!s.points.bait[0];
  const shopReady = !!s.points.purchase[0] && !!s.points.purchase[1];
  const fruitReady = !!s.points.fruit[0];

  const testWebhook = async () => {
    try {
      await api.webhookTest();
      setWhTest("ok");
    } catch (e) {
      setWhTest(String(e));
    }
  };

  return (
    <div className="pb-4 pt-2">
      <Section title="Fishing">
        <Row
          title="Auto zoom"
          sub="Resets the camera to a known zoom before fishing so the bar lands in the same place."
          right={<Toggle value={s.features.auto_zoom} onChange={(v) => { update((x) => void (x.features.auto_zoom = v)); if (v) setOpen("zoom"); }} />}
          open={open === "zoom"}
          onToggle={() => toggle("zoom")}
        >
          <Field label="Zoom out steps">
            <Stepper value={s.zoom.out_steps} min={1} max={30} onChange={(v) => update((x) => void (x.zoom.out_steps = v))} />
          </Field>
          <Field label="Zoom in steps">
            <Stepper value={s.zoom.in_steps} min={0} max={30} onChange={(v) => update((x) => void (x.zoom.in_steps = v))} />
          </Field>
          <Field label="Step delay">
            <Slider value={s.zoom.step_delay_ms} min={20} max={400} step={10} format={(v) => `${v} ms`} onChange={(v) => update((x) => void (x.zoom.step_delay_ms = v))} />
          </Field>
        </Row>
        <Row
          title="Auto bait"
          sub="Re-selects your bait before every cast so the rod never fishes empty."
          right={
            <>
              {s.features.auto_bait && !baitReady && <Pill tone="warn">point needed</Pill>}
              <Toggle value={s.features.auto_bait} onChange={(v) => { update((x) => void (x.features.auto_bait = v)); if (v) setOpen("bait"); }} />
            </>
          }
          open={open === "bait"}
          onToggle={() => toggle("bait")}
        >
          <Steps>
            <Step n={1} title={<>Press <Kbd>{s.keys.rod.toUpperCase()}</Kbd> to open the rod menu</>} sub="The rod key is set in Setup › Inventory keys." done />
            <Step n={2} title="Click the bait" sub="Pick the top bait in the rod menu." done={baitReady}>
              <PointField target="bait1" value={s.points.bait[0]} />
            </Step>
            <Step n={3} title="Backup click" sub="Optional. If set, the bot clicks here, then the bait again." done={!!s.points.bait[1]} last>
              <PointField target="bait2" value={s.points.bait[1]} clearable onCleared={() => update((x) => void (x.points.bait[1] = null))} />
            </Step>
          </Steps>
        </Row>
        <Row
          title="Auto buy bait"
          sub="Stand next to the bait barrel on the dock. Runs once at start and again every N catches."
          right={
            <>
              {s.features.auto_purchase && !shopReady && <Pill tone="warn">points needed</Pill>}
              <Toggle value={s.features.auto_purchase} onChange={(v) => { update((x) => void (x.features.auto_purchase = v)); if (v) setOpen("buy"); }} />
            </>
          }
          open={open === "buy"}
          onToggle={() => toggle("buy")}
        >
          <Steps>
            <Step n={1} title="Hold the shop key" sub="Held down next to the barrel until its shop dialog opens, like holding it yourself." done>
              <KeyCapture single value={s.keys.shop} onChange={(v) => update((x) => void (x.keys.shop = v))} />
              <Slider value={s.purchase.hold_shop_key_ms} min={500} max={5000} step={100} width={150} format={(v) => `${(v / 1000).toFixed(1)} s`} onChange={(v) => update((x) => void (x.purchase.hold_shop_key_ms = v))} />
            </Step>
            <Step n={2} title="Click the Confirm button" sub="Pick the button that confirms the purchase." done={!!s.points.purchase[0]}>
              <PointField target="purchase1" value={s.points.purchase[0]} clearable onCleared={() => update((x) => void (x.points.purchase[0] = null))} />
            </Step>
            <Step n={3} title="Click the quantity box and type the amount" sub="Pick the number field in the shop." done={!!s.points.purchase[1]}>
              <PointField target="purchase2" value={s.points.purchase[1]} clearable onCleared={() => update((x) => void (x.points.purchase[1] = null))} />
              <Stepper value={s.purchase.amount} min={1} max={9999} step={10} suffix="bait" onChange={(v) => update((x) => void (x.purchase.amount = v))} />
            </Step>
            <Step n={4} title="Click the Cancel button" sub="Optional. Clicked after the second Confirm to close the dialog." done={!!s.points.purchase[2]}>
              <PointField target="purchase3" value={s.points.purchase[2]} clearable onCleared={() => update((x) => void (x.points.purchase[2] = null))} />
            </Step>
            <Step n={5} title="Back to fishing" sub="Clicks the quantity box once more, then right-clicks the cast point." done last />
          </Steps>
          <div className="mt-4 pt-3 border-t border-line">
            <Field label="Buy every">
              <Stepper value={s.purchase.every_n_catches} min={1} max={500} suffix="fish" onChange={(v) => update((x) => void (x.purchase.every_n_catches = v))} />
            </Field>
            <Field label="Pause between clicks">
              <Slider value={s.purchase.click_delay_ms} min={200} max={3000} step={50} format={(v) => `${v} ms`} onChange={(v) => update((x) => void (x.purchase.click_delay_ms = v))} />
            </Field>
          </div>
        </Row>
      </Section>

      <Section title="Devil fruits">
        <Row
          title="Store fruits"
          sub="After a fruit drop, moves it to your fruit slots and re-equips the rod."
          right={
            <>
              {s.features.fruit_storage && !fruitReady && <Pill tone="warn">point needed</Pill>}
              <Toggle disabled={!ocrAvailable} value={s.features.fruit_storage} onChange={(v) => { update((x) => void (x.features.fruit_storage = v)); if (v) setOpen("store"); }} />
            </>
          }
          open={open === "store"}
          onToggle={() => toggle("store")}
          disabled={!ocrAvailable}
        >
          <Steps>
            <Step n={1} title="Fruit slot keys" sub="The two hotbar slots the fruit is moved through. Keep them empty." done>
              <KeyCapture single value={s.keys.fruit_slot_1} onChange={(v) => update((x) => void (x.keys.fruit_slot_1 = v))} />
              <KeyCapture single value={s.keys.fruit_slot_2} onChange={(v) => update((x) => void (x.keys.fruit_slot_2 = v))} />
            </Step>
            <Step n={2} title="Click the Store button" sub="Pick where the Store button appears after switching to a fruit slot." done={fruitReady}>
              <PointField target="fruit1" value={s.points.fruit[0]} />
            </Step>
            <Step n={3} title="Backup click" sub="Optional. If set, the bot clicks here, then the Store button again." done={!!s.points.fruit[1]}>
              <PointField target="fruit2" value={s.points.fruit[1]} clearable onCleared={() => update((x) => void (x.points.fruit[1] = null))} />
            </Step>
            <Step n={4} title={<>Press <Kbd>{s.keys.rod.toUpperCase()}</Kbd> to re-equip the rod</>} sub="The rod key is set in Setup." done last />
          </Steps>
          <div className="mt-4 pt-3 border-t border-line">
            <Field label="Dialog wait">
              <Slider value={s.fruit_storage.dialog_wait_ms} min={200} max={3000} step={50} format={(v) => `${v} ms`} onChange={(v) => update((x) => void (x.fruit_storage.dialog_wait_ms = v))} />
            </Field>
            <Field label="After drop">
              <Slider value={s.fruit_storage.after_drop_ms} min={200} max={4000} step={50} format={(v) => `${v} ms`} onChange={(v) => update((x) => void (x.fruit_storage.after_drop_ms = v))} />
            </Field>
          </div>
        </Row>
      </Section>

      <Section title="Discord">
        <Row
          title="Webhook"
          sub={s.webhook.url ? "Sends progress, fruit drops and world spawns to your channel." : "Paste a channel webhook URL to enable."}
          right={<Toggle value={s.webhook.enabled} onChange={(v) => { update((x) => void (x.webhook.enabled = v)); if (v) setOpen("wh"); }} />}
          open={open === "wh"}
          onToggle={() => toggle("wh")}
        >
          <div className="flex gap-2 mb-3">
            <TextField
              type="password"
              mono
              placeholder="https://discord.com/api/webhooks/…"
              value={s.webhook.url}
              onChange={(v) => update((x) => void (x.webhook.url = v.trim()))}
            />
            <Button size="md" onClick={testWebhook} disabled={!s.webhook.enabled || !s.webhook.url} icon={<Send size={13} />}>
              Test
            </Button>
          </div>
          {whTest !== "idle" && (
            <div className="mb-3">{whTest === "ok" ? <Pill tone="ok">message delivered</Pill> : <Pill tone="bad">{whTest}</Pill>}</div>
          )}
          <Field label="Progress every">
            <Stepper value={s.webhook.progress_every_n} min={1} max={500} suffix="fish" onChange={(v) => update((x) => void (x.webhook.progress_every_n = v))} />
          </Field>
          <div className="mt-2 border-t border-line">
            {(
              [
                ["progress", "Progress updates"],
                ["fruit_drop", "Devil fruit caught"],
                ["spawn", "World spawn (reads the drop message area)"],
                ["purchase", "Bait purchased"],
                ["recovery", "Recovery / stuck"],
              ] as const
            ).map(([k, label]) => (
              <div key={k} className="flex items-center h-10 border-b border-line">
                <div className="text-fg-dim">{label}</div>
                <div className="ml-auto">
                  <Toggle value={s.webhook[k]} disabled={(k === "fruit_drop" || k === "spawn") && !ocrAvailable} onChange={(v) => update((x) => void (x.webhook[k] = v))} />
                </div>
              </div>
            ))}
            {s.webhook.fruit_drop && ocrAvailable && (
              <div className="flex items-center h-10 border-b border-line pl-4">
                <div className="text-fg-dim">Legendary drops only</div>
                <div className="ml-auto">
                  <Toggle value={s.webhook.legendary_only} onChange={(v) => update((x) => void (x.webhook.legendary_only = v))} />
                </div>
              </div>
            )}
          </div>
          {s.webhook.spawn && ocrAvailable && (
            <div className="mt-3">
              <Field label="Spawn check every">
                <Slider value={s.ocr.spawn_check_interval_s} min={2} max={30} step={1} format={(v) => `${v} s`} onChange={(v) => update((x) => void (x.ocr.spawn_check_interval_s = v))} />
              </Field>
              <Field label="Spawn cooldown">
                <Slider value={s.ocr.spawn_cooldown_s / 60} min={1} max={30} step={1} format={(v) => `${v} min`} onChange={(v) => update((x) => void (x.ocr.spawn_cooldown_s = v * 60))} />
              </Field>
            </div>
          )}
        </Row>
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
