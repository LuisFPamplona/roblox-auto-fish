use crate::core::platform::GameWindow;
use crate::core::types::{PxRect, WindowInfo};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::HiDpi::{
    GetDpiForWindow, SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetAncestor, GetClientRect, GetForegroundWindow, GetWindowThreadProcessId,
    IsIconic, IsWindow, IsWindowVisible, WindowFromPoint, GA_ROOT,
};

pub struct Win32Window {
    class: Vec<u16>,
    title: Vec<u16>,
}
struct PhysicalCoordinates(DPI_AWARENESS_CONTEXT);
impl PhysicalCoordinates {
    fn enter() -> Self {
        Self(unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) })
    }
}
impl Drop for PhysicalCoordinates {
    fn drop(&mut self) {
        if !self.0 .0.is_null() {
            unsafe {
                SetThreadDpiAwarenessContext(self.0);
            }
        }
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

impl Win32Window {
    pub fn new(title: &str) -> Self {
        Self {
            class: wide("WINDOWSCLIENT"),
            title: wide(title),
        }
    }
    pub(crate) fn hwnd(&self) -> Option<HWND> {
        unsafe {
            let h = FindWindowW(PCWSTR(self.class.as_ptr()), PCWSTR(self.title.as_ptr())).ok()?;
            if h.is_invalid() || !IsWindow(Some(h)).as_bool() {
                return None;
            }
            let mut pid = 0;
            GetWindowThreadProcessId(h, Some(&mut pid));
            // Read only the executable path, never process memory or credentials.
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut path = [0u16; 32768];
            let mut len = path.len() as u32;
            let ok = QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(path.as_mut_ptr()),
                &mut len,
            )
            .is_ok();
            let _ = CloseHandle(process);
            if !ok {
                return None;
            }
            let path = String::from_utf16_lossy(&path[..len as usize]);
            let name = std::path::Path::new(&path).file_name()?.to_str()?;
            if !name.eq_ignore_ascii_case("RobloxPlayerBeta.exe") {
                return None;
            }
            Some(h)
        }
    }
}

impl GameWindow for Win32Window {
    fn owns_point(&self, point: crate::core::types::PxPoint) -> bool {
        self.hwnd().is_some_and(|h| unsafe {
            GetAncestor(
                WindowFromPoint(POINT {
                    x: point.x,
                    y: point.y,
                }),
                GA_ROOT,
            ) == h
        })
    }
    fn focus(&self) -> bool {
        self.hwnd()
            .is_some_and(|h| unsafe { GetForegroundWindow() == h })
    }
    fn find(&self) -> Option<WindowInfo> {
        // WGC textures and DWM bounds are physical pixels, including at 125/150% DPI.
        let _coordinates = PhysicalCoordinates::enter();
        let h = self.hwnd()?;
        unsafe {
            if !IsWindowVisible(h).as_bool() || IsIconic(h).as_bool() {
                return None;
            }
            let mut rc = RECT::default();
            GetClientRect(h, &mut rc).ok()?;
            let mut origin = POINT {
                x: rc.left,
                y: rc.top,
            };
            if !ClientToScreen(h, &mut origin).as_bool() {
                return None;
            }
            let client = PxRect {
                x: origin.x,
                y: origin.y,
                w: rc.right - rc.left,
                h: rc.bottom - rc.top,
            };
            if client.is_empty() {
                return None;
            }
            let dpi = GetDpiForWindow(h);
            Some(WindowInfo {
                client,
                is_foreground: GetForegroundWindow() == h,
                visible: true,
                dpi: if dpi == 0 { 96 } else { dpi },
            })
        }
    }
}
