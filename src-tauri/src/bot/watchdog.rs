use std::time::Duration;

use crate::events::BotState;

use super::ctx::Ctx;

pub fn max_state_age(s: BotState) -> Duration {
    Duration::from_secs(match s {
        BotState::InitialSetup => 45,
        BotState::Casting => 20,
        BotState::WaitingForBite => 90,
        BotState::Tracking => 90,
        BotState::PostCatch => 30,
        BotState::Purchasing => 60,
        BotState::StoringFruit => 45,
        BotState::Recovering => 30,
        BotState::WaitingForRoblox | BotState::Stopped | BotState::Paused => 0,
    })
}

pub enum Verdict {
    Ok,
    Stuck(String),
}

pub fn check(ctx: &Ctx) -> Verdict {
    let hb = Duration::from_secs_f32(ctx.settings.read().watchdog.heartbeat_timeout_s);
    let age = ctx.heartbeat_age();
    if age > hb {
        return Verdict::Stuck(format!("no heartbeat for {}s", age.as_secs()));
    }
    let state = ctx.state();
    let max = max_state_age(state);
    if !max.is_zero() && ctx.state_age() > max {
        return Verdict::Stuck(format!("{state:?} exceeded {}s", max.as_secs()));
    }
    Verdict::Ok
}

pub fn run(ctx: &Ctx, on_stuck: Box<dyn FnOnce(String) + Send>) {
    while ctx.alive() {
        for _ in 0..20 {
            if !ctx.alive() {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if !ctx.alive() {
            return;
        }
        if let Verdict::Stuck(reason) = check(ctx) {
            ctx.log_error(&format!("Watchdog: {reason}"));
            on_stuck(reason);
            return;
        }
    }
}
