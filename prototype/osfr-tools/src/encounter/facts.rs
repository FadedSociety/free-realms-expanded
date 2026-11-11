//! What a fight reports to the rest of the server: kills and fight events (combat design
//! section 12). The engine fills `EncounterOutcome::kills` and `::facts`; gameplay's
//! `quest_combat` turns them into quest events. The older `defeated`, `abilities_used` and
//! `win_completed` fields are still filled exactly as before, until quest credit stops reading
//! them.
//!
//! Frozen by the combat seam: the types and their fields do not change. Each piece fills its
//! own fields through the `stamp_kill_*` hook in its own module.
use super::*;

/// How strong an enemy kind is; health and damage follow it (`scale`).
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    /// A crate, an egg, a spawner.
    Object,
    Minion,
    #[default]
    Normal,
    Tough,
    Elite,
    /// A battle's boss.
    Boss,
    DungeonBoss,
    /// An open-world named boss.
    NamedBoss,
    WorldBoss,
}

/// One enemy this player gets credit for.
#[derive(Clone, Debug, PartialEq)]
pub struct KillFact {
    /// The enemy's definition id (what `defeat` quest goals name as `model_id`).
    pub kind: u32,
    pub guid: u64,
    pub rank: Rank,
    pub tags: Vec<String>,
    pub zone: u32,
    pub area: Option<u32>,
    /// The battle, dungeon or fight the kill happened in.
    pub fight: Option<u32>,
    /// One name on the kill's credit list.
    pub solo: bool,
    /// The credited player's active profile (job) id.
    pub job: u32,
    pub ability: Option<i32>,
    pub item: Option<i32>,
}

/// Something that happened in a fight, other than a kill.
#[derive(Clone, Debug, PartialEq)]
pub enum FightFact {
    Entered {
        fight: u32,
    },
    Goal {
        fight: u32,
        goal: String,
        done: u32,
        of: u32,
    },
    Won {
        fight: u32,
    },
    Lost {
        fight: u32,
    },
    Freed {
        kind: u32,
        guid: u64,
    },
    /// An accepted ability cast and the kind of its enemy target, if it had one.
    Used {
        ability: i32,
        on_kind: Option<u32>,
    },
}

impl EncounterEngine {
    /// This player's credit for one defeated enemy: the old `defeated` entry and its kill fact,
    /// always together. `definition` 0 or below names no quest target and records nothing.
    /// `shared` marks a kill another player made that this player is credited for.
    pub(super) fn record_kill(
        &self,
        definition: i32,
        guid: u64,
        boss: bool,
        shared: bool,
        out: &mut EncounterOutcome,
    ) {
        let Ok(kind) = u32::try_from(definition) else {
            return;
        };
        if kind == 0 {
            return;
        }
        out.defeated.push(definition);
        let mut fact = KillFact {
            kind,
            guid,
            rank: if boss { Rank::Boss } else { Rank::Normal },
            tags: Vec::new(),
            // Not the last zone the solo tick saw: a shared-field kill right after an arena
            // would report the arena.
            zone: self
                .dungeon()
                .map_or(self.content.world.zone_id, |d| d.zone.zone_id),
            area: None,
            fight: (self.state.active.as_ref())
                .map(|a| self.fight_state.doors.fight(a.encounter_id)),
            solo: !shared
                && self.arena.acting.is_none()
                && self
                    .field_sim
                    .kills
                    .hits
                    .get(&guid)
                    .is_none_or(|hitters| hitters.len() <= 1),
            job: u32::try_from(self.player_profile).unwrap_or(0),
            ability: None,
            item: None,
        };
        self.stamp_kill_scale(&mut fact);
        self.stamp_kill_kit(&mut fact);
        self.stamp_kill_run(&mut fact);
        out.kills.push(fact);
    }
}
