use serde::{Deserialize, Serialize};

use crate::core::types::{Frame, Rgb};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Palette {
    pub bar: Rgb,
    pub fish: Rgb,
    pub marker: Rgb,
    pub tolerance: u8,
    pub min_bar_height_px: usize,
    pub min_bar_aspect: f32,
    pub min_bar_fill: f32,
    pub min_row_fraction: f32,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            bar: Rgb::new(85, 170, 255),
            fish: Rgb::new(25, 25, 25),
            marker: Rgb::new(255, 255, 255),
            tolerance: 8,
            min_bar_height_px: 40,
            min_bar_aspect: 1.5,
            min_bar_fill: 0.3,
            min_row_fraction: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bbox {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

impl Bbox {
    pub fn w(&self) -> usize {
        self.x1 - self.x0 + 1
    }
    pub fn h(&self) -> usize {
        self.y1 - self.y0 + 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn len(&self) -> usize {
        self.end - self.start + 1
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    pub fn mid(&self) -> f32 {
        (self.start + self.end) as f32 / 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    pub bar: Bbox,
    pub fish: Span,
    pub marker: Span,
    pub fish_center: f32,
    pub marker_center: f32,
    pub error: f32,
}

fn row_counts(f: &Frame, color: Rgb, tol: u8) -> Vec<u32> {
    f.rgba
        .chunks_exact(f.w * 4)
        .map(|row| {
            row.chunks_exact(4)
                .filter(|px| color.within(px[0], px[1], px[2], tol))
                .count() as u32
        })
        .collect()
}

fn col_counts(f: &Frame, color: Rgb, tol: u8) -> Vec<u32> {
    let mut cols = vec![0u32; f.w];
    for row in f.rgba.chunks_exact(f.w * 4) {
        for (c, px) in cols.iter_mut().zip(row.chunks_exact(4)) {
            if color.within(px[0], px[1], px[2], tol) {
                *c += 1;
            }
        }
    }
    cols
}

fn largest_run(counts: &[u32], min: u32, max_gap: usize) -> Option<Span> {
    let mut best: Option<Span> = None;
    let mut cur: Option<usize> = None;
    let mut gap = 0usize;
    let mut last_hit = 0usize;
    let consider = |start: usize, end: usize, best: &mut Option<Span>| {
        let span = Span { start, end };
        if best.map_or(true, |b| span.len() > b.len()) {
            *best = Some(span);
        }
    };
    for (i, &n) in counts.iter().enumerate() {
        if n >= min {
            gap = 0;
            last_hit = i;
            if cur.is_none() {
                cur = Some(i);
            }
        } else if let Some(s) = cur {
            gap += 1;
            if gap > max_gap {
                consider(s, last_hit, &mut best);
                cur = None;
                gap = 0;
            }
        }
    }
    if let Some(s) = cur {
        consider(s, last_hit, &mut best);
    }
    best
}

const BAR_EDGE_GAP: usize = 4;
const MIN_BLOCK_ROWS: usize = 8;

fn long_run_mask(counts: &[u32], min: u32, min_len: usize) -> Vec<bool> {
    let mut mask = vec![false; counts.len()];
    let mut start: Option<usize> = None;
    for i in 0..=counts.len() {
        let hit = i < counts.len() && counts[i] >= min;
        match (start, hit) {
            (None, true) => start = Some(i),
            (Some(s), false) => {
                if i - s >= min_len {
                    mask[s..i].iter_mut().for_each(|m| *m = true);
                }
                start = None;
            }
            _ => {}
        }
    }
    mask
}

pub fn find_bar(f: &Frame, p: &Palette) -> Option<Bbox> {
    if f.w == 0 || f.h == 0 {
        return None;
    }
    let cols = col_counts(f, p.bar, p.tolerance);
    let max_col = *cols.iter().max()?;
    if (max_col as usize) < p.min_bar_height_px / 2 {
        return None;
    }
    let xs = largest_run(&cols, (max_col / 2).max(3), 1)?;
    if xs.len() < 4 {
        return None;
    }
    let strip = f.crop(xs.start, 0, xs.len(), f.h);
    let rows = row_counts(&strip, p.bar, p.tolerance);
    let dark = row_counts(&strip, p.fish, p.tolerance);
    let white = row_counts(&strip, p.marker, p.tolerance);
    let min_row = ((xs.len() as f32) * 0.5).max(1.0) as u32;
    let block = long_run_mask(&dark, min_row, MIN_BLOCK_ROWS);
    let filled: Vec<u32> = (0..rows.len())
        .map(|i| u32::from(rows[i] >= min_row || white[i] >= min_row || block[i]))
        .collect();
    let ys = largest_run(&filled, 1, BAR_EDGE_GAP)?;
    let bbox = Bbox {
        x0: xs.start,
        y0: ys.start,
        x1: xs.end,
        y1: ys.end,
    };
    if bbox.h() < p.min_bar_height_px || (bbox.h() as f32) < p.min_bar_aspect * bbox.w() as f32 {
        return None;
    }
    let blue: u32 = rows[ys.start..=ys.end].iter().sum();
    let fill = blue as f32 / (bbox.w() * bbox.h()) as f32;
    if fill < p.min_bar_fill {
        return None;
    }
    Some(bbox)
}

pub fn find_marker(inner: &Frame, p: &Palette) -> Option<Span> {
    let rows = row_counts(inner, p.marker, p.tolerance);
    let min = ((inner.w as f32) * p.min_row_fraction).max(1.0) as u32;
    largest_run(&rows, min, 1)
}

pub fn find_fish(inner: &Frame, p: &Palette, max_gap: usize) -> Option<Span> {
    let rows = row_counts(inner, p.fish, p.tolerance);
    let min = ((inner.w as f32) * p.min_row_fraction).max(1.0) as u32;
    largest_run(&rows, min, max_gap)
}

pub fn read(f: &Frame, p: &Palette) -> Option<Reading> {
    read_locked(f, p, None)
}

const MARKER_MARGIN: usize = 8;

pub fn read_locked(f: &Frame, p: &Palette, lock: Option<Bbox>) -> Option<Reading> {
    let mut bar = find_bar(f, p)?;
    if let Some(l) = lock {
        if l.y1 < f.h {
            bar.y0 = l.y0;
            bar.y1 = l.y1;
        }
    }
    let inner = f.crop(bar.x0, bar.y0, bar.w(), bar.h());
    let margin = MARKER_MARGIN.min(bar.y0);
    let ext = f.crop(
        bar.x0,
        bar.y0 - margin,
        bar.w(),
        bar.h() + margin + MARKER_MARGIN,
    );
    let raw = find_marker(&ext, p)?;
    let h = inner.h.max(1) as f32;
    let marker_center = ((raw.mid() - margin as f32) / h).clamp(0.0, 1.0);
    let marker = Span {
        start: raw.start.saturating_sub(margin).min(inner.h - 1),
        end: raw.end.saturating_sub(margin).min(inner.h - 1),
    };
    let max_gap = (marker.len() * 2).max(2);
    let fish = find_fish(&inner, p, max_gap)?;
    let fish_center = fish.mid() / h;
    Some(Reading {
        bar,
        fish,
        marker,
        fish_center,
        marker_center,
        error: fish_center - marker_center,
    })
}

pub fn has_bar(f: &Frame, p: &Palette) -> bool {
    find_bar(f, p).is_some()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Confidence {
    pub bar: f32,
    pub fish: f32,
    pub marker: f32,
    pub score: f32,
}

pub fn confidence(f: &Frame, p: &Palette) -> Confidence {
    let total = (f.w * f.h).max(1) as f32;
    let bar = row_counts(f, p.bar, p.tolerance).iter().sum::<u32>() as f32 / total;
    let fish = row_counts(f, p.fish, p.tolerance).iter().sum::<u32>() as f32 / total;
    let marker = row_counts(f, p.marker, p.tolerance).iter().sum::<u32>() as f32 / total;
    let mut score = 0.0;
    if find_bar(f, p).is_some() {
        score += 0.4;
    }
    if fish > 0.02 {
        score += 0.3;
    }
    if marker > 0.005 {
        score += 0.3;
    }
    Confidence {
        bar,
        fish,
        marker,
        score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: usize, h: usize, paint: impl Fn(usize, usize) -> Rgb) -> Frame {
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let c = paint(x, y);
                let i = (y * w + x) * 4;
                rgba[i] = c.r;
                rgba[i + 1] = c.g;
                rgba[i + 2] = c.b;
                rgba[i + 3] = 255;
            }
        }
        Frame::new(w, h, rgba)
    }

    #[test]
    fn detects_bar_fish_marker() {
        let p = Palette::default();
        let bg = Rgb::new(0, 0, 0);
        let f = frame(60, 200, |x, y| {
            let in_bar = (10..50).contains(&x) && (20..180).contains(&y);
            if !in_bar {
                return bg;
            }
            if (60..90).contains(&y) {
                return p.fish;
            }
            if (120..126).contains(&y) {
                return p.marker;
            }
            p.bar
        });
        let r = read(&f, &p).expect("reading");
        assert_eq!(
            r.bar,
            Bbox {
                x0: 10,
                y0: 20,
                x1: 49,
                y1: 179
            }
        );
        assert_eq!(r.fish, Span { start: 40, end: 69 });
        assert_eq!(
            r.marker,
            Span {
                start: 100,
                end: 105
            }
        );
        assert!(r.error < 0.0);
    }

    #[test]
    fn block_at_bar_end_keeps_bbox_stable() {
        let p = Palette::default();
        let bg = Rgb::new(0, 0, 0);
        let paint = |block: std::ops::Range<usize>| {
            move |x: usize, y: usize| -> Rgb {
                if !(10..50).contains(&x) {
                    return bg;
                }
                if (16..18).contains(&y) || (182..184).contains(&y) {
                    return p.fish;
                }
                if !(20..180).contains(&y) {
                    return bg;
                }
                if block.contains(&y) {
                    return p.fish;
                }
                if (100..103).contains(&y) {
                    return p.marker;
                }
                p.bar
            }
        };
        let mid = read(&frame(60, 200, paint(60..90)), &p).expect("mid");
        let low = read(&frame(60, 200, paint(150..180)), &p).expect("bottom");
        let high = read(&frame(60, 200, paint(20..50)), &p).expect("top");
        assert_eq!(
            mid.bar,
            Bbox {
                x0: 10,
                y0: 20,
                x1: 49,
                y1: 179
            }
        );
        assert_eq!(low.bar, mid.bar);
        assert_eq!(high.bar, mid.bar);
        assert_eq!(
            low.fish,
            Span {
                start: 130,
                end: 159
            }
        );
        assert_eq!(high.fish, Span { start: 0, end: 29 });
    }

    #[test]
    fn lock_overrides_vertical_extent() {
        let p = Palette::default();
        let bg = Rgb::new(0, 0, 0);
        let f = frame(60, 200, |x, y| {
            if !(10..50).contains(&x) || !(30..180).contains(&y) {
                return bg;
            }
            if (60..90).contains(&y) {
                return p.fish;
            }
            if (120..123).contains(&y) {
                return p.marker;
            }
            p.bar
        });
        let lock = Bbox {
            x0: 0,
            y0: 20,
            x1: 0,
            y1: 179,
        };
        let r = read_locked(&f, &p, Some(lock)).expect("reading");
        assert_eq!(
            r.bar,
            Bbox {
                x0: 10,
                y0: 20,
                x1: 49,
                y1: 179
            }
        );
        assert_eq!(r.fish, Span { start: 40, end: 69 });
        assert_eq!(
            r.marker,
            Span {
                start: 100,
                end: 102
            }
        );
    }

    #[test]
    fn marker_just_above_locked_bar_is_still_read() {
        let p = Palette::default();
        let bg = Rgb::new(0, 0, 0);
        let f = frame(60, 200, |x, y| {
            if !(10..50).contains(&x) {
                return bg;
            }
            if (16..19).contains(&y) {
                return p.marker;
            }
            if !(20..180).contains(&y) {
                return bg;
            }
            if (25..55).contains(&y) {
                return p.fish;
            }
            p.bar
        });
        let lock = Bbox {
            x0: 0,
            y0: 20,
            x1: 0,
            y1: 179,
        };
        let r = read_locked(&f, &p, Some(lock)).expect("reading");
        assert_eq!(r.bar.y0, 20);
        assert_eq!(r.marker, Span { start: 0, end: 0 });
        assert_eq!(r.marker_center, 0.0);
        assert!(r.error > 0.0);
    }

    #[test]
    fn no_bar_returns_none() {
        let p = Palette::default();
        let f = frame(30, 30, |_, _| Rgb::new(10, 10, 10));
        assert!(find_bar(&f, &p).is_none());
        assert!(read(&f, &p).is_none());
    }

    #[test]
    fn scattered_blue_noise_is_not_a_bar() {
        let p = Palette::default();
        let f = frame(120, 200, |x, y| {
            if (x * 7 + y * 13) % 11 == 0 {
                p.bar
            } else {
                Rgb::new(0, 0, 0)
            }
        });
        assert!(find_bar(&f, &p).is_none());
    }

    #[test]
    fn wide_blue_patch_is_not_a_bar() {
        let p = Palette::default();
        let f = frame(200, 100, |x, y| {
            if (10..190).contains(&x) && (20..70).contains(&y) {
                p.bar
            } else {
                Rgb::new(0, 0, 0)
            }
        });
        assert!(find_bar(&f, &p).is_none());
    }

    #[test]
    fn fish_gap_merging() {
        let p = Palette::default();
        let f = frame(10, 50, |_, y| {
            if (5..10).contains(&y) || (12..20).contains(&y) {
                p.fish
            } else {
                p.bar
            }
        });
        let s = find_fish(&f, &p, 3).unwrap();
        assert_eq!(s, Span { start: 5, end: 19 });
        let s = find_fish(&f, &p, 1).unwrap();
        assert_eq!(s, Span { start: 12, end: 19 });
    }
}
