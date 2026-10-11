import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BotState,
  DropInfo, SpawnInfo,
  LogLine,
  OcrTest,
  OverlaySession,
  OverlayTarget,
  RegionsSession,
  PxRect,
  Reading,
  RegionPreview,
  RelPoint,
  RelRect,
  Settings,
  Snapshot,
  Stats,
  WindowInfo,
} from "./types";

export const api = {
  snapshot: () => invoke<Snapshot>("snapshot"),
  botStart: () => invoke<void>("bot_start"),
  botPause: () => invoke<void>("bot_pause"),
  botStop: () => invoke<void>("bot_stop"),
  botToggle: () => invoke<void>("bot_toggle"),
  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (settings: Settings) => invoke<void>("settings_set", { settings }),
  settingsReset: () => invoke<Settings>("settings_reset"),
  presetList: () => invoke<string[]>("preset_list"),
  presetSave: (name: string) => invoke<void>("preset_save", { name }),
  presetLoad: (name: string) => invoke<Settings>("preset_load", { name }),
  presetDelete: (name: string) => invoke<void>("preset_delete", { name }),
  legacyImport: (json: string) => invoke<Settings>("legacy_import", { json }),
  overlayOpen: (target: OverlayTarget) => invoke<OverlaySession>("overlay_open", { target }),
  overlayCommit: (commit: { target: OverlayTarget; region?: RelRect | null; point?: RelPoint | null }) =>
    invoke<Settings>("overlay_commit", { commit }),
  overlayCancel: () => invoke<void>("overlay_cancel"),
  overlayOpenRegions: () => invoke<RegionsSession>("overlay_open_regions"),
  overlayPending: () => invoke<{ kind: "single"; session: OverlaySession } | { kind: "regions"; session: RegionsSession } | null>("overlay_pending"),
  overlayCommitRegions: (commit: { bar: RelRect; drop: RelRect }) => invoke<Settings>("overlay_commit_regions", { commit }),
  panelPlacementChanged: () => invoke<void>("panel_placement_changed"),
  regionPreview: (region: RelRect, maxDim = 320) => invoke<RegionPreview>("region_preview", { region, maxDim }),
  panelVisible: () => invoke<boolean>("panel_visible"),
  ocrTest: () => invoke<OcrTest>("ocr_test"),
  webhookTest: () => invoke<void>("webhook_test"),
  detectBarRegion: () => invoke<RelRect>("detect_bar_region"),
  hudSetOffset: (offset: RelPoint) => invoke<void>("hud_set_offset", { offset }),
  hudToggle: () => invoke<boolean>("hud_toggle"),
  panelShow: () => invoke<void>("panel_show"),
  guideOpen: () => invoke<void>("guide_open"),
  guideHide: () => invoke<void>("guide_hide"),
  panelHide: () => invoke<void>("panel_hide"),
  panelToggle: () => invoke<void>("panel_toggle"),
  quit: () => invoke<void>("app_quit"),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
  openDataDir: () => invoke<void>("open_data_dir"),
  dataDir: () => invoke<string>("data_dir"),
};

type Events = {
  "bot:state": { kind: "state"; state: BotState; detail: string | null };
  "bot:stats": { kind: "stats" } & Stats;
  "bot:log": { kind: "log" } & LogLine;
  "bot:reading": Reading | null;
  "bot:fruit_drop": { kind: "fruit_drop" } & DropInfo;
  "bot:fruit_spawn": { kind: "fruit_spawn" } & SpawnInfo;
  "bot:purchase": { kind: "purchase"; amount: number };
  "bot:recovery": { kind: "recovery"; attempt: number; reason: string };
  "roblox:changed": WindowInfo | null;
  "overlay:roblox": PxRect;
  "overlay:session": OverlaySession;
  "overlay:regions": RegionsSession;
  "overlay:close": null;
  "overlay:view": null;
  "ui:visibility": boolean;
  "guide:open": null;
  "settings:changed": Settings;
  "panel:visible": boolean;
};

export function on<K extends keyof Events>(name: K, cb: (payload: Events[K]) => void): Promise<UnlistenFn> {
  return listen<Events[K]>(name, (e) => cb(e.payload));
}
