use crate::core::platform::{Ocr, PlatformError, Result};
use crate::core::types::Frame;
use std::cell::RefCell;
use std::sync::OnceLock;
use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::System::Com::{CoDecrementMTAUsage, CoIncrementMTAUsage, CO_MTA_USAGE_COOKIE};
use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};

// Each thread owns its apartment and engine. No unsafe Send/Sync assertions.
// The API explicitly allows its opaque usage cookie to be released on another
// thread. It is not a COM object or an engine; no Send/Sync assertion is used.
pub(super) struct MtaLease(usize);
impl MtaLease {
    pub(super) fn acquire() -> Option<Self> {
        unsafe {
            CoIncrementMTAUsage()
                .ok()
                .map(|cookie| Self(cookie.0 as usize))
        }
    }
}
impl Drop for MtaLease {
    fn drop(&mut self) {
        unsafe {
            let _ = CoDecrementMTAUsage(CO_MTA_USAGE_COOKIE(self.0 as *mut _));
        }
    }
}

struct ThreadOcr {
    engine: Option<OcrEngine>,
    initialized: bool,
    _mta: Option<MtaLease>,
}
impl ThreadOcr {
    fn new() -> Self {
        let mta = MtaLease::acquire();
        let initialized = unsafe { RoInitialize(RO_INIT_MULTITHREADED).is_ok() };
        let engine = if initialized && mta.is_some() {
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
            _mta: mta,
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

pub struct WindowsOcr {
    available: OnceLock<bool>,
    _mta: Option<MtaLease>,
}
impl WindowsOcr {
    pub fn new() -> Self {
        // Acquire off the UI thread: an implicit MTA must not change Tauri's
        // main thread apartment before it initializes WebView2 as an STA.
        let mta = std::thread::spawn(MtaLease::acquire).join().ok().flatten();
        Self {
            available: OnceLock::new(),
            _mta: mta,
        }
    }
}
impl Default for WindowsOcr {
    fn default() -> Self {
        Self::new()
    }
}
impl Ocr for WindowsOcr {
    fn available(&self) -> bool {
        if self._mta.is_none() {
            return false;
        }
        // Tauri's UI thread can already belong to an STA. Check availability
        // on a dedicated MTA without changing the UI apartment or sharing COM.
        *self.available.get_or_init(|| {
            std::thread::spawn(|| ENGINE.with(|e| e.borrow().engine.is_some()))
                .join()
                .unwrap_or(false)
        })
    }
    fn read(&self, frame: &Frame) -> Result<String> {
        if self._mta.is_none() {
            return Err(PlatformError::OcrUnavailable);
        }
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
            for px in bgra.as_chunks_mut::<4>().0 {
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
