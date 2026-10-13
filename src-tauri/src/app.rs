use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use tauri::{AppHandle, Emitter, Manager};

use crate::bot::Bot;
use crate::config::{Settings, Store};
use crate::core::platform::{self, Platform};
use crate::core::types::WindowInfo;
use crate::events::BotEvent;
use crate::webhook::WebhookQueue;
use crate::{hotkeys, tray, windows};

pub struct AppState {
    pub platform: Platform,
    pub settings: Arc<RwLock<Settings>>,
    pub roblox: Arc<RwLock<Option<WindowInfo>>>,
    pub store: Arc<Store>,
    pub bot: Arc<Bot>,
    pub webhook: Arc<WebhookQueue>,
    pub overlay_session: Mutex<Option<serde_json::Value>>,
    pub resume_after_overlay: AtomicBool,
    pub panel_requested: AtomicBool,
    events_tx: Sender<BotEvent>,
    events_rx: Mutex<Option<Receiver<BotEvent>>>,
}

pub fn data_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("autofish-local")
}

pub fn build_state() -> AppState {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    init_logging(&dir);

    let store = Arc::new(Store::new(dir));
    let settings = Arc::new(RwLock::new(store.load()));
    let platform = platform::build();
    let roblox = Arc::new(RwLock::new(platform.window.find()));
    let webhook = WebhookQueue::start(Arc::clone(&settings));
    let (tx, rx) = bounded::<BotEvent>(512);
    let bot = Bot::new(
        platform.clone(),
        Arc::clone(&settings),
        Arc::clone(&roblox),
        tx.clone(),
        Arc::clone(&webhook),
        Arc::clone(&store),
    );

    AppState {
        platform,
        settings,
        roblox,
        store,
        bot,
        webhook,
        overlay_session: Mutex::new(None),
        resume_after_overlay: AtomicBool::new(false),
        panel_requested: AtomicBool::new(false),
        events_tx: tx,
        events_rx: Mutex::new(Some(rx)),
    }
}

pub fn setup(app: &AppHandle, st: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(rx) = st.events_rx.lock().take() {
        spawn_event_forwarder(app.clone(), rx);
    }
    spawn_roblox_watcher(
        st.platform.clone(),
        Arc::clone(&st.roblox),
        st.events_tx.clone(),
        Arc::clone(&st.bot),
    );
    spawn_panel_fallback(app.clone());

    tray::build(app)?;
    hotkeys::register(app, &st.settings.read().hotkeys)?;
    windows::init(app);
    tracing::info!("AutoFish Local started");
    Ok(())
}

fn init_logging(dir: &std::path::Path) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    let file = tracing_appender::rolling::daily(dir.join("logs"), "gpo-autofish.log");
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(file).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stderr))
        .try_init();
}

fn spawn_event_forwarder(app: AppHandle, rx: Receiver<BotEvent>) {
    std::thread::Builder::new()
        .name("event-forwarder".into())
        .spawn(move || {
            for ev in rx.iter() {
                // These channels use nullable payloads in the frontend. An
                // internally tagged enum cannot serialize a None newtype.
                let emitted = match &ev {
                    BotEvent::Reading(value) => app.emit(ev.channel(), value),
                    BotEvent::Roblox(value) => app.emit(ev.channel(), value),
                    _ => app.emit(ev.channel(), &ev),
                };
                if let Err(error) = emitted {
                    tracing::warn!("event delivery failed on {}: {error}", ev.channel());
                }
                match &ev {
                    BotEvent::State { state, .. } => {
                        if state.is_active() && app.state::<AppState>().roblox.read().is_some() {
                            // Controls must not cover the pixels used by GDI tracking.
                            windows::hide_panel(&app);
                            windows::hide_guide(&app);
                        }
                        tray::update(&app, *state);
                    }
                    BotEvent::Roblox(info) => windows::on_roblox_changed(&app, *info),
                    _ => {}
                }
            }
        })
        .expect("spawn event forwarder");
}

fn spawn_panel_fallback(app: AppHandle) {
    std::thread::Builder::new()
        .name("panel-fallback".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_secs(8));
            let st = app.state::<AppState>();
            if !st.panel_requested.load(std::sync::atomic::Ordering::SeqCst) {
                tracing::warn!("frontend did not report ready; showing panel anyway");
                windows::show_panel(&app);
            }
        })
        .expect("spawn panel fallback");
}

fn spawn_roblox_watcher(
    platform: Platform,
    roblox: Arc<RwLock<Option<WindowInfo>>>,
    tx: Sender<BotEvent>,
    bot: Arc<Bot>,
) {
    std::thread::Builder::new()
        .name("roblox-watcher".into())
        .spawn(move || {
            let mut last: Option<WindowInfo> = None;
            let mut first = true;
            let mut last_ui_focus = false;
            loop {
                let now = platform.window.find();
                let ui_focus = windows::ui_is_foreground();
                if !now.is_some_and(|w| w.visible && w.is_foreground) {
                    platform.input.release_all();
                    if last.is_some_and(|w| w.visible && w.is_foreground) && bot.is_running() {
                        bot.pause();
                    }
                }
                if first || now != last || ui_focus != last_ui_focus {
                    first = false;
                    *roblox.write() = now;
                    let _ = tx.try_send(BotEvent::Roblox(now));
                    last = now;
                    last_ui_focus = ui_focus;
                }
                std::thread::sleep(Duration::from_millis(if now.is_some() { 50 } else { 500 }));
            }
        })
        .expect("spawn roblox watcher");
}
