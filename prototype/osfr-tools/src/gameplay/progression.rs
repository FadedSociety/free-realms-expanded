//! Progress-driven achievements over authoritative gameplay state.
//!
//! Job rank and completed-quest counters are read from the same saved state the
//! rest of gameplay mutates. Only a counter that actually moved is observed, so
//! an ordinary request or tick emits no progress packet and clones no reward
//! state when nothing was earned. Every transaction that can move a counter
//! settles its achievements here: job XP (`apply_encounter`, mining/harvesting
//! and the derby), quest completion, and the minigame payouts applied by
//! `tick`. The request-only hook left tick and shared-field rewards unnoticed
//! until the next reconnect. A claim/title capacity failure defers the award to
//! a later transaction instead of failing the reward transaction, because the
//! same settlement runs inside the shared-field tick.
//!
//! Three kinds of counter feed the rules in `achievements.json` (the key vocabulary is in its
//! `counter_binding_report`):
//! * observed: a value read from saved state (job rank, quests, pets, mounts, recipes, other
//!   achievements), settled by `settle_achievement_counters` around every request and tick;
//! * events: `npc_quest_event` is the one funnel every kill, dungeon win, minigame result, pet
//!   trick and item use already passes through, so `achievement_events_for` turns the same
//!   event into counter increments there (a hook in another file calls `achievement_event`);
//! * `waiting:<event>`: raised by a game or hook that does not exist yet; nothing fires them.
use super::*;

/// Session-only achievement state, boxed in `Gameplay` (it is cloned at every staging site).
#[derive(Clone, Debug, Default)]
pub(super) struct AchievementSession {
    /// The last overworld region the player stood in; 0 before the first classification.
    region: u32,
    /// Where `region` was classified, so a player standing still costs one comparison.
    at: Option<[f32; 2]>,
}

/// The achievement number inside a key (`wiki:achievement:097:music-student` -> 97).
fn key_number(key: &str) -> Option<u32> {
    key.split(':').nth(2)?.parse().ok()
}

impl Gameplay {
    /// Data-bound counter values: one rank counter per unlocked job, the completed regional
    /// story quest total, and every state-derived counter of a rule that is still open.
    pub(super) fn achievement_counters(&self) -> BTreeMap<String, u32> {
        let mut counters = BTreeMap::<String, u32>::new();
        if let Some(profiles) = &self.profiles {
            for profile in profiles
                .state()
                .unlocked
                .iter()
                .copied()
                .chain(profiles.state().experience.keys().copied())
            {
                if let Some(slug) = self.achievement_bindings.job_slug(profile) {
                    counters.insert(
                        format!("job_rank:{slug}"),
                        profiles.rank(profile).saturating_sub(1),
                    );
                }
            }
        }
        if let Some(quests) = &self.npc_quests {
            // A repeatable quest taken again keeps counting (`QuestState::completed_count`).
            counters.insert(
                "quest_completed_count".into(),
                u32::try_from(quests.state().completed_count()).unwrap_or(1_000_000),
            );
        }
        self.observe_derived_counters(&mut counters);
        counters
    }

    /// Counters computed from saved state for every rule that has not completed yet.
    fn observe_derived_counters(&self, counters: &mut BTreeMap<String, u32>) {
        let state = self.progression_extras.achievement_state_ref();
        let done_numbers: BTreeSet<u32> = state
            .completed
            .iter()
            .filter_map(|key| key_number(key))
            .collect();
        for rule in self.achievement_bindings.rules() {
            if state.completed.contains(&rule.key) || counters.contains_key(&rule.counter_key) {
                continue;
            }
            if let Some(value) = self.derived_counter(&rule.counter_key, &done_numbers) {
                counters.insert(rule.counter_key.clone(), value);
            }
        }
    }

    fn derived_counter(&self, key: &str, done_numbers: &BTreeSet<u32>) -> Option<u32> {
        let (kind, arg) = key.split_once(':').unwrap_or((key, ""));
        match kind {
            "quest_done" => {
                let id: u32 = arg.parse().ok()?;
                let quest = self.npc_quests.as_ref()?.state().quests.get(&id)?;
                Some(u32::from(
                    quest.status == crate::quest_engine::QuestStatus::Completed,
                ))
            }
            "completed_of" => Some(
                arg.split('|')
                    .filter_map(|n| n.parse::<u32>().ok())
                    .filter(|n| done_numbers.contains(n))
                    .count() as u32,
            ),
            "job_rank_max" => {
                let profiles = self.profiles.as_ref()?;
                let state = profiles.state();
                state
                    .unlocked
                    .iter()
                    .copied()
                    .chain(state.experience.keys().copied())
                    .map(|profile| profiles.rank(profile))
                    .max()
            }
            "opening_complete" => Some(u32::from(self.opening.as_ref()?.complete())),
            "pets_owned" => Some(u32::try_from(self.pets.pet_count()).unwrap_or(u32::MAX)),
            "mounts_owned" => {
                let mounts = self.mounts.as_ref()?.catalog();
                Some(mounts.owned_mounts(self.inventory.state()).len() as u32)
            }
            "mounts_owned_of" => {
                // The named rides by mount definition id (United We Stand's five combiner rides).
                let wanted: BTreeSet<i32> = arg.split('|').filter_map(|n| n.parse().ok()).collect();
                let mounts = self.mounts.as_ref()?.catalog();
                let owned = mounts
                    .owned_mounts(self.inventory.state())
                    .into_iter()
                    .filter(|m| wanted.contains(&m.id))
                    .count();
                Some(owned as u32)
            }
            "recipes_all" => {
                // (R) Sacred Grove blacksmithing = the original recipe ids below 100; Sunstone
                // Valley = the expansion ids from 169.
                let (job, group) = arg.split_once(':')?;
                let prefix = format!("{job}.");
                let ids: Vec<u32> = self
                    .recipe_catalog
                    .records
                    .iter()
                    .filter(|r| r.key.starts_with(&prefix))
                    .filter_map(|r| r.recipe_id)
                    .filter(|id| match group {
                        "lt100" => *id < 100,
                        "ge100" => *id >= 100,
                        _ => false,
                    })
                    .collect();
                Some(u32::from(
                    !ids.is_empty() && ids.iter().all(|id| self.learned_recipes.contains(id)),
                ))
            }
            _ => None,
        }
    }

    /// Remember the last overworld region the player stood in: a kill inside a battle
    /// instance counts for the region the fight was started from.
    pub(super) fn note_achievement_region(&mut self, character: &CharacterState) {
        let at = [character.position[0], character.position[2]];
        if self.achievement_bindings.rules().is_empty() || self.achievement_session.at == Some(at) {
            return;
        }
        let region = self
            .region_table()
            .and_then(|table| table.region_at(character.zone_id, at[0], at[1]));
        if let Some(region) = region {
            self.achievement_session.region = region;
            self.achievement_session.at = Some(at);
        }
    }

    /// 150/4: every live row, before any 150/7 or 150/3 can mean anything to the client.
    pub(super) fn append_achievement_definitions(&mut self, out: &mut Outcome) {
        if self.achievement_bindings.client_rows().is_empty() {
            return;
        }
        let state = self.progression_extras.achievement_state_ref();
        out.replies.push(self.achievement_bindings.initialize_packet(state));
    }

    /// Counter keys of the form `<prefix>:<id>|<id>...` that name this id.
    fn list_keys_for(&self, prefix: &str, id: u32) -> Vec<String> {
        let mut keys: BTreeSet<&str> = BTreeSet::new();
        for rule in self.achievement_bindings.rules() {
            if let Some(list) = rule.counter_key.strip_prefix(prefix) {
                if list.split('|').any(|n| n.parse::<u32>().ok() == Some(id)) {
                    keys.insert(rule.counter_key.as_str());
                }
            }
        }
        keys.into_iter().map(str::to_owned).collect()
    }

    /// A winning minigame settle that carries no result facts (`npc_minigame_win_id_only`: the
    /// song runs of the Music Stand, cooking, smelting). The result-event path counts the same
    /// win through `MinigameResult`; a settle takes exactly one of the two.
    pub(super) fn achievement_minigame_win(&mut self, minigame_id: u32, out: &mut Outcome) {
        let events = self
            .list_keys_for("minigame_win:", minigame_id)
            .into_iter()
            .map(|key| (key, 1))
            .collect();
        self.apply_achievement_events(events, out);
    }

    /// The counter increments one quest event is worth, read before the quest engine takes it.
    pub(super) fn achievement_events_for(
        &self,
        event: &crate::quest_engine::QuestEvent,
    ) -> Vec<(String, u32)> {
        use crate::quest_engine::{DanceTeam, QuestEvent};
        let rules = self.achievement_bindings.rules();
        let mut out: Vec<(String, u32)> = Vec::new();
        if rules.is_empty() {
            return out;
        }
        let keys_with = |prefix: &str| -> BTreeSet<String> {
            rules
                .iter()
                .filter(|r| r.counter_key.starts_with(prefix))
                .map(|r| r.counter_key.clone())
                .collect()
        };
        match event {
            QuestEvent::CompleteDungeon { encounter_id } => {
                if let Some(dungeon) = self.achievement_bindings.dungeon_for_fight(*encounter_id) {
                    out.push((format!("dungeon_clear:{dungeon}"), 1));
                }
            }
            QuestEvent::Kill { fact } => {
                let Some(name) = self.achievement_bindings.kind_name(fact.kind) else {
                    return out;
                };
                for key in keys_with("kill:") {
                    let mut parts = key.splitn(3, ':').skip(1);
                    let region: u32 = parts
                        .next()
                        .and_then(|r| r.parse().ok())
                        .unwrap_or(u32::MAX);
                    let pattern = parts.next().unwrap_or("");
                    if (region == 0 || region == self.achievement_session.region) && name.contains(pattern)
                    {
                        out.push((key, 1));
                    }
                }
            }
            QuestEvent::MinigameResult {
                minigame_id,
                result,
            } if result.won => {
                out.extend(self.list_keys_for("minigame_win:", *minigame_id).into_iter().map(|k| (k, 1)));
                if self
                    .activities
                    .cards()
                    .is_some_and(|cards| cards.is_duel_row(*minigame_id))
                {
                    out.push(("tcg_win".into(), 1));
                    for key in keys_with("tcg_win_under:") {
                        let limit: Option<u32> = key["tcg_win_under:".len()..].parse().ok();
                        if let (Some(limit), Some(elapsed)) = (limit, result.elapsed_seconds) {
                            if elapsed < limit {
                                out.push((key, 1));
                            }
                        }
                    }
                }
            }
            QuestEvent::PetTrick { .. } => out.push(("pet_trick".into(), 1)),
            QuestEvent::UseItem { quantity, .. } if *quantity > 0 => {
                out.push(("consumable_use".into(), *quantity));
            }
            QuestEvent::JoinDanceOff { team } => out.push((
                match team {
                    DanceTeam::Vampire => "join_dance_team:vampire",
                    DanceTeam::Werewolf => "join_dance_team:werewolf",
                }
                .into(),
                1,
            )),
            _ => {}
        }
        out.retain(|(key, _)| rules.iter().any(|r| &r.counter_key == key));
        out
    }

    /// Raise one named achievement event. A hook in another builder's file calls this.
    #[allow(dead_code)]
    pub(crate) fn achievement_event(&mut self, key: &str, amount: u32, out: &mut Outcome) {
        self.apply_achievement_events(vec![(key.to_owned(), amount)], out);
    }

    /// Add event increments to every rule bound to those counters. A refusal (capacity, bad
    /// amount) is logged and never fails the transaction that raised the event.
    pub(super) fn apply_achievement_events(
        &mut self,
        events: Vec<(String, u32)>,
        out: &mut Outcome,
    ) {
        if events.is_empty() {
            return;
        }
        if let Err(error) = self.try_apply_achievement_events(events, out) {
            eprintln!("achievement: event deferred: {error}");
        }
    }

    fn try_apply_achievement_events(
        &mut self,
        events: Vec<(String, u32)>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let rewards = self.reward_snapshot();
        let mut achievements = crate::progression_extras::AchievementProgression::new(
            self.achievement_bindings.rules().to_vec(),
            self.progression_extras.achievement_state(),
        )?;
        let mut packets = Vec::new();
        let mut completed = Vec::new();
        for (key, amount) in events {
            let transition = achievements.on_event(&key, amount, &rewards)?;
            packets.extend(transition.packets);
            if !transition.newly_completed.is_empty() {
                packets.extend(transition.reward_packets);
                completed.extend(transition.newly_completed);
            }
        }
        if completed.is_empty() {
            // Progress only: keep the counters and tell the client; nothing is awarded.
            if !packets.is_empty() {
                self.progression_extras
                    .set_achievement_state(achievements.state().clone());
                out.replies.extend(packets);
                out.changed = true;
            }
            return Ok(());
        }
        self.commit_achievements(&achievements, packets, &completed, out)?;
        // Another achievement may be the sum of the ones just completed.
        self.sync_achievement_progress(out, None)
    }

    /// Stage the whole completion before committing it. A refusal after one
    /// claim was issued would otherwise leave a partial award that can never be
    /// retried; a deferred award leaves no title, claim or counter behind.
    fn commit_achievements(
        &mut self,
        achievements: &crate::progression_extras::AchievementProgression,
        mut packets: Vec<Vec<u8>>,
        completed: &[String],
        out: &mut Outcome,
    ) -> io::Result<()> {
        let mut staged = self.progression_extras.clone();
        for binding in self.achievement_bindings.claim_bindings() {
            if completed.contains(&binding.achievement_key) {
                staged.issue_server_grant(binding.claim_id, binding.grant.clone())?;
            }
        }
        packets.extend(staged.award_completed_achievement_titles(
            completed,
            self.achievement_bindings.title_bindings(),
        )?);
        staged.set_achievement_state(achievements.state().clone());
        self.progression_extras = staged;
        out.replies.extend(packets);
        out.changed = true;
        Ok(())
    }

    /// Observe authoritative persisted progress. Job promotions and completed
    /// regional quests feed the data-bound achievement rules; only a newly
    /// completed rule emits packets, grants its title or marks the character
    /// changed (R). Progress-only observation stays session-local.
    ///
    /// A capacity refusal (claim inbox or title book full) defers the award to
    /// a later transaction instead of failing the reward transaction that moved
    /// the counter. `apply_encounter` also runs inside the shared-field tick,
    /// where an `Err` would freeze every player in the instance; the moved
    /// counter is re-observed by the next transaction once capacity exists.
    pub(super) fn sync_achievement_progress(
        &mut self,
        out: &mut Outcome,
        only: Option<&BTreeSet<String>>,
    ) -> io::Result<()> {
        if self.achievement_bindings.rules().is_empty() {
            return Ok(());
        }
        // A completion can finish a sum of achievements (`completed_of`): look again, with every
        // counter, until a pass completes nothing.
        let mut only = only;
        for _ in 0..4 {
            match self.try_sync_achievement_progress(out, only) {
                Ok(true) => only = None,
                Ok(false) => break,
                Err(error) => {
                    eprintln!("achievement: settlement deferred: {error}");
                    break;
                }
            }
        }
        Ok(())
    }

    /// One observation pass; `true` when it completed an achievement.
    fn try_sync_achievement_progress(
        &mut self,
        out: &mut Outcome,
        only: Option<&BTreeSet<String>>,
    ) -> io::Result<bool> {
        let counters = self.achievement_counters();
        let rewards = self.reward_snapshot();
        let before = self.progression_extras.achievement_state();
        let mut achievements = crate::progression_extras::AchievementProgression::new(
            self.achievement_bindings.rules().to_vec(),
            before,
        )?;
        let mut packets = Vec::new();
        let mut completed = Vec::new();
        for (counter, value) in counters {
            if only.is_some_and(|keys| !keys.contains(&counter)) {
                continue;
            }
            let transition = achievements.observe(&counter, value, &rewards)?;
            if !transition.newly_completed.is_empty() {
                packets.extend(transition.packets);
                packets.extend(transition.reward_packets);
                completed.extend(transition.newly_completed);
            }
        }
        if completed.is_empty() {
            return Ok(false);
        }
        self.commit_achievements(&achievements, packets, &completed, out)?;
        Ok(true)
    }

    /// World entry: catch up on anything earned while no transaction watched (a pet or ride
    /// granted by a path that does not settle). The packets go out now and the state saves with
    /// the next change or the logout save; a login alone never forces a save, so a replayed or
    /// refused request after reconnecting still changes nothing persisted.
    pub(super) fn sync_achievement_at_login(&mut self, out: &mut Outcome) -> io::Result<()> {
        let changed = out.changed;
        self.sync_achievement_progress(out, None)?;
        out.changed = changed;
        Ok(())
    }

    /// Compare the counters with a snapshot taken before one authoritative
    /// transaction and observe only the ones that moved. A transaction that
    /// changed nothing relevant does no achievement work.
    pub(super) fn settle_achievement_counters(
        &mut self,
        before: &BTreeMap<String, u32>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let moved: BTreeSet<String> = self
            .achievement_counters()
            .into_iter()
            .filter(|(key, value)| before.get(key) != Some(value))
            .map(|(key, _)| key)
            .collect();
        if !moved.is_empty() {
            self.sync_achievement_progress(out, Some(&moved))?;
        }
        Ok(())
    }
}

/// The funnel and the derived counters, walked on a real `Gameplay` with the shipped
/// `achievements.json` (the bundled content set is the fallback of `AchievementBindings::load`).
#[cfg(test)]
mod tests {
    use super::super::t013_objective_events::game;
    use super::*;
    use crate::encounter::{KillFact, Rank};
    use crate::quest_engine::{MinigameResult, QuestEvent};

    fn done(game: &Gameplay, number: u32) -> bool {
        game.progression_extras
            .achievement_state_ref()
            .completed
            .iter()
            .any(|key| key_number(key) == Some(number))
    }

    fn kill(kind: u32) -> QuestEvent {
        QuestEvent::Kill {
            fact: KillFact {
                kind,
                guid: 1,
                rank: Rank::Normal,
                tags: Vec::new(),
                zone: 0,
                area: None,
                fight: None,
                solo: true,
                job: 43,
                ability: None,
                item: None,
            },
        }
    }

    fn win(id: u32) -> QuestEvent {
        QuestEvent::MinigameResult {
            minigame_id: id,
            result: MinigameResult {
                won: true,
                elapsed_seconds: Some(100),
                score: None,
                placement: None,
                knocked_out: false,
            },
        }
    }

    #[test]
    fn kills_count_in_the_region_the_player_stood_in() {
        let mut game = game();
        let mut out = Outcome::default();
        const THUGAWUG: u32 = 4115;
        game.achievement_session.region = 8; // Snowhill: not Arrested Development's Southern Wilds
        for _ in 0..5 {
            game.npc_quest_event(kill(THUGAWUG), &mut out).unwrap();
        }
        assert!(!done(&game, 179));
        game.achievement_session.region = 1;
        for _ in 0..5 {
            game.npc_quest_event(kill(THUGAWUG), &mut out).unwrap();
        }
        assert!(done(&game, 179), "Arrested Development I after 5 kills");
        assert!(!done(&game, 180), "II needs 50");
    }

    #[test]
    fn an_enemy_that_exists_only_in_a_fight_file_counts_for_its_region() {
        // Sweetwater Climb (fight 14029) is the only Wugachug bixie fight.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/content/fights/14029.json");
        let fight: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let kind = fight["kinds"]
            .as_object()
            .unwrap()
            .values()
            .find(|k| k["name"].as_str().is_some_and(|n| n.to_lowercase().contains("bixie")))
            .and_then(|k| k["definition_id"].as_u64())
            .expect("a bixie kind in the fight") as u32;
        let mut game = game();
        let mut out = Outcome::default();
        game.achievement_session.region = 9; // Wugachug
        for _ in 0..5 {
            game.npc_quest_event(kill(kind), &mut out).unwrap();
        }
        assert!(done(&game, 191), "Bugging Out I");
    }

    #[test]
    fn a_dungeon_win_clears_the_dungeon_its_fight_belongs_to() {
        let mut game = game();
        let mut out = Outcome::default();
        // 13003 is the Bixie Hive goal fight in dungeons/bixie_hive.json.
        game.npc_quest_event(QuestEvent::CompleteDungeon { encounter_id: 13003 }, &mut out)
            .unwrap();
        assert!(done(&game, 106));
        assert!(!done(&game, 107));
    }

    #[test]
    fn minigame_wins_complete_their_rows_and_the_sum_row_follows() {
        let mut game = game();
        let mut out = Outcome::default();
        // A loss counts nothing.
        let loss = QuestEvent::MinigameResult {
            minigame_id: 201,
            result: MinigameResult {
                won: false,
                elapsed_seconds: None,
                score: None,
                placement: None,
                knocked_out: false,
            },
        };
        game.npc_quest_event(loss, &mut out).unwrap();
        assert!(!done(&game, 97));
        for id in 201..=205 {
            game.npc_quest_event(win(id), &mut out).unwrap();
        }
        // Checkers (game 21) and a chess row (20) raise the result event.
        game.npc_quest_event(win(21), &mut out).unwrap();
        game.npc_quest_event(win(20), &mut out).unwrap();
        assert!(done(&game, 104), "Checkers Boss");
        assert!(done(&game, 105), "Chess Master");
        for number in 97..=101 {
            assert!(done(&game, number), "Feeling the Beat row {number}");
        }
        assert!(done(&game, 102), "Maestro: all five songs");
    }

    #[test]
    fn pet_tricks_and_item_rewards_pay_once() {
        let mut game = game();
        let mut out = Outcome::default();
        game.npc_quest_event(QuestEvent::PetTrick { trick_id: 1 }, &mut out)
            .unwrap();
        assert!(done(&game, 254) && !done(&game, 255));
        // Mini Holiday Tree (item 911083) is queued as a claim once, when the row completes.
        assert_eq!(game.progression_extras.pending_count(), 0);
        game.achievement_event("waiting:gifting_tree_saved", 1, &mut out);
        assert!(done(&game, 309));
        assert_eq!(game.progression_extras.pending_count(), 1);
        game.achievement_event("waiting:gifting_tree_saved", 1, &mut out);
        assert_eq!(game.progression_extras.pending_count(), 1);
    }
}
