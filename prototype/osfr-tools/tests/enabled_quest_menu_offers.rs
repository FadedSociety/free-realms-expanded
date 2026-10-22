//! T384: every enabled quest is offered by its own giver through the real NPC
//! menu, and no disabled quest is.
//!
//! The enabled set, the giver union and the titles come from the production
//! loader (`QuestEngine::from_split_content`), the giver's position from
//! `NpcInteraction::load` (the row the 26/8 range check reads) and the clicked
//! runtime actor from `world::NpcVisibility` (the 35/2 draw set and its
//! `content_key`). The raw records in `tests/support/quest_records.rs` only add
//! what the loader does not expose: prerequisite ids, the job/level gate and the
//! authored title id.
//!
//! Every offer goes through `Gameplay::handle_in_world` as a client sends it:
//! 26/8 interact (constructor `0x008b6bc0`), the menu's Talk row through 26/10
//! (`0x00a36f80`), then the 26/3 quest row through 26/6 (`0x009f5890`). The
//! reply is the 49/1 card (reader `0x00c7bb60`, body `0x00c7b7a0`, reward bundle
//! `0x008e7930`), decoded with the production reader. Another quest the giver
//! offers is accepted with 49/2 (`0x00a86400`) and a turn-in a Talk opens (49/13,
//! `0x00c7b990`) is finished with 49/14 (`0x00a864c0`), as a player would.
//! Goodbye is answered with the empty 26/4 EndDialog (client reader
//! `0x00aa0870`). No new wire.
//!
//! The save a quest needs is one a player reaches by playing: its whole
//! prerequisite chain completed and its job active at its `min_level` on the
//! local 100-XP rank curve. Loading a save costs seconds (`NpcVisibility::load`
//! dominates), so quests share one per (job, chain size), except that a
//! mutually exclusive quest (T465) gets its own: a quest inside
//! another's chain has a strictly smaller chain, so no shared save completes a
//! quest it must offer. A disabled prerequisite can only be the historical
//! completed entry `repair_counters` keeps; [`HISTORICAL_PREREQUISITE`] names the
//! two enabled quests T314 already reports for that, and any other one fails.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    CHARACTER, Fixture, QUEST_CHOICE, TALK_LABEL, accept, assert_packet_changes_nothing,
    character_at_for, content_dir, dialogue_choice, menu_option, npc_interact, npc_select,
    pending_offer, quest_end_reply, quest_reply, quest_status, read_json, saved,
    seed_active_profile,
};
use osfr_tools::{
    character_store::CharacterState,
    content::{Job, VersionedFile},
    game_store,
    gameplay::{Gameplay, GameplayState, Outcome},
    npc_interaction::{
        NpcInteraction, QuestOfferPacket49_1, decode_dialogue_26_3, decode_quest_offer_49_1,
        goodbye_dialogue_choice, quest_dialogue_choice,
    },
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestStatus},
    world::NpcVisibility,
};
use quest_records::{RawQuest, read_quests};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

const OTHER: u64 = CHARACTER + 1;
/// Enabled records on the T384 tree (tip `33200d2`). Fewer means content was
/// disabled or lost; the failure prints the new count.
const SHIPPED_ENABLED: usize = 50;
/// Enabled quests whose chain runs through a disabled record (`3020013`,
/// `3100019`), so only a historical save reaches their offer. T314 reports both;
/// re-enabling or re-pointing the prerequisite is a content task.
const HISTORICAL_PREREQUISITE: [u32; 2] = [3_020_014, 3_100_020];
/// The ordinary client requests the gate sends. A dev `realm` command
/// (offer, goto, place) is a 15/x chat request and never one of these.
const MENU_REQUESTS: [[u8; 4]; 5] = [
    [26, 0, 8, 0],
    [26, 0, 10, 0],
    [26, 0, 6, 0],
    [49, 0, 2, 0],
    [49, 0, 14, 0],
];
/// `NpcInteraction::build_options` lists Talk first, as menu option 1.
const TALK_OPTION: i32 = 1;
/// More Talks than any giver has quests.
const MAX_TALKS: usize = 16;

type Offer = (Vec<[u32; 5]>, QuestOfferPacket49_1);

/// The shipped catalogue through its production loaders, loaded once per run.
struct Shipped {
    engine: QuestEngine,
    records: BTreeMap<u32, RawQuest>,
    npcs: NpcInteraction,
    /// Content NPC key -> the runtime actor the client clicks, from every zone's
    /// draw set (regional rows are all drawn in the open world, zone 0).
    actors: BTreeMap<u64, u64>,
}

fn shipped() -> &'static Shipped {
    static SHIPPED: OnceLock<Shipped> = OnceLock::new();
    SHIPPED.get_or_init(|| {
        let root = content_dir();
        let visibility = NpcVisibility::load(&root).expect("the NPC draw set loads");
        let zones: BTreeSet<u32> = visibility.model_actors().map(|(zone, ..)| zone).collect();
        let mut actors = BTreeMap::new();
        for target in zones
            .into_iter()
            .flat_map(|zone| visibility.teleport_targets(zone))
        {
            if let Some(key) = visibility.content_key(target.guid) {
                actors.entry(key).or_insert(target.guid);
            }
        }
        Shipped {
            engine: QuestEngine::from_split_content(
                &root,
                CHARACTER,
                CharacterEligibility::default(),
                None,
            )
            .expect("the split quest catalogue loads"),
            records: read_quests(&root)
                .into_iter()
                .map(|quest| (quest.id, quest))
                .collect(),
            npcs: NpcInteraction::load(&root.join("npcs"), &root.join("stores.json"))
                .expect("NPC content loads"),
            actors,
        }
    })
}

/// One giver's walk inside a shared save.
struct Walk {
    giver: u64,
    /// The quests of the save this giver gives.
    targets: Vec<u32>,
    offers: Result<Vec<Offer>, String>,
}

#[derive(Default)]
struct Pass {
    walks: Vec<Walk>,
    /// Quests whose giver has no placed, drawable actor: no menu can offer them.
    unplaced: Vec<u32>,
    /// Disabled prerequisites seeded as historical entries, by quest.
    historical: BTreeMap<u32, Vec<u32>>,
}

impl Shipped {
    fn enabled(&self) -> Vec<u32> {
        self.records
            .keys()
            .copied()
            .filter(|id| self.engine.is_enabled(*id))
            .collect()
    }

    /// Where `character_id` stands to click `npc` (its content row), and the
    /// runtime actor it clicks.
    fn giver(&self, character_id: u64, npc: u64) -> Option<(CharacterState, u64)> {
        let (zone, position) = self.npcs.location(npc)?;
        Some((
            character_at_for(character_id, zone, position),
            *self.actors.get(&npc)?,
        ))
    }

    /// The authored title id the 26/3 row and the 49/1 card carry.
    fn title(&self, quest: u32) -> u32 {
        u32::try_from(self.records[&quest].title_string_id).unwrap_or(0)
    }

    /// Every quest that must be completed before `quest`, transitively. The
    /// loader already refused unknown prerequisite ids.
    fn chain(&self, quest: u32) -> BTreeSet<u32> {
        let mut chain = BTreeSet::new();
        let mut open = self.records[&quest].prerequisite_quest_ids.clone();
        while let Some(id) = open.pop() {
            if chain.insert(id) {
                open.extend(&self.records[&id].prerequisite_quest_ids);
            }
        }
        chain
    }

    /// `base` after completing every chain of `quests` (one job), with that job
    /// active at their highest level. Also returns the disabled (historical)
    /// prerequisites it seeded, by quest.
    fn eligible(
        &self,
        base: &GameplayState,
        quests: &[u32],
    ) -> (GameplayState, BTreeMap<u32, Vec<u32>>) {
        let mut state = base.clone();
        let mut historical = BTreeMap::new();
        let progress = &mut state
            .npc_quests
            .as_mut()
            .expect("the fixture binds the quest catalogue")
            .quests;
        for &quest in quests {
            let chain = self.chain(quest);
            let disabled: Vec<u32> = chain
                .iter()
                .copied()
                .filter(|id| !self.engine.is_enabled(*id))
                .collect();
            if !disabled.is_empty() {
                historical.insert(quest, disabled);
            }
            for id in chain {
                // Empty counters: `repair_counters` fills an enabled quest's to
                // its required counts and keeps a disabled one as historical.
                progress.insert(
                    id,
                    QuestProgress {
                        status: QuestStatus::Completed,
                        counts: Vec::new(),
                    },
                );
            }
        }
        if let Some(job) = self.records[&quests[0]].job {
            seed_active_profile(&mut state, job, &[]);
        }
        if let Some(level) = quests
            .iter()
            .filter_map(|quest| self.records[quest].min_level)
            .max()
        {
            let profiles = state.profiles.as_mut().expect("the fixture has profiles");
            // `Profiles::rank`: 1 + XP / 100.
            profiles
                .experience
                .insert(profiles.active, level.saturating_sub(1) * 100);
        }
        (state, historical)
    }

    /// An offer from `giver`'s menu names one of its own enabled quests, with
    /// that quest's authored title on the 26/3 row and on the 49/1 card.
    fn check_offer(&self, giver: u64, (rows, offer): &Offer) -> Result<(), String> {
        let quest = offer.quest_id;
        if !self.engine.is_enabled(quest) {
            return Err(format!("offered disabled quest {quest}"));
        }
        if !self.engine.quest_ids_for_giver(giver).any(|id| id == quest) {
            return Err(format!("offered quest {quest}, which it does not give"));
        }
        let title = self.title(quest);
        if rows.first() != Some(&quest_dialogue_choice(title)) || offer.name_string_id != title {
            return Err(format!(
                "quest {quest}: 26/3 row {:?} and 49/1 title {} are not its title {title}",
                rows.first(),
                offer.name_string_id
            ));
        }
        Ok(())
    }

    /// One save per (job, chain size) of `quests`; in each, walk to every giver
    /// of those quests and take every offer its menu makes. A live event (T418,
    /// the scavenger hunt) is offered only while an admin has it open, so every
    /// walk runs with all of them open. Taking one mutually exclusive quest
    /// (T465) closes the others, so each of them gets a save of its own.
    fn walk_layers(&self, fixture: &Fixture, quests: &[u32]) -> Pass {
        let base = fixture.game(None).state();
        let live = fixture.open_live_events(self.engine.live_events().keys().copied());
        let mut layers = BTreeMap::<(Option<i32>, usize, Option<u32>), Vec<u32>>::new();
        for &quest in quests {
            let record = &self.records[&quest];
            let exclusive = (!record.excludes_quest_ids.is_empty()).then_some(quest);
            let key = (record.job, self.chain(quest).len(), exclusive);
            layers.entry(key).or_default().push(quest);
        }
        let mut pass = Pass::default();
        for layer in layers.values() {
            let (state, historical) = self.eligible(&base, layer);
            pass.historical.extend(historical);
            let mut givers = BTreeMap::<u64, Vec<u32>>::new();
            for &quest in layer {
                let giver = self.records[&quest].giver_npc_id;
                givers.entry(giver).or_default().push(quest);
            }
            let start = character_at_for(CHARACTER, 0, [0.0; 3]);
            let mut game = fixture.game(Some(state));
            game.set_world_dir(Some(live.clone())).unwrap();
            let mut client = Client::new(game, start);
            for (giver, targets) in givers {
                let Some((at, actor)) = self.giver(CHARACTER, giver) else {
                    pass.unplaced.extend(targets);
                    continue;
                };
                client.at = at;
                let offers = client.walk(actor, None);
                pass.walks.push(Walk {
                    giver,
                    targets,
                    offers,
                });
            }
        }
        pass
    }

    /// The first enabled quest a fresh character may take (no chain, job or
    /// level), offered through its giver's menu with the fixture's own save.
    fn root_offer(&self, fixture: &Fixture) -> (Client, u32, u64) {
        let quest = self
            .enabled()
            .into_iter()
            .find(|quest| {
                let record = &self.records[quest];
                record.prerequisite_quest_ids.is_empty()
                    && record.job.is_none()
                    && record.min_level.is_none()
            })
            .expect("an enabled root quest");
        let giver = self.records[&quest].giver_npc_id;
        let (at, actor) = self.giver(CHARACTER, giver).expect("a placed giver");
        let mut client = Client::new(fixture.game(None), at);
        let offers = client
            .walk(actor, Some(quest))
            .unwrap_or_else(|error| panic!("quest {quest}: {error}"));
        for offer in &offers {
            self.check_offer(giver, offer)
                .unwrap_or_else(|error| panic!("giver {giver}: {error}"));
        }
        (client, quest, actor)
    }
}

/// What one Talk and its 26/3 quest row produced.
enum Step {
    Offer(Offer),
    TurnIn(u32),
    Nothing,
}

/// One character's session. Every request goes through
/// `Gameplay::handle_in_world` and is kept for the dev-command scan.
struct Client {
    game: Gameplay,
    at: CharacterState,
    now: u64,
    sent: Vec<Vec<u8>>,
}

impl Client {
    fn new(game: Gameplay, at: CharacterState) -> Self {
        Self {
            game,
            at,
            now: 1_000,
            sent: Vec::new(),
        }
    }

    fn send(&mut self, packet: Vec<u8>) -> std::io::Result<Outcome> {
        self.now += 10;
        let out = self.game.handle_in_world(&packet, &self.at, self.now);
        self.sent.push(packet);
        out
    }

    /// 26/8, the menu's Talk row (26/10), then dialogue row `row` (26/6).
    /// Returns the 26/3 rows shown and the reply to the chosen row.
    fn talk(&mut self, actor: u64, row: i32) -> Result<(Vec<[u32; 5]>, Outcome), String> {
        let menu = self
            .send(npc_interact(actor))
            .map_err(|error| format!("26/8 refused: {error}"))?;
        let talk = menu_option(&menu, TALK_LABEL);
        let shown = self
            .send(npc_select(actor, talk))
            .map_err(|error| format!("Talk refused: {error}"))?;
        let rows = shown
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
            .and_then(|packet| decode_dialogue_26_3(packet).ok())
            .ok_or("Talk opened no 26/3 dialogue")?
            .choices;
        let reply = self
            .send(dialogue_choice(row))
            .map_err(|error| format!("26/6 refused: {error}"))?;
        Ok((rows, reply))
    }

    fn step(&mut self, actor: u64) -> Result<Step, String> {
        let (rows, reply) = self.talk(actor, QUEST_CHOICE)?;
        let mut offers = reply
            .replies
            .iter()
            .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
            .map(|packet| decode_quest_offer_49_1(packet))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("a 49/1 does not decode: {error:?}"))?;
        if offers.len() > 1 {
            return Err(format!("one Talk sent {} offers", offers.len()));
        }
        if let Some(offer) = offers.pop() {
            return Ok(Step::Offer((rows, offer)));
        }
        // 49/13: quest id at 14..18 (`0x00c7b990`).
        Ok(reply
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[49, 0, 13, 0]) && packet.len() >= 18)
            .map_or(Step::Nothing, |end| {
                Step::TurnIn(u32::from_le_bytes(end[14..18].try_into().unwrap()))
            }))
    }

    /// Talk to `actor` until it offers `target` (with `None`, until it offers
    /// nothing more), accepting every other offer and finishing every turn-in
    /// it opens. Returns the offers in order; with a target, the last is its.
    fn walk(&mut self, actor: u64, target: Option<u32>) -> Result<Vec<Offer>, String> {
        let mut offers = Vec::new();
        for _ in 0..MAX_TALKS {
            match self.step(actor)? {
                Step::Offer(offer) => {
                    let id = offer.1.quest_id;
                    offers.push(offer);
                    if Some(id) == target {
                        return Ok(offers);
                    }
                    let accepted = self
                        .send(quest_reply(id, true))
                        .map_err(|error| format!("accepting {id} failed: {error}"))?;
                    if !accepted.changed {
                        return Err(format!("the offer of {id} could not be accepted"));
                    }
                }
                Step::TurnIn(id) => {
                    self.send(quest_end_reply(id))
                        .map_err(|error| format!("turning in {id} failed: {error}"))?;
                }
                Step::Nothing if target.is_none() => return Ok(offers),
                Step::Nothing => {
                    let ids: Vec<u32> = offers.iter().map(|offer| offer.1.quest_id).collect();
                    return Err(format!("the menu stopped offering after {ids:?}"));
                }
            }
        }
        Err(format!("still offering after {MAX_TALKS} Talks"))
    }

    /// The ordinary Talk sequence from `from`, opened by `interact`, offers
    /// nothing and leaves the state and the save bytes equal. Refused requests
    /// are expected; whatever is accepted must not reach a 49/1.
    fn assert_offers_nothing(
        &mut self,
        from: &CharacterState,
        interact: Vec<u8>,
        actor: u64,
        label: &str,
    ) {
        let before = (self.game.state(), saved(&self.game));
        let home = std::mem::replace(&mut self.at, from.clone());
        let mut replies = Vec::new();
        for packet in [
            interact,
            npc_select(actor, TALK_OPTION),
            dialogue_choice(QUEST_CHOICE),
        ] {
            if let Ok(out) = self.send(packet) {
                replies.extend(out.replies);
            }
        }
        self.at = home;
        assert!(
            !replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
            "{label} offered a quest"
        );
        assert_eq!(self.game.state(), before.0, "{label} changed the state");
        assert_eq!(saved(&self.game), before.1, "{label} changed the save");
    }
}

#[test]
fn every_enabled_quest_is_offered_by_its_own_giver() {
    let shipped = shipped();
    let fixture = Fixture::new();
    let enabled = shipped.enabled();
    let pass = shipped.walk_layers(&fixture, &enabled);
    let mut failures: Vec<String> = pass
        .unplaced
        .iter()
        .map(|quest| format!("quest {quest}: its giver has no placed, drawable actor"))
        .collect();
    for walk in &pass.walks {
        let offers = match &walk.offers {
            Ok(offers) => offers,
            Err(error) => {
                failures.push(format!("giver {} {:?}: {error}", walk.giver, walk.targets));
                continue;
            }
        };
        let ids: Vec<u32> = offers.iter().map(|offer| offer.1.quest_id).collect();
        for target in &walk.targets {
            let count = ids.iter().filter(|id| *id == target).count();
            if count != 1 {
                failures.push(format!(
                    "quest {target}: giver {} offered it {count} times ({ids:?})",
                    walk.giver
                ));
            }
        }
        for offer in offers {
            if let Err(error) = shipped.check_offer(walk.giver, offer) {
                failures.push(format!("giver {}: {error}", walk.giver));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "enabled quests the real menu does not offer:\n{}",
        failures.join("\n")
    );
    assert!(
        pass.historical
            .keys()
            .all(|quest| HISTORICAL_PREREQUISITE.contains(quest)),
        "enabled quests only a save with a disabled prerequisite reaches: {:?}",
        pass.historical
    );
    println!(
        "{} enabled quests offered in {} giver walks; historical prerequisites: {:?}",
        enabled.len(),
        pass.walks.len(),
        pass.historical
    );
}

#[test]
fn a_disabled_quest_is_never_offered() {
    let shipped = shipped();
    let fixture = Fixture::new();
    // A disabled record with objectives is refused by the disabled flag alone;
    // an empty one is refused twice over and is not sampled. The loader binds
    // a quest to its own giver only, so that giver is the one menu that could
    // leak it, stale row listing included. Each walk takes every offer the
    // giver makes to a save that completed the record's chain.
    let sample: Vec<u32> = shipped
        .records
        .values()
        .filter(|record| !shipped.engine.is_enabled(record.id) && !record.objectives.is_empty())
        .map(|record| record.id)
        .collect();
    let pass = shipped.walk_layers(&fixture, &sample);
    for walk in &pass.walks {
        let offers = walk
            .offers
            .as_ref()
            .unwrap_or_else(|error| panic!("giver {}: {error}", walk.giver));
        for offer in offers {
            shipped
                .check_offer(walk.giver, offer)
                .unwrap_or_else(|error| panic!("giver {}: {error}", walk.giver));
        }
    }
    assert!(
        !pass.walks.is_empty(),
        "no disabled record has a placed giver"
    );
    println!(
        "disabled records walked at {} givers; no placed giver: {:?}",
        pass.walks.len(),
        pass.unplaced
    );
}

#[test]
fn an_out_of_range_or_wrong_giver_click_offers_nothing() {
    let shipped = shipped();
    let fixture = Fixture::new();
    let (mut client, quest, actor) = shipped.root_offer(&fixture);
    // Back to a quest the giver would offer, so every refusal below is the click's.
    assert!(client.send(quest_reply(quest, false)).unwrap().changed);
    let at = client.at.clone();

    // The wrong giver: a placed, drawable NPC that gives no enabled quest, so
    // its Talk has nothing to offer. A giver of disabled quests only comes
    // first (its Talk opens a real quest dialogue); any other NPC is the
    // fallback once content enables those.
    let (other_at, other) = shipped
        .actors
        .keys()
        .filter(|npc| {
            shipped
                .engine
                .quest_ids_for_giver(**npc)
                .all(|id| !shipped.engine.is_enabled(id))
        })
        .filter_map(|npc| {
            let no_quests = shipped.engine.quest_ids_for_giver(*npc).next().is_none();
            Some((no_quests, shipped.giver(CHARACTER, *npc)?))
        })
        .min_by_key(|(no_quests, _)| *no_quests)
        .map(|(_, giver)| giver)
        .expect("a placed NPC that gives no enabled quest");
    let mut far = at.clone();
    far.position[0] += 100.0;
    let mut wrong_zone = at.clone();
    wrong_zone.zone_id += 1;
    let truncated = npc_interact(actor)[..12].to_vec();
    let mut bad_flag = npc_interact(actor);
    bad_flag[12] = 2;
    for (label, from, interact, target) in [
        ("an out-of-range click", far, npc_interact(actor), actor),
        ("a wrong-zone click", wrong_zone, npc_interact(actor), actor),
        (
            "a Talk at the wrong giver",
            other_at,
            npc_interact(other),
            other,
        ),
        ("a truncated 26/8", at.clone(), truncated, actor),
        ("a 26/8 with an invalid flag", at.clone(), bad_flag, actor),
    ] {
        client.assert_offers_nothing(&from, interact, target, label);
    }
    // Control: the same save at the giver is offered the quest again.
    let offers = client
        .walk(actor, Some(quest))
        .unwrap_or_else(|error| panic!("quest {quest} after the refusals: {error}"));
    assert_eq!(offers.len(), 1, "only the declined quest was left to offer");
}

#[test]
fn the_offer_survives_a_real_reload_and_is_per_character() {
    let shipped = shipped();
    let fixture = Fixture::new();
    // A giver with two enabled root quests under one job: one Talk offers one,
    // the next Talk the other.
    let mut roots = BTreeMap::<(u64, Option<i32>), Vec<u32>>::new();
    for quest in shipped.enabled() {
        let record = &shipped.records[&quest];
        if record.prerequisite_quest_ids.is_empty() && record.min_level.is_none() {
            roots
                .entry((record.giver_npc_id, record.job))
                .or_default()
                .push(quest);
        }
    }
    let ((giver, _), pair) = roots
        .into_iter()
        .find(|(_, quests)| quests.len() >= 2)
        .expect("a giver with two root quests under one job");
    let (at, actor) = shipped.giver(CHARACTER, giver).expect("a placed giver");
    let (state, _) = shipped.eligible(&fixture.game(None).state(), &pair);
    let mut a = Client::new(fixture.game(Some(state)), at.clone());
    let Ok(Step::Offer((_, first))) = a.step(actor) else {
        panic!("giver {giver} offers nothing")
    };
    assert!(pair.contains(&first.quest_id));
    accept(&mut a.game, &at, first.quest_id, 2_000);

    // The production save envelope, then the bind a reconnecting session runs.
    let path = fixture.path().join("t384-character.json");
    let reload = |game: &Gameplay| {
        let snapshot = game_store::Snapshot {
            character: at.clone(),
            gameplay: Some(game.state()),
            pets: None,
        };
        game_store::save(&path, &snapshot).expect("the save writes");
        let loaded = game_store::load_or_create(&path, at.clone()).expect("the save loads");
        Client::new(fixture.game(loaded.gameplay), at.clone())
    };
    let mut a = reload(&a.game);
    assert_eq!(quest_status(&a.game, first.quest_id), QuestStatus::Active);
    assert_eq!(pending_offer(&a.game), None);
    let Ok(Step::Offer((_, second))) = a.step(actor) else {
        panic!("the reloaded giver offers nothing")
    };
    assert_ne!(second.quest_id, first.quest_id);
    assert!(pair.contains(&second.quest_id));
    // A reload at the pending offer keeps it, and A accepts it afterwards.
    let mut a = reload(&a.game);
    assert_eq!(pending_offer(&a.game), Some(second.quest_id));

    // A second character has its own offer set and cannot accept A's offer.
    let (state, _) = shipped.eligible(&fixture.game_as(OTHER, None).state(), &pair);
    let mut b = Client::new(
        fixture.game_as(OTHER, Some(state)),
        character_at_for(OTHER, at.zone_id, at.position),
    );
    let Ok(Step::Offer((_, own))) = b.step(actor) else {
        panic!("giver {giver} offers the second character nothing")
    };
    assert_eq!(
        own.quest_id, first.quest_id,
        "A's accept does not change B's offer"
    );
    let b_at = b.at.clone();
    assert_packet_changes_nothing(
        &mut b.game,
        &b_at,
        &quest_reply(second.quest_id, true),
        3_000,
        "B accepting A's pending offer",
    );
    assert_eq!(pending_offer(&b.game), Some(first.quest_id));
    // Each accepts its own offer.
    accept(&mut b.game, &b_at, first.quest_id, 3_000);
    accept(&mut a.game, &at, second.quest_id, 3_000);
}

#[test]
fn the_offer_menu_does_not_consume_the_quest_or_the_actor() {
    let shipped = shipped();
    let fixture = Fixture::new();
    let (mut client, quest, actor) = shipped.root_offer(&fixture);
    assert!(client.send(quest_reply(quest, false)).unwrap().changed);
    let before = (client.game.state(), saved(&client.game));

    // Goodbye on the quest dialogue: the empty 26/4 EndDialog, no offer.
    let goodbye = goodbye_dialogue_choice()[0] as i32;
    let (rows, closed) = client.talk(actor, goodbye).expect("the giver talks");
    let title = shipped.title(quest);
    assert_eq!(
        rows,
        [quest_dialogue_choice(title), goodbye_dialogue_choice()]
    );
    assert_eq!(closed.replies, [[26, 0, 4, 0]]);
    assert_eq!(client.game.state(), before.0, "Goodbye changed the state");
    assert_eq!(saved(&client.game), before.1, "Goodbye changed the save");

    // The same giver reopens and offers the same quest again.
    let Ok(Step::Offer((_, offer))) = client.step(actor) else {
        panic!("the giver did not reopen its offer")
    };
    assert_eq!(offer.quest_id, quest);
    // Declining it leaves no quest, item, coin or XP change behind.
    assert!(client.send(quest_reply(quest, false)).unwrap().changed);
    assert_eq!(client.game.state(), before.0, "the offer changed the state");
    assert_eq!(saved(&client.game), before.1, "the offer changed the save");
}

#[test]
fn the_gate_uses_no_dev_commands() {
    let shipped = shipped();
    let fixture = Fixture::new();
    let (client, quest, _) = shipped.root_offer(&fixture);
    for header in &MENU_REQUESTS[..3] {
        assert!(
            client.sent.iter().any(|packet| packet.starts_with(header)),
            "the pass never sent {header:?}"
        );
    }
    let other = client.sent.iter().find(|packet| {
        !MENU_REQUESTS
            .iter()
            .any(|header| packet.starts_with(header))
    });
    assert_eq!(
        other, None,
        "a request that is not an ordinary menu request"
    );
    // No teleport either: the player stood on the giver's content row.
    assert_eq!(
        Some((client.at.zone_id, client.at.position)),
        shipped.npcs.location(shipped.records[&quest].giver_npc_id)
    );
}

#[test]
fn the_enabled_set_is_the_shipped_one() {
    let shipped = shipped();
    let enabled = shipped.enabled();
    assert!(
        enabled.len() >= SHIPPED_ENABLED,
        "the production loader enables {} quests; the T384 tree shipped {SHIPPED_ENABLED}",
        enabled.len()
    );
    // `from_split_content` refuses a quest outside the jobs.json union; this
    // repeats the rule for the enabled set with the production row type.
    let jobs: VersionedFile<Job> =
        serde_json::from_value(read_json(&content_dir().join("jobs.json")))
            .expect("jobs.json decodes");
    let listed: BTreeSet<u32> = jobs
        .records
        .iter()
        .flat_map(|job| job.quest_ids.iter().copied())
        .collect();
    let orphans: Vec<u32> = enabled
        .iter()
        .copied()
        .filter(|quest| !listed.contains(quest))
        .collect();
    assert!(orphans.is_empty(), "enabled quests in no job: {orphans:?}");
    let no_giver: Vec<u32> = enabled
        .iter()
        .copied()
        .filter(|quest| {
            shipped
                .engine
                .giver_npc_id(*quest)
                .and_then(|giver| shipped.giver(CHARACTER, giver))
                .is_none()
        })
        .collect();
    assert!(
        no_giver.is_empty(),
        "enabled quests without a placed, drawable giver: {no_giver:?}"
    );
    println!("enabled quests (production loader): {}", enabled.len());
}
