export type RelPoint = { x: number; y: number };
export type RelRect = { x: number; y: number; w: number; h: number };
export type PxPoint = { x: number; y: number };
export type PxRect = { x: number; y: number; w: number; h: number };
export type Rgb = { r: number; g: number; b: number };

export type WindowInfo = { client: PxRect; is_foreground: boolean; visible: boolean; dpi: number };

export type BotState =
  | "stopped"
  | "waiting_for_roblox"
  | "initial_setup"
  | "casting"
  | "waiting_for_bite"
  | "tracking"
  | "post_catch"
  | "purchasing"
  | "storing_fruit"
  | "recovering"
  | "paused";

export type Lifetime = {
  fish: number;
  failed: number;
  fruits: number;
  bait_purchased: number;
  runtime_s: number;
  sessions: number;
  last_fruit: string | null;
  last_spawn: string | null;
};

export type Stats = {
  fish: number;
  failed: number;
  fruits: number;
  bait_purchased: number;
  success_rate: number;
  runtime_s: number;
  restarts: number;
  last_fruit: string | null;
  last_spawn: string | null;
  total: Lifetime;
};

export type LogLevel = "debug" | "info" | "warn" | "error";
export type LogLine = { ts: number; level: LogLevel; msg: string };

export type Span = { start: number; end: number };
export type Bbox = { x0: number; y0: number; x1: number; y1: number };
export type Reading = {
  bar: Bbox;
  fish: Span;
  marker: Span;
  fish_center: number;
  marker_center: number;
  error: number;
  hold: boolean;
  predicted_error: number;
  fish_velocity: number;
  marker_velocity: number;
  origin_dx: number;
};
export type Confidence = { bar: number; fish: number; marker: number; score: number };

export type DropInfo = { text: string; is_legendary: boolean };
export type SpawnInfo = { text: string; name: string | null; location: string | null };

export type Settings = {
  version: number;
  regions: { bar: RelRect; drop: RelRect };
  points: {
    fishing: RelPoint;
    purchase: [RelPoint | null, RelPoint | null, RelPoint | null];
    fruit: [RelPoint | null, RelPoint | null];
    bait: [RelPoint | null, RelPoint | null];
  };
  keys: { rod: string; fruit_slot_1: string; fruit_slot_2: string; shop: string };
  fishing: {
    control: {
      mode: "lookahead" | "physics";
      lookahead_ms: number;
      hysteresis: number;
      velocity_smoothing: number;
      invert: boolean;
      physics: { accel_hold: number; accel_release: number; max_speed: number; latency_ms: number; calibrated_at: number };
    };
    palette: { bar: Rgb; fish: Rgb; marker: Rgb; tolerance: number; min_bar_height_px: number; min_bar_aspect: number; min_bar_fill: number; min_row_fraction: number };
    scan_timeout_s: number;
    track_timeout_s: number;
    min_track_s: number;
    bite_confirm_frames: number;
    lost_frames: number;
    wait_after_catch_s: number;
    cast_hold_ms: number;
    scan_hz: number;
    track_hz: number;
    trace: boolean;
  };
  features: {
    auto_zoom: boolean;
    auto_mouse_position: boolean;
    auto_bait: boolean;
    fruit_storage: boolean;
    auto_purchase: boolean;
  };
  purchase: {
    amount: number;
    every_n_catches: number;
    hold_shop_key_ms: number;
    after_key_ms: number;
    click_delay_ms: number;
    after_type_ms: number;
  };
  zoom: { out_steps: number; in_steps: number; step_delay_ms: number; sequence_delay_ms: number };
  fruit_storage: { key_settle_ms: number; click_settle_ms: number; dialog_wait_ms: number; after_drop_ms: number };
  ocr: { spawn_check_interval_s: number; spawn_cooldown_s: number; post_catch_reads: number; post_catch_read_gap_ms: number };
  lexicon: { fruits: string[]; drop_phrases: string[]; drop_keywords: string[]; spawn_keywords: string[]; catch_phrases: string[]; fail_phrases: string[]; fuzzy_threshold: number };
  webhook: {
    url: string;
    enabled: boolean;
    progress_every_n: number;
    progress: boolean;
    fruit_drop: boolean;
    spawn: boolean;
    purchase: boolean;
    recovery: boolean;
    legendary_only: boolean;
  };
  hotkeys: { toggle: string; overlay: string; quit: string; hide_hud: string };
  ui: { theme: string; hud_offset: RelPoint; hud_visible: boolean; panel_offset: RelPoint; panel_size: [number, number]; log_level: string };
  watchdog: { enabled: boolean; heartbeat_timeout_s: number; max_restarts: number; restart_backoff_s: number };
  auto_update: boolean;
};

export type Snapshot = {
  state: BotState;
  paused: boolean;
  stats: Stats;
  roblox: WindowInfo | null;
  ocr_available: boolean;
  settings: Settings;
  version: string;
};

export type OverlayTarget =
  | "bar_region"
  | "drop_region"
  | "fishing_point"
  | "purchase1"
  | "purchase2"
  | "purchase3"
  | "fruit1"
  | "fruit2"
  | "bait1"
  | "bait2";

export type OverlaySession = {
  target: OverlayTarget;
  roblox: PxRect;
  overlay_origin: PxPoint;
  region: RelRect | null;
  point: RelPoint | null;
};

export type RegionsSession = { roblox: PxRect; bar: RelRect; drop: RelRect };

export type Geometry = { bar: Bbox; fish: Span; marker: Span; fish_center: number; marker_center: number; error: number };

export type RegionPreview = {
  width: number;
  height: number;
  png_base64: string;
  confidence: Confidence;
  reading: Geometry | null;
};

export type OcrTest = { text: string; drop: DropInfo | null; spawn: SpawnInfo | null };

export const STATE_LABEL: Record<BotState, string> = {
  stopped: "Idle",
  waiting_for_roblox: "Waiting for Roblox",
  initial_setup: "Setting up",
  casting: "Casting",
  waiting_for_bite: "Waiting for bite",
  tracking: "Reeling",
  post_catch: "Checking catch",
  purchasing: "Buying bait",
  storing_fruit: "Storing fruit",
  recovering: "Recovering",
  paused: "Paused",
};

export const TARGET_LABEL: Record<OverlayTarget, string> = {
  bar_region: "Fishing bar area",
  drop_region: "Drop message area",
  fishing_point: "Cast point",
  purchase1: "Shop confirm button",
  purchase2: "Shop quantity box",
  purchase3: "Shop cancel button",
  fruit1: "Fruit slot",
  fruit2: "Fruit slot (backup)",
  bait1: "Bait slot",
  bait2: "Bait slot (backup)",
};
