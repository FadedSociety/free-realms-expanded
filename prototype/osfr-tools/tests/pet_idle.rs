//! T531: pet idle life and thought bubbles, through the world tick (`follow_owner` then
//! `care_tick`, which `Gameplay::pet_tick` runs) and through `Gameplay` itself. A pet settled
//! beside a still owner plays idle and bathroom animations, lies down after a while and stands up
//! when the owner moves; a dog or cat shakes itself dry after the owner leaves water; a non-vanity
//! pet shows a thought bubble (35/27) for its lowest need. The owner and the viewers see the same
//! 35/8 and 35/27 packets, nothing is saved, and a walking pet shows none of it.
#[path = "support/pet_care.rs"]
mod care;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;

use care::*;
use npc_quest_chain::{CHARACTER, Fixture};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    pets::{PetContent, PetService},
    startup_world::parse_add_npc,
    world_events::piece_052::ANIM_ID_STAND,
};
use pet_inventory::InventoryFixture;
use pet_wire::{perform_trick, summon, water};
use serde_json::{Value, json};

/// 1,000 s after adoption: the same base the other pet suites use.
const START: u64 = 1_000_000;
/// The world tick, in ms.
const TICK: u64 = 500;
/// The follow rule's stillness before a pet walks to its settle spot (`follow.settle_delay_ms`).
const SETTLE_DELAY: u64 = 750;
/// A vanity Pet Pal (T454).
const CRAB_ITEM: i32 = 7885;
const DISMISS: [u8; 8] = [53, 0, 4, 0, 0, 0, 0, 0];
/// The cat model the trick suite also uses.
const CAT: i32 = 50;
/// Model 364 (the dog) knows this trick and has an animation row for it.
const TRICK: u32 = 11;

/// A shipped `care.idle` number.
fn rule(key: &str) -> u64 {
    pets_json()["care"]["idle"][key].as_u64().unwrap()
}

fn icon(key: &str) -> i32 {
    rule(key) as i32
}

/// What one 35/8 asked for.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Play {
    animation: i32,
    base: bool,
}

/// The client's 35/27 fields (reader `008d7880` after the base words `008d6830`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bubble {
    icon: i32,
    seconds: i32,
}

/// Every 35/8 for `actor`: base words, u64 guid, i32 slot, i32 0, u8 base flag, 21 bytes.
fn plays(packets: &[Vec<u8>], actor: u64) -> Vec<Play> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 8, 0]))
        .map(|packet| {
            assert_eq!(packet.len(), 21);
            assert_eq!(u64::from_le_bytes(packet[4..12].try_into().unwrap()), actor);
            assert_eq!(u32_at(packet, 16), 0);
            Play {
                animation: u32_at(packet, 12) as i32,
                base: packet[20] == 1,
            }
        })
        .collect()
}

/// Every 35/27 for `actor`: base words, u64 guid, i32 icon, i32 seconds, u8 clear flag (0).
fn bubbles(packets: &[Vec<u8>], actor: u64) -> Vec<Bubble> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 27, 0]))
        .map(|packet| {
            assert_eq!(
                packet.len(),
                21,
                "base words 4, guid 8, icon 4, seconds 4, clear 1"
            );
            assert_eq!(u64::from_le_bytes(packet[4..12].try_into().unwrap()), actor);
            assert_eq!(packet[20], 0, "a bubble is never cleared by the server");
            Bubble {
                icon: u32_at(packet, 12) as i32,
                seconds: u32_at(packet, 16) as i32,
            }
        })
        .collect()
}

fn visual(packets: &[Vec<u8>]) -> Vec<Vec<u8>> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 8, 0]) || packet.starts_with(&[35, 0, 27, 0]))
        .cloned()
        .collect()
}

/// What one tick showed.
struct Seen {
    at: u64,
    plays: Vec<Play>,
    bubbles: Vec<Bubble>,
    /// The owner's 35/8 and 35/27 packets, as sent.
    raw: Vec<Vec<u8>>,
}

/// A pet standing up: the base returns to standing, then the posture's exit transition plays.
fn assert_stand_up(seen: &Seen) {
    assert_eq!(
        seen.plays.first(),
        Some(&Play {
            animation: ANIM_ID_STAND,
            base: true
        }),
        "the base returns to standing"
    );
    assert_eq!(seen.plays.len(), 2, "then the exit transition");
    assert!(!seen.plays[1].base);
}

/// The owner's pet summoned at `START`: the tick below does what `Gameplay::pet_tick` does.
struct Session {
    service: PetService,
    fixture: InventoryFixture,
    pet: u32,
    actor: u64,
    owner: [f32; 3],
}

impl Session {
    fn new(mut service: PetService, pet: u32, seed: u64) -> Self {
        service.set_idle_seed(seed);
        let mut fixture = kit();
        send(&mut service, &mut fixture, &summon(pet), START);
        Self {
            service,
            fixture,
            pet,
            actor: PetService::pet_guid(OWNER),
            owner: [0.0; 3],
        }
    }

    /// A dog on edited content, its needs set to `needs` (food, groom, play).
    fn dog_with(content: PetContent, needs: [f32; 3], seed: u64) -> Self {
        let (service, pet) = dog_with(content.clone());
        Self::new(with_needs(service, content, pet, needs), pet, seed)
    }

    /// One world tick at `at`: the follow step, then the care tick. The owner and the viewers must
    /// see the same idle packets.
    fn tick(&mut self, at: u64) -> Seen {
        let mut wire = self
            .service
            .follow_owner(OWNER, self.owner, |_, _| None, at);
        let care = self.service.care_tick(&mut self.fixture.inventory, at);
        assert!(!care.changed, "idle life saves nothing");
        wire.private.extend(care.private);
        wire.shared.extend(care.shared);
        assert_eq!(
            visual(&wire.private),
            visual(&wire.shared),
            "the viewers see what the owner sees"
        );
        Seen {
            at,
            plays: plays(&wire.private, self.actor),
            bubbles: bubbles(&wire.private, self.actor),
            raw: visual(&wire.private),
        }
    }

    /// Ticks every `TICK` ms from `from` to `to`; only the ticks that showed something.
    fn run(&mut self, from: u64, to: u64) -> Vec<Seen> {
        (from..=to)
            .step_by(TICK as usize)
            .map(|at| self.tick(at))
            .filter(|seen| !seen.plays.is_empty() || !seen.bubbles.is_empty())
            .collect()
    }

    /// Ticks until the pet lies down (`rest_after_ms` must already be due) and returns that tick.
    fn lie_down(&mut self) -> Seen {
        let mut at = START;
        loop {
            at += TICK;
            let seen = self.tick(at);
            if seen.plays.iter().any(|play| play.base) {
                return seen;
            }
            assert!(at < START + 10_000, "the pet lies down");
        }
    }

    fn send(&mut self, packet: &[u8], at: u64) {
        send(&mut self.service, &mut self.fixture, packet, at);
    }
}

/// `service` reloaded from its own save with `pet`'s needs set and its clock at `START`, so the
/// summon decays nothing and the live decay (0.02 per second) runs from these values.
fn with_needs(service: PetService, content: PetContent, pet: u32, needs: [f32; 3]) -> PetService {
    let mut document = service.document_value().unwrap();
    let row = document["pets"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == pet)
        .unwrap();
    let mood = needs.iter().sum::<f32>() / 3.0;
    row["needs"] = json!({"food": needs[0], "groom": needs[1], "play": needs[2], "mood": mood});
    row["updated_at"] = json!(START / 1_000);
    PetService::from_document(content, Some(document)).unwrap()
}

/// The shipped model's idle animation slots, bathroom slots and shake-off slot.
fn rows(model: i32) -> (Vec<i32>, Vec<i32>, Option<i32>) {
    let content = PetContent::bundled().unwrap();
    let idle = content.model(model).unwrap().idle.as_ref().unwrap();
    (
        idle.animations.iter().map(|row| row.animation_id).collect(),
        idle.bathroom.iter().map(|row| row.animation_id).collect(),
        idle.shake_off.map(|row| row.animation_id),
    )
}

fn content_idle(key: &str, value: Value) -> PetContent {
    content_with(|document| document["care"]["idle"][key] = value)
}

/// A quiet pet: no first idle animation or rest within the test.
fn no_idle() -> PetContent {
    content_with(|document| {
        document["care"]["idle"]["idle_after_ms"] = json!(3_600_000);
        document["care"]["idle"]["rest_after_ms"] = json!(3_600_000);
    })
}

const SEEDS: std::ops::Range<u64> = 0..8;

#[test]
fn the_thought_bubble_bytes_match_the_client_reader() {
    let mut session = Session::dog_with(PetContent::bundled().unwrap(), [10.0, 90.0, 90.0], 1);
    let seen = session.run(START, START + 120_000);
    let first = seen
        .iter()
        .find(|seen| !seen.bubbles.is_empty())
        .expect("a hungry pet shows a bubble within two minutes");
    // Rebuilt from the reader: base words 35 and 27, guid, icon, seconds, clear flag.
    let mut expected = vec![35, 0, 27, 0];
    expected.extend(session.actor.to_le_bytes());
    expected.extend(icon("hunger_icon").to_le_bytes());
    expected.extend((rule("bubble_seconds") as i32).to_le_bytes());
    expected.push(0);
    assert_eq!(expected.len(), 21);
    assert_eq!(first.raw, [expected]);
}

/// F809: the four bubble ids are client `ActorCompositeEffectDefinitions.xml` rows, not OSFR's
/// 39/40/41/42 (three unrelated particle effects and one undefined): 5524
/// `PFX_petbubble_bone_loop` (a thought balloon with a dog bone), 5241
/// `PFX_dirt_brown_dog_dirt-indicator_loop` (dirty), 10 `PFX_emo_question-marks_red_head`
/// (bored) and 5099 `PFX_emo_grumble_red_head` (bad mood). A data edit drops one of these only
/// with new client evidence.
#[test]
fn the_bubble_icons_are_client_thought_and_emote_effects() {
    assert_eq!(icon("hunger_icon"), 5524);
    assert_eq!(icon("hygiene_icon"), 5241);
    assert_eq!(icon("play_icon"), 10);
    assert_eq!(icon("sad_icon"), 5099);
}

#[test]
fn a_hungry_pet_shows_the_hunger_icon_inside_the_window_and_never_more_often_than_the_minimum() {
    let (min, max) = (rule("bubble_min_ms"), rule("bubble_max_ms"));
    for seed in SEEDS {
        let mut session =
            Session::dog_with(PetContent::bundled().unwrap(), [10.0, 90.0, 90.0], seed);
        let seen = session.run(START, START + 20 * 60_000);
        let seen: Vec<&Seen> = seen
            .iter()
            .filter(|seen| !seen.bubbles.is_empty())
            .collect();
        let times: Vec<u64> = seen.iter().map(|seen| seen.at).collect();
        assert!(times.len() >= 15, "seed {seed}: {times:?}");
        for seen in &seen {
            assert_eq!(
                seen.bubbles,
                [Bubble {
                    icon: icon("hunger_icon"),
                    seconds: rule("bubble_seconds") as i32
                }],
                "seed {seed}"
            );
        }
        // The first tick only schedules; the first bubble comes a full gap later.
        assert!(
            (min..=max + TICK).contains(&(times[0] - START)),
            "seed {seed}: {times:?}"
        );
        for pair in times.windows(2) {
            let gap = pair[1] - pair[0];
            assert!((min..=max + TICK).contains(&gap), "seed {seed}: gap {gap}");
        }
    }
}

#[test]
fn the_lowest_need_picks_the_icon_and_a_contented_pet_shows_none() {
    let first_icon = |content: PetContent, needs: [f32; 3]| {
        let mut session = Session::dog_with(content, needs, 3);
        let seen = session.run(START, START + 3 * 60_000);
        let icons: Vec<i32> = seen
            .iter()
            .flat_map(|seen| seen.bubbles.iter().map(|bubble| bubble.icon))
            .collect();
        assert!(
            icons.windows(2).all(|pair| pair[0] == pair[1]),
            "one reason at a time: {icons:?}"
        );
        icons.first().copied()
    };
    let bundled = || PetContent::bundled().unwrap();
    let hunger = icon("hunger_icon");
    let hygiene = icon("hygiene_icon");
    let play = icon("play_icon");
    assert_eq!(first_icon(bundled(), [10.0, 50.0, 60.0]), Some(hunger));
    assert_eq!(first_icon(bundled(), [50.0, 10.0, 60.0]), Some(hygiene));
    assert_eq!(first_icon(bundled(), [50.0, 60.0, 10.0]), Some(play));
    assert_eq!(
        first_icon(bundled(), [10.0, 10.0, 10.0]),
        Some(hunger),
        "a tie goes to food, then groom"
    );
    assert_eq!(
        first_icon(bundled(), [50.0, 10.0, 10.0]),
        Some(hygiene),
        "a tie goes to groom before play"
    );
    // Nothing is low and the mood is fine: no bubble.
    assert_eq!(first_icon(bundled(), [40.0, 40.0, 40.0]), None);

    // The threshold is data: raised above the needs, a need shows its icon.
    let raised = content_idle("need_bubble_threshold", json!(60));
    assert_eq!(first_icon(raised, [40.0, 50.0, 55.0]), Some(hunger));
    // A low mood with no low need shows the sad icon (R: mood is the need average today, so
    // the thresholds are split to reach it).
    let sad = content_with(|document| {
        document["care"]["idle"]["need_bubble_threshold"] = json!(0);
        document["care"]["idle"]["mood_bubble_threshold"] = json!(60);
    });
    assert_eq!(
        first_icon(sad, [40.0, 40.0, 40.0]),
        Some(icon("sad_icon")),
        "sad"
    );
}

#[test]
fn a_vanity_pet_shows_no_bubbles() {
    let content = PetContent::bundled().unwrap();
    let crab_model = content.model_id(&content.definition_for_item(CRAB_ITEM).unwrap().model);
    assert!(content.is_vanity(crab_model));
    let crab_definition = content.definition_for_item(CRAB_ITEM).unwrap();
    let (mut service, dog_id) = dog();
    let crab = service.adopt_vendor_grant(2, &crab_definition, 0).unwrap();
    // Both pets are starving; only the vanity flag differs.
    let service = with_needs(service, content.clone(), crab, [5.0; 3]);
    let service = with_needs(service, content, dog_id, [5.0; 3]);
    let mut session = Session::new(service.clone(), crab, 4);
    assert!(
        session
            .run(START, START + 10 * 60_000)
            .iter()
            .all(|seen| seen.bubbles.is_empty()),
        "a vanity pet is silent"
    );
    let mut session = Session::new(service, dog_id, 4);
    assert!(
        session
            .run(START, START + 10 * 60_000)
            .iter()
            .any(|seen| !seen.bubbles.is_empty()),
        "the same needs on a dog do show bubbles"
    );
}

/// The canary of the settled check: remove `active.pose.settled()` from `idle_tick` and this fails.
#[test]
fn a_walking_pet_never_idles_and_a_still_one_does() {
    let content = PetContent::bundled().unwrap();
    for seed in SEEDS {
        let (service, pet) = dog();
        // Needs stay high, so no bubble either: any packet here is an idle animation.
        let service = with_needs(service, content.clone(), pet, [95.0; 3]);
        let mut session = Session::new(service, pet, seed);
        // 10 minutes of walking: far past `idle_after_ms` and `rest_after_ms`.
        for at in (START..=START + 10 * 60_000).step_by(TICK as usize) {
            session.owner[2] += 1.0;
            let seen = session.tick(at);
            assert!(
                seen.raw.is_empty(),
                "seed {seed}: a walking pet showed {seen:?}",
                seen = seen.raw
            );
        }

        // The owner stops: the same pet now idles.
        let still = session.run(START + 10 * 60_000 + TICK, START + 11 * 60_000);
        assert!(
            still.iter().any(|seen| !seen.plays.is_empty()),
            "seed {seed}: a still owner's pet never animated"
        );
    }
}

#[test]
fn a_settled_pet_idles_after_its_delay_then_on_schedule_with_its_models_rows() {
    let (min, max, after) = (
        rule("idle_min_ms"),
        rule("idle_max_ms"),
        rule("idle_after_ms"),
    );
    let (idle, bathroom, _) = rows(364);
    for seed in SEEDS {
        let content = content_idle("rest_after_ms", json!(3_600_000));
        let mut session = Session::dog_with(content, [95.0; 3], seed);
        let seen = session.run(START, START + 8 * 60_000);
        let times: Vec<u64> = seen.iter().map(|seen| seen.at).collect();
        // Nothing before the pet had stood still for `idle_after_ms`; and soon after.
        assert!(
            times[0] >= START + SETTLE_DELAY + after,
            "seed {seed}: {times:?}"
        );
        assert!(
            times[0] <= START + SETTLE_DELAY + after + 5_000,
            "seed {seed}: {times:?}"
        );
        for pair in times.windows(2) {
            let gap = pair[1] - pair[0];
            assert!((min..=max + TICK).contains(&gap), "seed {seed}: gap {gap}");
        }
        assert!(times.len() >= 7, "seed {seed}: {times:?}");
        for seen in &seen {
            assert_eq!(seen.plays.len(), 1, "one animation per window");
            let play = seen.plays[0];
            assert!(!play.base, "an idle animation is a one-shot");
            assert!(
                idle.contains(&play.animation) || bathroom.contains(&play.animation),
                "seed {seed}: {play:?} is not one of the model's rows"
            );
        }
    }
}

#[test]
fn the_bathroom_chance_is_data() {
    let (idle, bathroom, _) = rows(364);
    assert!(idle.iter().all(|id| !bathroom.contains(id)));
    let ids = |chance: f64| {
        let content = content_with(|document| {
            document["care"]["idle"]["bathroom_chance"] = json!(chance);
            document["care"]["idle"]["rest_after_ms"] = json!(3_600_000);
        });
        let mut session = Session::dog_with(content, [95.0; 3], 5);
        session
            .run(START, START + 10 * 60_000)
            .iter()
            .map(|seen| seen.plays[0].animation)
            .collect::<Vec<_>>()
    };
    let always = ids(1.0);
    assert!(always.len() > 8 && always.iter().all(|id| bathroom.contains(id)));
    let never = ids(0.0);
    assert!(never.len() > 8 && never.iter().all(|id| idle.contains(id)));
}

#[test]
fn a_pet_lies_down_after_its_delay_fidgets_and_stands_when_the_owner_moves() {
    let (min, max, after, rest) = (
        rule("idle_min_ms"),
        rule("idle_max_ms"),
        rule("idle_after_ms"),
        rule("rest_after_ms"),
    );
    let content = PetContent::bundled().unwrap();
    let model = content.model(364).unwrap().idle.as_ref().unwrap();
    for seed in SEEDS {
        let mut session = Session::dog_with(content.clone(), [95.0; 3], seed);
        let seen = session.run(START, START + 6 * 60_000);
        let lying = seen
            .iter()
            .position(|seen| seen.plays.iter().any(|play| play.base))
            .unwrap_or_else(|| panic!("seed {seed}: the pet never lay down"));
        let down = &seen[lying];
        // The loop is the base animation, then the enter transition plays over it.
        let [looped, enter] = down.plays[..] else {
            panic!("seed {seed}: {:?}", down.plays)
        };
        assert!(looped.base && !enter.base);
        let posture = model
            .rest_postures
            .iter()
            .find(|posture| {
                posture.enter.animation_id == enter.animation
                    && posture.loops.contains(&looped.animation)
            })
            .unwrap_or_else(|| panic!("seed {seed}: no posture has {looped:?} and {enter:?}"));
        assert!(down.at >= START + SETTLE_DELAY + rest, "seed {seed}");
        assert!(
            down.at <= START + SETTLE_DELAY + rest + 20_000,
            "seed {seed}"
        );
        // Before it lay down: idle animations only. After: this posture's fidgets only.
        let (idle, bathroom, _) = rows(364);
        for before in &seen[..lying] {
            assert!(before.at >= START + SETTLE_DELAY + after);
            assert!(before.plays.iter().all(|play| !play.base
                && (idle.contains(&play.animation) || bathroom.contains(&play.animation))));
        }
        let fidgets: Vec<i32> = posture.fidgets.iter().map(|row| row.animation_id).collect();
        assert!(
            seen.len() > lying + 2,
            "seed {seed}: it fidgets while it lies"
        );
        let mut last = down.at + u64::from(posture.enter.duration_ms);
        for fidget in &seen[lying + 1..] {
            assert_eq!(fidget.plays.len(), 1);
            assert!(!fidget.plays[0].base && fidgets.contains(&fidget.plays[0].animation));
            let gap = fidget.at - last;
            assert!((min..=max + TICK).contains(&gap), "seed {seed}: {gap}");
            last = fidget.at;
        }

        // The owner moves: the base animation returns to standing, the exit transition plays, and
        // no old timer runs on: the next idle animation is a full delay after the pet settles.
        let moved_at = START + 6 * 60_000 + TICK;
        session.owner = [4.0, 0.0, 0.0];
        let up = session.tick(moved_at);
        assert_stand_up(&up);
        assert_eq!(
            up.plays[1].animation, posture.exit.animation_id,
            "seed {seed}"
        );
        let after_up = session.run(moved_at + TICK, moved_at + 3 * 60_000);
        let (idle, bathroom, _) = rows(364);
        assert!(
            after_up[0].at >= moved_at + SETTLE_DELAY + after,
            "seed {seed}"
        );
        assert!(after_up[0].plays.iter().all(|play| !play.base
            && (idle.contains(&play.animation) || bathroom.contains(&play.animation))));
    }
}

/// F809/T490: a routed follow click sends the pet walking on the next tick, so the server pose
/// stops counting as settled and a pet that lay down stands up instead of fidgeting in place.
#[test]
fn a_follow_click_wakes_a_lying_pet_and_pauses_its_idle_life() {
    let content = content_with(|document| {
        document["care"]["idle"]["rest_after_ms"] = json!(0);
        document["care"]["idle"]["idle_after_ms"] = json!(3_600_000);
        document["care"]["idle"]["idle_min_ms"] = json!(1_000);
        document["care"]["idle"]["idle_max_ms"] = json!(1_000);
    });
    let mut session = Session::dog_with(content, [95.0; 3], 11);
    let lying = session.lie_down();
    assert!(
        session
            .service
            .view(OWNER)
            .unwrap()
            .base_animation
            .is_some()
    );
    // A refused click changes nothing: the pet stays where it lies, still settled and with no
    // destination stored. The 53/26 reply (flag 0) is the only thing sent.
    let actor = session.actor;
    let can_go = |flag: bool| {
        let mut packet = vec![53, 0, 26];
        packet.extend(actor.to_le_bytes());
        packet.push(u8::from(flag));
        packet
    };
    let before = session.service.view(OWNER).unwrap();
    for target in [
        [f32::NAN, 0.0, 0.0, 0.0],
        [1e38, 1e38, 1e38, 0.0],
        [0.0, 1e38, 0.0, 0.0],
        [100.0, 0.0, 0.0, 0.0],
    ] {
        let refused =
            session
                .service
                .follow_location(target, session.owner, |_, _| Some(0.0), lying.at + 1);
        assert_eq!(refused.private, vec![can_go(false)], "{target:?}");
        assert_eq!(session.service.view(OWNER).unwrap(), before, "{target:?}");
    }
    let still = session.tick(lying.at + TICK);
    assert!(
        still.plays.is_empty() && still.bubbles.is_empty(),
        "a refused click leaves the rest in place: {:?}",
        still.plays
    );
    assert_eq!(
        session.service.view(OWNER).unwrap().base_animation,
        before.base_animation
    );
    // Click "go there", within the client's 8-40 m band but far from the settle spot: the server
    // walks the pet there, so the rest posture ends on the next tick instead of fidgeting on in
    // place.
    let accepted = session.service.follow_location(
        [20.0, 0.0, 0.0, 0.0],
        session.owner,
        |_, _| Some(0.0),
        lying.at + TICK,
    );
    assert_eq!(accepted.private, vec![can_go(true)]);
    let up = session.tick(lying.at + 2 * TICK);
    assert_stand_up(&up);
    assert_eq!(session.service.view(OWNER).unwrap().base_animation, None);
    // It walks to the clicked spot, so no rest pose resumes while the walk runs.
    let walked = session.run(lying.at + 3 * TICK, lying.at + 6_000);
    assert!(
        walked
            .iter()
            .all(|seen| seen.plays.iter().all(|play| !play.base)),
        "a walking pet does not lie down"
    );
}

/// F809: a viewer who arrives while the pet lies down is sent the rest loop as a base animation
/// after the AddNpc, so it does not see a standing pet through the rest.
#[test]
fn a_late_viewer_is_sent_the_rest_loop_with_the_pet_spawn() {
    let content = content_with(|document| {
        document["care"]["idle"]["rest_after_ms"] = json!(0);
        document["care"]["idle"]["idle_after_ms"] = json!(3_600_000);
    });
    let mut session = Session::dog_with(content, [95.0; 3], 12);
    let looped = session
        .lie_down()
        .plays
        .into_iter()
        .find(|play| play.base)
        .expect("the lying tick has a base loop")
        .animation;
    let view = session.service.view(OWNER).expect("the pet is out");
    assert_eq!(view.base_animation, Some(looped));
    let packets = view.spawn_packets().expect("the AddNpc builds");
    assert_eq!(packets.len(), 2, "the AddNpc and the base loop");
    assert_eq!(packets[0], view.spawn_packet().unwrap());
    let mut expected = vec![35, 0, 8, 0];
    expected.extend(session.actor.to_le_bytes());
    expected.extend(looped.to_le_bytes());
    expected.extend([0u8; 4]);
    expected.push(1);
    assert_eq!(
        packets[1], expected,
        "the base rest loop the pose is playing"
    );
}

/// F809: the rest ends on every way out of "settled", not only the owner moving: entering water
/// and starting a care action stand a lying pet up too.
#[test]
fn water_or_a_care_action_also_stands_a_lying_pet_up() {
    let content = content_with(|document| {
        document["care"]["idle"]["rest_after_ms"] = json!(0);
        document["care"]["idle"]["idle_after_ms"] = json!(3_600_000);
    });

    let mut session = Session::dog_with(content.clone(), [95.0; 3], 14);
    let lying = session.lie_down();
    session.send(&water(true), lying.at + TICK);
    assert_stand_up(&session.tick(lying.at + 2 * TICK));

    let mut session = Session::dog_with(content, [95.0; 3], 15);
    let lying = session.lie_down();
    session.send(&perform_trick(TRICK), lying.at + TICK);
    assert_stand_up(&session.tick(lying.at + 2 * TICK));
}

#[test]
fn care_pauses_the_idle_life_and_restarts_its_delay() {
    // A groom or a trick running is not "settled": the idle clock starts over when it ends.
    let (service, pet) = dog();
    let content = PetContent::bundled().unwrap();
    let mut session = Session::new(with_needs(service, content, pet, [60.0; 3]), pet, 2);
    // The pet settles; a trick starts just before its first idle animation is due.
    session.run(START, START + 8_000);
    let trick = PetContent::bundled()
        .unwrap()
        .model(364)
        .unwrap()
        .trick_animations
        .iter()
        .find(|row| row.trick_id == TRICK)
        .unwrap()
        .clone();
    session.send(&perform_trick(TRICK), START + 8_500);
    let trick_ends = START + 8_500 + u64::from(trick.start_delay_ms) + u64::from(trick.duration_ms);
    let seen = session.run(START + 9_000, trick_ends + TICK);
    let (idle, bathroom, _) = rows(364);
    assert!(
        seen.iter().flat_map(|seen| &seen.plays).all(|play| {
            ![idle.as_slice(), bathroom.as_slice()]
                .concat()
                .contains(&play.animation)
        }),
        "no idle animation while the trick runs: {:?}",
        seen.iter().map(|seen| &seen.plays).collect::<Vec<_>>()
    );
    let later = session.run(trick_ends + 2 * TICK, trick_ends + 60_000);
    assert!(
        later[0].at >= trick_ends + rule("idle_after_ms"),
        "the delay starts over after the trick"
    );
}

#[test]
fn leaving_water_shakes_a_dog_or_cat_once_it_stands_still() {
    let shake = rows(364).2.expect("a dog shakes");
    assert_eq!(rows(CAT).2, Some(shake), "a cat shakes too");
    for model in [364, CAT] {
        let content = no_idle();
        let (service, pet) = model_pet(content.clone(), model);
        let mut session = Session::new(with_needs(service, content, pet, [95.0; 3]), pet, 6);
        let mut at = START;
        // Settled, then in the water: nothing shows, and nothing is armed by being wet.
        assert!(session.run(at, at + 5_000).is_empty());
        at += 5_000;
        session.send(&water(true), at);
        assert!(session.run(at, at + 5_000).is_empty());
        at += 5_000;
        // Out again: one shake at the next tick, a one-shot on the pet, then quiet.
        session.send(&water(false), at);
        let seen = session.run(at, at + 30_000);
        assert_eq!(seen.len(), 1, "model {model}: exactly one shake");
        assert_eq!(
            seen[0].plays,
            [Play {
                animation: shake,
                base: false
            }]
        );
        assert!(seen[0].at <= at + TICK);
        // "Out" with no water before is a client that is already dry: nothing is armed.
        at += 30_000;
        for _ in 0..1_000 {
            session.send(&water(false), at);
        }
        assert!(session.run(at, at + 30_000).is_empty(), "model {model}");
        // Rapid in/out toggling arms at most one shake, and it still plays at most once.
        for step in 0..100 {
            session.send(&water(step % 2 == 0), at + step);
        }
        let shakes = session.run(at + 100, at + 30_000);
        assert_eq!(
            shakes.len(),
            1,
            "model {model}: one shake from the toggle spam"
        );
        assert_eq!(
            shakes[0].plays,
            [Play {
                animation: shake,
                base: false
            }]
        );
    }
}

#[test]
fn a_pet_waits_for_its_walk_to_end_before_it_shakes_and_entering_water_cancels_it() {
    let content = no_idle();
    let (service, pet) = dog_with(content.clone());
    let mut session = Session::new(with_needs(service, content, pet, [95.0; 3]), pet, 7);
    session.run(START, START + 5_000);
    session.send(&water(true), START + 5_000);
    session.send(&water(false), START + 6_000);
    // The owner walks out of the water: no shake while it walks.
    let mut at = START + 6_000;
    for _ in 0..20 {
        session.owner[2] += 1.0;
        at += TICK;
        let seen = session.tick(at);
        assert!(seen.plays.is_empty(), "no shake while walking");
    }
    let seen = session.run(at + TICK, at + 20_000);
    assert_eq!(seen.len(), 1, "one shake once it stands still");
    assert!(seen[0].at >= at + SETTLE_DELAY);

    // Out of the water but back in before it stood still: the shake is cancelled.
    session.send(&water(true), at + 30_000);
    session.send(&water(false), at + 30_100);
    session.send(&water(true), at + 30_200);
    assert!(session.run(at + 30_200, at + 60_000).is_empty());
}

#[test]
fn a_model_that_cannot_shake_never_does_and_one_without_rows_is_quiet_but_still_shows_bubbles() {
    // A model with idle rows and no shake-off row.
    let document = pets_json();
    let plain = document["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|model| {
            model["idle"].is_object()
                && model["idle"]["shake_off"].is_null()
                && model["vanity"] != json!(true)
        })
        .expect("some pet cannot shake")["model_id"]
        .as_i64()
        .unwrap() as i32;
    let content = no_idle();
    let (service, pet) = model_pet(content.clone(), plain);
    let mut session = Session::new(with_needs(service, content, pet, [95.0; 3]), pet, 8);
    session.run(START, START + 5_000);
    session.send(&water(true), START + 5_000);
    session.send(&water(false), START + 6_000);
    assert!(session.run(START + 6_000, START + 60_000).is_empty());

    // The dog's rows removed: no animation at all, no panic, and the bubble still works.
    let content = content_with(|document| {
        model_row(document, 364)
            .as_object_mut()
            .unwrap()
            .remove("idle");
        document["care"]["idle"]["idle_after_ms"] = json!(0);
        document["care"]["idle"]["rest_after_ms"] = json!(0);
    });
    let mut session = Session::dog_with(content, [10.0, 90.0, 90.0], 9);
    session.send(&water(true), START + 1_000);
    session.send(&water(false), START + 2_000);
    let seen = session.run(START, START + 5 * 60_000);
    assert!(
        seen.iter().all(|seen| seen.plays.is_empty()),
        "no rows, no animation"
    );
    assert!(
        seen.iter().any(|seen| !seen.bubbles.is_empty()),
        "the bubble needs no rows"
    );
}

/// F809/F833: a clock that ran backwards (a restart or a replayed timestamp) shows no bubble and
/// never errors or saves the tick, even when the bubble is due at that earlier time.
#[test]
fn a_backwards_clock_shows_no_bubble_and_never_errors() {
    let content = content_with(|document| {
        document["care"]["idle"]["bubble_min_ms"] = json!(1_000);
        document["care"]["idle"]["bubble_max_ms"] = json!(1_000);
    });
    let mut session = Session::dog_with(content, [10.0, 90.0, 90.0], 13);
    // The pet's needs clock is `START` (its summon). Tick a full bubble gap behind it: the first
    // tick schedules the bubble for `earlier + 1 s`, and the second one is due then, so only the
    // backwards guard in `decayed` can keep the hungry pet quiet. A zero-elapsed regression would
    // show the hunger icon here.
    let earlier = START - 10_000;
    let schedule = session
        .service
        .care_tick(&mut session.fixture.inventory, earlier);
    assert!(bubbles(&schedule.private, session.actor).is_empty());
    let care = session
        .service
        .care_tick(&mut session.fixture.inventory, earlier + 1_000);
    assert!(!care.changed, "a backwards clock saves nothing");
    assert!(bubbles(&care.private, session.actor).is_empty());
    assert!(
        care.private.is_empty() && care.shared.is_empty(),
        "no bubble and no live-needs status: {:?}",
        care.private
    );
}

#[test]
fn nothing_is_saved_and_a_dismiss_or_a_new_summon_starts_over() {
    let mut session = Session::dog_with(PetContent::bundled().unwrap(), [10.0, 90.0, 90.0], 10);
    let roster = session.service.document_value();
    let seen = session.run(START, START + 5 * 60_000);
    assert!(seen.iter().any(|seen| !seen.plays.is_empty()));
    assert!(seen.iter().any(|seen| !seen.bubbles.is_empty()));
    assert!(
        seen.iter()
            .any(|seen| seen.plays.iter().any(|play| play.base)),
        "it lay down"
    );
    assert_eq!(
        session.service.document_value(),
        roster,
        "session-only visuals"
    );

    // Dismissed while it lies down, then summoned again: the old timers do not run.
    let at = START + 5 * 60_000 + TICK;
    session.send(&DISMISS, at);
    assert!(
        session.run(at + TICK, at + 5 * 60_000).is_empty(),
        "no pet, no idle"
    );
    session.send(&summon(session.pet), at + 10 * 60_000);
    let again = session.run(at + 10 * 60_000 + TICK, at + 10 * 60_000 + 80_000);
    let first = &again[0];
    assert!(
        first.at
            >= at + 10 * 60_000 + SETTLE_DELAY + rule("idle_after_ms").min(rule("bubble_min_ms")),
        "a new summon waits again: {:?}",
        again.iter().map(|seen| seen.at).collect::<Vec<_>>()
    );
    assert!(
        again
            .iter()
            .all(|seen| !seen.plays.iter().any(|play| play.base)),
        "it has not lain down again yet"
    );
}

#[test]
fn bad_idle_content_is_rejected_at_load() {
    assert!(loads(|_| ()).is_ok(), "the shipped content loads");
    for (key, bad) in [
        ("idle_min_ms", json!(999)),
        ("idle_min_ms", json!(0)),
        ("idle_max_ms", json!(3_600_001)),
        ("idle_after_ms", json!(3_600_001)),
        ("rest_after_ms", json!(3_600_001)),
        ("bathroom_chance", json!(1.5)),
        ("bathroom_chance", json!(-0.1)),
        ("bubble_min_ms", json!(999)),
        ("bubble_max_ms", json!(3_600_001)),
        ("bubble_seconds", json!(0)),
        ("bubble_seconds", json!(61)),
        ("need_bubble_threshold", json!(100.5)),
        ("need_bubble_threshold", json!(-1)),
        ("mood_bubble_threshold", json!(101)),
        ("hunger_icon", json!(0)),
        ("hygiene_icon", json!(-3)),
        ("play_icon", json!(0)),
        ("sad_icon", json!(0)),
        ("unknown_key", json!(1)),
    ] {
        assert!(
            loads(|document| document["care"]["idle"][key] = bad.clone()).is_err(),
            "{key} = {bad}"
        );
    }
    // The range must be ordered.
    assert!(
        loads(|document| {
            document["care"]["idle"]["idle_min_ms"] = json!(40_000);
            document["care"]["idle"]["idle_max_ms"] = json!(39_999);
        })
        .is_err()
    );
    assert!(
        loads(|document| {
            document["care"]["idle"]["bubble_min_ms"] = json!(60_000);
            document["care"]["idle"]["bubble_max_ms"] = json!(59_999);
        })
        .is_err()
    );
    // The model rows.
    let edits: [(&str, &dyn Fn(&mut Value)); 11] = [
        ("an animation id of 0", &|idle| {
            idle["animations"][0]["animation_id"] = json!(0)
        }),
        ("a run time of 0", &|idle| {
            idle["animations"][0]["duration_ms"] = json!(0)
        }),
        ("a run time over a minute", &|idle| {
            idle["bathroom"][0]["duration_ms"] = json!(60_001)
        }),
        ("a posture with no loops", &|idle| {
            idle["rest_postures"][0]["loops"] = json!([])
        }),
        ("a loop id of 0", &|idle| {
            idle["rest_postures"][0]["loops"] = json!([0])
        }),
        ("a posture enter of 0 ms", &|idle| {
            idle["rest_postures"][0]["enter"]["duration_ms"] = json!(0)
        }),
        ("a fidget id of -1", &|idle| {
            idle["rest_postures"][0]["fidgets"][0]["animation_id"] = json!(-1)
        }),
        ("a shake-off id of 0", &|idle| {
            idle["shake_off"]["animation_id"] = json!(0)
        }),
        ("no source", &|idle| idle["src"] = json!("")),
        ("65 animations", &|idle| {
            let row = idle["animations"][0].clone();
            idle["animations"] = json!(vec![row; 65]);
        }),
        ("an unknown key", &|idle| idle["extra"] = json!(1)),
    ];
    for (why, edit) in edits {
        assert!(
            loads(|document| edit(&mut model_row(document, 364)["idle"])).is_err(),
            "{why}"
        );
    }
}

/// The real route: `Gameplay::pet_tick` runs the follow step and the idle life on the world tick,
/// and the owner's replies and the viewers' `nearby` packets carry the same visuals.
#[test]
fn a_pet_idles_and_shows_bubbles_through_gameplay_for_the_owner_and_the_viewers() {
    let fixture = Fixture::minimal(CHARACTER);
    let content = PetContent::bundled().unwrap();
    let (service, pet) = dog();
    let service = with_needs(service, content, pet, [10.0, 90.0, 90.0]);
    let mut who = CharacterState {
        guid: CHARACTER,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let mut game = Gameplay::load(fixture.path(), None).unwrap();
    game.adopt_pets(service.document_value()).unwrap();
    game.initialize_world(&who, START).unwrap();
    game.handle_in_world(&summon(pet), &who, START).unwrap();
    let actor = PetService::pet_guid(who.guid);
    let (idle, bathroom, _) = rows(364);

    // Walking: no animation, even for ten minutes. (A hungry pet's bubble needs no stillness.)
    let mut bubbled = false;
    let mut at = START;
    while at < START + 10 * 60_000 {
        at += TICK;
        who.position[2] += 1.0;
        let out = game.pet_tick(&who, at).expect("a pet is out");
        assert!(
            plays(&out.replies, actor).is_empty(),
            "walking: {:?}",
            out.replies
        );
        assert!(
            plays(&out.nearby, actor).is_empty(),
            "walking: {:?}",
            out.nearby
        );
        bubbled |= !bubbles(&out.nearby, actor).is_empty();
    }
    // Standing still: the pet animates (before it lies down), for both audiences. `idle_seen`
    // accepts only the model's own idle/bathroom rows; T533's hunt animations are allowed beside
    // them but never count as the idle life.
    let mut idle_seen = false;
    for _ in 0..(45_000 / TICK) {
        at += TICK;
        let out = game.pet_tick(&who, at).expect("a pet is out");
        // T533: a settled dog also hunts (sniff 5520, dig 5525), and the one save an idle tick may
        // make is the find of its chest (model 151, `sg_treasure_chest_01.adr`): any other saving
        // tick is a regression.
        let found = out.replies.iter().any(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && parse_add_npc(packet).is_ok_and(|npc| npc.model_id == 151)
        });
        assert!(
            !out.changed || found,
            "only a find may save on an idle tick"
        );
        assert_eq!(visual(&out.replies), visual(&out.nearby));
        for play in plays(&out.nearby, actor) {
            if idle.contains(&play.animation) || bathroom.contains(&play.animation) {
                assert!(!play.base, "an idle animation is a one-shot");
                idle_seen = true;
            } else {
                assert!(
                    [5520, 5525].contains(&play.animation),
                    "unexpected animation {}",
                    play.animation
                );
            }
        }
        for bubble in bubbles(&out.nearby, actor) {
            assert_eq!(bubble.icon, icon("hunger_icon"));
            bubbled = true;
        }
    }
    assert!(idle_seen && bubbled, "idle {idle_seen}, bubbled {bubbled}");
}
