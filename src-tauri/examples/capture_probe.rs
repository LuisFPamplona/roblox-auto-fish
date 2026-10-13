//! Local diagnostics only: prints capture metadata, never images or OCR text.
use gpo_autofish_lib::config::Settings;
use gpo_autofish_lib::core::platform::windows::{
    capture::GdiCapture, wgc::WindowCapture, window::Win32Window,
};
use gpo_autofish_lib::core::platform::{Capture, GameWindow};
use gpo_autofish_lib::core::vision;
use std::time::Duration;

fn main() {
    let path = dirs::config_dir()
        .unwrap()
        .join("autofish-local/settings.json");
    let settings: Settings = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    settings.validate().unwrap();
    let window = Win32Window::new("Roblox");
    let capture: Box<dyn Capture> = if std::env::args().any(|arg| arg == "--wgc") {
        Box::new(WindowCapture::new())
    } else {
        Box::new(GdiCapture::new())
    };
    let watch = std::env::args().any(|arg| arg == "--watch");
    if !watch {
        std::thread::sleep(Duration::from_secs(8));
    }
    for _ in 0..if watch { 120 } else { 8 } {
        match window.find() {
            None => println!("Roblox not found"),
            Some(info) => {
                println!(
                    "client={:?}, foreground={}",
                    info.client, info.is_foreground
                );
                for (name, region) in [
                    ("bar", settings.regions.bar),
                    ("drop", settings.regions.drop),
                ] {
                    match capture.grab(region.to_px(&info.client)) {
                        Err(error) => println!("{name}: {error}"),
                        Ok(frame) => {
                            let black = frame
                                .rgba
                                .as_chunks::<4>()
                                .0
                                .iter()
                                .filter(|p| p[0] == 0 && p[1] == 0 && p[2] == 0)
                                .count();
                            println!(
                                "{name}: {}x{}, black={:.1}%, confidence={:?}, bar_detected={}",
                                frame.w,
                                frame.h,
                                black as f64 * 100.0 / (frame.w * frame.h) as f64,
                                vision::confidence(&frame, &settings.fishing.palette),
                                vision::has_bar(&frame, &settings.fishing.palette)
                            );
                        }
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
