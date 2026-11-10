//! Earned pets in `Gameplay`: the verified facts that move a pet's earn counters, and the tick
//! step that hands over the pets they earn. The table and the counters live in
//! `crate::pets::earn`; nothing here trusts a client value, every fact was already settled by
//! its own system before it reaches these functions.
use super::*;
use crate::{quest_engine::QuestEvent, station_cash::EarnSource};

impl Gameplay {
    /// Count one quest-engine fact for the earn table. The quest engine sees every dungeon
    /// clear, kill, gather, craft and pet trick, with or without a quest that wants it. Its
    /// `FinishMinigame` is not counted here: fishing raises one per catch and harvesting and
    /// mining one per run, so minigames are counted by `pet_earn_minigame`. Its `JobLevel` is
    /// raised only for an open quest goal, so ranks are read by `pet_earn_job_ranks`.
    pub(in crate::gameplay) fn pet_earn_quest_event(&mut self, event: &QuestEvent) {
        let pets = &mut self.pets;
        match *event {
            QuestEvent::CompleteDungeon { encounter_id } => {
                pets.earn_bump("dungeon", 1);
                pets.earn_bump(&format!("dungeon:{encounter_id}"), 1);
            }
            QuestEvent::KillActorKind { count, .. } => pets.earn_bump("kill", count),
            QuestEvent::GatherResource { quantity, .. } => pets.earn_bump("gather", quantity),
            QuestEvent::CraftRecipe { quantity, .. } => pets.earn_bump("craft", quantity),
            QuestEvent::PetTrick { .. } => pets.earn_bump("pet_trick", 1),
            _ => {}
        }
    }

    /// Count one minigame that ended with a result: a kart race, a derby match, a card duel, a
    /// Pirate's Plunder game or a client game's winning result record. Called once per settle.
    pub(in crate::gameplay) fn pet_earn_minigame(&mut self, minigame_id: u32) {
        self.pets.earn_bump("minigame", 1);
        self.pets.earn_bump(&format!("minigame:{minigame_id}"), 1);
    }

    /// Read every unlocked job's rank into the rank counters. Called where a rank can have
    /// changed (`sync_job_level_objectives`: an XP award, a payout, a turn-in, the developer
    /// level), never on a steady tick.
    pub(in crate::gameplay) fn pet_earn_job_ranks(&mut self) {
        let Some(profiles) = &self.profiles else {
            return;
        };
        for &profile in &profiles.state().unlocked {
            let rank = profiles.rank(profile);
            self.pets.earn_reach("job_level", rank);
            self.pets.earn_reach(&format!("job:{profile}"), rank);
        }
    }

    /// Count one paid completion (the Station Cash earn sources) and its activity day. A dungeon
    /// clear is counted by its quest-engine fact, so it adds only the day here.
    pub(in crate::gameplay) fn pet_earn_source(&mut self, source: EarnSource) {
        let pets = &mut self.pets;
        match source {
            EarnSource::Quest { id } => {
                pets.earn_bump("quest", 1);
                pets.earn_bump(&format!("quest:{id}"), 1);
            }
            EarnSource::KartRace => pets.earn_bump("kart_race", 1),
            EarnSource::DerbyMatch => pets.earn_bump("derby_match", 1),
            EarnSource::SoccerMatch => pets.earn_bump("soccer_match", 1),
            EarnSource::ChessGame { .. } => pets.earn_bump("chess_game", 1),
            EarnSource::DungeonClear { .. } => {}
        }
        pets.earn_day(crate::station_cash::activity_day(current_unix_seconds()));
    }

    /// The pet tick's earn step: pet items a reward left in the bag become pets, then every due
    /// row of the earn table is paid, each with one chat line naming the pet and the next goal.
    pub(super) fn claim_earned_pets(&mut self, now_ms: u64, out: &mut Outcome) {
        let now = now_ms / 1_000;
        self.pets
            .set_earn_day(crate::station_cash::activity_day(current_unix_seconds()));
        let adopted = self.pets.adopt_bag_items(&mut self.inventory, now);
        out.changed |= !adopted.is_empty();
        out.replies.extend(adopted);

        let claim = self.pets.claim_earned(now);
        for pet in &claim.earned {
            out.replies.push(chat::system(&format!(
                "You earned a new pet: {}! ({}) Open your Pets window to meet it.",
                pet.name, pet.goal
            )));
            if let Some((name, goal, have, at)) = &pet.next {
                out.replies.push(chat::system(&format!(
                    "Next pet: {name}. {goal} ({have}/{at})."
                )));
            }
        }
        if claim.roster_full {
            out.replies.push(chat::system(
                "You earned a new pet, but your pet list is full. It will join you when there is room.",
            ));
        }
        out.changed |= !claim.earned.is_empty();
        out.replies.extend(claim.replies);
        // Counters alone mark a save now and then, so a string of kills is one save.
        out.changed |= self.pets.take_earn_unsaved(now_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::t013_objective_events::game;

    fn rules(rows: serde_json::Value) -> crate::pets::EarnRules {
        let content = crate::pets::PetService::bundled_content().unwrap();
        let table = serde_json::json!({ "schema_version": 1, "enabled": true, "rows": rows });
        crate::pets::EarnRules::from_json(&serde_json::to_vec(&table).unwrap(), &content).unwrap()
    }

    fn row(key: &str, counter: &str, at: u32, item: i32) -> serde_json::Value {
        serde_json::json!({
            "key": key, "counter": counter, "at": at, "item_id": item, "name": key,
            "goal": "goal", "kind": "reconstructed", "basis": "test"
        })
    }

    fn who() -> CharacterState {
        CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        }
    }

    fn pets(game: &Gameplay) -> usize {
        game.pets_document()
            .and_then(|roster| roster["pets"].as_array().map(Vec::len))
            .unwrap_or(0)
    }

    /// The two hub hooks: a paid completion (`pay_earned_station_cash`, here a quest turn-in and a
    /// kart race) and a quest-engine fact (`npc_quest_event`, here a dungeon clear and a settled
    /// minigame) move their counters without a Marketplace or a quest engine, and the pet tick
    /// pays each due row once.
    #[test]
    fn completions_and_quest_engine_facts_earn_pets_on_the_pet_tick() {
        let mut game = game();
        game.pets.set_earn_rules(rules(serde_json::json!([
            row("quest", "quest", 2, 70_352),
            row("that-quest", "quest:77", 1, 17_841),
            row("race", "kart_race", 1, 70_353),
            row("dungeon", "dungeon:12303", 1, 70_354),
            row("game", "minigame", 2, 70_355),
        ])));
        let mut out = Outcome::default();
        game.pay_earned_station_cash(EarnSource::Quest { id: 5 }, &mut out)
            .unwrap();
        assert!(
            game.pet_tick(&who(), 10_000)
                .is_none_or(|tick| !tick.changed)
        );
        assert_eq!(pets(&game), 0, "one quest is one short");

        game.pay_earned_station_cash(EarnSource::Quest { id: 77 }, &mut out)
            .unwrap();
        game.pay_earned_station_cash(EarnSource::KartRace, &mut out)
            .unwrap();
        game.npc_quest_event(
            QuestEvent::CompleteDungeon {
                encounter_id: 12_303,
            },
            &mut out,
        )
        .unwrap();
        // A fish catch or a harvest run raises the quest fact alone and is no minigame here.
        game.npc_quest_event(QuestEvent::FinishMinigame { minigame_id: 9 }, &mut out)
            .unwrap();
        assert_eq!(game.pets.earn_count("minigame"), 0);
        for _ in 0..2 {
            let result = crate::quest_engine::MinigameResult::finished_at(None);
            game.credit_minigame(9, result, &mut out).unwrap();
        }
        assert_eq!(game.pets.earn_count("minigame"), 2, "one per settled game");
        let tick = game
            .pet_tick(&who(), 11_000)
            .expect("the earn step has something to send");
        assert!(tick.changed, "a new pet marks the save");
        assert_eq!(pets(&game), 5);
        let lines = tick
            .replies
            .iter()
            .filter(|packet| String::from_utf8_lossy(packet).contains("You earned a new pet"))
            .count();
        assert_eq!(lines, 5, "one line per pet");
        assert!(tick.replies.contains(&game.pets.list_packet()));

        // Nothing pays twice, and more of the same facts change nothing.
        game.pay_earned_station_cash(EarnSource::Quest { id: 77 }, &mut out)
            .unwrap();
        game.pet_tick(&who(), 12_000);
        assert_eq!(pets(&game), 5);
    }

    /// A job rank reaches its counter through a real XP award, with no quest that has a
    /// job-level goal (the quest engine's `JobLevel` fact exists only for such a goal).
    #[test]
    fn a_rank_gained_by_an_xp_award_earns_the_rank_pet() {
        let mut game = game();
        game.pets.set_earn_rules(rules(serde_json::json!([row(
            "rank",
            "job_level",
            2,
            70_352
        )])));
        let profile = game.profiles().unwrap().active();
        assert_eq!(game.job_level(profile), 1);
        game.pet_tick(&who(), 10_000);
        assert_eq!(pets(&game), 0);

        let mut out = Outcome::default();
        for _ in 0..500 {
            if game.job_level(profile) >= 2 {
                break;
            }
            game.award_job_xp(100, &mut out).unwrap();
        }
        assert!(game.job_level(profile) >= 2, "the awards reached rank 2");
        assert_eq!(game.pets.earn_count("job_level"), game.job_level(profile));
        game.pet_tick(&who(), 11_000);
        assert_eq!(pets(&game), 1, "the rank pet is paid");
    }

    /// A pet item a reward left in the bag (the reward applier keeps no grant list) is adopted by
    /// the next pet tick and leaves the bag.
    #[test]
    fn a_reward_pet_item_in_the_bag_is_adopted_on_the_pet_tick() {
        let mut game = game();
        game.inventory
            .grant(0, 70_352, 1)
            .expect("the pet item is a known definition");
        let held = |game: &Gameplay| {
            game.inventory
                .state()
                .items
                .iter()
                .filter(|item| item.definition_id == 70_352)
                .count()
        };
        assert_eq!((held(&game), pets(&game)), (1, 0));
        let tick = game.pet_tick(&who(), 20_000).expect("the adoption is sent");
        assert!(tick.changed);
        assert_eq!((held(&game), pets(&game)), (0, 1));
    }
}
