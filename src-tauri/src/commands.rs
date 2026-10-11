use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::app::AppState;
use crate::config::{import_legacy, Settings};
use crate::core::types::{PxPoint, PxRect, RelPoint, RelRect, WindowInfo};
use crate::core::vision;
use crate::events::{BotState, Stats};
use crate::hotkeys;
use crate::windows;

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub state: BotState,
    pub paused: bool,
    pub stats: Stats,
    pub roblox: Option<WindowInfo>,
    pub ocr_available: bool,
    pub settings: Settings,
    pub version: String,
}

#[tauri::command]
pub fn snapshot(app: AppHandle, st: State<'_, AppState>) -> Snapshot {
    Snapshot {
        state: st.bot.state(),
        paused: st.bot.is_paused(),
        stats: st.bot.ctx().session.lock().stats(),
        roblox: *st.roblox.read(),
        ocr_available: st.platform.ocr.available(),
        settings: st.settings.read().clone(),
        version: app.package_info().version.to_string(),
    }
}

#[tauri::command]
pub fn bot_start(st: State<'_, AppState>) {
    st.bot.start();
}

#[tauri::command]
pub fn bot_pause(st: State<'_, AppState>) {
    st.bot.pause();
}

#[tauri::command]
pub fn bot_stop(st: State<'_, AppState>) {
    st.platform.input.set_enabled(false);
    st.bot.stop();
}

#[tauri::command]
pub fn bot_toggle(st: State<'_, AppState>) {
    st.bot.toggle();
}

#[tauri::command]
pub fn settings_get(st: State<'_, AppState>) -> Settings {
    st.settings.read().clone()
}

#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    st: State<'_, AppState>,
    mut settings: Settings,
) -> Result<(), String> {
    settings.auto_update = false;
    settings.validate()?;
    let (hotkeys_changed, hud_changed) = {
        let cur = st.settings.read();
        settings.ui.panel_offset = cur.ui.panel_offset;
        settings.ui.panel_size = cur.ui.panel_size;
        (
            cur.hotkeys != settings.hotkeys,
            cur.ui.hud_visible != settings.ui.hud_visible
                || cur.ui.hud_offset != settings.ui.hud_offset,
        )
    };
    let previous = st.settings.read().clone();
    if hotkeys_changed {
        if let Err(e) = hotkeys::register(&app, &settings.hotkeys) {
            let _ = hotkeys::register(&app, &previous.hotkeys);
            return Err(e);
        }
    }
    if let Err(e) = st.store.save(&settings) {
        if hotkeys_changed {
            let _ = hotkeys::register(&app, &previous.hotkeys);
        }
        return Err(e.to_string());
    }
    *st.settings.write() = settings.clone();
    if hud_changed {
        windows::reposition(&app);
    }
    let _ = app.emit("settings:changed", &settings);
    Ok(())
}

#[tauri::command]
pub fn settings_reset(app: AppHandle, st: State<'_, AppState>) -> Result<Settings, String> {
    let s = Settings::default();
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn preset_list(st: State<'_, AppState>) -> Vec<String> {
    st.store.list_presets()
}

#[tauri::command]
pub fn preset_save(st: State<'_, AppState>, name: String) -> Result<(), String> {
    st.store
        .save_preset(&name, &st.settings.read())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preset_load(
    app: AppHandle,
    st: State<'_, AppState>,
    name: String,
) -> Result<Settings, String> {
    let s = st.store.load_preset(&name).map_err(|e| e.to_string())?;
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn preset_delete(st: State<'_, AppState>, name: String) -> Result<(), String> {
    st.store.delete_preset(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn legacy_import(
    app: AppHandle,
    st: State<'_, AppState>,
    json: String,
) -> Result<Settings, String> {
    let rect = st
        .roblox
        .read()
        .map(|w| w.client)
        .ok_or("Open Roblox first so coordinates can be converted")?;
    let s = import_legacy(&json, rect).map_err(|e| e.to_string())?;
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayTarget {
    BarRegion,
    DropRegion,
    FishingPoint,
    Purchase1,
    Purchase2,
    Purchase3,
    Fruit1,
    Fruit2,
    Bait1,
    Bait2,
}

impl OverlayTarget {
    pub fn is_region(self) -> bool {
        matches!(self, OverlayTarget::BarRegion | OverlayTarget::DropRegion)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlaySession {
    pub target: OverlayTarget,
    pub roblox: PxRect,
    pub overlay_origin: PxPoint,
    pub region: Option<RelRect>,
    pub point: Option<RelPoint>,
}

#[tauri::command]
pub fn overlay_open(
    app: AppHandle,
    st: State<'_, AppState>,
    target: OverlayTarget,
) -> Result<OverlaySession, String> {
    let roblox = st
        .roblox
        .read()
        .map(|w| w.client)
        .ok_or("Roblox window not found")?;
    let origin = windows::show_overlay(&app, roblox, true)?;
    let s = st.settings.read();
    let (region, point) = match target {
        OverlayTarget::BarRegion => (Some(s.regions.bar), None),
        OverlayTarget::DropRegion => (Some(s.regions.drop), None),
        OverlayTarget::FishingPoint => (None, Some(s.points.fishing)),
        OverlayTarget::Purchase1 => (None, s.points.purchase[0]),
        OverlayTarget::Purchase2 => (None, s.points.purchase[1]),
        OverlayTarget::Purchase3 => (None, s.points.purchase[2]),
        OverlayTarget::Fruit1 => (None, s.points.fruit[0]),
        OverlayTarget::Fruit2 => (None, s.points.fruit[1]),
        OverlayTarget::Bait1 => (None, s.points.bait[0]),
        OverlayTarget::Bait2 => (None, s.points.bait[1]),
    };
    let session = OverlaySession {
        target,
        roblox,
        overlay_origin: origin,
        region,
        point,
    };
    *st.overlay_session.lock() = Some(serde_json::json!({ "kind": "single", "session": session }));
    if let Some(o) = app.get_webview_window("overlay") {
        let _ = o.emit("overlay:session", &session);
    }
    Ok(session)
}

#[derive(Debug, Deserialize)]
pub struct OverlayCommit {
    pub target: OverlayTarget,
    pub region: Option<RelRect>,
    pub point: Option<RelPoint>,
}

#[tauri::command]
pub fn overlay_commit(
    app: AppHandle,
    st: State<'_, AppState>,
    commit: OverlayCommit,
) -> Result<Settings, String> {
    let mut s = st.settings.read().clone();
    match commit.target {
        OverlayTarget::BarRegion => s.regions.bar = commit.region.ok_or("region required")?,
        OverlayTarget::DropRegion => s.regions.drop = commit.region.ok_or("region required")?,
        OverlayTarget::FishingPoint => s.points.fishing = commit.point.ok_or("point required")?,
        OverlayTarget::Purchase1 => s.points.purchase[0] = commit.point,
        OverlayTarget::Purchase2 => s.points.purchase[1] = commit.point,
        OverlayTarget::Purchase3 => s.points.purchase[2] = commit.point,
        OverlayTarget::Fruit1 => s.points.fruit[0] = commit.point,
        OverlayTarget::Fruit2 => s.points.fruit[1] = commit.point,
        OverlayTarget::Bait1 => s.points.bait[0] = commit.point,
        OverlayTarget::Bait2 => s.points.bait[1] = commit.point,
    }
    windows::hide_overlay(&app);
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn overlay_cancel(app: AppHandle) {
    windows::hide_overlay(&app);
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionsSession {
    pub roblox: PxRect,
    pub bar: RelRect,
    pub drop: RelRect,
}

pub fn open_regions_editor(app: &AppHandle) -> Result<RegionsSession, String> {
    let st = app.state::<AppState>();
    let roblox = st
        .roblox
        .read()
        .map(|w| w.client)
        .ok_or("Roblox window not found")?;
    windows::show_overlay(app, roblox, true)?;
    let s = st.settings.read();
    let session = RegionsSession {
        roblox,
        bar: s.regions.bar,
        drop: s.regions.drop,
    };
    drop(s);
    *st.overlay_session.lock() = Some(serde_json::json!({ "kind": "regions", "session": session }));
    if let Some(o) = app.get_webview_window("overlay") {
        let _ = o.emit("overlay:regions", &session);
    }
    Ok(session)
}

#[tauri::command]
pub fn overlay_pending(st: State<'_, AppState>) -> Option<serde_json::Value> {
    st.overlay_session.lock().clone()
}

#[tauri::command]
pub fn overlay_open_regions(app: AppHandle) -> Result<RegionsSession, String> {
    open_regions_editor(&app)
}

#[derive(Debug, Deserialize)]
pub struct RegionsCommit {
    pub bar: RelRect,
    pub drop: RelRect,
}

#[tauri::command]
pub fn overlay_commit_regions(
    app: AppHandle,
    st: State<'_, AppState>,
    commit: RegionsCommit,
) -> Result<Settings, String> {
    let mut s = st.settings.read().clone();
    s.regions.bar = commit.bar;
    s.regions.drop = commit.drop;
    windows::hide_overlay(&app);
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn panel_placement_changed(app: AppHandle) {
    windows::save_panel_placement(&app);
}

#[derive(Debug, Serialize)]
pub struct RegionPreview {
    pub width: usize,
    pub height: usize,
    pub png_base64: String,
    pub confidence: vision::Confidence,
    pub reading: Option<vision::Reading>,
}

#[tauri::command]
pub async fn region_preview(
    st: State<'_, AppState>,
    region: RelRect,
    max_dim: Option<u32>,
) -> Result<RegionPreview, String> {
    let roblox = st
        .roblox
        .read()
        .map(|w| w.client)
        .ok_or("Roblox window not found")?;
    let capture = st.platform.capture.clone();
    let palette = st.settings.read().fishing.palette;
    region.validate()?;
    let max_dim = max_dim.unwrap_or(320).clamp(32, 1024) as usize;
    blocking(move || {
        let frame = capture
            .grab(region.to_px(&roblox))
            .map_err(|e| e.to_string())?;
        let confidence = vision::confidence(&frame, &palette);
        let reading = vision::read(&frame, &palette);
        let small = frame.downscale(max_dim);
        let png = encode_png(&small)?;
        Ok(RegionPreview {
            width: frame.w,
            height: frame.h,
            png_base64: png,
            confidence,
            reading,
        })
    })
    .await
}

#[derive(Debug, Serialize)]
pub struct OcrTest {
    pub text: String,
    pub drop: Option<crate::core::fruit::DropInfo>,
    pub spawn: Option<crate::core::fruit::SpawnInfo>,
}

#[tauri::command]
pub async fn ocr_test(st: State<'_, AppState>) -> Result<OcrTest, String> {
    let roblox = st
        .roblox
        .read()
        .map(|w| w.client)
        .ok_or("Roblox window not found")?;
    let region = st.settings.read().regions.drop;
    let lex = st.settings.read().lexicon.clone();
    let platform = st.platform.clone();
    blocking(move || {
        if !platform.ocr.available() {
            return Err("Windows OCR is not available on this system".into());
        }
        let frame = platform
            .capture
            .grab(region.to_px(&roblox))
            .map_err(|e| e.to_string())?;
        let text = platform.ocr.read(&frame).map_err(|e| e.to_string())?;
        Ok(OcrTest {
            drop: crate::core::fruit::detect_drop(&lex, &text),
            spawn: crate::core::fruit::detect_spawn(&lex, &text),
            text,
        })
    })
    .await
}

#[tauri::command]
pub async fn webhook_test(st: State<'_, AppState>) -> Result<(), String> {
    let wh = st.webhook.clone();
    blocking(move || wh.test()).await
}

#[tauri::command]
pub async fn detect_bar_region(app: AppHandle, st: State<'_, AppState>) -> Result<RelRect, String> {
    let roblox = st
        .roblox
        .read()
        .map(|w| w.client)
        .ok_or("Roblox window not found")?;
    let capture = st.platform.capture.clone();
    let palette = st.settings.read().fishing.palette;
    st.bot.pause();
    let hidden = windows::suspend_for_capture(&app)?;
    let result = blocking(move || {
        std::thread::sleep(std::time::Duration::from_millis(75));
        let frame = capture.grab(roblox).map_err(|e| e.to_string())?;
        let bbox = vision::find_bar(&frame, &palette)
            .ok_or("No fishing bar visible. Cast your line, wait for the bar, then try again.")?;
        let pad_x = (bbox.w() as f32 * 0.35).ceil() as i32;
        let pad_y = (bbox.h() as f32 * 0.15).ceil() as i32;
        let px = PxRect {
            x: roblox.x + bbox.x0 as i32 - pad_x,
            y: roblox.y + bbox.y0 as i32 - pad_y,
            w: bbox.w() as i32 + pad_x * 2,
            h: bbox.h() as i32 + pad_y * 2,
        };
        let x0 = px.x.max(roblox.x);
        let y0 = px.y.max(roblox.y);
        let x1 = px.right().min(roblox.right());
        let y1 = px.bottom().min(roblox.bottom());
        Ok(PxRect {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        }
        .to_rel(&roblox))
    })
    .await;
    windows::restore_after_capture(&app, &hidden);
    result
}

#[tauri::command]
pub fn hud_set_offset(
    app: AppHandle,
    st: State<'_, AppState>,
    offset: RelPoint,
) -> Result<(), String> {
    let mut s = st.settings.read().clone();
    s.ui.hud_offset = offset;
    settings_set(app, st, s)
}

#[tauri::command]
pub fn hud_toggle(app: AppHandle, st: State<'_, AppState>) -> Result<bool, String> {
    let mut s = st.settings.read().clone();
    s.ui.hud_visible = !s.ui.hud_visible;
    let v = s.ui.hud_visible;
    settings_set(app.clone(), st, s)?;
    windows::set_hud_visible(&app, v);
    Ok(v)
}

#[tauri::command]
pub fn panel_show(app: AppHandle, st: State<'_, AppState>) {
    st.panel_requested
        .store(true, std::sync::atomic::Ordering::SeqCst);
    windows::show_panel(&app);
}

#[tauri::command]
pub fn panel_toggle(app: AppHandle) {
    windows::toggle_panel(&app);
}

#[tauri::command]
pub fn panel_hide(app: AppHandle) {
    windows::hide_panel(&app);
}

#[tauri::command]
pub fn panel_visible(app: AppHandle) -> bool {
    windows::panel_visible(&app)
}

#[tauri::command]
pub fn guide_open(app: AppHandle) -> Result<(), String> {
    windows::show_guide(&app)
}

#[tauri::command]
pub fn guide_hide(app: AppHandle) {
    windows::hide_guide(&app);
}

#[tauri::command]
pub fn app_quit(app: AppHandle, st: State<'_, AppState>) {
    st.bot.stop();
    app.exit(0);
}

#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    if url != "https://www.roblox.com/games/1730877806/Grand-Piece-Online" {
        return Err("This external destination is not allowed".into());
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|_| "Could not open Roblox page".into())
}

#[tauri::command]
pub fn open_data_dir(app: AppHandle, st: State<'_, AppState>) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(st.store.dir().display().to_string(), None::<&str>)
        .map_err(|_| "Could not open application data directory".into())
}

#[tauri::command]
pub fn data_dir(st: State<'_, AppState>) -> String {
    st.store.dir().display().to_string()
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

fn encode_png(frame: &crate::core::types::Frame) -> Result<String, String> {
    use base64::Engine;
    use image::ImageEncoder;
    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new_with_quality(
        &mut out,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::NoFilter,
    )
    .write_image(
        &frame.rgba,
        frame.w as u32,
        frame.h as u32,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(out))
}
