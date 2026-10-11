use std::time::Duration;

use crate::core::types::{Key, MouseButton, PxPoint, RelPoint};
use crate::events::BotState;

use super::ctx::Ctx;

const WHEEL_STEP: i32 = 120;

pub fn fishing_point(ctx: &Ctx) -> Option<PxPoint> {
    let rect = ctx.roblox_rect()?;
    let s = ctx.settings.read();
    let rel = if s.features.auto_mouse_position {
        RelPoint { x: 0.5, y: 0.33 }
    } else {
        s.points.fishing
    };
    Some(rel.to_px(&rect))
}

fn rel_to_px(ctx: &Ctx, p: RelPoint) -> Option<PxPoint> {
    ctx.roblox_rect().map(|r| p.to_px(&r))
}

pub fn click(ctx: &Ctx, p: PxPoint) -> bool {
    let input = &ctx.platform.input;
    input.move_to(p);
    if !ctx.sleep_ms(40) {
        return false;
    }
    input.button(MouseButton::Left, true);
    if !ctx.sleep_ms(40) {
        input.button(MouseButton::Left, false);
        return false;
    }
    input.button(MouseButton::Left, false);
    true
}

pub fn right_click(ctx: &Ctx, p: PxPoint) -> bool {
    let input = &ctx.platform.input;
    input.move_to(p);
    if !ctx.sleep_ms(40) {
        return false;
    }
    input.button(MouseButton::Right, true);
    if !ctx.sleep_ms(40) {
        input.button(MouseButton::Right, false);
        return false;
    }
    input.button(MouseButton::Right, false);
    true
}

pub fn key_tap(ctx: &Ctx, k: Key) -> bool {
    ctx.platform.input.key(k, true);
    let ok = ctx.sleep_ms(30);
    ctx.platform.input.key(k, false);
    ok
}

pub fn key_hold(ctx: &Ctx, k: Key, d: Duration) -> bool {
    ctx.platform.input.key(k, true);
    let ok = ctx.sleep(d);
    ctx.platform.input.key(k, false);
    ok
}

pub fn wheel_steps(ctx: &Ctx, steps: u32, dir: i32, step_delay_ms: u32) -> bool {
    for _ in 0..steps {
        ctx.platform.input.wheel(WHEEL_STEP * dir);
        if !ctx.sleep_ms(step_delay_ms) {
            return false;
        }
    }
    true
}

fn click_pair(ctx: &Ctx, primary: RelPoint, backup: Option<RelPoint>, settle_ms: u32) -> bool {
    let Some(p) = rel_to_px(ctx, primary) else {
        return false;
    };
    if !click(ctx, p) || !ctx.sleep_ms(settle_ms) {
        return false;
    }
    if let Some(b) = backup.and_then(|b| rel_to_px(ctx, b)) {
        if !click(ctx, b) || !ctx.sleep_ms(settle_ms) {
            return false;
        }
        if !click(ctx, p) || !ctx.sleep_ms(settle_ms) {
            return false;
        }
    }
    true
}

pub fn select_bait(ctx: &Ctx) -> bool {
    let s = ctx.settings();
    if !s.features.auto_bait {
        return true;
    }
    let Some(primary) = s.points.bait[0] else {
        ctx.log_warn("Auto bait enabled but bait point not set");
        return true;
    };
    ctx.log_debug("Selecting bait");
    click_pair(ctx, primary, s.points.bait[1], 300)
}

pub fn zoom_reset(ctx: &Ctx) -> bool {
    let z = ctx.settings().zoom;
    ctx.log_debug("Zoom: resetting camera");
    if let Some(p) = fishing_point(ctx) {
        ctx.platform.input.move_to(p);
        if !ctx.sleep_ms(120) {
            return false;
        }
    }
    if !wheel_steps(ctx, z.out_steps, -1, z.step_delay_ms) {
        return false;
    }
    if !ctx.sleep_ms(z.sequence_delay_ms) {
        return false;
    }
    wheel_steps(ctx, z.in_steps, 1, z.step_delay_ms)
}

pub fn initial_setup(ctx: &Ctx) -> bool {
    ctx.set_state(BotState::InitialSetup, None);
    let s = ctx.settings();
    if s.features.auto_zoom && (!zoom_reset(ctx) || !ctx.sleep_ms(800)) {
        return false;
    }
    if s.features.auto_purchase && !purchase(ctx) {
        return false;
    }
    if s.features.auto_bait {
        if !key_tap(ctx, Key::Char(s.keys.rod)) || !ctx.sleep_ms(500) {
            return false;
        }
        if !select_bait(ctx) {
            return false;
        }
    }
    ctx.sleep_ms(1000)
}

pub fn cast(ctx: &Ctx) -> bool {
    ctx.set_state(BotState::Casting, None);
    let Some(p) = fishing_point(ctx) else {
        ctx.log_warn("Roblox window not found; cannot cast");
        return false;
    };
    let hold = ctx.settings.read().fishing.cast_hold_ms;
    ctx.platform.input.move_to(p);
    if !ctx.sleep_ms(80) {
        return false;
    }
    if !right_click(ctx, p) || !ctx.sleep_ms(250) {
        return false;
    }
    ctx.hold_mouse(true);
    let ok = ctx.sleep_ms(hold);
    ctx.hold_mouse(false);
    ok
}

pub fn purchase(ctx: &Ctx) -> bool {
    let s = ctx.settings();
    let (Some(confirm), Some(quantity)) = (
        s.points.purchase[0].and_then(|p| rel_to_px(ctx, p)),
        s.points.purchase[1].and_then(|p| rel_to_px(ctx, p)),
    ) else {
        ctx.log_warn("Auto purchase: confirm and quantity points not both set");
        return true;
    };
    ctx.set_state(BotState::Purchasing, None);
    ctx.log_info(&format!("Buying {} bait", s.purchase.amount));
    let p = &s.purchase;
    let delay = p.click_delay_ms;

    ctx.ensure_roblox_focus();
    if !key_hold(
        ctx,
        Key::Char(s.keys.shop),
        Duration::from_millis(p.hold_shop_key_ms as u64),
    ) {
        return false;
    }
    if !ctx.sleep_ms(p.after_key_ms) {
        return false;
    }
    if !click(ctx, confirm) || !ctx.sleep_ms(delay) {
        return false;
    }
    if !click(ctx, quantity) || !ctx.sleep_ms(delay + 300) {
        return false;
    }
    ctx.platform.input.key(Key::Control, true);
    key_tap(ctx, Key::Char('a'));
    ctx.platform.input.key(Key::Control, false);
    if !ctx.sleep_ms(100) {
        return false;
    }
    key_tap(ctx, Key::Delete);
    if !ctx.sleep_ms(100) {
        return false;
    }
    for c in p.amount.to_string().chars() {
        if !key_tap(ctx, Key::Char(c)) || !ctx.sleep_ms(50) {
            return false;
        }
    }
    if !ctx.sleep_ms(p.after_type_ms) {
        return false;
    }
    if !click(ctx, confirm) || !ctx.sleep_ms(delay) {
        return false;
    }
    if let Some(cancel) = s.points.purchase[2].and_then(|p| rel_to_px(ctx, p)) {
        if !click(ctx, cancel) || !ctx.sleep_ms(delay) {
            return false;
        }
    }
    if !click(ctx, quantity) || !ctx.sleep_ms(delay) {
        return false;
    }
    if let Some(fp) = fishing_point(ctx) {
        if !right_click(ctx, fp) || !ctx.sleep_ms(delay) {
            return false;
        }
    }
    {
        let mut sess = ctx.session.lock();
        sess.bait_purchased += p.amount;
        sess.since_purchase = 0;
    }
    ctx.emit_stats();
    ctx.emit(crate::events::BotEvent::Purchase { amount: p.amount });
    if s.webhook.purchase {
        ctx.webhook.purchase(p.amount);
    }
    true
}

pub fn store_fruit(ctx: &Ctx) -> bool {
    let s = ctx.settings();
    if !s.features.fruit_storage {
        return true;
    }
    let Some(fruit_primary) = s.points.fruit[0] else {
        ctx.log_warn("Fruit storage enabled but fruit point not set");
        return true;
    };
    ctx.set_state(BotState::StoringFruit, None);
    ctx.log_info("Storing fruit");
    let fs = &s.fruit_storage;

    if let Some(fp) = fishing_point(ctx) {
        if !click(ctx, fp) || !ctx.sleep_ms(1000) {
            return false;
        }
        if !click(ctx, fp) || !ctx.sleep_ms(1000) {
            return false;
        }
    }

    for slot in [s.keys.fruit_slot_1, s.keys.fruit_slot_2] {
        if !key_tap(ctx, Key::Char(slot)) || !ctx.sleep_ms(fs.key_settle_ms) {
            return false;
        }
        if !click_pair(ctx, fruit_primary, s.points.fruit[1], fs.click_settle_ms) {
            return false;
        }
        if !ctx.sleep_ms(fs.dialog_wait_ms) {
            return false;
        }
        if !key_hold(ctx, Key::Backspace, Duration::from_millis(100)) {
            return false;
        }
        if !ctx.sleep_ms(fs.after_drop_ms) {
            return false;
        }
    }

    if !ctx.sleep_ms(1000) {
        return false;
    }
    if !key_tap(ctx, Key::Char(s.keys.rod)) || !ctx.sleep_ms(800) {
        return false;
    }
    if !select_bait(ctx) {
        return false;
    }
    if let Some(fp) = fishing_point(ctx) {
        ctx.platform.input.move_to(fp);
    }
    ctx.sleep_ms(300)
}
