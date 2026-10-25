//! T521: the job toolbar follows the equipped slot-7 weapon.
//!
//! The 36/5 basic slot's name argument is what the client displays
//! (`0x008ee3d0` passes the row's fourth dword through the string table to
//! `0x009cf920` / `HandlerActionBar:setButtonData`; `ScriptsBase.lua:5211-5244`
//! takes the name from the ActionBar data source and only re-reads the icon).
//! A matching weapon names the basic slot with its `ClientItemDefinition`
//! `name_id`; no matching weapon empties the combat bar like a noncombat job
//! (R, OSFR #124). These tests drive the production request path in
//! `Gameplay::handle_in_world`.
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/job_playtest.rs"]
mod job_playtest;

use job_playtest::bar_ids;

static SERIAL: AtomicU64 = AtomicU64::new(0);

const BRAWLER: i32 = 43;
const STARTER_NAME: i32 = 131914;
/// Second Brawler hammer of the same class, `name_id` 131915.
const SECOND: i32 = 75031;
const SECOND_NAME: i32 = 131915;
/// A class-5 Brawler hammer: profile 43 may equip it, but the job's cast gate
/// (class 71) refuses it, so it leaves the job unarmed.
const WRONG_CLASS: i32 = 13658;
/// A class-21 slot-7 item profile 45 may wear, but class 45 is not unlocked.
const LOCKED_PROFILE_ITEM: i32 = 75587;

/// The eight-slot bar of an armed Brawler at level 1: basic, Leg Sweep (row 1).
const ARMED_BAR: [Option<i32>; 8] = [Some(4895), Some(62018), None, None, None, None, None, None];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/opening-anvil-placement-v1/content");
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t521-weapon-bar-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let source = entry.unwrap().path();
            if source.is_file() {
                fs::copy(&source, path.join(source.file_name().unwrap())).unwrap();
            }
        }
        // The pinned campaign predates the canonical class requirements.
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/opening-abilities.json"),
            path.join("opening-abilities.json"),
        )
        .unwrap();
        // The Warrior starter cudgel, used for an inactive job's outfit.
        job_playtest::install_starter_weapon_definitions(&path);
        Self(path)
    }

    fn content(&self) -> &Path {
        &self.0
    }

    /// A Brawler session entered through the real 26/13 request. The opening
    /// package's initial save equips the Brawler starter hammer, so the bar is
    /// armed from the first packet.
    fn game(&self) -> (Gameplay, CharacterState) {
        let mut seed = Gameplay::load(self.content(), None).unwrap().state();
        {
            let profiles = seed.profiles.as_mut().unwrap();
            profiles.active = 1;
            profiles.unlocked.extend([2, 11, 12, 32, 35, BRAWLER]);
        }
        let mut game = Gameplay::load(self.content(), Some(seed)).unwrap();
        game.set_dev_tools(true);
        game.set_role(AccountRole::Admin);
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [0.0; 3],
        };
        let switched = game
            .handle_in_world(&select(BRAWLER), &character, 100)
            .unwrap();
        assert!(
            switched
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[36, 0, 5, 0])),
            "the job switch sends the toolbar"
        );
        (game, character)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn command(family: u16, sub: u16, fields: &[i32]) -> Vec<u8> {
    [
        family.to_le_bytes().to_vec(),
        sub.to_le_bytes().to_vec(),
        fields
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect(),
    ]
    .concat()
}

fn select(job: i32) -> Vec<u8> {
    command(26, 13, &[job])
}

fn unequip(slot: i32, profile: i32) -> Vec<u8> {
    command(42, 2, &[slot, profile])
}

fn equip(item: i32, profile: i32, slot: i32) -> Vec<u8> {
    command(42, 3, &[item, profile, slot])
}

fn equipped_item(game: &Gameplay, profile: i32, slot: i32) -> i32 {
    game.state()
        .inventory
        .equipment
        .iter()
        .find(|entry| entry.profile_id == profile && entry.slot == slot)
        .map_or(0, |entry| entry.item_id)
}

/// Grant one definition through the production `realm give` path and return
/// the new instance id.
fn give(game: &mut Gameplay, character: &CharacterState, definition: i32, now: u64) -> i32 {
    let out = game
        .handle_in_world(
            &chat::local_request(&format!("realm give {definition}")),
            character,
            now,
        )
        .unwrap();
    assert!(!out.replies.is_empty(), "realm give {definition}");
    game.state()
        .inventory
        .items
        .iter()
        .rev()
        .find(|item| item.definition_id == definition)
        .expect("the granted item instance exists")
        .id
}

fn toolbars(out: &Outcome) -> Vec<&Vec<u8>> {
    out.replies
        .iter()
        .filter(|packet| packet.starts_with(&[36, 0, 5, 0]))
        .collect()
}

/// The one toolbar a committed weapon change resends.
fn sole_bar(out: &Outcome) -> &Vec<u8> {
    let bars = toolbars(out);
    assert_eq!(bars.len(), 1, "one toolbar resend");
    bars[0]
}

fn slots(bar: &Vec<u8>) -> Vec<Option<i32>> {
    bar_ids(std::slice::from_ref(bar))
}

/// The name id in the type-3 basic entry (toolbar header 12, kind, id, energy,
/// icon, then the name the client resolves at `0x008ee3d0`).
fn basic_name(bar: &[u8]) -> i32 {
    assert!(bar.starts_with(&[36, 0, 5, 0]));
    assert_eq!(
        i32::from_le_bytes(bar[12..16].try_into().unwrap()),
        3,
        "the basic slot is a filled type-3 entry"
    );
    i32::from_le_bytes(bar[28..32].try_into().unwrap())
}

fn definitions(out: &Outcome) -> Vec<i32> {
    out.replies
        .iter()
        .filter(|packet| packet.starts_with(&[36, 0, 13, 0]))
        .map(|packet| i32::from_le_bytes(packet[4..8].try_into().unwrap()))
        .collect()
}

#[test]
fn equip_relabels_the_basic_attack() {
    let fixture = Fixture::new();
    let (mut game, character) = fixture.game();
    let second = give(&mut game, &character, SECOND, 200);
    let out = game
        .handle_in_world(&equip(second, BRAWLER, 7), &character, 300)
        .unwrap();
    let bar = sole_bar(&out);
    assert_eq!(slots(bar), ARMED_BAR);
    assert_eq!(
        basic_name(bar),
        SECOND_NAME,
        "the basic slot names the equipped hammer"
    );
    // Only the bar label changes: the basic ability definition still carries
    // the ability's own name (420172), not the weapon's.
    let basic = out
        .replies
        .iter()
        .find(|packet| {
            packet.starts_with(&[36, 0, 13, 0]) && packet[4..8] == 4895_i32.to_le_bytes()
        })
        .expect("the basic ability definition precedes the bar");
    assert_eq!(
        i32::from_le_bytes(basic[10..14].try_into().unwrap()),
        420172
    );
    // The label derives from the saved equipment: a reconnect rebuilds it.
    let saved = game.state();
    let mut reconnected = Gameplay::load(fixture.content(), Some(saved)).unwrap();
    let entered = reconnected.initialize_world(&character, 500).unwrap();
    let bar = toolbars(&entered)
        .into_iter()
        .next()
        .expect("world entry sends the toolbar");
    assert_eq!(
        basic_name(bar),
        SECOND_NAME,
        "a reconnect keeps the equipped weapon's name"
    );
    // Replaying the same equip is a no-op and sends no second bar.
    let replay = game
        .handle_in_world(&equip(second, BRAWLER, 7), &character, 400)
        .unwrap();
    assert!(toolbars(&replay).is_empty(), "a no-op replay sends no bar");
}

#[test]
fn unarmed_bar_is_empty_and_rearm_restores_it() {
    let fixture = Fixture::new();
    let (mut game, character) = fixture.game();
    let starter = equipped_item(&game, BRAWLER, 7);
    assert_ne!(starter, 0, "the starter hammer is equipped");

    // Unequip: eight empty slots and no ability definitions (R, OSFR #124).
    let out = game
        .handle_in_world(&unequip(7, BRAWLER), &character, 200)
        .unwrap();
    let bar = sole_bar(&out);
    assert!(
        slots(bar).iter().all(Option::is_none),
        "an unarmed combat job gets the empty bar"
    );
    assert!(
        definitions(&out).is_empty(),
        "no ability definitions while unarmed"
    );

    // A wrong-class weapon is also unarmed: profile 43 may equip class 5, but
    // its cast gate requires class 71.
    let wrong = give(&mut game, &character, WRONG_CLASS, 300);
    let out = game
        .handle_in_world(&equip(wrong, BRAWLER, 7), &character, 400)
        .unwrap();
    assert!(
        slots(sole_bar(&out)).iter().all(Option::is_none),
        "a wrong-class weapon empties the bar"
    );

    // Re-equipping the starter through the item-record request (42/8) restores
    // the full job bar and its name.
    let starter_item = game
        .state()
        .inventory
        .items
        .iter()
        .find(|item| item.id == starter)
        .expect("the starter instance exists")
        .clone();
    let out = game
        .handle_in_world(
            &command(
                42,
                8,
                &[starter_item.definition_id, starter_item.tint, BRAWLER, 7],
            ),
            &character,
            500,
        )
        .unwrap();
    let bar = sole_bar(&out);
    assert_eq!(slots(bar), ARMED_BAR);
    assert_eq!(basic_name(bar), STARTER_NAME);
}

#[test]
fn unrelated_or_refused_equip_sends_no_bar() {
    let fixture = Fixture::new();
    let (mut game, character) = fixture.game();

    // Another slot: unequip and re-equip the active job's chest.
    let chest = equipped_item(&game, BRAWLER, 3);
    assert_ne!(chest, 0, "the Brawler wears a chest");
    let out = game
        .handle_in_world(&unequip(3, BRAWLER), &character, 200)
        .unwrap();
    assert!(
        toolbars(&out).is_empty(),
        "another slot's unequip sends no bar"
    );
    let out = game
        .handle_in_world(&equip(chest, BRAWLER, 3), &character, 300)
        .unwrap();
    assert!(
        toolbars(&out).is_empty(),
        "another slot's equip sends no bar"
    );

    // An inactive job's outfit: the Warrior cudgel for profile 32. It is a
    // real accepted equip, but the active Brawler bar does not change.
    let cudgel = give(&mut game, &character, 29963, 400);
    let out = game
        .handle_in_world(&equip(cudgel, 32, 7), &character, 500)
        .unwrap();
    assert!(
        toolbars(&out).is_empty(),
        "an inactive job's equip sends no bar"
    );

    // A refused equip (class 45 is not unlocked) changes nothing.
    let locked = give(&mut game, &character, LOCKED_PROFILE_ITEM, 600);
    let before = game.state();
    let out = game
        .handle_in_world(&equip(locked, 45, 7), &character, 700)
        .unwrap();
    assert_eq!(game.state(), before, "a refused equip changes nothing");
    assert!(toolbars(&out).is_empty(), "a refused equip sends no bar");

    // A no-op replay of the equipped starter.
    let starter = equipped_item(&game, BRAWLER, 7);
    let out = game
        .handle_in_world(&equip(starter, BRAWLER, 7), &character, 800)
        .unwrap();
    assert!(toolbars(&out).is_empty(), "a no-op replay sends no bar");
}
