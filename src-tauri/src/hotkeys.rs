use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::app::AppState;
use crate::config::Hotkeys;
use crate::windows;

#[derive(Clone, Copy)]
enum Action {
    Toggle,
    Overlay,
    Quit,
    HideHud,
    Stop,
}

pub fn validate(keys: &Hotkeys) -> Result<(), String> {
    let mut seen: Vec<Shortcut> = Vec::new();
    for combo in [&keys.toggle, &keys.overlay, &keys.quit, &keys.hide_hud] {
        if combo.trim().is_empty() {
            continue;
        }
        let shortcut: Shortcut = combo.parse().map_err(|_| "Invalid hotkey")?;
        let emergency: Shortcut = "Ctrl+Alt+F12"
            .parse()
            .map_err(|_| "Invalid emergency shortcut")?;
        if shortcut == emergency || seen.contains(&shortcut) {
            return Err(
                "Hotkeys must be distinct; Ctrl+Alt+F12 is reserved for emergency stop".into(),
            );
        }
        seen.push(shortcut);
    }
    Ok(())
}

pub fn register(app: &AppHandle, keys: &Hotkeys) -> Result<(), String> {
    validate(keys)?;
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|e| e.to_string())?;
    let bindings = [
        ("Ctrl+Alt+F12", Action::Stop),
        (keys.toggle.as_str(), Action::Toggle),
        (keys.overlay.as_str(), Action::Overlay),
        (keys.quit.as_str(), Action::Quit),
        (keys.hide_hud.as_str(), Action::HideHud),
    ];
    for (combo, action) in bindings {
        if combo.trim().is_empty() {
            continue;
        }
        let shortcut: Shortcut = combo
            .parse()
            .map_err(|_| format!("Invalid hotkey: {combo}"))?;
        gs.on_shortcut(shortcut, move |app, _sc, ev| {
            if ev.state() != ShortcutState::Pressed {
                return;
            }
            dispatch(app, action);
        })
        .map_err(|e| format!("Could not bind {combo}: {e}"))?;
    }
    Ok(())
}

fn dispatch(app: &AppHandle, action: Action) {
    let st = app.state::<AppState>();
    match action {
        Action::Toggle => {
            st.bot.toggle();
        }
        Action::Overlay => {
            if windows::overlay_visible(app) {
                windows::hide_overlay(app);
            } else if let Err(e) = crate::commands::open_regions_editor(app) {
                tracing::warn!("regions editor: {e}");
            }
        }
        Action::Quit => {
            st.bot.stop();
            app.exit(0);
        }
        Action::Stop => {
            st.bot.ctx().platform.input.set_enabled(false);
            st.bot.stop();
        }
        Action::HideHud => {
            let mut s = st.settings.read().clone();
            s.ui.hud_visible = !s.ui.hud_visible;
            let v = s.ui.hud_visible;
            *st.settings.write() = s.clone();
            let _ = st.store.save(&s);
            windows::set_hud_visible(app, v);
        }
    }
}
