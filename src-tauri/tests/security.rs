use gpo_autofish_lib::config::{Hotkeys, Settings};
use gpo_autofish_lib::core::types::{PxRect, RelPoint, RelRect};
use gpo_autofish_lib::{hotkeys, webhook};

#[test]
fn configuration_rejects_capture_outside_game_and_invalid_timings() {
    let mut settings = Settings::default();
    assert!(settings.validate().is_ok());
    settings.regions.bar.x = 0.99;
    assert!(settings.validate().is_err());
    settings = Settings::default();
    settings.fishing.track_timeout_s = -1.0;
    assert!(settings.validate().is_err());
    settings = Settings::default();
    settings.fishing.scan_hz = 100_000;
    assert!(settings.validate().is_err());
    settings = Settings::default();
    settings.ocr.spawn_check_interval_s = f32::NAN;
    assert!(settings.validate().is_err());
}

#[test]
fn points_on_normalized_edge_stay_inside_client_pixels() {
    let base = PxRect {
        x: -100,
        y: 50,
        w: 100,
        h: 100,
    };
    let point = RelPoint { x: 1.0, y: 1.0 }.to_px(&base);
    assert_eq!(point.x, -1);
    assert_eq!(point.y, 149);
    assert!(RelRect {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 1.0
    }
    .validate()
    .is_err());
}

#[test]
fn emergency_binding_cannot_be_removed_or_shadowed() {
    let mut keys = Hotkeys::default();
    keys.toggle = "Ctrl+Alt+F12".into();
    assert!(hotkeys::validate(&keys).is_err());
    keys = Hotkeys::default();
    keys.quit = keys.toggle.clone();
    assert!(hotkeys::validate(&keys).is_err());
}

#[test]
fn webhooks_require_explicit_enablement_and_valid_destination() {
    let mut settings = Settings::default();
    assert!(!settings.auto_update);
    assert!(!settings.webhook.enabled);
    settings.webhook.url = "http://127.0.0.1/private".into();
    settings.webhook.enabled = true;
    assert!(settings.validate().is_err());
    assert!(webhook::WebhookQueue::disabled().test().is_err());
}
