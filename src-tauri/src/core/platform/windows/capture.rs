use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible,
};

use crate::core::platform::{Capture, GameWindow, PlatformError, Result};
use crate::core::types::{Frame, PxRect};

pub struct GdiCapture;

struct Occlusion {
    target: HWND,
    rect: PxRect,
    reached: bool,
    blocked: bool,
}
unsafe extern "system" fn above_window(h: HWND, data: LPARAM) -> BOOL {
    let state = &mut *(data.0 as *mut Occlusion);
    if h == state.target {
        state.reached = true;
        return BOOL(0);
    }
    if IsWindowVisible(h).as_bool() && !IsIconic(h).as_bool() {
        let mut r = RECT::default();
        if GetWindowRect(h, &mut r).is_ok()
            && r.left < state.rect.right()
            && r.right > state.rect.x
            && r.top < state.rect.bottom()
            && r.bottom > state.rect.y
        {
            state.blocked = true;
            return BOOL(0);
        }
    }
    BOOL(1)
}

fn capture_target(rect: PxRect) -> Result<(HWND, PxRect)> {
    let window = super::window::Win32Window::new("Roblox");
    let h = window.hwnd().ok_or(PlatformError::WindowNotFound)?;
    let info = window.find().ok_or(PlatformError::WindowNotFound)?;
    let r = info.client;
    if rect.is_empty()
        || rect.w > 8192
        || rect.h > 8192
        || (rect.w as i64 * rect.h as i64) > 16_777_216
        || rect.x < r.x
        || rect.y < r.y
        || rect.x as i64 + rect.w as i64 > r.x as i64 + r.w as i64
        || rect.y as i64 + rect.h as i64 > r.y as i64 + r.h as i64
    {
        return Err(PlatformError::Capture(
            "region outside Roblox or resource limit".into(),
        ));
    }
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut pid));
        if !info.is_foreground && pid != GetCurrentProcessId() {
            return Err(PlatformError::Capture(
                "Roblox or preview UI must be active".into(),
            ));
        }
        let mut state = Occlusion {
            target: h,
            rect,
            reached: false,
            blocked: false,
        };
        let _ = EnumWindows(Some(above_window), LPARAM(&mut state as *mut _ as isize));
        if state.blocked || !state.reached {
            return Err(PlatformError::Capture(
                "Roblox region is covered by another window".into(),
            ));
        }
    }
    Ok((h, r))
}

impl GdiCapture {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GdiCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl Capture for GdiCapture {
    fn grab(&self, rect: PxRect) -> Result<Frame> {
        let (target, client) = capture_target(rect)?;
        let w = rect.w;
        let h = rect.h;
        unsafe {
            let screen = GetDC(Some(target));
            if screen.is_invalid() {
                return Err(PlatformError::Capture("GetDC".into()));
            }
            let mem = CreateCompatibleDC(Some(screen));
            let bmp = CreateCompatibleBitmap(screen, w, h);
            if mem.is_invalid() || bmp.is_invalid() {
                if !bmp.is_invalid() {
                    let _ = DeleteObject(bmp.into());
                }
                if !mem.is_invalid() {
                    let _ = DeleteDC(mem);
                }
                ReleaseDC(Some(target), screen);
                return Err(PlatformError::Capture("GDI allocation failed".into()));
            }
            let old = SelectObject(mem, bmp.into());
            let ok = BitBlt(
                mem,
                0,
                0,
                w,
                h,
                Some(screen),
                rect.x - client.x,
                rect.y - client.y,
                SRCCOPY,
            )
            .is_ok();

            let mut info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut buf = vec![0u8; (w * h * 4) as usize];
            // GetDIBits requires the bitmap to be deselected from its DC.
            SelectObject(mem, old);
            let lines = GetDIBits(
                mem,
                bmp,
                0,
                h as u32,
                Some(buf.as_mut_ptr() as *mut _),
                &mut info,
                DIB_RGB_COLORS,
            );

            let _ = DeleteObject(bmp.into());
            let _ = DeleteDC(mem);
            ReleaseDC(Some(target), screen);

            if !ok || lines != h as u32 {
                return Err(PlatformError::Capture("BitBlt/GetDIBits".into()));
            }
            let (after, after_client) = capture_target(rect)?;
            if after != target || after_client != client {
                return Err(PlatformError::Capture("Roblox moved during capture".into()));
            }
            for px in buf.chunks_exact_mut(4) {
                px.swap(0, 2);
                px[3] = 255;
            }
            Ok(Frame::new(w as usize, h as usize, buf))
        }
    }
}
