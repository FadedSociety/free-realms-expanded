//! PB2 group play: the tunable values of shared credit and the job a group row shows. They come
//! from `group_play.json` in the content directory; the shipped file is compiled in, so a
//! directory without it gets the shipped values, and a present but bad file refuses the load.
//!
//! Shared credit (who gets a kill, collect or objective another member made): a member of the
//! same group, in the same zone instance, and either inside the same battle or dungeon instance
//! (the whole instance counts) or within `credit_range_m` in the open world. The quest engine
//! then counts it only for a member on that step, like any other event.
use serde::Deserialize;
use std::{fs, io, path::Path};

use crate::social_lists::MemberJob;

const FILE: &str = "group_play.json";
const SHIPPED: &str = include_str!("../data/content/group_play.json");
const MAX_BYTES: u64 = 64 * 1024;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GroupPlay {
    pub schema_version: u32,
    /// (R) Open-world reach of shared credit, in metres.
    pub credit_range_m: f32,
    /// Per job: the icon and badge image set its group row shows (captured profile records).
    pub job_icons: Vec<JobIcon>,
    pub source: Vec<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct JobIcon {
    pub profile_id: i32,
    pub icon: i32,
    pub badge: i32,
}

impl GroupPlay {
    /// The shipped values, which the file's own tests check.
    pub fn shipped() -> Self {
        Self::parse(SHIPPED.as_bytes()).unwrap_or(Self {
            schema_version: 1,
            credit_range_m: 0.0,
            job_icons: Vec::new(),
            source: Vec::new(),
        })
    }

    /// The content directory's file, or the shipped one when it has none.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join(FILE);
        if !path.is_file() {
            return Ok(Self::shipped());
        }
        let refuse = |e: &dyn std::fmt::Display| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {e}", path.display()),
            )
        };
        if fs::metadata(&path).map_err(|e| refuse(&e))?.len() > MAX_BYTES {
            return Err(refuse(&"file too large"));
        }
        Self::parse(&fs::read(&path).map_err(|e| refuse(&e))?).map_err(|e| refuse(&e))
    }

    fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rules: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut ids: Vec<i32> = rules.job_icons.iter().map(|j| j.profile_id).collect();
        ids.sort_unstable();
        ids.dedup();
        if rules.schema_version != 1
            || !(1.0..=1000.0).contains(&rules.credit_range_m)
            || ids.len() != rules.job_icons.len()
            || rules.job_icons.len() > 256
            || rules.source.is_empty()
        {
            return Err("group play rules out of bounds".into());
        }
        Ok(rules)
    }

    /// The row a member with `profile_id` at `rank` shows. A job with no icon row keeps the
    /// client's own defaults for the icons (-1, as the record constructor `008dcd60` sets).
    pub fn member_job(&self, profile_id: i32, rank: i32) -> MemberJob {
        let icons = self.job_icons.iter().find(|j| j.profile_id == profile_id);
        MemberJob {
            profile_id,
            icon: icons.map_or(-1, |j| j.icon),
            badge: icons.map_or(-1, |j| j.badge),
            rank,
        }
    }

    /// Whether a credit made at `from` reaches a member standing at `to` (horizontal metres).
    /// `whole_instance` is a battle or dungeon instance, where distance does not matter.
    pub fn in_reach(&self, from: [f32; 3], to: [f32; 3], whole_instance: bool) -> bool {
        let flat = ((from[0] - to[0]).powi(2) + (from[2] - to[2]).powi(2)).sqrt();
        whole_instance || flat <= self.credit_range_m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_rules_load_and_name_the_medic_row() {
        let rules = GroupPlay::parse(SHIPPED.as_bytes()).unwrap();
        assert_eq!(rules.credit_range_m, 60.0);
        let medic = rules.member_job(11, 7);
        assert_eq!((medic.icon, medic.badge, medic.rank), (1340, 13, 7));
        assert_eq!(
            rules.member_job(999, 1).icon,
            -1,
            "an unknown job keeps the defaults"
        );
        assert!(rules.in_reach([0.0; 3], [59.0, 30.0, 0.0], false));
        assert!(!rules.in_reach([0.0; 3], [61.0, 0.0, 0.0], false));
        assert!(rules.in_reach([0.0; 3], [600.0, 0.0, 0.0], true));
    }

    #[test]
    fn a_bad_file_is_refused() {
        let bad = SHIPPED.replace("\"credit_range_m\": 60.0", "\"credit_range_m\": 0.0");
        assert!(GroupPlay::parse(bad.as_bytes()).is_err());
        let dup = SHIPPED.replace("\"profile_id\": 2,", "\"profile_id\": 1,");
        assert!(GroupPlay::parse(dup.as_bytes()).is_err());
    }
}
