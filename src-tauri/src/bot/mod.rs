pub mod actions;
pub mod ctx;
pub mod machine;
pub mod session;
pub mod trace;
pub mod watchdog;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::thread::JoinHandle;

use crossbeam_channel::Sender;
use parking_lot::{Mutex, RwLock};

use crate::config::{Settings, Store};
use crate::core::platform::Platform;
use crate::core::types::WindowInfo;
use crate::events::{BotEvent, BotState};
use crate::webhook::WebhookQueue;
use ctx::Ctx;

pub struct Bot {
    ctx: Arc<Ctx>,
    thread: Mutex<Option<JoinHandle<()>>>,
    watchdog: Mutex<Option<JoinHandle<()>>>,
    paused: AtomicBool,
    lifecycle: Mutex<()>,
}

fn join_unless_self(h: JoinHandle<()>) {
    if h.thread().id() != std::thread::current().id() {
        let _ = h.join();
    }
}

impl Bot {
    pub fn new(
        platform: Platform,
        settings: Arc<RwLock<Settings>>,
        roblox: Arc<RwLock<Option<WindowInfo>>>,
        events: Sender<BotEvent>,
        webhook: Arc<WebhookQueue>,
        store: Arc<Store>,
    ) -> Arc<Self> {
        let ctx = Arc::new(Ctx::new(platform, settings, roblox, events, webhook, store));
        Arc::new(Self {
            ctx,
            thread: Mutex::new(None),
            watchdog: Mutex::new(None),
            paused: AtomicBool::new(false),
            lifecycle: Mutex::new(()),
        })
    }

    pub fn ctx(&self) -> &Arc<Ctx> {
        &self.ctx
    }

    pub fn is_running(&self) -> bool {
        self.ctx.running.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn state(&self) -> BotState {
        self.ctx.state()
    }

    pub fn start(self: &Arc<Self>) {
        let _lifecycle = self.lifecycle.lock();
        if self.is_running() {
            return;
        }
        self.halt();
        let resume = self.paused.swap(false, Ordering::SeqCst);
        if resume {
            self.ctx.session.lock().resume();
        } else {
            let mut sess = self.ctx.session.lock();
            *sess = sess.next_session();
        }
        self.ctx.emit_stats();
        self.spawn_loop(resume);
        self.spawn_watchdog();
        self.ctx
            .log_info(if resume { "Resumed" } else { "Started" });
    }

    pub fn pause(&self) {
        let _lifecycle = self.lifecycle.lock();
        self.ctx.platform.input.set_enabled(false);
        if !self.is_running() {
            return;
        }
        self.paused.store(true, Ordering::SeqCst);
        self.ctx.running.store(false, Ordering::SeqCst);
        self.ctx.set_state(BotState::Paused, None);
        self.ctx.emit(BotEvent::Reading(None));
        self.ctx.log_info("Paused");
        self.halt();
        self.ctx.session.lock().pause();
        self.ctx.emit_stats();
    }

    pub fn stop(&self) {
        let _lifecycle = self.lifecycle.lock();
        self.ctx.platform.input.set_enabled(false);
        let was = self.is_running() || self.is_paused();
        self.paused.store(false, Ordering::SeqCst);
        self.ctx.running.store(false, Ordering::SeqCst);
        self.ctx.set_state(BotState::Stopped, None);
        self.ctx.emit(BotEvent::Reading(None));
        if was {
            self.ctx.log_info("Stopped");
        }
        self.halt();
        self.ctx.session.lock().pause();
        self.ctx.emit_stats();
    }

    pub fn toggle(self: &Arc<Self>) {
        if self.is_running() {
            self.pause();
        } else {
            self.start();
        }
    }

    fn halt(&self) {
        self.ctx.running.store(false, Ordering::SeqCst);
        if let Some(h) = self.watchdog.lock().take() {
            join_unless_self(h);
        }
        if let Some(h) = self.thread.lock().take() {
            join_unless_self(h);
        }
        self.ctx.release_mouse();
        self.ctx.emit(BotEvent::Reading(None));
    }

    fn spawn_loop(&self, skip_setup: bool) {
        self.ctx.running.store(true, Ordering::SeqCst);
        self.ctx.platform.input.set_enabled(true);
        self.ctx.touch();
        let ctx = Arc::clone(&self.ctx);
        let h = std::thread::Builder::new()
            .name("bot-loop".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    machine::run(&ctx, skip_setup);
                }));
                ctx.release_mouse();
                ctx.platform.input.set_enabled(false);
                ctx.running.store(false, Ordering::SeqCst);
                ctx.emit(BotEvent::Reading(None));
                ctx.session.lock().pause();
                ctx.emit_stats();
                if result.is_ok() && !matches!(ctx.state(), BotState::Paused | BotState::Stopped) {
                    ctx.set_state(BotState::Stopped, Some("Loop ended; start manually".into()));
                }
                if result.is_err() {
                    ctx.log_error("Bot loop panicked; stopped");
                    ctx.running.store(false, Ordering::SeqCst);
                    ctx.set_state(BotState::Stopped, Some("panic".into()));
                }
            })
            .expect("spawn bot loop");
        *self.thread.lock() = Some(h);
    }

    fn spawn_watchdog(self: &Arc<Self>) {
        if !self.ctx.settings.read().watchdog.enabled {
            return;
        }
        let ctx = Arc::clone(&self.ctx);
        let weak: Weak<Bot> = Arc::downgrade(self);
        let on_stuck: Box<dyn FnOnce(String) + Send> = Box::new(move |reason| {
            if let Some(bot) = weak.upgrade() {
                bot.ctx.platform.input.set_enabled(false);
                std::thread::spawn(move || bot.restart_loop(&reason));
            }
        });
        let h = std::thread::Builder::new()
            .name("bot-watchdog".into())
            .spawn(move || watchdog::run(&ctx, on_stuck))
            .expect("spawn watchdog");
        *self.watchdog.lock() = Some(h);
    }

    pub fn restart_loop(self: &Arc<Self>, reason: &str) {
        let attempt = {
            let mut s = self.ctx.session.lock();
            s.restarts += 1;
            s.restarts
        };
        self.ctx.emit(BotEvent::Recovery {
            attempt,
            reason: reason.to_string(),
        });
        self.ctx.webhook.recovery(attempt, reason);
        self.ctx
            .log_error("Watchdog stopped automation; review the game and start manually");
        self.stop();
    }
}
