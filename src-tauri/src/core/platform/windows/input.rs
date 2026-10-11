use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, SendInput, VkKeyScanW, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
    KEYEVENTF_UNICODE, MAPVK_VK_TO_VSC, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL, MOUSEINPUT, VIRTUAL_KEY, VK_BACK, VK_CONTROL,
    VK_DELETE, VK_ESCAPE, VK_RETURN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN,
};

use crate::core::platform::Input;
use crate::core::types::{Key, MouseButton, PxPoint, PxRect};

pub struct SendInputBackend;

impl SendInputBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SendInputBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn send(inputs: &[INPUT]) {
    unsafe {
        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

fn mouse(
    flags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS,
    dx: i32,
    dy: i32,
    data: i32,
) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data as u32,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn vk(key: VIRTUAL_KEY, up: bool) -> INPUT {
    let scan = unsafe { MapVirtualKeyW(key.0 as u32, MAPVK_VK_TO_VSC) } as u16;
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if scan != 0 {
        flags |= KEYEVENTF_SCANCODE;
    }
    if matches!(key, VK_DELETE) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn unicode(ch: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: ch,
                dwFlags: if up {
                    KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                } else {
                    KEYEVENTF_UNICODE
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn key_to_vk(k: Key) -> Option<VIRTUAL_KEY> {
    match k {
        Key::Backspace => Some(VK_BACK),
        Key::Delete => Some(VK_DELETE),
        Key::Enter => Some(VK_RETURN),
        Key::Escape => Some(VK_ESCAPE),
        Key::Control => Some(VK_CONTROL),
        Key::Char(c) => {
            let scan = unsafe { VkKeyScanW(c as u16) };
            if scan == -1 || (scan >> 8) & 0x07 != 0 {
                None
            } else {
                Some(VIRTUAL_KEY((scan & 0xff) as u16))
            }
        }
    }
}

impl Input for SendInputBackend {
    fn move_to(&self, p: PxPoint) {
        let s = self.screen();
        let nx = ((p.x - s.x) as i64 * 65535 / (s.w - 1).max(1) as i64) as i32;
        let ny = ((p.y - s.y) as i64 * 65535 / (s.h - 1).max(1) as i64) as i32;
        send(&[mouse(
            MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
            nx,
            ny,
            0,
        )]);
    }

    fn button(&self, b: MouseButton, down: bool) {
        let flag = match (b, down) {
            (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
            (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
            (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
            (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
        };
        send(&[mouse(flag, 0, 0, 0)]);
    }

    fn key(&self, k: Key, down: bool) {
        match key_to_vk(k) {
            Some(v) => send(&[vk(v, !down)]),
            None => {
                if let Key::Char(c) = k {
                    let mut buf = [0u16; 2];
                    for u in c.encode_utf16(&mut buf) {
                        send(&[unicode(*u, !down)]);
                    }
                }
            }
        }
    }

    fn wheel(&self, delta: i32) {
        send(&[mouse(MOUSEEVENTF_WHEEL, 0, 0, delta)]);
    }

    fn type_text(&self, s: &str) {
        for c in s.chars() {
            match key_to_vk(Key::Char(c)) {
                Some(v) => send(&[vk(v, false), vk(v, true)]),
                None => {
                    let mut buf = [0u16; 2];
                    for u in c.encode_utf16(&mut buf) {
                        send(&[unicode(*u, false), unicode(*u, true)]);
                    }
                }
            }
        }
    }

    fn cursor(&self) -> PxPoint {
        let mut p = windows::Win32::Foundation::POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut p);
        }
        PxPoint { x: p.x, y: p.y }
    }

    fn screen(&self) -> PxRect {
        unsafe {
            PxRect {
                x: GetSystemMetrics(SM_XVIRTUALSCREEN),
                y: GetSystemMetrics(SM_YVIRTUALSCREEN),
                w: GetSystemMetrics(SM_CXVIRTUALSCREEN),
                h: GetSystemMetrics(SM_CYVIRTUALSCREEN),
            }
        }
    }
}
