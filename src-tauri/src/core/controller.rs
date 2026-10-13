use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ControlMode {
    #[default]
    Lookahead,
    Physics,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Physics {
    pub accel_hold: f32,
    pub accel_release: f32,
    pub max_speed: f32,
    pub latency_ms: f32,
    pub calibrated_at: u64,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            accel_hold: -1.8,
            accel_release: 1.8,
            max_speed: 1.4,
            latency_ms: 80.0,
            calibrated_at: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct ControlGains {
    pub mode: ControlMode,
    pub lookahead_ms: f32,
    pub hysteresis: f32,
    pub velocity_smoothing: f32,
    pub invert: bool,
    pub physics: Physics,
}

impl Default for ControlGains {
    fn default() -> Self {
        Self {
            mode: ControlMode::Physics,
            lookahead_ms: 120.0,
            hysteresis: 0.015,
            velocity_smoothing: 0.6,
            invert: false,
            physics: Physics::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Decision {
    pub hold: bool,
    pub predicted_error: f32,
    pub fish_velocity: f32,
    pub marker_velocity: f32,
}

const MAX_OBSERVED_SPEED: f32 = 3.0;
const DEFAULT_ZONE_HALF: f32 = 0.125;

#[derive(Debug, Clone, Copy)]
pub struct Tracker {
    last: Option<(f32, f32, Instant)>,
    fish_v: f32,
    marker_v: f32,
    period: f32,
    holding: bool,
    zone_half: f32,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            last: None,
            fish_v: 0.0,
            marker_v: 0.0,
            period: 0.0,
            holding: false,
            zone_half: DEFAULT_ZONE_HALF,
        }
    }
}

impl Tracker {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_zone_half(&mut self, half: f32) {
        if half.is_finite() {
            self.zone_half = half.clamp(0.02, 0.5);
        }
    }

    pub fn step(&mut self, g: &ControlGains, fish: f32, marker: f32, now: Instant) -> Decision {
        if let Some((pf, pm, t)) = self.last {
            let dt = now.duration_since(t).as_secs_f32().clamp(0.001, 0.5);
            self.period = dt;
            let a = g.velocity_smoothing.clamp(0.0, 0.95).powf(dt * 60.0);
            let measured = (fish - pf) / dt;
            let modelled = match g.mode {
                ControlMode::Physics => {
                    let acc = if self.holding {
                        g.physics.accel_hold
                    } else {
                        g.physics.accel_release
                    };
                    let cap = g.physics.max_speed.abs().max(0.1);
                    (self.fish_v + acc * dt).clamp(-cap, cap)
                }
                ControlMode::Lookahead => self.fish_v,
            };
            self.fish_v = (a * modelled + (1.0 - a) * measured)
                .clamp(-MAX_OBSERVED_SPEED, MAX_OBSERVED_SPEED);
            self.marker_v = (a * self.marker_v + (1.0 - a) * ((marker - pm) / dt))
                .clamp(-MAX_OBSERVED_SPEED, MAX_OBSERVED_SPEED);
        }
        self.last = Some((fish, marker, now));

        let predicted = match g.mode {
            ControlMode::Lookahead => self.lookahead(g, fish, marker),
            ControlMode::Physics => self.switching(g, fish, marker),
        };
        let predicted = if g.invert { -predicted } else { predicted };
        let band = g.hysteresis.max(0.0);
        if predicted > band {
            self.holding = true;
        } else if predicted < -band {
            self.holding = false;
        }
        Decision {
            hold: self.holding,
            predicted_error: predicted,
            fish_velocity: self.fish_v,
            marker_velocity: self.marker_v,
        }
    }

    fn lookahead(&self, g: &ControlGains, fish: f32, marker: f32) -> f32 {
        let tau = g.lookahead_ms.max(0.0) / 1000.0;
        (fish + self.fish_v * tau) - (marker + self.marker_v * tau)
    }

    fn switching(&self, g: &ControlGains, fish: f32, marker: f32) -> f32 {
        let p = g.physics;
        let lat = p.latency_ms.max(0.0) / 1000.0 + 0.5 * self.period;
        let dir = if p.accel_hold >= 0.0 { 1.0 } else { -1.0 };
        let a_hold = p.accel_hold.abs().max(1e-3);
        let a_rel = p.accel_release.abs().max(1e-3);
        let a_now = if self.holding {
            p.accel_hold
        } else {
            p.accel_release
        };
        let v0 =
            (self.fish_v + 0.5 * a_now * self.period).clamp(-p.max_speed.abs(), p.max_speed.abs());
        let zone = fish + v0 * lat + 0.5 * a_now * lat * lat;
        let zone_v = (v0 + a_now * lat).clamp(-p.max_speed.abs(), p.max_speed.abs());
        let raw_target = marker + self.marker_v * lat;
        let (lo, hi) = (self.zone_half, 1.0 - self.zone_half);
        let target = raw_target.clamp(lo, hi);
        let beyond_wall = raw_target < lo || raw_target > hi;
        let e = dir * (target - zone);
        if beyond_wall {
            return e;
        }
        let v = dir * (zone_v - self.marker_v);
        let brake = if v > 0.0 { a_rel } else { a_hold };
        e - v * v.abs() / (2.0 * brake)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn gains() -> ControlGains {
        ControlGains {
            mode: ControlMode::Lookahead,
            lookahead_ms: 100.0,
            hysteresis: 0.01,
            velocity_smoothing: 0.0,
            ..Default::default()
        }
    }

    #[test]
    fn fish_below_marker_means_hold() {
        let mut t = Tracker::default();
        let d = t.step(&gains(), 0.7, 0.3, Instant::now());
        assert!(d.hold);
        let d = t.step(&gains(), 0.2, 0.6, Instant::now());
        assert!(!d.hold);
    }

    #[test]
    fn hysteresis_keeps_state_inside_band() {
        let mut t = Tracker::default();
        let g = ControlGains {
            hysteresis: 0.05,
            lookahead_ms: 0.0,
            ..gains()
        };
        assert!(t.step(&g, 0.6, 0.5, Instant::now()).hold);
        assert!(t.step(&g, 0.5, 0.52, Instant::now()).hold);
        assert!(!t.step(&g, 0.4, 0.5, Instant::now()).hold);
    }

    #[test]
    fn lookahead_uses_velocity() {
        let mut t = Tracker::default();
        let g = ControlGains {
            lookahead_ms: 200.0,
            ..gains()
        };
        let t0 = Instant::now();
        t.step(&g, 0.5, 0.5, t0);
        let d = t.step(&g, 0.5, 0.45, t0 + Duration::from_millis(16));
        assert!(d.marker_velocity < 0.0);
        assert!(d.predicted_error > 0.0);
        assert!(d.hold);
    }

    #[test]
    fn invert_flips_decision() {
        let mut t = Tracker::default();
        let g = ControlGains {
            invert: true,
            ..gains()
        };
        assert!(!t.step(&g, 0.7, 0.3, Instant::now()).hold);
    }

    struct Sim {
        marker: f32,
        marker_v: f32,
        fish: f32,
        fish_v: f32,
        physics: Physics,
        fish_half: f32,
        smooth: bool,
        queue: std::collections::VecDeque<bool>,
    }

    impl Sim {
        fn new(physics: Physics, latency_frames: usize) -> Self {
            Self {
                marker: 0.5,
                marker_v: 0.0,
                fish: 0.5,
                fish_v: 0.0,
                physics,
                fish_half: 0.125,
                smooth: false,
                queue: std::iter::repeat_n(false, latency_frames).collect(),
            }
        }

        fn tick(&mut self, hold: bool, dt: f32, frame: usize) {
            self.queue.push_back(hold);
            let applied = self.queue.pop_front().unwrap_or(false);
            let a = if applied {
                self.physics.accel_hold
            } else {
                self.physics.accel_release
            };
            self.fish_v =
                (self.fish_v + a * dt).clamp(-self.physics.max_speed, self.physics.max_speed);
            self.fish = (self.fish + self.fish_v * dt).clamp(0.0, 1.0);
            if self.fish <= 0.0 || self.fish >= 1.0 {
                self.fish_v = 0.0;
            }
            let t = frame as f32 * dt;
            if self.smooth {
                self.marker_v = 0.3 * (t * 2.1).sin();
            } else if (t / 1.5).fract() < dt / 1.5 {
                self.marker_v = if ((t / 1.5) as usize).is_multiple_of(2) {
                    0.25
                } else {
                    -0.25
                };
            }
            self.marker = (self.marker + self.marker_v * dt).clamp(0.08, 0.92);
        }

        fn on_target(&self) -> bool {
            (self.marker - self.fish).abs() <= self.fish_half
        }
    }

    fn run_sim(g: ControlGains, physics: Physics, latency_frames: usize) -> f32 {
        run_sim_with(g, physics, latency_frames, false)
    }

    fn run_sim_with(g: ControlGains, physics: Physics, latency_frames: usize, smooth: bool) -> f32 {
        run_sim_at(g, physics, latency_frames, smooth, 1.0 / 60.0)
    }

    fn run_sim_at(
        g: ControlGains,
        physics: Physics,
        latency_frames: usize,
        smooth: bool,
        dt: f32,
    ) -> f32 {
        let mut sim = Sim::new(physics, latency_frames);
        sim.smooth = smooth;
        let mut tracker = Tracker::default();
        let t0 = Instant::now();
        let mut hits = 0;
        let frames = (12.0 / dt) as usize;
        let warmup = (1.0 / dt) as usize;
        for i in 0..frames {
            let now = t0 + Duration::from_secs_f32(i as f32 * dt);
            let d = tracker.step(&g, sim.fish, sim.marker, now);
            sim.tick(d.hold, dt, i);
            if i > warmup && sim.on_target() {
                hits += 1;
            }
        }
        hits as f32 / (frames - warmup - 1) as f32
    }

    #[test]
    #[ignore]
    fn dump_sim() {
        let physics = Physics {
            accel_hold: 2.5,
            accel_release: -2.5,
            max_speed: 1.2,
            latency_ms: 50.0,
            calibrated_at: 0,
        };
        let g = ControlGains {
            mode: ControlMode::Physics,
            hysteresis: 0.0,
            velocity_smoothing: 0.3,
            physics,
            ..Default::default()
        };
        let mut sim = Sim::new(physics, 3);
        let mut tracker = Tracker::default();
        let dt = 1.0 / 60.0;
        let t0 = Instant::now();
        for i in 0..360 {
            let now = t0 + Duration::from_secs_f32(i as f32 * dt);
            let d = tracker.step(&g, sim.fish, sim.marker, now);
            sim.tick(d.hold, dt, i);
            if i % 4 == 0 {
                eprintln!(
                    "{i:3} fish {:.3} fv {:+.2} marker {:.3} mv {:+.2} pred {:+.3} hold {} on {}",
                    sim.fish,
                    sim.fish_v,
                    sim.marker,
                    sim.marker_v,
                    d.predicted_error,
                    d.hold as u8,
                    sim.on_target() as u8
                );
            }
        }
    }

    #[test]
    fn physics_controller_tracks_moving_target() {
        let physics = Physics {
            accel_hold: 2.5,
            accel_release: -2.5,
            max_speed: 1.2,
            latency_ms: 50.0,
            calibrated_at: 0,
        };
        let g = ControlGains {
            mode: ControlMode::Physics,
            hysteresis: 0.0,
            velocity_smoothing: 0.3,
            physics,
            ..Default::default()
        };
        let score = run_sim(g, physics, 3);
        assert!(score > 0.7, "on-target fraction {score}");
        let smooth = run_sim_with(g, physics, 3, true);
        assert!(smooth > 0.9, "smooth on-target fraction {smooth}");
    }

    #[test]
    fn physics_controller_tracks_at_low_frame_rate() {
        let physics = Physics::default();
        let g = ControlGains {
            mode: ControlMode::Physics,
            hysteresis: 0.0,
            velocity_smoothing: 0.3,
            physics,
            ..Default::default()
        };
        let score = run_sim_at(g, physics, 1, false, 1.0 / 8.0);
        let baseline = run_sim_at(
            ControlGains {
                mode: ControlMode::Lookahead,
                ..g
            },
            physics,
            1,
            false,
            1.0 / 8.0,
        );
        let smooth = run_sim_at(g, physics, 1, true, 1.0 / 8.0);
        assert!(score > 0.5, "on-target fraction at 8 fps {score}");
        assert!(smooth > 0.9, "smooth on-target fraction at 8 fps {smooth}");
        assert!(
            score > baseline,
            "physics {score} should beat lookahead {baseline} at 8 fps"
        );
    }

    #[test]
    fn lookahead_controller_is_reasonable_baseline() {
        let physics = Physics {
            accel_hold: -2.5,
            accel_release: 2.5,
            max_speed: 1.2,
            latency_ms: 50.0,
            calibrated_at: 0,
        };
        let g = ControlGains {
            mode: ControlMode::Lookahead,
            lookahead_ms: 150.0,
            hysteresis: 0.0,
            velocity_smoothing: 0.3,
            physics,
            ..Default::default()
        };
        let score = run_sim(g, physics, 3);
        assert!(score > 0.6, "on-target fraction {score}");
    }

    #[test]
    fn physics_controller_handles_downward_hold() {
        let physics = Physics {
            accel_hold: -2.5,
            accel_release: 2.5,
            max_speed: 1.2,
            latency_ms: 50.0,
            calibrated_at: 0,
        };
        let g = ControlGains {
            mode: ControlMode::Physics,
            hysteresis: 0.0,
            velocity_smoothing: 0.3,
            physics,
            ..Default::default()
        };
        let score = run_sim(g, physics, 3);
        assert!(score > 0.7, "on-target fraction {score}");
        let smooth = run_sim_with(g, physics, 3, true);
        assert!(smooth > 0.9, "smooth on-target fraction {smooth}");
    }
}
