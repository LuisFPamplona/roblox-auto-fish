#![cfg(windows)]

use gpo_autofish_lib::core::platform::windows::{
    capture::GdiCapture, ocr::WindowsOcr, wgc::WindowCapture,
};
use gpo_autofish_lib::core::platform::{Capture, Ocr};
use gpo_autofish_lib::core::types::{Frame, PxRect};

#[test]
fn capture_rejects_invalid_region_without_desktop_fallback() {
    let capture = GdiCapture::new();
    assert!(capture.grab(PxRect::default()).is_err());
    assert!(capture
        .grab(PxRect {
            x: 0,
            y: 0,
            w: 100_000,
            h: 100_000
        })
        .is_err());
}

#[test]
fn wgc_rejects_invalid_requests_and_worker_shuts_down() {
    let capture = WindowCapture::new();
    assert!(capture.grab(PxRect::default()).is_err());
    assert!(capture
        .grab(PxRect {
            x: 0,
            y: 0,
            w: 100_000,
            h: 100_000
        })
        .is_err());
    capture.release();
    drop(capture);
}

// Run explicitly with AUTOFISH_OCR_FIXTURE pointing at a synthetic image.
// This tests native WinRT, not recognition inside Roblox/GPO.
#[test]
#[ignore = "requires an installed Windows OCR language and synthetic fixture"]
fn winrt_recognizes_synthetic_text_on_independent_threads() {
    let path = std::env::var("AUTOFISH_OCR_FIXTURE").expect("synthetic OCR fixture path");
    let pixels = image::open(path).expect("synthetic fixture").to_rgba8();
    let frame = Frame::new(
        pixels.width() as usize,
        pixels.height() as usize,
        pixels.into_raw(),
    );
    eprintln!("Checking OCR availability");
    let ocr = std::sync::Arc::new(WindowsOcr::new());
    assert!(ocr.available(), "Windows OCR language unavailable");
    let jobs: Vec<_> = (0..2)
        .map(|_| {
            let frame = frame.clone();
            let ocr = ocr.clone();
            std::thread::spawn(move || {
                eprintln!("OCR worker entering read");
                let text = ocr.read(&frame).expect("native OCR read");
                eprintln!("OCR worker completed read: {text}");
                assert!(
                    text.to_lowercase().contains("devil fruit"),
                    "OCR result: {text}"
                );
            })
        })
        .collect();
    for job in jobs {
        job.join().expect("OCR worker");
    }
}
