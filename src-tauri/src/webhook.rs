use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use serde_json::{json, Value};

use crate::config::Settings;
use crate::core::fruit::{DropInfo, SpawnInfo};
use crate::events::Stats;

const COLOR_BLUE: u32 = 0x3B82F6;
const COLOR_PURPLE: u32 = 0x8B5CF6;
const COLOR_GOLD: u32 = 0xF59E0B;
const COLOR_GREEN: u32 = 0x22C55E;
const COLOR_RED: u32 = 0xEF4444;

pub struct WebhookQueue {
    tx: Option<Sender<(String, Value)>>,
    settings: Option<Arc<RwLock<Settings>>>,
}

static LAST_POST: Mutex<Option<Instant>> = Mutex::new(None);
pub fn validate_url(raw: &str) -> Result<reqwest::Url, String> {
    let invalid = || "Use an HTTPS discord.com/api/webhooks/<id>/<token> URL".to_string();
    if raw.len() > 512 {
        return Err(invalid());
    }
    let url = reqwest::Url::parse(raw).map_err(|_| invalid())?;
    if url.scheme() != "https"
        || url.host_str() != Some("discord.com")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid());
    }
    let parts: Vec<_> = url.path().split('/').collect();
    if parts.len() != 5
        || parts[1] != "api"
        || parts[2] != "webhooks"
        || !(17..=22).contains(&parts[3].len())
        || !parts[3].bytes().all(|b| b.is_ascii_digit())
        || !(20..=200).contains(&parts[4].len())
        || !parts[4]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(invalid());
    }
    Ok(url)
}

impl WebhookQueue {
    pub fn disabled() -> Self {
        Self {
            tx: None,
            settings: None,
        }
    }

    pub fn start(settings: Arc<RwLock<Settings>>) -> Arc<Self> {
        let (tx, rx) = bounded::<(String, Value)>(32);
        let s2 = Arc::clone(&settings);
        std::thread::Builder::new()
            .name("webhook".into())
            .spawn(move || worker(rx, s2))
            .expect("spawn webhook worker");
        Arc::new(Self {
            tx: Some(tx),
            settings: Some(settings),
        })
    }

    fn enabled(&self) -> bool {
        self.settings
            .as_ref()
            .map(|s| {
                let s = s.read();
                s.webhook.enabled && validate_url(&s.webhook.url).is_ok()
            })
            .unwrap_or(false)
    }

    fn send(&self, embed: Value) {
        if !self.enabled() {
            return;
        }
        if let Some(tx) = &self.tx {
            let Some(settings) = &self.settings else {
                return;
            };
            let url = settings.read().webhook.url.clone();
            // A full queue drops notifications, never stalls fishing or grows RAM.
            let _ = tx.try_send((
                url,
                json!({ "embeds": [embed], "allowed_mentions": { "parse": [] } }),
            ));
        }
    }

    pub fn test(&self) -> Result<(), String> {
        if !self.enabled() {
            return Err("Enable your valid webhook before sending a test".into());
        }
        let url = self
            .settings
            .as_ref()
            .map(|s| s.read().webhook.url.clone())
            .unwrap_or_default();
        if url.trim().is_empty() {
            return Err("Webhook URL is empty".into());
        }
        post(
            &url,
            &json!({ "embeds": [embed("Webhook connected", "AutoFish Local can reach this channel.", COLOR_GREEN, vec![])], "allowed_mentions": { "parse": [] } }),
        ).map(|_| ())
    }

    pub fn progress(&self, st: Stats) {
        self.send(embed(
            "Fishing progress",
            "",
            COLOR_BLUE,
            vec![
                field("Fish", &st.fish.to_string()),
                field("Fruits", &st.fruits.to_string()),
                field("Runtime", &fmt_runtime(st.runtime_s)),
                field("Success", &format!("{:.0}%", st.success_rate * 100.0)),
            ],
        ));
    }

    pub fn fruit_drop(&self, d: &DropInfo) {
        let (title, desc, color) = if d.is_legendary {
            (
                "Legendary devil fruit dropped",
                "Pity reset to 0. You got a **legendary** devil fruit!",
                COLOR_GOLD,
            )
        } else {
            (
                "Devil fruit dropped",
                "You got a devil fruit.",
                COLOR_PURPLE,
            )
        };
        self.send(embed(title, desc, color, vec![]));
    }

    pub fn spawn(&self, info: &SpawnInfo) {
        let at = info
            .location
            .as_deref()
            .map(|l| format!(" at **{l}**"))
            .unwrap_or_default();
        let (title, desc, color) = match &info.name {
            Some(n) => (
                "Devil fruit spawned",
                format!("**{n}** has spawned{at}."),
                COLOR_PURPLE,
            ),
            None => (
                "Devil fruit spawned",
                format!("A devil fruit has spawned{at}!"),
                COLOR_BLUE,
            ),
        };
        self.send(embed(title, &desc, color, vec![]));
    }

    pub fn purchase(&self, amount: u32) {
        self.send(embed(
            "Bait purchased",
            &format!("Bought {amount} bait."),
            COLOR_GREEN,
            vec![],
        ));
    }

    pub fn recovery(&self, attempt: u32, reason: &str) {
        let flag = self
            .settings
            .as_ref()
            .map(|s| s.read().webhook.recovery)
            .unwrap_or(false);
        if !flag {
            return;
        }
        self.send(embed(
            "Recovery",
            reason,
            COLOR_RED,
            vec![field("Attempt", &attempt.to_string())],
        ));
    }
}

fn embed(title: &str, desc: &str, color: u32, fields: Vec<Value>) -> Value {
    json!({
        "title": title,
        "description": desc,
        "color": color,
        "fields": fields,
        "footer": { "text": "AutoFish Local v4" },
        "timestamp": chrono_now(),
    })
}

fn field(name: &str, value: &str) -> Value {
    json!({ "name": name, "value": value, "inline": true })
}

fn fmt_runtime(s: u64) -> String {
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let rem = secs % 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn post(url: &str, body: &Value) -> Result<u16, String> {
    let url = validate_url(url)?;
    {
        let mut last = LAST_POST.lock();
        if last.is_some_and(|t| t.elapsed() < Duration::from_secs(5)) {
            return Err("Notification rate limit: wait five seconds".into());
        }
        *last = Some(Instant::now());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .map_err(|_| "Could not initialize notification client".to_string())?;
    let resp = client
        .post(url)
        .json(body)
        .send()
        .map_err(|_| "Notification request failed (details redacted)".to_string())?;
    let status = resp.status();
    if status.is_success() {
        Ok(status.as_u16())
    } else {
        Err(format!("Discord returned {}", status.as_u16()))
    }
}

fn worker(rx: Receiver<(String, Value)>, settings: Arc<RwLock<Settings>>) {
    for (destination, body) in rx.iter() {
        // Never redirect queued events to a newly configured destination.
        let s = settings.read();
        if !s.webhook.enabled || s.webhook.url != destination {
            continue;
        }
        drop(s);
        if let Err(e) = post(&destination, &body) {
            tracing::warn!("webhook: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restricts_destinations_without_network_requests() {
        let valid =
            "https://discord.com/api/webhooks/123456789012345678/abcdefghijklmnopqrstuvwxyz";
        assert!(validate_url(valid).is_ok());
        for bad in ["http://discord.com/api/webhooks/123456789012345678/abcdefghijklmnopqrstuvwxyz", "https://discord.com.evil.test/api/webhooks/123456789012345678/abcdefghijklmnopqrstuvwxyz", "https://127.0.0.1/api/webhooks/123456789012345678/abcdefghijklmnopqrstuvwxyz", "https://discord.com@evil.test/api/webhooks/123456789012345678/abcdefghijklmnopqrstuvwxyz", "https://discord.com/api/webhooks/123/short"] {
            let error = validate_url(bad).unwrap_err();
            assert!(!error.contains("abcdefghijklmnopqrstuvwxyz"));
        }
        assert!(validate_url(&format!("{valid}?wait=true")).is_err());
        assert!(validate_url(&format!("{valid}/extra")).is_err());
    }
    #[test]
    fn disabled_test_never_contacts_network() {
        assert!(WebhookQueue::disabled().test().is_err());
    }
}
