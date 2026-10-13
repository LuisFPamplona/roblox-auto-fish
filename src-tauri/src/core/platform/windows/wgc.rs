//! Window-scoped Windows Graphics Capture. Never creates a monitor capture item.
//! COM/D3D objects live on one worker; only cropped RGBA leaves that worker.
use super::ocr::MtaLease;
use super::window::Win32Window;
use crate::core::platform::{Capture, GameWindow, PlatformError, Result};
use crate::core::types::{Frame, PxRect};
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use windows::core::{factory, Interface};
use windows::Graphics::Capture::{
    Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::SizeInt32;
use windows::Win32::Foundation::{HMODULE, HWND, RECT};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Dxgi::{
    Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
    IDXGIDevice,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

fn failure(message: impl Into<String>) -> PlatformError {
    PlatformError::Capture(message.into())
}
fn native(error: windows::core::Error) -> PlatformError {
    failure(format!("Windows window capture: {}", error.code()))
}

enum Request {
    Grab(PxRect, Sender<Result<Frame>>),
    Release,
}
pub struct WindowCapture {
    tx: Mutex<Option<Sender<Request>>>,
    worker: Mutex<Option<JoinHandle<()>>>,
}
impl WindowCapture {
    pub fn new() -> Self {
        let (tx, rx) = bounded(8);
        let worker = std::thread::Builder::new()
            .name("roblox-window-capture".into())
            .spawn(move || work(rx))
            .expect("spawn capture worker");
        Self {
            tx: Mutex::new(Some(tx)),
            worker: Mutex::new(Some(worker)),
        }
    }
}
impl Default for WindowCapture {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for WindowCapture {
    fn drop(&mut self) {
        self.tx.get_mut().take();
        if let Some(worker) = self.worker.get_mut().take() {
            let _ = worker.join();
        }
    }
}
impl Capture for WindowCapture {
    fn grab(&self, rect: PxRect) -> Result<Frame> {
        validate_size(rect)?;
        let (tx, rx) = bounded(1);
        self.tx
            .lock()
            .as_ref()
            .ok_or_else(|| failure("Capture worker stopped"))?
            .try_send(Request::Grab(rect, tx))
            .map_err(|_| failure("Capture busy; retry"))?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|_| failure("Capture response timed out"))?
    }
    fn release(&self) {
        if let Some(tx) = self.tx.lock().as_ref() {
            let _ = tx.try_send(Request::Release);
        }
    }
}

fn validate_size(rect: PxRect) -> Result<()> {
    if rect.is_empty()
        || rect.w > 8192
        || rect.h > 8192
        || i64::from(rect.w) * i64::from(rect.h) > 16_777_216
    {
        return Err(failure("Invalid capture region or resource limit"));
    }
    Ok(())
}
fn inside(rect: PxRect, bounds: PxRect) -> bool {
    !rect.is_empty()
        && rect.x >= bounds.x
        && rect.y >= bounds.y
        && i64::from(rect.x) + i64::from(rect.w) <= i64::from(bounds.x) + i64::from(bounds.w)
        && i64::from(rect.y) + i64::from(rect.h) <= i64::from(bounds.y) + i64::from(bounds.h)
}
fn target(rect: PxRect) -> Result<(HWND, PxRect, PxRect)> {
    validate_size(rect)?;
    let window = Win32Window::new("Roblox");
    let hwnd = window.hwnd().ok_or(PlatformError::WindowNotFound)?;
    let info = window.find().ok_or(PlatformError::WindowNotFound)?;
    if !inside(rect, info.client) {
        return Err(failure("Region outside Roblox client"));
    }
    unsafe {
        let mut foreground_pid = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut foreground_pid));
        if !info.is_foreground && foreground_pid != GetCurrentProcessId() {
            return Err(failure("Roblox or preview UI must be active"));
        }
        let mut bounds = RECT::default();
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut bounds as *mut _ as *mut _,
            std::mem::size_of::<RECT>() as u32,
        )
        .map_err(native)?;
        let bounds = PxRect {
            x: bounds.left,
            y: bounds.top,
            w: bounds.right - bounds.left,
            h: bounds.bottom - bounds.top,
        };
        validate_size(bounds)?;
        if !inside(rect, bounds) {
            return Err(failure("Region outside captured window bounds"));
        }
        Ok((hwnd, info.client, bounds))
    }
}

struct Apartment {
    initialized: bool,
    _lease: Option<MtaLease>,
}
impl Apartment {
    fn new() -> Self {
        let lease = MtaLease::acquire();
        Self {
            initialized: unsafe { RoInitialize(RO_INIT_MULTITHREADED).is_ok() },
            _lease: lease,
        }
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                RoUninitialize();
            }
        }
    }
}
fn work(rx: Receiver<Request>) {
    let apartment = Apartment::new();
    let mut session: Option<Session> = None;
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Request::Release) | Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                session.take();
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            Ok(Request::Grab(rect, reply)) => {
                let result = (|| {
                    if !apartment.initialized || apartment._lease.is_none() {
                        return Err(failure("Windows capture apartment unavailable"));
                    }
                    let (hwnd, client, bounds) = target(rect)?;
                    if session.as_ref().is_none_or(|s| {
                        s.hwnd != hwnd || s.size.Width != bounds.w || s.size.Height != bounds.h
                    }) {
                        session.take();
                        session = Some(Session::new(hwnd)?);
                    }
                    let frame = session.as_mut().unwrap().grab(rect, bounds)?;
                    let (after, after_client, after_bounds) = target(rect)?;
                    if after != hwnd || after_client != client || after_bounds != bounds {
                        return Err(failure("Roblox moved or resized during capture; retry"));
                    }
                    Ok(frame)
                })();
                if result.is_err() {
                    session.take();
                }
                let _ = reply.try_send(result);
            }
        }
    }
    drop(session);
    drop(apartment);
}

struct Session {
    hwnd: HWND,
    size: SizeInt32,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    pool: Direct3D11CaptureFramePool,
    capture: GraphicsCaptureSession,
    staging: Option<(i32, i32, ID3D11Texture2D)>,
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.capture.Close();
        let _ = self.pool.Close();
    }
}
struct Captured(Direct3D11CaptureFrame);
impl Drop for Captured {
    fn drop(&mut self) {
        let _ = self.0.Close();
    }
}
struct Mapped<'a> {
    context: &'a ID3D11DeviceContext,
    texture: &'a ID3D11Texture2D,
}
impl Drop for Mapped<'_> {
    fn drop(&mut self) {
        unsafe {
            self.context.Unmap(self.texture, 0);
        }
    }
}
impl Session {
    fn new(hwnd: HWND) -> Result<Self> {
        if !GraphicsCaptureSession::IsSupported().map_err(native)? {
            return Err(failure("Windows Graphics Capture is unsupported"));
        }
        let interop: IGraphicsCaptureItemInterop =
            factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>().map_err(native)?;
        // Only the validated Roblox HWND. No CreateForMonitor/Desktop Duplication.
        let item: GraphicsCaptureItem = unsafe { interop.CreateForWindow(hwnd).map_err(native)? };
        let size = item.Size().map_err(native)?;
        validate_size(PxRect {
            x: 0,
            y: 0,
            w: size.Width,
            h: size.Height,
        })?;
        let mut device = None;
        let mut context = None;
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .map_err(native)?;
        }
        let device = device.ok_or_else(|| failure("D3D device unavailable"))?;
        let context = context.ok_or_else(|| failure("D3D context unavailable"))?;
        let dxgi: IDXGIDevice = device.cast().map_err(native)?;
        let winrt: IDirect3DDevice = unsafe {
            CreateDirect3D11DeviceFromDXGIDevice(&dxgi)
                .map_err(native)?
                .cast()
                .map_err(native)?
        };
        let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &winrt,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            2,
            size,
        )
        .map_err(native)?;
        let capture = pool.CreateCaptureSession(&item).map_err(native)?;
        capture.SetIsCursorCaptureEnabled(false).map_err(native)?;
        capture.StartCapture().map_err(native)?;
        Ok(Self {
            hwnd,
            size,
            device,
            context,
            pool,
            capture,
            staging: None,
        })
    }
    fn grab(&mut self, rect: PxRect, bounds: PxRect) -> Result<Frame> {
        let deadline = Instant::now() + Duration::from_millis(300);
        let mut captured = loop {
            match self.pool.TryGetNextFrame() {
                Ok(frame) => break Captured(frame),
                // WinRT returns a null frame with S_OK while the pool is empty;
                // windows-rs represents that null interface as Error(S_OK).
                Err(error) if matches!(error.code().0, 0 | -2147467261) => {}
                Err(error) => return Err(native(error)),
            }
            if Instant::now() >= deadline {
                return Err(failure("No frame received from Roblox"));
            }
            std::thread::sleep(Duration::from_millis(1));
        };
        // Prefer the most recent frame, closing superseded buffers immediately.
        for _ in 0..2 {
            if let Ok(frame) = self.pool.TryGetNextFrame() {
                captured = Captured(frame);
            } else {
                break;
            }
        }
        let content = captured.0.ContentSize().map_err(native)?;
        if content.Width != bounds.w || content.Height != bounds.h {
            return Err(failure("Window/frame geometry changed; retry"));
        }
        let surface = captured.0.Surface().map_err(native)?;
        let access: IDirect3DDxgiInterfaceAccess = surface.cast().map_err(native)?;
        let texture: ID3D11Texture2D = unsafe { access.GetInterface().map_err(native)? };
        let x = rect.x - bounds.x;
        let y = rect.y - bounds.y;
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe {
            texture.GetDesc(&mut desc);
        }
        if x < 0
            || y < 0
            || i64::from(x) + i64::from(rect.w) > i64::from(desc.Width)
            || i64::from(y) + i64::from(rect.h) > i64::from(desc.Height)
        {
            return Err(failure("Crop outside Roblox texture"));
        }
        if self
            .staging
            .as_ref()
            .is_none_or(|(w, h, _)| *w != rect.w || *h != rect.h)
        {
            let desc = D3D11_TEXTURE2D_DESC {
                Width: rect.w as u32,
                Height: rect.h as u32,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_STAGING,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                ..Default::default()
            };
            let mut staging = None;
            unsafe {
                self.device
                    .CreateTexture2D(&desc, None, Some(&mut staging))
                    .map_err(native)?;
            }
            self.staging = Some((
                rect.w,
                rect.h,
                staging.ok_or_else(|| failure("Staging texture unavailable"))?,
            ));
        }
        let staging = &self.staging.as_ref().unwrap().2;
        let crop = D3D11_BOX {
            left: x as u32,
            top: y as u32,
            right: (x + rect.w) as u32,
            bottom: (y + rect.h) as u32,
            front: 0,
            back: 1,
        };
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        unsafe {
            self.context
                .CopySubresourceRegion(staging, 0, 0, 0, 0, &texture, 0, Some(&crop));
            self.context
                .Map(staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
                .map_err(native)?;
        }
        let _unmap = Mapped {
            context: &self.context,
            texture: staging,
        };
        let row = rect.w as usize * 4;
        if mapped.pData.is_null() || (mapped.RowPitch as usize) < row {
            return Err(failure("Invalid mapped frame layout"));
        }
        let mut rgba = Vec::with_capacity(row * rect.h as usize);
        for i in 0..rect.h as usize {
            let pixels = unsafe {
                std::slice::from_raw_parts(
                    (mapped.pData as *const u8).add(i * mapped.RowPitch as usize),
                    row,
                )
            };
            rgba.extend_from_slice(pixels);
        }
        for pixel in rgba.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        Ok(Frame::new(rect.w as usize, rect.h as usize, rgba))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crop_cannot_escape_window_or_overflow_coordinates() {
        let bounds = PxRect {
            x: -900,
            y: 30,
            w: 800,
            h: 600,
        };
        assert!(inside(
            PxRect {
                x: -899,
                y: 31,
                w: 798,
                h: 598
            },
            bounds
        ));
        assert!(!inside(
            PxRect {
                x: -901,
                y: 30,
                w: 5,
                h: 5
            },
            bounds
        ));
        assert!(!inside(
            PxRect {
                x: -101,
                y: 30,
                w: 2,
                h: 5
            },
            bounds
        ));
        assert!(!inside(
            PxRect {
                x: i32::MAX,
                y: 30,
                w: 800,
                h: 600
            },
            bounds
        ));
        assert!(validate_size(PxRect {
            x: 0,
            y: 0,
            w: 100_000,
            h: 100_000
        })
        .is_err());
    }
}
