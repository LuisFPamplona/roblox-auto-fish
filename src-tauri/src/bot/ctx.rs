use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use parking_lot::{Mutex, RwLock};

use crate::config::{Settings, Store};
use crate::core::platform::Platform;
use crate::core::types::{MouseButton, PxRect, WindowInfo};
use crate::events::{now_ms, BotEvent, BotState, LogLevel, LogLine};
use crate::webhook::WebhookQueue;

use super::session::Session;

pub struct Ctx {
    pub platform: Platform,
    pub settings: Arc<RwLock<Settings>>,
    pub roblox: Arc<RwLock<Option<WindowInfo>>>,
    pub events: Sender<BotEvent>,
    pub webhook: Arc<WebhookQueue>,
    pub store: Arc<Store>,
    pub session: Mutex<Session>,
    pub running: AtomicBool,
    pub mouse_held: AtomicBool,
    heartbeat_ms: AtomicU64,
    state: Mutex<(BotState, Instant)>,
    start: Instant,
}

impl Ctx {
    pub fn new(
        platform: Platform,
        settings: Arc<RwLock<Settings>>,
        roblox: Arc<RwLock<Option<WindowInfo>>>,
        events: Sender<BotEvent>,
        webhook: Arc<WebhookQueue>,
        store: Arc<Store>,
    ) -> Self {
        let session = Session::with_base(store.load_stats());
        Self {
            platform,
            settings,
            roblox,
            events,
            webhook,
            store,
            session: Mutex::new(session),
            running: AtomicBool::new(false),
            mouse_held: AtomicBool::new(false),
            heartbeat_ms: AtomicU64::new(0),
            state: Mutex::new((BotState::Stopped, Instant::now())),
            start: Instant::now(),
        }
    }

    pub fn alive(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn touch(&self) {
        self.heartbeat_ms
            .store(self.start.elapsed().as_millis() as u64, Ordering::Relaxed);
    }

    pub fn heartbeat_age(&self) -> Duration {
        let now = self.start.elapsed().as_millis() as u64;
        Duration::from_millis(now.saturating_sub(self.heartbeat_ms.load(Ordering::Relaxed)))
    }

    pub fn state(&self) -> BotState {
        self.state.lock().0
    }

    pub fn state_age(&self) -> Duration {
        self.state.lock().1.elapsed()
    }

    pub fn set_state(&self, s: BotState, detail: Option<String>) {
        if !self.alive() && !matches!(s, BotState::Paused | BotState::Stopped) {
            return;
        }
        {
            let mut cur = self.state.lock();
            if cur.0 == s && detail.is_none() {
                return;
            }
            *cur = (s, Instant::now());
        }
        self.touch();
        self.emit(BotEvent::State { state: s, detail });
    }

    pub fn emit(&self, e: BotEvent) {
        let _ = self.events.try_send(e);
    }

    pub fn emit_stats(&self) {
        let stats = self.session.lock().stats();
        if let Err(e) = self.store.save_stats(&stats.total) {
            tracing::warn!("stats save: {e}");
        }
        self.emit(BotEvent::Stats(stats));
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        match level {
            LogLevel::Debug => tracing::debug!("{msg}"),
            LogLevel::Info => tracing::info!("{msg}"),
            LogLevel::Warn => tracing::warn!("{msg}"),
            LogLevel::Error => tracing::error!("{msg}"),
        }
        self.emit(BotEvent::Log(LogLine {
            ts: now_ms(),
            level,
            msg: msg.to_string(),
        }));
    }

    pub fn log_debug(&self, m: &str) {
        self.log(LogLevel::Debug, m)
    }
    pub fn log_info(&self, m: &str) {
        self.log(LogLevel::Info, m)
    }
    pub fn log_warn(&self, m: &str) {
        self.log(LogLevel::Warn, m)
    }
    pub fn log_error(&self, m: &str) {
        self.log(LogLevel::Error, m)
    }

    pub fn roblox_rect(&self) -> Option<PxRect> {
        self.roblox.read().map(|w| w.client)
    }

    pub fn roblox_in_front(&self) -> bool {
        self.platform
            .window
            .find()
            .is_some_and(|w| w.visible && w.is_foreground)
    }

    pub fn ensure_roblox_focus(&self) -> bool {
        if !self.roblox_in_front() {
            return false;
        }
        true
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().clone()
    }

    pub fn sleep(&self, d: Duration) -> bool {
        let end = Instant::now() + d;
        while Instant::now() < end {
            if !self.alive() {
                return false;
            }
            let left = end.saturating_duration_since(Instant::now());
            std::thread::sleep(left.min(Duration::from_millis(25)));
            self.touch();
        }
        self.alive()
    }

    pub fn sleep_ms(&self, ms: u32) -> bool {
        self.sleep(Duration::from_millis(ms as u64))
    }

    pub fn hold_mouse(&self, down: bool) {
        if self.mouse_held.swap(down, Ordering::SeqCst) != down {
            self.platform.input.button(MouseButton::Left, down);
        }
    }

    pub fn release_mouse(&self) {
        if self.mouse_held.swap(false, Ordering::SeqCst) {
            self.platform.input.button(MouseButton::Left, false);
        }
    }
}
