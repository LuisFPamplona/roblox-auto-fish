use crate::core::platform::{Ocr, PlatformError, Result};
use crate::core::types::Frame;
use std::cell::RefCell;
use std::sync::OnceLock;
use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};

// Each thread owns its apartment and engine. No unsafe Send/Sync assertions.
struct ThreadOcr {
    engine: Option<OcrEngine>,
    initialized: bool,
}
impl ThreadOcr {
    fn new() -> Self {
        let initialized = unsafe { RoInitialize(RO_INIT_MULTITHREADED).is_ok() };
        let engine = if initialized {
            OcrEngine::TryCreateFromUserProfileLanguages()
                .ok()
                .or_else(|| {
                    let language = Language::CreateLanguage(&HSTRING::from("en-US")).ok()?;
                    OcrEngine::TryCreateFromLanguage(&language).ok()
                })
        } else {
            None
        };
        Self {
            engine,
            initialized,
        }
    }
}
impl Drop for ThreadOcr {
    fn drop(&mut self) {
        self.engine.take();
        if self.initialized {
            unsafe {
                RoUninitialize();
            }
        }
    }
}
thread_local! { static ENGINE: RefCell<ThreadOcr> = RefCell::new(ThreadOcr::new()); }

#[derive(Default)]
pub struct WindowsOcr {
    available: OnceLock<bool>,
}
impl WindowsOcr {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Ocr for WindowsOcr {
    fn available(&self) -> bool {
        // Tauri's UI thread can already belong to an STA. Check availability
        // on a dedicated MTA without changing the UI apartment or sharing COM.
        *self.available.get_or_init(|| {
            std::thread::spawn(|| ENGINE.with(|e| e.borrow().engine.is_some()))
                .join()
                .unwrap_or(false)
        })
    }
    fn read(&self, frame: &Frame) -> Result<String> {
        if frame.w < 8 || frame.h < 8 {
            return Ok(String::new());
        }
        ENGINE.with(|state| {
            let state = state.borrow();
            let engine = state.engine.as_ref().ok_or(PlatformError::OcrUnavailable)?;
            let err = |e: windows::core::Error| PlatformError::Ocr(e.message());
            let max = OcrEngine::MaxImageDimension().map_err(err)? as usize;
            let frame = frame.downscale(max.max(1));
            let mut bgra = frame.rgba;
            for px in bgra.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
            let writer = DataWriter::new().map_err(err)?;
            writer.WriteBytes(&bgra).map_err(err)?;
            let buffer = writer.DetachBuffer().map_err(err)?;
            let bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
                &buffer,
                BitmapPixelFormat::Bgra8,
                frame.w as i32,
                frame.h as i32,
                BitmapAlphaMode::Ignore,
            )
            .map_err(err)?;
            let result = engine
                .RecognizeAsync(&bitmap)
                .map_err(err)?
                .join()
                .map_err(err)?;
            Ok(result.Text().map_err(err)?.to_string())
        })
    }
}
