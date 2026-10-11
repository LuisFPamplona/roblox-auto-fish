use serde::{Deserialize, Serialize};

use crate::core::controller::Decision;
use crate::core::fruit::{DropInfo, SpawnInfo};
use crate::core::types::WindowInfo;
use crate::core::vision::Reading;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrackFrame {
    #[serde(flatten)]
    pub reading: Reading,
    #[serde(flatten)]
    pub decision: Decision,
    pub origin_dx: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotState {
    Stopped,
    WaitingForRoblox,
    InitialSetup,
    Casting,
    WaitingForBite,
    Tracking,
    PostCatch,
    Purchasing,
    StoringFruit,
    Recovering,
    Paused,
}

impl BotState {
    pub fn is_active(self) -> bool {
        !matches!(self, BotState::Stopped | BotState::Paused)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Lifetime {
    pub fish: u32,
    pub failed: u32,
    pub fruits: u32,
    pub bait_purchased: u32,
    pub runtime_s: u64,
    pub sessions: u32,
    pub last_fruit: Option<String>,
    pub last_spawn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub fish: u32,
    pub failed: u32,
    pub fruits: u32,
    pub bait_purchased: u32,
    pub success_rate: f32,
    pub runtime_s: u64,
    pub restarts: u32,
    pub last_fruit: Option<String>,
    pub last_spawn: Option<String>,
    pub total: Lifetime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub ts: u64,
    pub level: LogLevel,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BotEvent {
    State {
        state: BotState,
        detail: Option<String>,
    },
    Stats(Stats),
    Log(LogLine),
    Reading(Option<TrackFrame>),
    FruitDrop(DropInfo),
    FruitSpawn(SpawnInfo),
    Purchase {
        amount: u32,
    },
    Recovery {
        attempt: u32,
        reason: String,
    },
    Roblox(Option<WindowInfo>),
}

impl BotEvent {
    pub fn channel(&self) -> &'static str {
        match self {
            BotEvent::State { .. } => "bot:state",
            BotEvent::Stats(_) => "bot:stats",
            BotEvent::Log(_) => "bot:log",
            BotEvent::Reading(_) => "bot:reading",
            BotEvent::FruitDrop(_) => "bot:fruit_drop",
            BotEvent::FruitSpawn(_) => "bot:fruit_spawn",
            BotEvent::Purchase { .. } => "bot:purchase",
            BotEvent::Recovery { .. } => "bot:recovery",
            BotEvent::Roblox(_) => "roblox:changed",
        }
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
