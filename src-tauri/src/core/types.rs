use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct PxPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct PxRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl PxRect {
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
    pub fn is_empty(&self) -> bool {
        self.w <= 0 || self.h <= 0
    }
    pub fn to_rel(&self, base: &PxRect) -> RelRect {
        RelRect {
            x: (self.x - base.x) as f32 / base.w.max(1) as f32,
            y: (self.y - base.y) as f32 / base.h.max(1) as f32,
            w: self.w as f32 / base.w.max(1) as f32,
            h: self.h as f32 / base.h.max(1) as f32,
        }
    }
}

impl PxPoint {
    pub fn to_rel(&self, base: &PxRect) -> RelPoint {
        RelPoint {
            x: (self.x - base.x) as f32 / base.w.max(1) as f32,
            y: (self.y - base.y) as f32 / base.h.max(1) as f32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct RelPoint {
    pub x: f32,
    pub y: f32,
}

impl RelPoint {
    pub fn validate(&self) -> Result<(), String> {
        if [self.x, self.y]
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        {
            Ok(())
        } else {
            Err("Point must lie inside the Roblox client area".into())
        }
    }
    pub fn to_px(&self, base: &PxRect) -> PxPoint {
        PxPoint {
            x: base.x
                + (self.x * base.w as f32)
                    .round()
                    .clamp(0.0, (base.w - 1).max(0) as f32) as i32,
            y: base.y
                + (self.y * base.h as f32)
                    .round()
                    .clamp(0.0, (base.h - 1).max(0) as f32) as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct RelRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RelRect {
    pub fn validate(&self) -> Result<(), String> {
        if [self.x, self.y, self.w, self.h]
            .iter()
            .all(|v| v.is_finite())
            && self.x >= 0.0
            && self.y >= 0.0
            && self.w > 0.0
            && self.h > 0.0
            && self.x + self.w <= 1.0
            && self.y + self.h <= 1.0
        {
            Ok(())
        } else {
            Err("Region must lie entirely inside the Roblox client area".into())
        }
    }
    pub fn to_px(&self, base: &PxRect) -> PxRect {
        PxRect {
            x: base.x + (self.x * base.w as f32).round() as i32,
            y: base.y + (self.y * base.h as f32).round() as i32,
            w: (self.w * base.w as f32).round().max(1.0) as i32,
            h: (self.h * base.h as f32).round().max(1.0) as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    #[inline]
    pub fn within(&self, r: u8, g: u8, b: u8, tol: u8) -> bool {
        self.r.abs_diff(r) <= tol && self.g.abs_diff(g) <= tol && self.b.abs_diff(b) <= tol
    }
}

#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

impl Frame {
    pub fn new(w: usize, h: usize, rgba: Vec<u8>) -> Self {
        debug_assert_eq!(rgba.len(), w * h * 4);
        Self { w, h, rgba }
    }

    #[inline]
    pub fn px(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let i = (y * self.w + x) * 4;
        (self.rgba[i], self.rgba[i + 1], self.rgba[i + 2])
    }

    pub fn crop(&self, x: usize, y: usize, w: usize, h: usize) -> Frame {
        let x1 = (x + w).min(self.w);
        let y1 = (y + h).min(self.h);
        let cw = x1.saturating_sub(x);
        let ch = y1.saturating_sub(y);
        let mut out = Vec::with_capacity(cw * ch * 4);
        for row in y..y1 {
            let s = (row * self.w + x) * 4;
            out.extend_from_slice(&self.rgba[s..s + cw * 4]);
        }
        Frame {
            w: cw,
            h: ch,
            rgba: out,
        }
    }

    pub fn downscale(&self, max_dim: usize) -> Frame {
        let longest = self.w.max(self.h);
        if longest <= max_dim || longest == 0 {
            return self.clone();
        }
        let factor = longest.div_ceil(max_dim);
        let nw = (self.w / factor).max(1);
        let nh = (self.h / factor).max(1);
        let mut out = Vec::with_capacity(nw * nh * 4);
        for y in 0..nh {
            let sy = y * factor;
            for x in 0..nw {
                let i = (sy * self.w + x * factor) * 4;
                out.extend_from_slice(&self.rgba[i..i + 4]);
            }
        }
        Frame {
            w: nw,
            h: nh,
            rgba: out,
        }
    }

    pub fn average_hash(&self) -> u64 {
        if self.w == 0 || self.h == 0 {
            return 0;
        }
        let mut cells = [0u32; 64];
        let mut counts = [0u32; 64];
        for y in 0..self.h {
            let cy = y * 8 / self.h;
            for x in 0..self.w {
                let cx = x * 8 / self.w;
                let (r, g, b) = self.px(x, y);
                let i = cy * 8 + cx;
                cells[i] += (r as u32 + g as u32 + b as u32) / 3;
                counts[i] += 1;
            }
        }
        let mut avg = 0u64;
        let mut vals = [0u32; 64];
        for i in 0..64 {
            vals[i] = cells[i].checked_div(counts[i]).unwrap_or(0);
            avg += vals[i] as u64;
        }
        let mean = (avg / 64) as u32;
        let mut hash = 0u64;
        for (i, v) in vals.iter().enumerate() {
            if *v > mean {
                hash |= 1 << i;
            }
        }
        hash
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub client: PxRect,
    pub is_foreground: bool,
    pub visible: bool,
    pub dpi: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Backspace,
    Delete,
    Enter,
    Escape,
    Control,
}
