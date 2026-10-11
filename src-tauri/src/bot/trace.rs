use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::core::controller::ControlGains;
use crate::events::TrackFrame;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReelMetrics {
    pub frames: u32,
    pub duration_ms: u64,
    pub fps: f32,
    pub on_target: f32,
    pub mean_abs_error: f32,
    pub max_abs_error: f32,
    pub flips_per_s: f32,
    pub blind_frames: u32,
    pub outcome: String,
}

pub struct Trace {
    dir: PathBuf,
    stem: String,
    started: Instant,
    frames: Vec<Value>,
    blind: u32,
    last_hold: Option<bool>,
    flips: u32,
    abs_err_sum: f32,
    abs_err_max: f32,
    on_target: u32,
    gains: ControlGains,
}

impl Trace {
    pub fn start(logs_dir: &Path, gains: ControlGains) -> std::io::Result<Self> {
        let dir = logs_dir.join("traces");
        fs::create_dir_all(&dir)?;
        let stem = format!("reel-{}", crate::events::now_ms());
        Ok(Self {
            dir,
            stem,
            started: Instant::now(),
            frames: Vec::with_capacity(1024),
            blind: 0,
            last_hold: None,
            flips: 0,
            abs_err_sum: 0.0,
            abs_err_max: 0.0,
            on_target: 0,
            gains,
        })
    }

    fn t_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    pub fn frame(&mut self, f: &TrackFrame, grab: Duration, read: Duration) {
        let r = &f.reading;
        let d = &f.decision;
        let fish_half = (r.fish.len() as f32 / 2.0) / r.bar.h().max(1) as f32;
        let err = r.error.abs();
        if err <= fish_half {
            self.on_target += 1;
        }
        self.abs_err_sum += err;
        self.abs_err_max = self.abs_err_max.max(err);
        if let Some(h) = self.last_hold {
            if h != d.hold {
                self.flips += 1;
            }
        }
        self.last_hold = Some(d.hold);
        self.frames.push(json!({
            "t": self.t_ms(),
            "bar": [r.bar.x0, r.bar.y0, r.bar.w(), r.bar.h()],
            "fish": [r.fish.start, r.fish.end],
            "marker": [r.marker.start, r.marker.end],
            "fish_c": r.fish_center,
            "marker_c": r.marker_center,
            "error": r.error,
            "fish_v": d.fish_velocity,
            "marker_v": d.marker_velocity,
            "predicted": d.predicted_error,
            "hold": d.hold,
            "grab_ms": grab.as_secs_f32() * 1000.0,
            "read_ms": read.as_secs_f32() * 1000.0,
        }));
    }

    pub fn blind(&mut self, bar_present: bool) {
        self.blind += 1;
        self.frames
            .push(json!({ "t": self.t_ms(), "blind": true, "bar_present": bar_present }));
    }

    pub fn finish(self, outcome: &str) -> (String, ReelMetrics) {
        let tracked = self
            .frames
            .iter()
            .filter(|f| f.get("blind").is_none())
            .count() as u32;
        let secs = self.started.elapsed().as_secs_f32().max(0.001);
        let metrics = ReelMetrics {
            frames: tracked,
            duration_ms: self.t_ms(),
            fps: (tracked + self.blind) as f32 / secs,
            on_target: if tracked > 0 {
                self.on_target as f32 / tracked as f32
            } else {
                0.0
            },
            mean_abs_error: if tracked > 0 {
                self.abs_err_sum / tracked as f32
            } else {
                0.0
            },
            max_abs_error: self.abs_err_max,
            flips_per_s: self.flips as f32 / secs,
            blind_frames: self.blind,
            outcome: outcome.to_string(),
        };
        let doc = json!({
            "reel": self.stem,
            "started_ms": crate::events::now_ms() - self.t_ms(),
            "gains": self.gains,
            "metrics": metrics,
            "frames": self.frames,
        });
        let _ = fs::write(
            self.dir.join(format!("{}.json", self.stem)),
            serde_json::to_vec_pretty(&doc).unwrap_or_default(),
        );
        append_index(&self.dir, &self.stem, &metrics, &self.gains);
        (
            format!(
                "{} ({} frames, on-target {:.0}%)",
                self.stem,
                tracked,
                metrics.on_target * 100.0
            ),
            metrics,
        )
    }
}

fn append_index(dir: &Path, stem: &str, m: &ReelMetrics, g: &ControlGains) {
    let path = dir.join("index.json");
    let mut list: Vec<Value> = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    list.push(json!({ "reel": stem, "metrics": m, "mode": g.mode, "lookahead_ms": g.lookahead_ms, "hysteresis": g.hysteresis, "latency_ms": g.physics.latency_ms }));
    if list.len() > 500 {
        let drop = list.len() - 500;
        list.drain(0..drop);
    }
    let _ = fs::write(path, serde_json::to_vec_pretty(&list).unwrap_or_default());
}
