//! T369: the Demo Derby Driver first chapter, and a finished derby as a server-observed
//! `finish_minigame` event.
//!
//! Records 3060004 ("Demo Derby: Rules of the Road", giver Dalton Narrowbrow) and 3060005
//! ("Demo Derby: License to Drive", giver Crash Davis) are the job's first playable chapter.
//! The finish credit comes only from the server's derby Results phase:
//! `Gameplay::pay_derby_completion` emits `QuestEvent::FinishMinigame { minigame_id: 39 }` (the
//! package's `game_id`, MiniGameData row 39 "Wildwood Demo Derby") after the once-per-instance
//! `take_completion` ledger hands over a finished match, whatever the flat payout. The shipped
//! package reaches Results through its score target, scored from bounded client 34/16 contact
//! reports; no `39/18` claim, cancel, reconnect or replay produces it.
//!
//! The tests drive the production request path: the 26/8 -> 26/10 NPC menu (trainer `Learn`,
//! derby offer, Talk), the 26/6 dialogue rows, the 49/1/49/2/49/13/49/14 quest handshake, the
//! 39/16 offer -> 39/5 Go -> 34/6 register -> phase/countdown acknowledgements -> 34/16 contacts
//! -> results lifecycle, the 26/42 close and a real `game_store` save/reload. Every packet shape
//! is an existing client-verified one; the field offsets are the readers cited in
//! `tests/support/npc_quest_chain.rs` (26/8 `0x008b6bc0`, 26/10 `0x00a36f80`, 26/6 `0x009f5890`,
//! 49/1 `0x00c7bb60`, 49/2 `0x00a86400`, 49/13 `0x00c7b990`, 49/14 `0x00a864c0`, 49/9
//! `0x00c7af60`, 49/10 `0x00c7b070`), re-checked for T369. Nothing here adds a packet.
//!
//! Fixture: the quest chapter's reference campaign plus the shipped derby package. The pinned
//! reference `profiles.json` predates the recovered job-outfit table, so the private copy gains
//! the active play package's profile-49 starter row (see `add_profile_49_outfit`). That is test
//! content, not a state insert, so learning the job and switching to it stay on the production
//! paths.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

#[path = "support/derby_play.rs"]
mod derby_play;

use derby_play::{countdown_for, hit_for, phase_for, register_for, start_request};
use npc_quest_chain::{
    ZoneQuest, ZoneQuests, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    coins, content_dir, counts, finish_quest, has_family, menu_option, npc_interact, npc_select,
    objective_activated, objective_complete, objective_update, offer, pending_turnin, profile_xp,
    quest_end_reply, quest_reply, quest_status, release_end_screen, saved, seed_quest, select_job,
    send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    derby::{DerbyMessage, DerbyPhase, ParticipantKey, VehiclePose, decode_inner, encode_inner},
    derby_gateway::DEMO_DERBY_DRIVER_PROFILE,
    game_store,
    gameplay::{Gameplay, GameplayState, Outcome},
    npc_interaction::NpcInteraction,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use quest_records::shipped_quest;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// The shipped arena drivers: the trainer gate and the chapter's two quest givers.
const MAD_JACK: u64 = 9_000_000_391;
const CRASH: u64 = 9_000_000_289;
const DALTON: u64 = 9_000_000_295;
/// `job_trainers.json` row 49 (Mad Jack teaches Demo Derby Driver).
const TRAINER_LABEL: u32 = 412_342;
/// `derby_gates.json` menu label.
const DERBY_LABEL: u32 = 3_053;
/// The fixture's second selectable job (unlocked and fully equipped by default).
const BRAWLER: i32 = 43;
/// Chapter quest ids and their authored rewards.
const RULES_OF_THE_ROAD: u32 = 3_060_004;
const LICENSE_TO_DRIVE: u32 = 3_060_005;
const RULES_COINS: u32 = 5;
const RULES_XP: u32 = 10;
const LICENSE_COINS: u32 = 20;
const LICENSE_XP: u32 = 25;
/// T313's shipped flat completion reward for one finished match.
const COMPLETION_COINS: u32 = 25;
const COMPLETION_XP: u32 = 10;
const CHARACTER: u64 = 369_001;
const OTHER_CHARACTER: u64 = 369_002;
const NOW: u64 = 10_000;
/// `data/content/derby_entry.json` rules used to drive a match to its results record.
const NPC_DRIVERS: usize = 7;
const KNOCKOUTS_TO_WIN: usize = 4;
const RESULTS_HOLD_MS: u64 = 10_000;

/// A quest-chapter fixture that also carries the shipped derby package.
struct Fixture {
    inner: npc_quest_chain::Fixture,
    guid: u64,
    interaction: NpcInteraction,
}

impl Fixture {
    fn new(guid: u64) -> Self {
        let inner = npc_quest_chain::Fixture::for_character(guid);
        for file in ["derby_entry.json", "derby_gates.json"] {
            fs::copy(content_dir().join(file), inner.path().join(file)).unwrap();
        }
        add_profile_49_outfit(inner.path());
        let interaction = NpcInteraction::load(
            &content_dir().join("npcs"),
            &content_dir().join("stores.json"),
        )
        .expect("the shipped NPC content loads");
        Self {
            inner,
            guid,
            interaction,
        }
    }

    fn game(&self) -> Gameplay {
        self.inner.game_as(self.guid, None)
    }

    fn game_from(&self, state: GameplayState) -> Gameplay {
        self.inner.game_as(self.guid, Some(state))
    }

    /// A character standing on the NPC's spawn, in the NPC's zone (the 12 m interaction reach).
    fn character(&self, npc: u64) -> CharacterState {
        let (zone_id, position) = self
            .interaction
            .location(npc)
            .unwrap_or_else(|| panic!("NPC {npc} is a positioned spawn"));
        CharacterState {
            guid: self.guid,
            zone_id,
            position,
            rotation: [0.0, 0.0, 1.0],
        }
    }
}

/// The pinned reference profile content predates the recovered job-outfit table. The active play
/// package (`lab/bundles/playtest-20261016T115811Z/content/profiles.json`, from the W82 report
/// `lab/reports/job-outfits-20261015/starter_items.json`) ships profile 49's starter pieces as
/// `[14481, 616, 623, 13998]`, the four class-22 derby pieces the packed inventory already
/// carries. Copy that row into the private fixture so the production learn/switch path can pass
/// the client's Chest/Legs check. Test content only; no state insert.
fn add_profile_49_outfit(path: &Path) {
    let file = path.join("profiles.json");
    let mut content: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    content["starter_items"]["49"] = json!([14481, 616, 623, 13998]);
    fs::write(&file, serde_json::to_vec(&content).unwrap()).unwrap();
}

fn unlocked(game: &Gameplay, profile: i32) -> bool {
    game.state()
        .profiles
        .as_ref()
        .expect("the fixture has profile content")
        .unlocked
        .contains(&profile)
}

fn experience(game: &Gameplay) -> BTreeMap<i32, u32> {
    game.state()
        .profiles
        .as_ref()
        .expect("the fixture has profile content")
        .experience
        .clone()
}

fn replies_text(outcome: &Outcome) -> String {
    outcome
        .replies
        .iter()
        .map(|packet| String::from_utf8_lossy(packet).into_owned())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Save through the production envelope and reload through the production world load.
fn save_and_reload(fixture: &Fixture, game: &Gameplay) -> Gameplay {
    let path: PathBuf = fixture
        .inner
        .path()
        .join(format!("character-{}.save", fixture.guid));
    let character = fixture.character(MAD_JACK);
    let snapshot = game_store::Snapshot {
        character,
        gameplay: Some(game.state()),
        pets: None,
    };
    game_store::save(&path, &snapshot).expect("the session save writes");
    let loaded =
        game_store::load_or_create(&path, snapshot.character).expect("the session save reads");
    fixture.inner.game_as(fixture.guid, loaded.gameplay)
}

/// Learn Demo Derby Driver from Mad Jack's real 26/8 -> 26/9 -> 26/10 menu, then pick it in the
/// ordinary 26/13 job list. No profile state is seeded for the positive path.
fn learn_and_activate(fixture: &Fixture, game: &mut Gameplay, now: &mut u64) {
    assert!(
        !unlocked(game, DEMO_DERBY_DRIVER_PROFILE),
        "the fixture starts without the derby job"
    );
    let character = fixture.character(MAD_JACK);
    *now += 10;
    let menu = npc_quest_chain::interact(game, &character, MAD_JACK, *now);
    let learn = menu_option(&menu, TRAINER_LABEL);
    *now += 1;
    let learned = send(game, &character, &npc_select(MAD_JACK, learn), *now);
    assert!(learned.changed, "the trainer menu must grant the job");
    assert!(
        unlocked(game, DEMO_DERBY_DRIVER_PROFILE),
        "the learned job must be unlocked"
    );
    assert!(
        game.profile_attachments(DEMO_DERBY_DRIVER_PROFILE)
            .unwrap()
            .starts_with(&4u32.to_le_bytes()),
        "the learned starter outfit (all four recovered pieces) is equipped for the new job"
    );
    *now += 1;
    let switched = send(
        game,
        &character,
        &select_job(DEMO_DERBY_DRIVER_PROFILE),
        *now,
    );
    assert!(
        switched.changed,
        "the learned job must be selectable: {}",
        replies_text(&switched)
    );
    assert_eq!(active_job(game), DEMO_DERBY_DRIVER_PROFILE);
}

/// Quest 3060004: offer, accept, the one Talk, its held 49/13 end screen and the 49/14 turn-in.
fn rules_of_the_road(fixture: &Fixture, game: &mut Gameplay, now: &mut u64) {
    let dalton = fixture.character(DALTON);
    *now += 10;
    offer(game, &dalton, DALTON, RULES_OF_THE_ROAD, *now);
    *now += 10;
    accept(game, &dalton, RULES_OF_THE_ROAD, *now);
    *now += 10;
    let talked = talk(game, &dalton, DALTON, *now);
    assert_eq!(counts(game, RULES_OF_THE_ROAD), vec![1]);
    assert_eq!(pending_turnin(game), Some(RULES_OF_THE_ROAD));
    assert!(
        !talked
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the end screen waits for the 26/6 dialogue choice"
    );
    *now += 10;
    release_end_screen(game, &dalton, RULES_OF_THE_ROAD, DALTON, 0, *now);
    *now += 10;
    finish_quest(
        game,
        &dalton,
        RULES_OF_THE_ROAD,
        DEMO_DERBY_DRIVER_PROFILE,
        RULES_COINS,
        RULES_XP,
        *now,
    );
}

/// Accept 3060005 through the real offer/accept packets.
fn accept_license(fixture: &Fixture, game: &mut Gameplay, now: &mut u64) {
    let crash = fixture.character(CRASH);
    *now += 10;
    offer(game, &crash, CRASH, LICENSE_TO_DRIVE, *now);
    *now += 10;
    accept(game, &crash, LICENSE_TO_DRIVE, *now);
    assert_eq!(counts(game, LICENSE_TO_DRIVE), vec![0, 0]);
}

/// T787: give the license quest's finish objective a result requirement (and a count)
/// before the session binds the catalogue. The shipped file in `data/content` is never
/// touched; `patch_quest` writes the fixture's narrowed copy.
fn patch_license_require(fixture: &Fixture, count: u32, require: Value) {
    fixture
        .inner
        .patch_quest("demo_derby_driver.json", LICENSE_TO_DRIVE, |record| {
            let goal = record["objectives"]
                .as_array_mut()
                .expect("license objectives")
                .iter_mut()
                .find(|goal| goal["kind"] == "finish_minigame")
                .expect("the license finish objective");
            goal["count"] = json!(count);
            goal["require"] = require;
        });
}

fn quit_request(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 6];
    bytes.extend(instance.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

fn pose_for(key: ParticipantKey, position: [f32; 3]) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: [
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            position[0],
            position[1],
            position[2],
            1.0,
        ]
        .map(f32::to_bits),
        vector_a_bits: [0; 4],
        vector_b_bits: [0; 4],
        controls_bits: [0; 3],
        flags: [0; 4],
        sequence: 1,
    }))
    .unwrap()
}

fn offer_instance(outcome: &Outcome) -> i32 {
    outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .map(|packet| i32::from_le_bytes(packet[3..7].try_into().unwrap()))
        .expect("the derby offer")
}

/// Take the driver's real derby menu option; returns the offered match instance.
fn start_match(fixture: &Fixture, game: &mut Gameplay, driver: u64, now: &mut u64) -> i32 {
    let character = fixture.character(driver);
    *now += 10;
    let menu = npc_quest_chain::interact(game, &character, driver, *now);
    let option = menu_option(&menu, DERBY_LABEL);
    *now += 100;
    let entered = send(game, &character, &npc_select(driver, option), *now);
    assert!(
        entered.changed,
        "the admitted derby choice must start a match: {}",
        replies_text(&entered)
    );
    offer_instance(&entered)
}

/// 39/5 Go, 34/6 register, the client-owned phase acknowledgements and the countdown; returns the
/// seven server-created NPC driver keys.
fn run_to_running(
    fixture: &Fixture,
    game: &mut Gameplay,
    instance: i32,
    now: &mut u64,
) -> Vec<ParticipantKey> {
    let character = fixture.character(MAD_JACK);
    let key = ParticipantKey(character.guid);
    let started = send(game, &character, &start_request(instance), *now);
    assert!(
        started
            .replies
            .iter()
            .any(|packet| packet[..3] == [39, 0, 17]),
        "39/5 Go must be acknowledged"
    );
    assert!(
        started
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 20])),
        "the native Go sends the type-11 active carrier"
    );
    *now += 100;
    let setup = send(game, &character, &register_for(key), *now);
    let npcs: Vec<_> = setup
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[34, 0, 10]))
        .filter_map(|packet| match decode_inner(packet) {
            Ok(DerbyMessage::CreateVehicle(vehicle)) if vehicle.key != key => Some(vehicle.key),
            _ => None,
        })
        .collect();
    assert_eq!(
        npcs.len(),
        NPC_DRIVERS,
        "the shipped max_drivers fills NPCs"
    );
    for value in 3..=8 {
        *now += 10;
        let out = send(game, &character, &phase_for(key, value), *now);
        assert!(!out.replies.is_empty(), "phase {value}");
    }
    for value in (0..=4).rev() {
        *now += 10;
        let out = send(game, &character, &countdown_for(key, value), *now);
        assert!(!out.replies.is_empty(), "countdown {value}");
    }
    assert_eq!(game.derby_phase(), Some(DerbyPhase::Running));
    npcs
}

/// Knock out all seven NPC drivers, wait out their respawn, then knock out four more for the
/// shipped first-to-22 target. The last contact record is the results one; its outcome returns.
fn finish_match(
    fixture: &Fixture,
    game: &mut Gameplay,
    npcs: &[ParticipantKey],
    now: &mut u64,
) -> Outcome {
    let character = fixture.character(MAD_JACK);
    let key = ParticipantKey(character.guid);
    send(
        game,
        &character,
        &pose_for(key, [0.0, 0.0, 0.0]),
        *now + 200,
    );
    for (index, npc) in npcs.iter().enumerate() {
        let at = *now + 300 + index as u64 * 100;
        let out = send(game, &character, &hit_for(key, *npc, 100.0), at);
        assert!(
            out.changed,
            "the server resolves contact {index}: {}",
            replies_text(&out)
        );
        assert_eq!(
            game.derby_phase(),
            Some(DerbyPhase::Running),
            "the first seven knockouts stay below the 22-point target"
        );
    }
    let respawn_at = *now + 8_000;
    send(
        game,
        &character,
        &pose_for(key, [0.0, 0.0, 0.0]),
        respawn_at - 100,
    );
    let respawn = game.tick(&character, respawn_at).unwrap();
    assert!(
        respawn
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[34, 0, 18])),
        "the knocked-out drivers reset"
    );
    for (index, npc) in npcs.iter().take(KNOCKOUTS_TO_WIN).enumerate() {
        let at = respawn_at + 100 + index as u64 * 100;
        let out = send(game, &character, &hit_for(key, *npc, 100.0), at);
        if index + 1 == KNOCKOUTS_TO_WIN {
            assert_eq!(
                game.derby_phase(),
                Some(DerbyPhase::Results),
                "the shipped 22-point target ends the match"
            );
            *now = at;
            return out;
        }
    }
    panic!("the shipped 22-point target was not reached");
}

/// One match driven to its results record.
struct Finished {
    instance: i32,
    npcs: Vec<ParticipantKey>,
    results: Outcome,
}

fn play_to_results(fixture: &Fixture, game: &mut Gameplay, driver: u64, now: &mut u64) -> Finished {
    let instance = start_match(fixture, game, driver, now);
    let npcs = run_to_running(fixture, game, instance, now);
    let results = finish_match(fixture, game, &npcs, now);
    Finished {
        instance,
        npcs,
        results,
    }
}

/// The results hold enables the 26/42 Close; taking it retires the match and restores the job.
fn close_match(fixture: &Fixture, game: &mut Gameplay, finished_at: u64) -> Outcome {
    let character = fixture.character(MAD_JACK);
    let ready = game
        .tick(&character, finished_at + RESULTS_HOLD_MS)
        .unwrap();
    assert!(
        ready.replies.iter().any(|packet| {
            packet.starts_with(&[34, 0, 1])
                && decode_inner(packet) == Ok(DerbyMessage::GameState(11))
        }),
        "the results hold enables Close"
    );
    let closed = send(game, &character, &[26, 0, 42, 0], finished_at + 10_100);
    assert!(closed.changed, "an accepted 26/42 Close retires the match");
    assert!(!game.derby_active());
    closed
}

fn count_quest_packets(outcome: &Outcome, quest: u32, subtype: u16) -> usize {
    outcome
        .replies
        .iter()
        .filter(|packet| has_family(packet, 49, subtype, Some(quest)))
        .count()
}

#[test]
fn the_chapter_plays_and_credits_one_finished_derby_exactly_once() {
    let fixture = Fixture::new(CHARACTER);
    let mut game = fixture.game();
    let mut now = NOW;

    learn_and_activate(&fixture, &mut game, &mut now);
    assert_eq!(
        game.state().profiles.as_ref().unwrap().active,
        DEMO_DERBY_DRIVER_PROFILE,
        "the job gate is reachable outside a match"
    );
    rules_of_the_road(&fixture, &mut game, &mut now);
    accept_license(&fixture, &mut game, &mut now);

    let before_coins = coins(&game);
    let before_xp = profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE);
    let finished = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);

    // The finish objective advanced exactly once, in order, and opened the return Talk.
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![1, 0]);
    assert_eq!(
        count_quest_packets(&finished.results, LICENSE_TO_DRIVE, 9),
        1,
        "one 49/9 objective update: {}",
        replies_text(&finished.results)
    );
    assert_eq!(
        count_quest_packets(&finished.results, LICENSE_TO_DRIVE, 10),
        1
    );
    assert!(objective_update(&finished.results, LICENSE_TO_DRIVE, 1, 1));
    assert!(objective_complete(&finished.results, LICENSE_TO_DRIVE, 1));
    assert!(
        objective_activated(&finished.results, LICENSE_TO_DRIVE, 2),
        "the ordered second objective opens: {}",
        replies_text(&finished.results)
    );
    // The flat T313 completion reward is paid once, at the same server-observed transition.
    assert_eq!(
        coins(&game),
        before_coins + COMPLETION_COINS,
        "one flat completion payout"
    );
    assert_eq!(
        profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE),
        before_xp + COMPLETION_XP,
        "the completion XP goes to profile 49"
    );
    assert_eq!(
        game.state().derby.as_ref().unwrap().paid_matches,
        vec![finished.instance]
    );
    let paid_after_finish = game.state();

    // A replayed results record cannot credit or pay again.
    let key = ParticipantKey(fixture.guid);
    let replay = send(
        &mut game,
        &fixture.character(MAD_JACK),
        &hit_for(key, finished.npcs[0], 100.0),
        now + 100,
    );
    assert!(!replay.changed);
    assert!(replay.replies.is_empty());
    assert_eq!(game.state(), paid_after_finish);

    close_match(&fixture, &mut game, now);

    // A live delta arrives between the finish and the turn-in (a second finished match also pays,
    // T219). The chapter reward must land on top of it, never on the entry snapshot.
    let second = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);
    assert!(
        second.instance > finished.instance,
        "a paid instance id is never reissued"
    );
    assert_eq!(
        counts(&game, LICENSE_TO_DRIVE),
        vec![1, 0],
        "a second finished match cannot re-credit the completed objective"
    );
    let live_coins = coins(&game);
    let live_xp = profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE);
    assert_eq!(live_coins, before_coins + 2 * COMPLETION_COINS);
    assert_eq!(live_xp, before_xp + 2 * COMPLETION_XP);
    close_match(&fixture, &mut game, now);

    // The chapter's job XP is content: switch jobs before the turn-in and profile 49 still earns.
    now += 10;
    let switched = send(
        &mut game,
        &fixture.character(MAD_JACK),
        &select_job(BRAWLER),
        now,
    );
    assert!(switched.changed, "{}", replies_text(&switched));
    assert_eq!(active_job(&game), BRAWLER, "another job runs the turn-in");

    // Return to Crash Davis, release the held 49/13 and turn in.
    let crash = fixture.character(CRASH);
    let experience_before = experience(&game);
    now += 10;
    let talked = talk(&mut game, &crash, CRASH, now);
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(LICENSE_TO_DRIVE));
    assert!(
        !talked
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the return end screen is a 26/6 dialog choice"
    );
    now += 10;
    release_end_screen(&mut game, &crash, LICENSE_TO_DRIVE, CRASH, 0, now);
    now += 10;
    finish_quest(
        &mut game,
        &crash,
        LICENSE_TO_DRIVE,
        DEMO_DERBY_DRIVER_PROFILE,
        LICENSE_COINS,
        LICENSE_XP,
        now,
    );
    assert_eq!(
        coins(&game),
        live_coins + LICENSE_COINS,
        "the chapter reward applies to the live wallet"
    );
    assert_eq!(
        profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE),
        live_xp + LICENSE_XP,
        "the chapter XP goes to profile 49 even when another job runs"
    );
    for (profile, amount) in experience(&game) {
        if profile == DEMO_DERBY_DRIVER_PROFILE {
            continue;
        }
        assert_eq!(
            Some(&amount),
            experience_before.get(&profile),
            "the chapter reward must not move profile {profile}"
        );
    }
    assert_eq!(
        quest_status(&game, RULES_OF_THE_ROAD),
        QuestStatus::Completed
    );

    // Replayed turn-in packets change nothing.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &crash,
        &quest_end_reply(LICENSE_TO_DRIVE),
        now,
        "a replayed 49/14",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &crash,
        &quest_reply(LICENSE_TO_DRIVE, true),
        now,
        "a replayed 49/2 accept",
    );
    assert_eq!(coins(&game), live_coins + LICENSE_COINS);
}

/// T787: the derby reports this participant's own placement and knock-out record to a
/// `require` objective, and one settled instance credits it once even when the objective
/// still needs a second match. A replayed 34/16 in the Results phase is not a second
/// settle (`take_completion`'s `paid_matches` ledger), and a real save/reload keeps the
/// credit without persisting any result fact.
#[test]
fn second_settlement_of_same_instance_credits_once() {
    let fixture = Fixture::new(CHARACTER);
    patch_license_require(
        &fixture,
        2,
        json!({"max_placement": 1, "no_knockout": true}),
    );
    let mut game = fixture.game();
    let mut now = NOW;
    learn_and_activate(&fixture, &mut game, &mut now);
    rules_of_the_road(&fixture, &mut game, &mut now);
    accept_license(&fixture, &mut game, &mut now);

    let finished = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);
    assert_eq!(
        counts(&game, LICENSE_TO_DRIVE),
        vec![1, 0],
        "one finished match credits the two-completion objective once"
    );
    assert_eq!(
        count_quest_packets(&finished.results, LICENSE_TO_DRIVE, 9),
        1,
        "one 49/9 objective update: {}",
        replies_text(&finished.results)
    );
    assert!(objective_update(&finished.results, LICENSE_TO_DRIVE, 1, 1));
    assert!(
        !objective_complete(&finished.results, LICENSE_TO_DRIVE, 1),
        "the objective still needs a second match"
    );
    let credited = game.state().npc_quests.clone();

    // A repeated result record for the same instance must not count again.
    let replay = send(
        &mut game,
        &fixture.character(MAD_JACK),
        &hit_for(ParticipantKey(fixture.guid), finished.npcs[0], 100.0),
        now + 100,
    );
    assert!(!replay.changed);
    assert!(replay.replies.is_empty());
    assert_eq!(game.state().npc_quests, credited);
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![1, 0]);

    // The counters are saved quest progress; the result facts are not saved at all.
    let bytes = serde_json::to_vec(&game.state()).unwrap();
    assert!(
        !bytes
            .windows(b"knocked_out".len())
            .any(|window| window == b"knocked_out"),
        "the derby result facts were written into the save"
    );
    let reloaded = save_and_reload(&fixture, &game);
    assert_eq!(counts(&reloaded, LICENSE_TO_DRIVE), vec![1, 0]);
    assert_eq!(
        quest_status(&reloaded, LICENSE_TO_DRIVE),
        QuestStatus::Active
    );
}

/// T787: the same derby path reports a score, so a `min_score` the finished run did not
/// reach leaves the quest state byte-equal and sends no quest packet while the match
/// still settles and pays. Canary: ignore `min_score` in `event_matches` and this fails.
#[test]
fn result_below_threshold_changes_nothing() {
    let fixture = Fixture::new(CHARACTER);
    patch_license_require(&fixture, 1, json!({"min_score": 1_000_000_000}));
    let mut game = fixture.game();
    let mut now = NOW;
    learn_and_activate(&fixture, &mut game, &mut now);
    rules_of_the_road(&fixture, &mut game, &mut now);
    accept_license(&fixture, &mut game, &mut now);
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![0, 0]);
    let before = game.state().npc_quests.clone();

    let finished = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);
    assert_eq!(
        game.derby_phase(),
        Some(DerbyPhase::Results),
        "the match still reaches its results phase"
    );
    assert_eq!(
        count_quest_packets(&finished.results, LICENSE_TO_DRIVE, 9),
        0,
        "no objective update: {}",
        replies_text(&finished.results)
    );
    assert_eq!(
        count_quest_packets(&finished.results, LICENSE_TO_DRIVE, 10),
        0
    );
    assert_eq!(
        game.state().npc_quests,
        before,
        "a result below the bound changes no quest state"
    );
}

#[test]
fn a_cancelled_license_match_credits_and_pays_nothing() {
    let fixture = Fixture::new(CHARACTER);
    let mut game = fixture.game();
    let mut now = NOW;
    learn_and_activate(&fixture, &mut game, &mut now);
    rules_of_the_road(&fixture, &mut game, &mut now);
    accept_license(&fixture, &mut game, &mut now);

    let before_coins = coins(&game);
    let before_xp = profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE);
    let instance = start_match(&fixture, &mut game, MAD_JACK, &mut now);
    run_to_running(&fixture, &mut game, instance, &mut now);

    // The native mid-game quit (39/6) retires the match.
    let character = fixture.character(MAD_JACK);
    now += 1_000;
    let quit = send(&mut game, &character, &quit_request(instance), now);
    assert!(quit.changed);
    assert!(!game.derby_active());
    assert!(
        quit.replies.iter().any(|packet| packet[..3] == [39, 0, 19]),
        "the quit retires the minigame state"
    );
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![0, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(
        !quit
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "a cancelled match opens no end screen"
    );
    assert_eq!(coins(&game), before_coins, "cancelling pays nothing");
    assert_eq!(
        profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE),
        before_xp,
        "cancelling credits no derby XP"
    );
    assert!(
        game.state().derby.is_none(),
        "an unfinished match writes no paid ledger"
    );

    // The pending-game cancel (39/7) is the same refusal for an offered match.
    let instance = start_match(&fixture, &mut game, MAD_JACK, &mut now);
    let mut pending_cancel = quit_request(instance);
    pending_cancel[2] = 7;
    now += 100;
    let cancelled = send(&mut game, &character, &pending_cancel, now);
    assert!(cancelled.changed);
    assert!(!game.derby_active());
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![0, 0]);
    assert_eq!(coins(&game), before_coins);
}

#[test]
fn a_refused_completion_payout_still_credits_the_finish_and_keeps_the_session() {
    let fixture = Fixture::new(CHARACTER);
    let mut state = fixture.game().state();
    // The wire coin field is i32. Rules of the Road still fits; the flat completion cannot.
    state.inventory.coins = i32::MAX as u32 - RULES_COINS;
    let mut game = fixture.game_from(state);
    let mut now = NOW;
    learn_and_activate(&fixture, &mut game, &mut now);
    rules_of_the_road(&fixture, &mut game, &mut now);
    accept_license(&fixture, &mut game, &mut now);
    assert_eq!(coins(&game), i32::MAX as u32, "the wallet sits at the cap");

    let before_xp = profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE);
    let finished = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);

    // Exactly one bounded server line; no error and no second reward bundle.
    let lines: Vec<_> = finished
        .results
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[15, 0, 3, 0]))
        .collect();
    assert_eq!(lines.len(), 1, "one bounded refusal line: {lines:?}");
    assert!(
        String::from_utf8_lossy(lines[0]).contains("demolition derby completion reward"),
        "the line names the refused reward"
    );
    assert!(
        !finished
            .results
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "a refused grant sends no reward bundle"
    );
    assert_eq!(
        coins(&game),
        i32::MAX as u32,
        "the refused payout paid coins"
    );
    assert_eq!(
        profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE),
        before_xp,
        "the refused payout paid XP"
    );
    // The finished-match ledger is consumed, and the server-observed finish still credits the quest.
    assert_eq!(
        game.state().derby.as_ref().unwrap().paid_matches,
        vec![finished.instance]
    );
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![1, 0]);
    assert!(objective_complete(&finished.results, LICENSE_TO_DRIVE, 1));

    // The session keeps running: Close is accepted and the next NPC packet is answered.
    close_match(&fixture, &mut game, now);
    now += RESULTS_HOLD_MS;
    let crash = fixture.character(CRASH);
    let menu = npc_quest_chain::interact(&mut game, &crash, CRASH, now);
    assert!(
        menu.replies.iter().any(|packet| packet[..3] == [26, 0, 9]),
        "the session still answers packets after a player-local refusal"
    );
}

/// A package tuned to pay nothing (0 coins / 0 XP, the serde defaults) still credits the finish
/// once: the quest credit never depends on the flat payout amount.
#[test]
fn a_zero_reward_package_still_credits_the_finish_once() {
    let fixture = Fixture::new(CHARACTER);
    let entry = fixture.inner.path().join("derby_entry.json");
    let mut package: Value = serde_json::from_slice(&fs::read(&entry).unwrap()).unwrap();
    package["completion_reward_coins"] = json!(0);
    package["completion_reward_xp"] = json!(0);
    fs::write(&entry, serde_json::to_vec(&package).unwrap()).unwrap();
    let mut game = fixture.game();
    let mut now = NOW;
    learn_and_activate(&fixture, &mut game, &mut now);
    rules_of_the_road(&fixture, &mut game, &mut now);
    accept_license(&fixture, &mut game, &mut now);
    let (before_coins, before_xp) = (coins(&game), experience(&game));

    let finished = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![1, 0]);
    assert_eq!(
        count_quest_packets(&finished.results, LICENSE_TO_DRIVE, 10),
        1
    );
    assert!(
        !finished
            .results
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "a zero payout sends no reward bundle"
    );
    assert_eq!(
        (coins(&game), experience(&game)),
        (before_coins, before_xp),
        "a zero payout pays nothing"
    );
    assert_eq!(
        game.state().derby.as_ref().unwrap().paid_matches,
        vec![finished.instance],
        "the finish is recorded once"
    );
    let credited = game.state();

    let key = ParticipantKey(fixture.guid);
    let replay = send(
        &mut game,
        &fixture.character(MAD_JACK),
        &hit_for(key, finished.npcs[0], 100.0),
        now + 100,
    );
    assert!(!replay.changed, "a replayed results record credits nothing");
    assert_eq!(game.state(), credited);
}

#[test]
fn the_chapter_survives_the_three_reload_points() {
    let fixture = Fixture::new(CHARACTER);
    let mut game = fixture.game();
    let mut now = NOW;
    learn_and_activate(&fixture, &mut game, &mut now);
    rules_of_the_road(&fixture, &mut game, &mut now);
    accept_license(&fixture, &mut game, &mut now);

    // 1. Mid-match: the live match is connection-local; the accepted quest survives.
    let instance = start_match(&fixture, &mut game, MAD_JACK, &mut now);
    run_to_running(&fixture, &mut game, instance, &mut now);
    let mut reloaded = save_and_reload(&fixture, &game);
    assert!(!reloaded.derby_active(), "a match is never a save field");
    assert_eq!(
        quest_status(&reloaded, LICENSE_TO_DRIVE),
        QuestStatus::Active
    );
    assert_eq!(counts(&reloaded, LICENSE_TO_DRIVE), vec![0, 0]);
    assert!(reloaded.state().derby.is_none());

    // 2. After the finish: the credited objective and the paid ledger survive.
    game = reloaded;
    let before_coins = coins(&game);
    let before_xp = profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE);
    let finished = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);
    assert_eq!(counts(&game, LICENSE_TO_DRIVE), vec![1, 0]);
    reloaded = save_and_reload(&fixture, &game);
    assert_eq!(counts(&reloaded, LICENSE_TO_DRIVE), vec![1, 0]);
    assert_eq!(
        reloaded.state().derby.as_ref().unwrap().paid_matches,
        vec![finished.instance],
        "the paid ledger survives the reconnect"
    );
    assert_eq!(coins(&reloaded), before_coins + COMPLETION_COINS);
    assert_eq!(
        profile_xp(&reloaded, DEMO_DERBY_DRIVER_PROFILE),
        before_xp + COMPLETION_XP
    );

    // The return Talk after the reload opens the held end screen; reload again with it pending.
    game = reloaded;
    now += RESULTS_HOLD_MS;
    let crash = fixture.character(CRASH);
    talk(&mut game, &crash, CRASH, now);
    assert_eq!(pending_turnin(&game), Some(LICENSE_TO_DRIVE));
    reloaded = save_and_reload(&fixture, &game);
    assert_eq!(counts(&reloaded, LICENSE_TO_DRIVE), vec![1, 1]);
    assert_eq!(
        pending_turnin(&reloaded),
        Some(LICENSE_TO_DRIVE),
        "the pending turn-in survives the reconnect"
    );

    // 3. After the reward: completed, and no replay pays again.
    game = reloaded;
    let before_coins = coins(&game);
    let before_xp = profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE);
    now += 10;
    finish_quest(
        &mut game,
        &fixture.character(CRASH),
        LICENSE_TO_DRIVE,
        DEMO_DERBY_DRIVER_PROFILE,
        LICENSE_COINS,
        LICENSE_XP,
        now,
    );
    let mut reloaded = save_and_reload(&fixture, &game);
    let paid = reloaded.state();
    assert_eq!(
        quest_status(&reloaded, LICENSE_TO_DRIVE),
        QuestStatus::Completed
    );
    assert_eq!(pending_turnin(&reloaded), None);
    assert_eq!(coins(&reloaded), before_coins + LICENSE_COINS);
    assert_eq!(
        profile_xp(&reloaded, DEMO_DERBY_DRIVER_PROFILE),
        before_xp + LICENSE_XP
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut reloaded,
        &fixture.character(CRASH),
        &quest_end_reply(LICENSE_TO_DRIVE),
        now,
        "a 49/14 replay after a reconnect",
    );
    assert_eq!(reloaded.state(), paid);
}

#[test]
fn two_players_finish_their_own_matches_independently() {
    let first = Fixture::new(CHARACTER);
    let second = Fixture::new(OTHER_CHARACTER);
    let mut a = first.game();
    let mut b = second.game();
    let mut now = NOW;
    for (fixture, game) in [(&first, &mut a), (&second, &mut b)] {
        learn_and_activate(fixture, game, &mut now);
        rules_of_the_road(fixture, game, &mut now);
        accept_license(fixture, game, &mut now);
        now += 100;
    }

    // Player A cancels its license match: nothing is credited or paid on either side.
    let a_coins = coins(&a);
    let b_coins = coins(&b);
    let instance = start_match(&first, &mut a, MAD_JACK, &mut now);
    run_to_running(&first, &mut a, instance, &mut now);
    now += 1_000;
    let quit = send(
        &mut a,
        &first.character(MAD_JACK),
        &quit_request(instance),
        now,
    );
    assert!(quit.changed);
    assert!(!a.derby_active());
    assert_eq!(counts(&a, LICENSE_TO_DRIVE), vec![0, 0]);
    assert_eq!(coins(&a), a_coins);
    assert_eq!(counts(&b, LICENSE_TO_DRIVE), vec![0, 0]);
    assert_eq!(coins(&b), b_coins);

    // Player B finishes and credits its own quest and ledger only.
    let b_finished = play_to_results(&second, &mut b, MAD_JACK, &mut now);
    assert_eq!(counts(&b, LICENSE_TO_DRIVE), vec![1, 0]);
    assert_eq!(
        b.state().derby.as_ref().unwrap().paid_matches,
        vec![b_finished.instance]
    );
    assert_eq!(coins(&b), b_coins + COMPLETION_COINS);
    assert_eq!(counts(&a, LICENSE_TO_DRIVE), vec![0, 0]);
    assert_eq!(coins(&a), a_coins);
    assert!(a.state().derby.is_none());

    // Player A can still finish its own match later; it gets exactly one credit and payout.
    let a_finished = play_to_results(&first, &mut a, MAD_JACK, &mut now);
    assert_eq!(counts(&a, LICENSE_TO_DRIVE), vec![1, 0]);
    assert_eq!(
        a.state().derby.as_ref().unwrap().paid_matches,
        vec![a_finished.instance]
    );
    assert_eq!(coins(&a), a_coins + COMPLETION_COINS);
    assert_eq!(coins(&b), b_coins + COMPLETION_COINS);
    close_match(&second, &mut b, now);
    close_match(&first, &mut a, now);
    let b_snapshot = saved(&b);

    // Others see the temporary derby job: entering from another job announces the profile-49
    // outfit through the existing directed 35/7 path (T313's visibility rule), and the second
    // session's own save does not move.
    now += 10;
    let switched = send(
        &mut a,
        &first.character(MAD_JACK),
        &select_job(BRAWLER),
        now,
    );
    assert!(switched.changed, "{}", replies_text(&switched));
    assert_eq!(active_job(&a), BRAWLER);
    now += 10;
    let entered = npc_quest_chain::interact(&mut a, &first.character(MAD_JACK), MAD_JACK, now);
    let option = menu_option(&entered, DERBY_LABEL);
    now += 100;
    let started = send(
        &mut a,
        &first.character(MAD_JACK),
        &npc_select(MAD_JACK, option),
        now,
    );
    // The live job is the temporary Derby Driver; the save keeps the job it replaced (T313).
    assert_eq!(
        a.active_profile(),
        Some(DEMO_DERBY_DRIVER_PROFILE),
        "{}",
        replies_text(&started)
    );
    assert_eq!(active_job(&a), BRAWLER, "the temporary job is never saved");
    let outfit = a.profile_attachments(DEMO_DERBY_DRIVER_PROFILE).unwrap();
    assert!(
        !outfit.starts_with(&0i32.to_le_bytes()),
        "the outfit is visible"
    );
    let announced = started
        .nearby
        .iter()
        .find(|packet| {
            packet.starts_with(&[35, 0, 7, 0])
                && packet.get(4..12) == Some(&first.guid.to_le_bytes()[..])
        })
        .expect("entering the derby announces the visible outfit");
    assert_eq!(&announced[12..announced.len() - 4], &outfit[..]);
    assert_eq!(
        saved(&b),
        b_snapshot,
        "one session's entry must not move the other's save"
    );

    // The completed chapter cannot be credited again by this later match.
    let instance = offer_instance(&started);
    let npcs = run_to_running(&first, &mut a, instance, &mut now);
    let results = finish_match(&first, &mut a, &npcs, &mut now);
    assert_eq!(
        counts(&a, LICENSE_TO_DRIVE),
        vec![1, 0],
        "the finish objective stays at one credit"
    );
    assert_eq!(
        count_quest_packets(&results, LICENSE_TO_DRIVE, 10),
        0,
        "a completed objective cannot complete twice"
    );
    assert_eq!(
        a.state().derby.as_ref().unwrap().paid_matches,
        vec![a_finished.instance, instance],
        "each finished match keeps one ledger row"
    );
}

#[test]
fn forged_requests_and_gated_refusals_change_nothing() {
    let fixture = Fixture::new(CHARACTER);
    let mut game = fixture.game();
    let mut now = NOW;
    let mad_jack = fixture.character(MAD_JACK);
    let crash = fixture.character(CRASH);

    // No match, no derby packets: a forged collision and a forged result claim are inert.
    let key = ParticipantKey(fixture.guid);
    now += 10;
    let forged_hit = send(
        &mut game,
        &mad_jack,
        &hit_for(key, ParticipantKey(1), 100.0),
        now,
    );
    assert!(!forged_hit.changed);
    assert!(forged_hit.replies.is_empty());
    now += 10;
    let forged_result = send(&mut game, &mad_jack, &[39, 0, 18, 0], now);
    assert!(!forged_result.changed);
    assert!(forged_result.replies.is_empty());

    // The license needs the rules first: the missing prerequisite offers nothing.
    learn_and_activate(&fixture, &mut game, &mut now);
    now += 10;
    assert_no_offer(
        &mut game,
        &crash,
        CRASH,
        &mut now,
        "License to Drive before Rules of the Road",
    );

    // A forged accept, a forged end reply and an out-of-range interact change nothing.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &crash,
        &quest_reply(LICENSE_TO_DRIVE, true),
        now,
        "a forged 49/2 accept for a quest that was never offered",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &crash,
        &quest_end_reply(LICENSE_TO_DRIVE),
        now,
        "a forged 49/14 before the quest is active",
    );
    let before = saved(&game);
    let far = CharacterState {
        position: [
            crash.position[0] + 100.0,
            crash.position[1],
            crash.position[2],
        ],
        ..crash
    };
    now += 10;
    assert!(
        game.handle_in_world(&npc_interact(CRASH), &far, now)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an out-of-range 26/8 is refused"
    );
    assert_eq!(
        saved(&game),
        before,
        "an out-of-range 26/8 changed the state"
    );

    // A pre-finish turn-in of the licence changes nothing once it is accepted.
    rules_of_the_road(&fixture, &mut game, &mut now);
    accept_license(&fixture, &mut game, &mut now);
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &crash,
        &quest_end_reply(LICENSE_TO_DRIVE),
        now,
        "a 49/14 before the first objective is complete",
    );

    // The trainer's menu is still usable after every refusal, and the job gate still holds.
    now += 10;
    let menu = npc_quest_chain::interact(&mut game, &mad_jack, MAD_JACK, now);
    let option = menu_option(&menu, DERBY_LABEL);
    now += 100;
    let entered = send(&mut game, &mad_jack, &npc_select(MAD_JACK, option), now);
    assert!(entered.changed, "{}", replies_text(&entered));
    assert!(game.derby_active());
    // The derby cap cannot be bypassed by a second 26/8/26/10 while a match is offered.
    now += 100;
    let before = saved(&game);
    let replay = send(&mut game, &mad_jack, &npc_interact(MAD_JACK), now);
    assert!(!replay.changed);
    assert_eq!(
        saved(&game),
        before,
        "an already-playing entry changed state"
    );
}

// ---- T666: "Demo Derby: Stunt Double" (3060030), Jace Wildwheel's first Wildwood quest ----
//
// One `finish_minigame` goal on the derby package (game 39), the same server-observed finish
// 3060005 uses, and a Talk hand-in at the giver. The reward, text ids and end text are read from
// `data/content/quests/demo_derby_driver.json`, so retuning the record keeps these tests.

/// Jace Wildwheel, a Wilds `legacy_data` row that lists 3060030 in `quest_ids_given`.
const JACE: u64 = 9_000_000_357;
const STUNT_DOUBLE: u32 = 3_060_030;

/// Jace and Tucker are `wilds.json` rows that spawn as their own content key; Kanan's row
/// lives in `stillwater_crossing.json`.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "wilds.json",
    files: &[(KANAN, "stillwater_crossing.json")],
    runtime: &[],
};

/// The shipped record mapped to the card ids, end text and reward words the runner pins.
fn stunt_double() -> ZoneQuest {
    ZoneQuest::from_record(
        shipped_quest(STUNT_DOUBLE),
        JACE,
        JACE,
        DEMO_DERBY_DRIVER_PROFILE,
    )
}

/// Learn the job, then take Jace's offer and accept it. Returns Jace's position.
fn accept_stunt_double(fixture: &Fixture, game: &mut Gameplay, now: &mut u64) -> CharacterState {
    learn_and_activate(fixture, game, now);
    *now += 10;
    MAP.accept(game, CHARACTER, &stunt_double(), now);
    assert_eq!(counts(game, STUNT_DOUBLE), vec![0]);
    MAP.at(CHARACTER, JACE)
}

#[test]
fn stunt_double_is_credited_by_a_finished_derby_and_paid_from_data() {
    let fixture = Fixture::new(CHARACTER);
    let mut game = fixture.game();
    let mut now = NOW;
    let quest = stunt_double();
    assert!(
        quest.coins > 0 && quest.profile_xp > 0,
        "the record pays something"
    );

    let jace = accept_stunt_double(&fixture, &mut game, &mut now);

    // Before the match: a Talk, an early 49/14 and a repeat 49/2 change nothing.
    assert_no_offer(&mut game, &jace, JACE, &mut now, "Jace mid-quest");
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &jace,
        &quest_end_reply(STUNT_DOUBLE),
        now,
        "an early 49/14",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &jace,
        &quest_reply(STUNT_DOUBLE, true),
        now,
        "a repeat 49/2",
    );

    // One finished match credits the goal once; it opens no end screen by itself.
    let finished = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);
    assert_eq!(counts(&game, STUNT_DOUBLE), vec![1]);
    assert!(objective_update(&finished.results, STUNT_DOUBLE, 1, 1));
    assert!(objective_complete(&finished.results, STUNT_DOUBLE, 1));
    assert_eq!(pending_turnin(&game), None);
    assert!(
        !finished
            .results
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the finish alone opens no end screen"
    );
    close_match(&fixture, &mut game, now);

    // The finished goal survives a save and reload; the hand-in then pays exactly the record.
    let mut game = save_and_reload(&fixture, &game);
    assert_eq!(quest_status(&game, STUNT_DOUBLE), QuestStatus::Active);
    assert_eq!(counts(&game, STUNT_DOUBLE), vec![1]);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE));
    now += 100;
    MAP.talk(&mut game, CHARACTER, JACE, now);
    now += 10;
    MAP.pay_held_turnin(
        &mut game,
        CHARACTER,
        &quest,
        DEMO_DERBY_DRIVER_PROFILE,
        &mut now,
    );
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(
        profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE),
        xp_before + quest.profile_xp
    );

    // A replayed hand-in or accept pays nothing more.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &jace,
        &quest_end_reply(STUNT_DOUBLE),
        now,
        "a replayed 49/14",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &jace,
        &quest_reply(STUNT_DOUBLE, true),
        now,
        "a replayed 49/2",
    );
}

#[test]
fn stunt_double_is_offered_only_to_an_active_derby_driver() {
    let fixture = Fixture::new(CHARACTER);
    let mut game = fixture.game();
    let mut now = NOW;
    assert!(!unlocked(&game, DEMO_DERBY_DRIVER_PROFILE));
    let jace = fixture.character(JACE);
    assert_no_offer(&mut game, &jace, JACE, &mut now, "Jace, not a derby driver");
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &jace,
        &quest_reply(STUNT_DOUBLE, true),
        now,
        "a forged 49/2 for Stunt Double",
    );
}

#[test]
fn a_cancelled_match_does_not_credit_stunt_double() {
    let fixture = Fixture::new(CHARACTER);
    let mut game = fixture.game();
    let mut now = NOW;
    let jace = accept_stunt_double(&fixture, &mut game, &mut now);
    let instance = start_match(&fixture, &mut game, MAD_JACK, &mut now);
    run_to_running(&fixture, &mut game, instance, &mut now);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE));

    now += 1_000;
    let quit = send(
        &mut game,
        &fixture.character(MAD_JACK),
        &quit_request(instance),
        now,
    );
    assert!(quit.changed);
    assert!(!game.derby_active());
    assert_eq!(counts(&game, STUNT_DOUBLE), vec![0]);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), coins_before, "cancelling pays nothing");
    assert_eq!(profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE), xp_before);
    // Jace still has nothing to hand in.
    assert_no_offer(&mut game, &jace, JACE, &mut now, "Jace after a cancel");
}

// ---- T674: "Demo Derby: Seeking a Sponsor" (3060016), Tucker Thunderjolt's Series 2 finale ----
//
// A finished derby (game 39, as above), then a Talk to Kanan in Stillwater Crossing, whose row
// carries the `quests_accepted` binding that makes her the formal turn-in. It follows Tucker's
// Troubles (3060013). Card ids and the reward are read from the shipped record.

/// Tucker Thunderjolt (giver, a Wilds `legacy_data` row) and Kanan (Stillwater Crossing).
const TUCKER: u64 = 9_000_000_495;
const KANAN: u64 = 9_000_000_124;
const TUCKERS_TROUBLES: u32 = 3_060_013;
const SEEKING_A_SPONSOR: u32 = 3_060_016;

fn sponsor() -> ZoneQuest {
    ZoneQuest::from_record(
        shipped_quest(SEEKING_A_SPONSOR),
        TUCKER,
        KANAN,
        DEMO_DERBY_DRIVER_PROFILE,
    )
}

/// An active derby driver (learned through Mad Jack) who has finished Tucker's Troubles.
fn sponsor_candidate(fixture: &Fixture, now: &mut u64) -> Gameplay {
    let mut game = fixture.game();
    learn_and_activate(fixture, &mut game, now);
    let mut state = game.state();
    seed_quest(&mut state, TUCKERS_TROUBLES, QuestStatus::Completed, &[1]);
    fixture.game_from(state)
}

#[test]
fn seeking_a_sponsor_is_a_finished_derby_then_kanan_and_pays_from_data() {
    let fixture = Fixture::new(CHARACTER);
    let mut now = NOW;
    let mut game = sponsor_candidate(&fixture, &mut now);
    let quest = sponsor();
    assert!(
        quest.coins > 0 && quest.profile_xp > 0,
        "the record pays something"
    );
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, SEEKING_A_SPONSOR), vec![0, 0]);

    // Kanan first, an early 49/14 and a repeat 49/2 credit nothing: the derby comes first.
    let kanan = MAP.at(CHARACTER, KANAN);
    let before = saved(&game);
    MAP.talk(&mut game, CHARACTER, KANAN, now);
    now += 10;
    assert_eq!(counts(&game, SEEKING_A_SPONSOR), vec![0, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(saved(&game), before, "Kanan's Talk before the derby");
    assert_packet_changes_nothing(
        &mut game,
        &kanan,
        &quest_end_reply(SEEKING_A_SPONSOR),
        now,
        "an early 49/14",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, TUCKER),
        &quest_reply(SEEKING_A_SPONSOR, true),
        now,
        "a repeat 49/2",
    );

    // One finished match credits the derby goal once; it opens no end screen by itself.
    let finished = play_to_results(&fixture, &mut game, MAD_JACK, &mut now);
    assert_eq!(counts(&game, SEEKING_A_SPONSOR), vec![1, 0]);
    assert!(objective_update(&finished.results, SEEKING_A_SPONSOR, 1, 1));
    assert!(objective_complete(&finished.results, SEEKING_A_SPONSOR, 1));
    assert!(objective_activated(&finished.results, SEEKING_A_SPONSOR, 2));
    assert_eq!(pending_turnin(&game), None);
    close_match(&fixture, &mut game, now);

    // The derby goal survives a save and reload; Kanan's Talk holds the end screen and the
    // hand-in pays exactly the record.
    let mut game = save_and_reload(&fixture, &game);
    assert_eq!(counts(&game, SEEKING_A_SPONSOR), vec![1, 0]);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE));
    now += 100;
    MAP.turn_in(
        &mut game,
        CHARACTER,
        &quest,
        DEMO_DERBY_DRIVER_PROFILE,
        &mut now,
    );
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(
        profile_xp(&game, DEMO_DERBY_DRIVER_PROFILE),
        xp_before + quest.profile_xp
    );
    assert_eq!(
        quest_status(&game, SEEKING_A_SPONSOR),
        QuestStatus::Completed
    );

    // A replayed hand-in or accept pays nothing more.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &kanan,
        &quest_end_reply(SEEKING_A_SPONSOR),
        now,
        "a replayed 49/14",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, TUCKER),
        &quest_reply(SEEKING_A_SPONSOR, true),
        now,
        "a replayed 49/2",
    );
}

#[test]
fn seeking_a_sponsor_is_offered_only_after_tuckers_troubles_to_a_derby_driver() {
    let fixture = Fixture::new(CHARACTER);
    let mut now = NOW;
    let tucker = MAP.at(CHARACTER, TUCKER);

    // Not a derby driver, even with Tucker's Troubles done.
    let mut state = fixture.game().state();
    seed_quest(&mut state, TUCKERS_TROUBLES, QuestStatus::Completed, &[1]);
    let mut game = fixture.game_from(state);
    assert_no_offer(&mut game, &tucker, TUCKER, &mut now, "Tucker, no derby job");
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &tucker,
        &quest_reply(SEEKING_A_SPONSOR, true),
        now,
        "a forged 49/2 without the derby job",
    );

    // The derby job without Tucker's Troubles.
    let mut game = fixture.game();
    learn_and_activate(&fixture, &mut game, &mut now);
    assert_no_offer(&mut game, &tucker, TUCKER, &mut now, "Tucker, before Betty");
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &tucker,
        &quest_reply(SEEKING_A_SPONSOR, true),
        now,
        "a forged 49/2 before Tucker's Troubles",
    );

    // Positive control: with both, Tucker offers it.
    let mut game = sponsor_candidate(&fixture, &mut now);
    now += 10;
    offer(&mut game, &tucker, TUCKER, SEEKING_A_SPONSOR, now);
}

#[test]
fn a_cancelled_match_does_not_credit_seeking_a_sponsor_and_kanan_is_the_turn_in() {
    let fixture = Fixture::new(CHARACTER);
    let mut now = NOW;
    let mut game = sponsor_candidate(&fixture, &mut now);
    MAP.accept(&mut game, CHARACTER, &sponsor(), &mut now);
    let instance = start_match(&fixture, &mut game, MAD_JACK, &mut now);
    run_to_running(&fixture, &mut game, instance, &mut now);

    now += 1_000;
    let quit = send(
        &mut game,
        &fixture.character(MAD_JACK),
        &quit_request(instance),
        now,
    );
    assert!(quit.changed);
    assert_eq!(counts(&game, SEEKING_A_SPONSOR), vec![0, 0]);
    assert_eq!(pending_turnin(&game), None);

    // The loader binds Tucker as giver and Kanan's `quests_accepted` row as the turn-in.
    let engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility::default(),
        None,
    )
    .expect("the shipped split catalogue loads");
    assert!(engine.is_enabled(SEEKING_A_SPONSOR));
    assert_eq!(engine.giver_npc_id(SEEKING_A_SPONSOR), Some(TUCKER));
    assert_eq!(engine.turnin_npc_id(SEEKING_A_SPONSOR), Some(KANAN));
}
