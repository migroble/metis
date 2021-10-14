use chrono_tz::{Etc::UTC, ParseError, Tz};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use slotmap::{DefaultKey, SlotMap};
use std::{default::Default, str::FromStr};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Schedule")]
struct ScheduleDef {
    #[serde(getter = "Schedule::to_string")]
    expr: String,
}

impl From<ScheduleDef> for Schedule {
    fn from(def: ScheduleDef) -> Schedule {
        Schedule::from_str(&def.expr).expect("Error parsing cron expression")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    #[serde(with = "ScheduleDef")]
    pub sched: Schedule,
    pub msg: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChannelData {
    pub tz: Tz,
    pub reminders: SlotMap<DefaultKey, Reminder>,
}

impl Default for ChannelData {
    fn default() -> Self {
        Self {
            tz: UTC,
            reminders: SlotMap::new(),
        }
    }
}

impl ChannelData {
    pub fn set_tz(&mut self, tz_str: &str) -> Result<(), ParseError> {
        self.tz = tz_str.parse()?;

        Ok(())
    }
}
