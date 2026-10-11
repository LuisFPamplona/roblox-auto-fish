use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::controller::ControlGains;
use crate::core::fruit::Lexicon;
use crate::core::types::{PxRect, RelPoint, RelRect};
use crate::core::vision::Palette;

pub const SETTINGS_VERSION: u32 = 9;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Regions {
    pub bar: RelRect,
    pub drop: RelRect,
}

impl Default for Regions {
    fn default() -> Self {
        Self {
            bar: RelRect {
                x: 0.546,
                y: 0.322,
                w: 0.121,
                h: 0.436,
            },
            drop: RelRect {
                x: 0.369,
                y: 0.052,
                w: 0.259,
                h: 0.113,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Points {
    pub fishing: RelPoint,
    pub purchase: [Option<RelPoint>; 3],
    pub fruit: [Option<RelPoint>; 2],
    pub bait: [Option<RelPoint>; 2],
}

impl Default for Points {
    fn default() -> Self {
        Self {
            fishing: RelPoint { x: 0.5, y: 0.33 },
            purchase: [None, None, None],
            fruit: [None, None],
            bait: [None, None],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Keys {
    pub rod: char,
    pub fruit_slot_1: char,
    pub fruit_slot_2: char,
    pub shop: char,
}

impl Default for Keys {
    fn default() -> Self {
        Self {
            rod: '1',
            fruit_slot_1: '2',
            fruit_slot_2: '3',
            shop: 'e',
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Fishing {
    pub control: ControlGains,
    pub palette: Palette,
    pub scan_timeout_s: f32,
    pub track_timeout_s: f32,
    pub min_track_s: f32,
    pub bite_confirm_frames: u32,
    pub lost_frames: u32,
    pub wait_after_catch_s: f32,
    pub cast_hold_ms: u32,
    pub scan_hz: u32,
    pub track_hz: u32,
    pub trace: bool,
}

impl Default for Fishing {
    fn default() -> Self {
        Self {
            control: ControlGains::default(),
            palette: Palette::default(),
            scan_timeout_s: 15.0,
            track_timeout_s: 30.0,
            min_track_s: 0.8,
            bite_confirm_frames: 2,
            lost_frames: 4,
            wait_after_catch_s: 1.0,
            cast_hold_ms: 1000,
            scan_hz: 15,
            track_hz: 60,
            trace: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Features {
    pub auto_zoom: bool,
    pub auto_mouse_position: bool,
    pub auto_bait: bool,
    pub fruit_storage: bool,
    pub auto_purchase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Purchase {
    pub amount: u32,
    pub every_n_catches: u32,
    pub hold_shop_key_ms: u32,
    pub after_key_ms: u32,
    pub click_delay_ms: u32,
    pub after_type_ms: u32,
}

impl Default for Purchase {
    fn default() -> Self {
        Self {
            amount: 100,
            every_n_catches: 10,
            hold_shop_key_ms: 3000,
            after_key_ms: 2000,
            click_delay_ms: 1000,
            after_type_ms: 1500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Zoom {
    pub out_steps: u32,
    pub in_steps: u32,
    pub step_delay_ms: u32,
    pub sequence_delay_ms: u32,
}

impl Default for Zoom {
    fn default() -> Self {
        Self {
            out_steps: 10,
            in_steps: 8,
            step_delay_ms: 100,
            sequence_delay_ms: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FruitStorage {
    pub key_settle_ms: u32,
    pub click_settle_ms: u32,
    pub dialog_wait_ms: u32,
    pub after_drop_ms: u32,
}

impl Default for FruitStorage {
    fn default() -> Self {
        Self {
            key_settle_ms: 500,
            click_settle_ms: 500,
            dialog_wait_ms: 800,
            after_drop_ms: 1200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OcrSettings {
    pub spawn_check_interval_s: f32,
    pub spawn_cooldown_s: f32,
    pub post_catch_reads: u32,
    pub post_catch_read_gap_ms: u32,
}

impl Default for OcrSettings {
    fn default() -> Self {
        Self {
            spawn_check_interval_s: 4.0,
            spawn_cooldown_s: 900.0,
            post_catch_reads: 3,
            post_catch_read_gap_ms: 250,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Webhook {
    pub url: String,
    pub enabled: bool,
    pub progress_every_n: u32,
    pub progress: bool,
    pub fruit_drop: bool,
    pub spawn: bool,
    pub purchase: bool,
    pub recovery: bool,
    pub legendary_only: bool,
}

impl Default for Webhook {
    fn default() -> Self {
        Self {
            url: String::new(),
            enabled: false,
            progress_every_n: 10,
            progress: true,
            fruit_drop: true,
            spawn: true,
            purchase: true,
            recovery: true,
            legendary_only: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub toggle: String,
    pub overlay: String,
    pub quit: String,
    pub hide_hud: String,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            toggle: "F1".into(),
            overlay: "F2".into(),
            quit: "F3".into(),
            hide_hud: "F4".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ui {
    pub theme: String,
    pub hud_offset: RelPoint,
    pub hud_visible: bool,
    pub panel_offset: RelPoint,
    pub panel_size: [u32; 2],
    pub log_level: String,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            hud_offset: RelPoint { x: 0.5, y: 0.95 },
            hud_visible: true,
            panel_offset: RelPoint { x: 1.0, y: 0.5 },
            panel_size: [440, 640],
            log_level: "info".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Watchdog {
    pub enabled: bool,
    pub heartbeat_timeout_s: f32,
    pub max_restarts: u32,
    pub restart_backoff_s: f32,
}

impl Default for Watchdog {
    fn default() -> Self {
        Self {
            enabled: true,
            heartbeat_timeout_s: 30.0,
            max_restarts: 5,
            restart_backoff_s: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub regions: Regions,
    pub points: Points,
    pub keys: Keys,
    pub fishing: Fishing,
    pub features: Features,
    pub purchase: Purchase,
    pub zoom: Zoom,
    pub fruit_storage: FruitStorage,
    pub ocr: OcrSettings,
    pub lexicon: Lexicon,
    pub webhook: Webhook,
    pub hotkeys: Hotkeys,
    pub ui: Ui,
    pub watchdog: Watchdog,
    pub auto_update: bool,
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        self.regions.bar.validate()?;
        self.regions.drop.validate()?;
        self.points.fishing.validate()?;
        for p in self
            .points
            .purchase
            .iter()
            .chain(self.points.fruit.iter())
            .chain(self.points.bait.iter())
            .flatten()
        {
            p.validate()?;
        }
        self.ui.hud_offset.validate()?;
        self.ui.panel_offset.validate()?;
        // Bound all nested numeric values before durations, allocations or loops.
        fn bounded(v: &serde_json::Value) -> bool {
            match v {
                serde_json::Value::Number(n) => n
                    .as_f64()
                    .is_some_and(|x| x.is_finite() && x.abs() <= 100_000.0),
                serde_json::Value::Object(o) => {
                    o.iter().all(|(k, v)| k == "calibrated_at" || bounded(v))
                }
                serde_json::Value::Array(a) => a.len() <= 256 && a.iter().all(bounded),
                serde_json::Value::String(s) => s.len() <= 2048,
                _ => true,
            }
        }
        if !bounded(&serde_json::to_value(self).map_err(|_| "Invalid settings")?) {
            return Err("Settings exceed resource limits".into());
        }
        let f = &self.fishing;
        let durations = [
            f.scan_timeout_s,
            f.track_timeout_s,
            f.min_track_s,
            f.wait_after_catch_s,
            self.ocr.spawn_check_interval_s,
            self.ocr.spawn_cooldown_s,
            self.watchdog.heartbeat_timeout_s,
            self.watchdog.restart_backoff_s,
        ];
        if durations
            .iter()
            .any(|x| !x.is_finite() || *x < 0.0 || *x > 3600.0)
            || !(1.0..=120.0).contains(&f.scan_timeout_s)
            || !(1.0..=120.0).contains(&f.track_timeout_s)
            || self.watchdog.heartbeat_timeout_s < 1.0
            || !(1..=60).contains(&f.scan_hz)
            || !(1..=120).contains(&f.track_hz)
            || !(1..=120).contains(&f.bite_confirm_frames)
            || !(1..=120).contains(&f.lost_frames)
            || self.ocr.spawn_check_interval_s < 1.0
            || self.ocr.post_catch_reads > 10
            || self.purchase.amount > 10_000
            || self.zoom.out_steps > 100
            || self.zoom.in_steps > 100
        {
            return Err("Unsafe timing, rate or automation limits".into());
        }
        let control = &f.control;
        if !(0.0..=1.0).contains(&control.velocity_smoothing)
            || !(0.0..=1.0).contains(&control.hysteresis)
            || control.lookahead_ms < 0.0
            || control.physics.latency_ms < 0.0
            || control.physics.max_speed <= 0.0
            || !(0.0..=1.0).contains(&f.palette.min_bar_fill)
            || !(0.0..=1.0).contains(&f.palette.min_row_fraction)
            || f.palette.min_bar_aspect <= 0.0
            || !(0.0..=1.0).contains(&self.lexicon.fuzzy_threshold)
        {
            return Err("Invalid detection or controller parameters".into());
        }
        for list in [
            &self.lexicon.fruits,
            &self.lexicon.drop_phrases,
            &self.lexicon.drop_keywords,
            &self.lexicon.spawn_keywords,
            &self.lexicon.catch_phrases,
            &self.lexicon.fail_phrases,
        ] {
            if list.iter().any(|s| s.trim().is_empty() || s.len() > 128) {
                return Err("Invalid OCR keywords".into());
            }
        }
        if [
            self.keys.rod,
            self.keys.shop,
            self.keys.fruit_slot_1,
            self.keys.fruit_slot_2,
        ]
        .iter()
        .any(|c| !c.is_ascii_alphanumeric())
        {
            return Err("Automation keys must be ASCII letters or digits".into());
        }
        crate::hotkeys::validate(&self.hotkeys)?;
        if self.webhook.enabled {
            crate::webhook::validate_url(&self.webhook.url)?;
        }
        Ok(())
    }
    pub fn fruit_alerts(&self) -> bool {
        self.webhook.enabled && (self.webhook.spawn || self.webhook.fruit_drop)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            regions: Regions::default(),
            points: Points::default(),
            keys: Keys::default(),
            fishing: Fishing::default(),
            features: Features::default(),
            purchase: Purchase::default(),
            zoom: Zoom::default(),
            fruit_storage: FruitStorage::default(),
            ocr: OcrSettings::default(),
            lexicon: Lexicon::default(),
            webhook: Webhook::default(),
            hotkeys: Hotkeys::default(),
            ui: Ui::default(),
            watchdog: Watchdog::default(),
            auto_update: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON configuration (contents redacted)")]
    Json(#[from] serde_json::Error),
    #[error("invalid preset name")]
    BadName,
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.dir.join("logs")
    }

    fn settings_path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    fn presets_dir(&self) -> PathBuf {
        self.dir.join("presets")
    }

    fn stats_path(&self) -> PathBuf {
        self.dir.join("stats.json")
    }

    pub fn load_stats(&self) -> crate::events::Lifetime {
        fs::read_to_string(self.stats_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_stats(&self, l: &crate::events::Lifetime) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.dir)?;
        write_atomic(&self.stats_path(), &serde_json::to_vec_pretty(l)?)
    }

    pub fn load(&self) -> Settings {
        let mut settings = match fs::read_to_string(self.settings_path()) {
            Ok(s) => serde_json::from_str::<Settings>(&s).unwrap_or_else(|_| {
                tracing::warn!("settings parse failed; using defaults");
                Settings::default()
            }),
            Err(_) => Settings::default(),
        };
        if settings.version < SETTINGS_VERSION {
            if settings.version < 6 && settings.fishing.control.physics.calibrated_at == 0 {
                settings.fishing.control = crate::core::controller::ControlGains::default();
            }
            if settings.version < 8 {
                let old = settings.points.purchase;
                settings.points.purchase = [old[2], old[1], None];
            }
            if settings.version < 9 {
                settings.fishing.trace = false;
            }
            settings.version = SETTINGS_VERSION;
            let _ = self.save(&settings);
        }
        settings.auto_update = false;
        if settings.validate().is_err() {
            tracing::warn!("Invalid settings; using safe defaults");
            return Settings::default();
        }
        settings
    }

    pub fn save(&self, s: &Settings) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.dir)?;
        write_atomic(&self.settings_path(), &serde_json::to_vec_pretty(s)?)
    }

    pub fn list_presets(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(rd) = fs::read_dir(self.presets_dir()) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "json") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        out.push(stem.to_string());
                    }
                }
            }
        }
        out.sort();
        out
    }

    fn preset_path(&self, name: &str) -> Result<PathBuf, ConfigError> {
        let reserved = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];
        let ok = !name.is_empty()
            && name.trim() == name
            && !reserved.contains(&name.to_ascii_uppercase().as_str())
            && name.len() <= 64
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ');
        if !ok {
            return Err(ConfigError::BadName);
        }
        Ok(self.presets_dir().join(format!("{name}.json")))
    }

    pub fn save_preset(&self, name: &str, s: &Settings) -> Result<(), ConfigError> {
        fs::create_dir_all(self.presets_dir())?;
        let mut preset = s.clone();
        preset.webhook = Webhook::default();
        preset.auto_update = false;
        write_atomic(
            &self.preset_path(name)?,
            &serde_json::to_vec_pretty(&preset)?,
        )
    }

    pub fn load_preset(&self, name: &str) -> Result<Settings, ConfigError> {
        let s = fs::read_to_string(self.preset_path(name)?)?;
        let mut preset: Settings = serde_json::from_str(&s)?;
        preset.webhook = Webhook::default();
        preset.auto_update = false;
        Ok(preset)
    }

    pub fn delete_preset(&self, name: &str) -> Result<(), ConfigError> {
        fs::remove_file(self.preset_path(name)?)?;
        Ok(())
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct LegacyArea {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug, Deserialize)]
struct LegacyLayout {
    area: Option<LegacyArea>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct LegacySettings {
    auto_purchase_enabled: bool,
    auto_purchase_amount: u32,
    loops_per_purchase: u32,
    point_coords: std::collections::HashMap<String, [i32; 2]>,
    fruit_coords: std::collections::HashMap<String, [i32; 2]>,
    fishing_location: Option<[i32; 2]>,
    fruit_storage_enabled: bool,
    fruit_storage_key: String,
    fruit_storage_key_2: String,
    rod_key: String,
    auto_bait_enabled: bool,
    top_bait_coords: Option<[i32; 2]>,
    top_bait_coords_2: Option<[i32; 2]>,
    kp: f32,
    kd: f32,
    scan_timeout: f32,
    wait_after_loss: f32,
    webhook_url: String,
    #[serde(rename = "webhook_enabled")]
    _webhook_enabled: bool,
    webhook_interval: u32,
    layout_settings: std::collections::HashMap<String, LegacyLayout>,
    zoom_settings: std::collections::HashMap<String, serde_json::Value>,
}

pub fn import_legacy(json: &str, roblox: PxRect) -> Result<Settings, ConfigError> {
    let l: LegacySettings = serde_json::from_str(json)?;
    let mut s = Settings::default();
    let rel = |p: [i32; 2]| crate::core::types::PxPoint { x: p[0], y: p[1] }.to_rel(&roblox);
    let rect = |a: &LegacyArea| {
        PxRect {
            x: a.x,
            y: a.y,
            w: a.width,
            h: a.height,
        }
        .to_rel(&roblox)
    };

    if let Some(a) = l.layout_settings.get("bar").and_then(|x| x.area.as_ref()) {
        s.regions.bar = rect(a);
    }
    if let Some(a) = l.layout_settings.get("drop").and_then(|x| x.area.as_ref()) {
        s.regions.drop = rect(a);
    }
    if let Some(p) = l.fishing_location {
        s.points.fishing = rel(p);
    }
    s.points.purchase = [
        l.point_coords.get("3").map(|p| rel(*p)),
        l.point_coords.get("2").map(|p| rel(*p)),
        None,
    ];
    s.points.fruit[0] = l.fruit_coords.get("fruit_point").map(|p| rel(*p));
    s.points.fruit[1] = l.fruit_coords.get("fruit_point_2").map(|p| rel(*p));
    s.points.bait[0] = l.top_bait_coords.map(rel);
    s.points.bait[1] = l.top_bait_coords_2.map(rel);
    if let Some(c) = l.rod_key.chars().next() {
        s.keys.rod = c;
    }
    if let Some(c) = l.fruit_storage_key.chars().next() {
        s.keys.fruit_slot_1 = c;
    }
    if let Some(c) = l.fruit_storage_key_2.chars().next() {
        s.keys.fruit_slot_2 = c;
    }
    s.features.auto_purchase = l.auto_purchase_enabled;
    s.features.fruit_storage = l.fruit_storage_enabled;
    s.features.auto_bait = l.auto_bait_enabled;
    if l.auto_purchase_amount > 0 {
        s.purchase.amount = l.auto_purchase_amount;
    }
    if l.loops_per_purchase > 0 {
        s.purchase.every_n_catches = l.loops_per_purchase;
    }
    if l.scan_timeout > 0.0 {
        s.fishing.scan_timeout_s = l.scan_timeout;
    }
    if l.wait_after_loss > 0.0 {
        s.fishing.wait_after_catch_s = l.wait_after_loss;
    }
    s.webhook.url = l.webhook_url;
    // Imported files never grant permission for external transmission.
    s.webhook.enabled = false;
    if l.webhook_interval > 0 {
        s.webhook.progress_every_n = l.webhook_interval;
    }
    let zoom = &l.zoom_settings;
    s.features.auto_zoom = zoom
        .get("auto_zoom_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    s.features.auto_mouse_position = zoom
        .get("auto_mouse_position_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if let Some(n) = zoom.get("zoom_in_steps").and_then(|v| v.as_u64()) {
        s.zoom.in_steps = n as u32;
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_defaults() {
        let s = Settings::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert_eq!(back.version, SETTINGS_VERSION);
        assert_eq!(back.keys.rod, '1');
    }

    #[test]
    fn partial_json_fills_defaults() {
        let s: Settings = serde_json::from_str(r#"{"features":{"auto_bait":true}}"#).unwrap();
        assert!(s.features.auto_bait);
        assert_eq!(s.purchase.amount, 100);
    }

    #[test]
    fn legacy_import_converts_to_relative() {
        let legacy = r#"{
          "fishing_location":[960,360],
          "point_coords":{"1":[801,943],"2":[962,941],"3":[1113,943]},
          "layout_settings":{"bar":{"area":{"x":1049,"y":348,"width":233,"height":471}}},
          "rod_key":"1","fruit_storage_key":"2","fruit_storage_key_2":"3",
          "kp":0.2,"kd":0.4,"webhook_url":"https://x","webhook_enabled":true
        }"#;
        let s = import_legacy(
            legacy,
            PxRect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1080,
            },
        )
        .unwrap();
        assert!((s.points.fishing.x - 0.5).abs() < 0.001);
        assert!(s.points.purchase[0].is_some());
        assert!(s.points.purchase[1].is_some());
        assert!((s.regions.bar.w - 233.0 / 1920.0).abs() < 0.001);
        assert!(!s.webhook.enabled);
    }
}
