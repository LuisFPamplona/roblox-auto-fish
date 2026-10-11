import { create } from "zustand";
import { api, on } from "./ipc";
import type { BotState, LogLine, Reading, Settings, Stats, WindowInfo } from "./types";

type Store = {
  ready: boolean;
  error: string | null;
  state: BotState;
  detail: string | null;
  paused: boolean;
  stats: Stats;
  roblox: WindowInfo | null;
  ocrAvailable: boolean;
  settings: Settings | null;
  version: string;
  log: LogLine[];
  reading: Reading | null;
  saving: boolean;
  lastEvent: { kind: string; text: string; ts: number } | null;
  init: () => Promise<void>;
  refresh: () => Promise<void>;
  update: (mutate: (s: Settings) => void) => Promise<void>;
  setSettings: (s: Settings) => void;
};

const EMPTY_STATS: Stats = {
  fish: 0,
  failed: 0,
  fruits: 0,
  bait_purchased: 0,
  success_rate: 0,
  runtime_s: 0,
  restarts: 0,
  last_fruit: null,
  last_spawn: null,
  total: { fish: 0, failed: 0, fruits: 0, bait_purchased: 0, runtime_s: 0, sessions: 0, last_fruit: null, last_spawn: null },
};

let subscribed = false;
let pendingReading: Reading | null | undefined;
let readingTimer: number | undefined;

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const useStore = create<Store>((set, get) => ({
  ready: false,
  error: null,
  state: "stopped",
  detail: null,
  paused: false,
  stats: EMPTY_STATS,
  roblox: null,
  ocrAvailable: false,
  settings: null,
  version: "",
  log: [],
  reading: null,
  saving: false,
  lastEvent: null,

  init: async () => {
    if (!subscribed) {
      subscribed = true;
      on("bot:state", (p) => set({ state: p.state, detail: p.detail, paused: p.state === "paused" }));
      on("bot:stats", (p) => set({ stats: p }));
      on("bot:log", (p) => set((s) => ({ log: [...s.log.slice(-399), { ts: p.ts, level: p.level, msg: p.msg }] })));
      on("bot:reading", (p) => {
        pendingReading = p;
        if (readingTimer !== undefined) return;
        readingTimer = window.setTimeout(() => {
          readingTimer = undefined;
          if (pendingReading !== undefined) set({ reading: pendingReading });
          pendingReading = undefined;
        }, 66);
      });
      on("roblox:changed", (p) => set({ roblox: p }));
      on("settings:changed", (p) => set({ settings: p }));
      on("bot:fruit_drop", (p) =>
        set({ lastEvent: { kind: "fruit", text: p.is_legendary ? "Legendary devil fruit!" : "Devil fruit dropped!", ts: Date.now() } }),
      );
      on("bot:fruit_spawn", (p) => set({ lastEvent: { kind: "spawn", text: `${p.name ?? "A fruit"} spawned${p.location ? ` at ${p.location}` : ""}`, ts: Date.now() } }));
      on("bot:purchase", (p) => set({ lastEvent: { kind: "purchase", text: `Bought ${p.amount} bait`, ts: Date.now() } }));
      on("bot:recovery", (p) => set({ lastEvent: { kind: "recovery", text: `Recovered (#${p.attempt})`, ts: Date.now() } }));
    }
    for (let attempt = 0; attempt < 20; attempt++) {
      try {
        await get().refresh();
        return;
      } catch (e) {
        set({ error: String(e) });
        await sleep(250 * Math.min(attempt + 1, 8));
      }
    }
  },

  refresh: async () => {
    const snap = await api.snapshot();
    set({
      ready: true,
      error: null,
      state: snap.state,
      paused: snap.paused,
      stats: snap.stats,
      roblox: snap.roblox,
      ocrAvailable: snap.ocr_available,
      settings: snap.settings,
      version: snap.version,
    });
  },

  update: async (mutate) => {
    const cur = get().settings;
    if (!cur) return;
    const next = structuredClone(cur);
    mutate(next);
    set({ settings: next, saving: true, error: null });
    try {
      await api.settingsSet(next);
    } catch (e) {
      set({ settings: cur, error: String(e) });
    } finally {
      set({ saving: false });
    }
  },

  setSettings: (s) => set({ settings: s }),
}));

export const isActive = (s: BotState) => s !== "stopped" && s !== "paused";
