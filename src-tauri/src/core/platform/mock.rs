use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;

use super::{Capture, GameWindow, Input, Ocr, Platform, PlatformError, Result};
use crate::core::types::{Frame, Key, MouseButton, PxPoint, PxRect, WindowInfo};

#[derive(Default)]
pub struct MockWindow {
    pub info: Mutex<Option<WindowInfo>>,
}

impl GameWindow for MockWindow {
    fn find(&self) -> Option<WindowInfo> {
        *self.info.lock()
    }
    fn focus(&self) -> bool {
        true
    }
}

#[derive(Default)]
pub struct ScriptedCapture {
    pub frames: Mutex<VecDeque<Frame>>,
    pub last: Mutex<Option<Frame>>,
}

impl ScriptedCapture {
    pub fn push(&self, f: Frame) {
        self.frames.lock().push_back(f);
    }
}

impl Capture for ScriptedCapture {
    fn grab(&self, _rect: PxRect) -> Result<Frame> {
        let mut q = self.frames.lock();
        if let Some(f) = q.pop_front() {
            *self.last.lock() = Some(f.clone());
            return Ok(f);
        }
        self.last
            .lock()
            .clone()
            .ok_or_else(|| PlatformError::Capture("no frames".into()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Move(PxPoint),
    Button(MouseButton, bool),
    Key(Key, bool),
    Wheel(i32),
    Text(String),
}

#[derive(Default)]
pub struct RecordingInput {
    pub events: Mutex<Vec<InputEvent>>,
    pub pos: Mutex<PxPoint>,
}

impl Input for RecordingInput {
    fn move_to(&self, p: PxPoint) {
        *self.pos.lock() = p;
        self.events.lock().push(InputEvent::Move(p));
    }
    fn button(&self, b: MouseButton, down: bool) {
        self.events.lock().push(InputEvent::Button(b, down));
    }
    fn key(&self, k: Key, down: bool) {
        self.events.lock().push(InputEvent::Key(k, down));
    }
    fn wheel(&self, delta: i32) {
        self.events.lock().push(InputEvent::Wheel(delta));
    }
    fn type_text(&self, s: &str) {
        self.events.lock().push(InputEvent::Text(s.to_string()));
    }
    fn cursor(&self) -> PxPoint {
        *self.pos.lock()
    }
    fn screen(&self) -> PxRect {
        PxRect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
        }
    }
}

pub struct NoOcr;

impl Ocr for NoOcr {
    fn available(&self) -> bool {
        false
    }
    fn read(&self, _frame: &Frame) -> Result<String> {
        Err(PlatformError::OcrUnavailable)
    }
}

#[derive(Default)]
pub struct ScriptedOcr {
    pub texts: Mutex<VecDeque<String>>,
}

impl Ocr for ScriptedOcr {
    fn available(&self) -> bool {
        true
    }
    fn read(&self, _frame: &Frame) -> Result<String> {
        Ok(self.texts.lock().pop_front().unwrap_or_default())
    }
}

pub fn platform() -> Platform {
    Platform {
        window: Arc::new(MockWindow::default()),
        capture: Arc::new(ScriptedCapture::default()),
        input: Arc::new(RecordingInput::default()),
        ocr: Arc::new(NoOcr),
    }
}
