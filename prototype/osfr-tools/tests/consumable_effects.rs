//! T473: the consumable effect stack. Packet layouts are pinned to the client readers
//! (35/13 `008d71b0`, 35/14 `008d6890`, 35/15 `008d7230`); the 35/41, 35/42, 38/16 and 38/17
//! packets are decoded with the piece_056 decoders, compiled with `#[path]` like its own test.

#[path = "../src/restoration/piece_056.rs"]
#[allow(dead_code)]
mod piece_056;

use osfr_tools::consumable_effects::{
    Boost, Boosted, EffectStack, Emit, Kind, MAX_DELAY_MS, MAX_DURATION_MS, Refusal, Spec,
    encode_remove_temporary_appearance, encode_temporary_appearance, encode_update_scale,
};
use piece_056::{
    decode_add_composite_effect, decode_add_effect_tag, decode_remove_composite_effect,
    decode_remove_effect_tag,
};

const GUID: u64 = 0x5255_5354_0000_0007;
const POS: [f32; 3] = [1.0, 2.0, 3.0];
const MIN: u64 = 60_000;

fn transform(duration_ms: u64) -> Spec {
    Spec {
        kind: Kind::Transform {
            model_id: 4321,
            poof_effect_id: 77,
        },
        duration_ms,
        icon_id: 3843,
        name_id: 100,
    }
}

fn aura(duration_ms: u64, delay_ms: u64) -> Spec {
    Spec {
        kind: Kind::Aura {
            composite_effect_id: 555,
            delay_ms,
        },
        duration_ms,
        icon_id: 3844,
        name_id: 101,
    }
}

fn scale(scale: f32, duration_ms: u64) -> Spec {
    Spec {
        kind: Kind::Scale { scale },
        duration_ms,
        icon_id: 3845,
        name_id: 102,
    }
}

/// L087: a booster takes a buff row and raises rewards until it ends; a second one of its kind
/// replaces it, and the other kind runs beside it.
#[test]
fn a_booster_raises_rewards_until_it_ends() {
    let booster = |raises, percent| Spec {
        kind: Kind::Boost { raises, percent },
        duration_ms: 30 * MIN,
        icon_id: 1157,
        name_id: 2440,
    };
    let mut stack = EffectStack::default();
    let (_, emit) = stack
        .apply(GUID, POS, booster(Boosted::Stars, 50), 0)
        .unwrap();
    assert_eq!(heads(&emit.to_self), [(38, 16)]);
    assert!(emit.to_nearby.is_empty(), "a booster changes no look");
    stack
        .apply(GUID, POS, booster(Boosted::Coins, 25), 0)
        .unwrap();
    assert_eq!(
        stack.boost(),
        Boost {
            stars: 50,
            coins: 25
        }
    );
    assert_eq!(
        (stack.boost().coins(10), stack.boost().stars(15)),
        (13, 23),
        "half rounds up"
    );
    let (_, emit) = stack
        .apply(GUID, POS, booster(Boosted::Stars, 25), MIN)
        .unwrap();
    assert_eq!(
        heads(&emit.to_self),
        [(38, 17), (38, 16)],
        "the new one replaces the old"
    );
    assert_eq!(
        stack.boost(),
        Boost {
            stars: 25,
            coins: 25
        }
    );
    assert_eq!(
        heads(&stack.tick(GUID, 30 * MIN).to_self),
        [(38, 17)],
        "the coin booster ends on time"
    );
    assert_eq!(
        stack.boost(),
        Boost {
            stars: 25,
            coins: 0
        }
    );
    assert_eq!(
        stack.admits(&booster(Boosted::Coins, 0)),
        Err(Refusal::Percent)
    );
}

/// (family, sub) of each packet.
fn heads(packets: &[Vec<u8>]) -> Vec<(u16, u16)> {
    packets
        .iter()
        .map(|p| {
            (
                u16::from_le_bytes([p[0], p[1]]),
                u16::from_le_bytes([p[2], p[3]]),
            )
        })
        .collect()
}

fn tag_of_41(packet: &[u8]) -> i32 {
    decode_add_composite_effect(packet).unwrap().tag_id
}

fn tag_of_42(packet: &[u8]) -> i32 {
    decode_remove_composite_effect(packet).unwrap().1
}

#[test]
fn layouts_follow_the_client_readers() {
    // 35/13 reader 008d71b0: u64 guid, f32 scale.
    let p = encode_update_scale(GUID, 2.5);
    assert_eq!(p.len(), 16);
    assert_eq!(heads(&[p.clone()]), [(35, 13)]);
    assert_eq!(u64::from_le_bytes(p[4..12].try_into().unwrap()), GUID);
    assert_eq!(f32::from_le_bytes(p[12..16].try_into().unwrap()), 2.5);
    // 35/14 reader 008d6890: i32 model first, then u64 guid.
    let p = encode_temporary_appearance(4321, GUID);
    assert_eq!(p.len(), 16);
    assert_eq!(heads(&[p.clone()]), [(35, 14)]);
    assert_eq!(i32::from_le_bytes(p[4..8].try_into().unwrap()), 4321);
    assert_eq!(u64::from_le_bytes(p[8..16].try_into().unwrap()), GUID);
    // 35/15 reader 008d7230: u64 guid, i32 0.
    let p = encode_remove_temporary_appearance(GUID);
    assert_eq!(p.len(), 16);
    assert_eq!(heads(&[p.clone()]), [(35, 15)]);
    assert_eq!(u64::from_le_bytes(p[4..12].try_into().unwrap()), GUID);
    assert_eq!(i32::from_le_bytes(p[12..16].try_into().unwrap()), 0);
}

#[test]
fn transform_apply_sends_poof_model_then_row() {
    let mut stack = EffectStack::default();
    let (id, emit) = stack.apply(GUID, POS, transform(30 * MIN), 1_000).unwrap();
    assert_eq!(heads(&emit.to_nearby), [(35, 16), (35, 14)]);
    assert_eq!(heads(&emit.to_self), [(35, 16), (35, 14), (38, 16)]);
    let (tag, row) = decode_add_effect_tag(&emit.to_self[2]).unwrap();
    assert_eq!((tag, row.unknown_1, row.unknown_20), (id, id, 3843));
    assert_eq!(row.unknown_7, 1800); // seconds
    assert_eq!(stack.visual().temporary_model, 4321);
}

#[test]
fn a_new_transform_replaces_the_old_without_a_revert() {
    let mut stack = EffectStack::default();
    let (first, _) = stack.apply(GUID, POS, transform(MIN), 0).unwrap();
    let (second, emit) = stack.apply(GUID, POS, transform(MIN), 10).unwrap();
    assert_ne!(first, second);
    assert_eq!(
        heads(&emit.to_self),
        [(38, 17), (35, 16), (35, 14), (38, 16)]
    );
    assert_eq!(decode_remove_effect_tag(&emit.to_self[0]).unwrap(), first);
    assert_eq!(stack.strip(GUID, first), None);
}

#[test]
fn transform_expiry_resyncs_the_aura_and_keeps_its_buff_id() {
    let mut stack = EffectStack::default();
    let (aura_id, _) = stack.apply(GUID, POS, aura(10 * MIN, 0), 0).unwrap();
    let (transform_id, _) = stack.apply(GUID, POS, transform(MIN), 0).unwrap();
    // The 35/14 above resynced the aura once already; remember its tag now.
    let before = stack.visual().auras[0].0;
    assert!(stack.tick(GUID, MIN - 1).is_empty());
    let emit = stack.tick(GUID, MIN);
    assert_eq!(
        heads(&emit.to_self),
        [(35, 15), (38, 17), (35, 42), (35, 41)]
    );
    assert_eq!(heads(&emit.to_nearby), [(35, 15), (35, 42), (35, 41)]);
    assert_eq!(
        decode_remove_effect_tag(&emit.to_self[1]).unwrap(),
        transform_id
    );
    assert_eq!(tag_of_42(&emit.to_self[2]), before);
    let fresh = tag_of_41(&emit.to_self[3]);
    assert_ne!(fresh, before);
    assert_eq!(stack.visual().auras, [(fresh, 555)]);
    // The aura still owns its buff id: stripping it names that id.
    assert!(stack.strip(GUID, aura_id).is_some());
    // Exactly once: nothing more for the expired transform.
    assert!(stack.tick(GUID, MIN * 2).is_empty());
}

#[test]
fn aura_and_scale_expire_exactly_once() {
    let mut stack = EffectStack::default();
    let (aura_id, _) = stack.apply(GUID, POS, aura(MIN, 0), 0).unwrap();
    let (scale_id, _) = stack.apply(GUID, POS, scale(2.0, 2 * MIN), 0).unwrap();
    let emit = stack.tick(GUID, MIN);
    assert_eq!(heads(&emit.to_self), [(35, 42), (38, 17)]);
    assert_eq!(decode_remove_effect_tag(&emit.to_self[1]).unwrap(), aura_id);
    assert!(stack.tick(GUID, MIN + 1).is_empty());
    let emit = stack.tick(GUID, 2 * MIN);
    assert_eq!(heads(&emit.to_self), [(35, 13), (38, 17)]);
    assert_eq!(heads(&emit.to_nearby), [(35, 13)]);
    assert_eq!(emit.to_self[0], encode_update_scale(GUID, 1.0));
    assert_eq!(
        decode_remove_effect_tag(&emit.to_self[1]).unwrap(),
        scale_id
    );
    assert!(stack.tick(GUID, 3 * MIN).is_empty());
}

#[test]
fn strip_ends_only_what_the_stack_owns() {
    let mut stack = EffectStack::default();
    let (id, _) = stack.apply(GUID, POS, scale(2.0, MIN), 0).unwrap();
    let before = stack.clone();
    for foreign in [0, 2, 4, 15255, id + 1, i32::MAX] {
        assert_eq!(stack.strip(GUID, foreign), None);
        assert_eq!(stack, before);
    }
    let emit = stack.strip(GUID, id).unwrap();
    assert_eq!(heads(&emit.to_self), [(35, 13), (38, 17)]);
    assert_eq!(stack.strip(GUID, id), None);
    assert!(stack.tick(GUID, 10 * MIN).is_empty());
}

#[test]
fn stripping_a_transform_reverts_and_resyncs_like_expiry() {
    let mut stack = EffectStack::default();
    stack.apply(GUID, POS, aura(10 * MIN, 0), 0).unwrap();
    let (id, _) = stack.apply(GUID, POS, transform(MIN), 0).unwrap();
    let emit = stack.strip(GUID, id).unwrap();
    assert_eq!(
        heads(&emit.to_self),
        [(35, 15), (38, 17), (35, 42), (35, 41)]
    );
}

#[test]
fn ids_stay_in_their_ranges_and_never_collide() {
    let mut stack = EffectStack::default();
    let mut seen = Vec::new();
    for round in 0..50_u64 {
        for spec in [transform(MIN), aura(MIN, 0), scale(1.5, MIN)] {
            let (id, _) = stack.apply(GUID, POS, spec, round).unwrap();
            assert!(id >= 1_000_000 && id != 2 && id != 4, "buff id {id}");
            assert!(!seen.contains(&id), "reused {id}");
            seen.push(id);
        }
        for (tag, _) in stack.visual().auras {
            assert!(tag >= 1_000_000 && tag != 2 && tag != 4, "world tag {tag}");
        }
    }
}

#[test]
fn a_delayed_aura_starts_once_after_its_delay() {
    let mut stack = EffectStack::default();
    let (_, emit) = stack.apply(GUID, POS, aura(MIN, 5_000), 100).unwrap();
    assert_eq!(heads(&emit.to_self), [(38, 16)]);
    assert!(emit.to_nearby.is_empty());
    assert!(stack.visual().auras.is_empty());
    assert!(stack.tick(GUID, 5_099).is_empty());
    let emit = stack.tick(GUID, 5_100);
    assert_eq!(heads(&emit.to_self), [(35, 41)]);
    assert_eq!(heads(&emit.to_nearby), [(35, 41)]);
    assert!(stack.tick(GUID, 5_200).is_empty());
    // A clock that went backwards fires nothing and does not panic.
    assert!(stack.tick(GUID, 0).is_empty());
    assert_eq!(stack.visual().auras.len(), 1);
}

#[test]
fn an_aura_that_expires_before_it_starts_sends_only_its_row_removal() {
    let mut stack = EffectStack::default();
    stack.apply(GUID, POS, aura(1_000, 5_000), 0).unwrap();
    let emit = stack.tick(GUID, 6_000);
    assert_eq!(heads(&emit.to_self), [(38, 17)]);
    assert!(emit.to_nearby.is_empty());
}

#[test]
fn the_clock_going_backwards_expires_nothing() {
    let mut stack = EffectStack::default();
    stack.apply(GUID, POS, scale(2.0, MIN), 1_000_000).unwrap();
    let before = stack.clone();
    assert!(stack.tick(GUID, 0).is_empty());
    assert_eq!(stack, before);
    // Deadlines that overflow saturate instead of panicking.
    let mut stack = EffectStack::default();
    stack
        .apply(GUID, POS, transform(MIN), u64::MAX - 1)
        .unwrap();
    assert!(stack.tick(GUID, u64::MAX - 1).is_empty());
}

#[test]
fn refusals_change_nothing() {
    let mut stack = EffectStack::default();
    stack.apply(GUID, POS, aura(MIN, 0), 0).unwrap();
    stack.apply(GUID, POS, scale(2.0, MIN), 0).unwrap();
    let before = stack.clone();
    let bad = [
        (scale(0.0, MIN), Refusal::Scale),
        (scale(f32::NAN, MIN), Refusal::Scale),
        (scale(f32::INFINITY, MIN), Refusal::Scale),
        (scale(5.5, MIN), Refusal::Scale),
        (transform(0), Refusal::Duration),
        (transform(MAX_DURATION_MS + 1), Refusal::Duration),
        (aura(MIN, MAX_DELAY_MS + 1), Refusal::Delay),
        (aura(MIN, u64::MAX), Refusal::Delay),
        (
            Spec {
                kind: Kind::Transform {
                    model_id: 0,
                    poof_effect_id: 0,
                },
                ..transform(MIN)
            },
            Refusal::Id,
        ),
        (
            Spec {
                kind: Kind::Aura {
                    composite_effect_id: -1,
                    delay_ms: 0,
                },
                ..aura(MIN, 0)
            },
            Refusal::Id,
        ),
    ];
    for (spec, why) in bad {
        assert_eq!(stack.admits(&spec), Err(why));
        assert_eq!(stack.apply(GUID, POS, spec, 5).unwrap_err(), why);
        assert_eq!(stack, before);
    }
    // The edges of the bounds are admitted.
    for ok in [
        scale(0.25, 1),
        scale(5.0, MAX_DURATION_MS),
        aura(1, MAX_DELAY_MS),
    ] {
        assert_eq!(stack.admits(&ok), Ok(()));
    }
}

#[test]
fn an_idle_stack_ticks_to_nothing() {
    let mut stack = EffectStack::default();
    assert_eq!(stack.tick(GUID, 123), Emit::default());
    assert_eq!(stack, EffectStack::default());
}

#[test]
fn resend_shows_the_live_state_with_remaining_time() {
    let mut stack = EffectStack::default();
    let (aura_id, _) = stack.apply(GUID, POS, aura(10 * MIN, 0), 0).unwrap();
    stack.apply(GUID, POS, transform(MIN), 0).unwrap();
    stack.apply(GUID, POS, scale(2.0, MIN), 0).unwrap();
    let packets = stack.resend_to_self(GUID, 30_000);
    assert_eq!(
        heads(&packets),
        [(35, 14), (35, 13), (35, 41), (38, 16), (38, 16), (38, 16)]
    );
    assert_eq!(tag_of_41(&packets[2]), stack.visual().auras[0].0);
    let rows: Vec<_> = packets[3..]
        .iter()
        .map(|p| decode_add_effect_tag(p).unwrap())
        .collect();
    let (_, aura_row) = rows.iter().find(|(tag, _)| *tag == aura_id).unwrap();
    assert_eq!(aura_row.unknown_7, 570); // 10 minutes minus 30 s
    assert!(EffectStack::default().resend_to_self(GUID, 0).is_empty());
}

/// L113: a `Boost` built with a percent the stack would refuse is computed exactly, or clamps at
/// `u32::MAX`, instead of overflowing (`100 + percent` used to panic in a debug build).
#[test]
fn a_huge_boost_percent_clamps_instead_of_overflowing() {
    let boost = Boost {
        stars: u32::MAX,
        coins: u32::MAX,
    };
    assert_eq!(
        boost.stars(10),
        429_496_740,
        "10 x (u32::MAX + 100) / 100, rounded"
    );
    assert_eq!(boost.coins(u32::MAX), u32::MAX);
    assert_eq!(boost.coins(0), 0);
}

/// B1: a burst takes no slot and no buff row: four run at once, a fifth ends the oldest early,
/// it plays on the named actor with the user as source, and each ends on time once.
#[test]
fn b1_bursts_run_four_at_once_and_each_ends_once() {
    let burst = |target| Spec {
        kind: Kind::Burst {
            composite_effect_id: 16517,
            target,
        },
        duration_ms: 10_000,
        icon_id: 0,
        name_id: 9042,
    };
    let mut stack = EffectStack::default();
    let mut tags = Vec::new();
    for n in 0..4 {
        let (tag, emit) = stack.apply(GUID, POS, burst(0), n).unwrap();
        assert_eq!(emit.to_self, emit.to_nearby, "no buff row, both lists see it");
        let added = decode_add_composite_effect(&emit.to_self[0]).unwrap();
        assert_eq!((added.guid, added.tag_id, added.source_guid), (GUID, tag, GUID));
        tags.push(tag);
    }
    let (fifth, emit) = stack.apply(GUID, POS, burst(7), 4).unwrap();
    assert_eq!(emit.to_nearby.len(), 2);
    assert_eq!(
        decode_remove_composite_effect(&emit.to_nearby[0]).unwrap(),
        (GUID, tags[0]),
        "the oldest gives way"
    );
    let added = decode_add_composite_effect(&emit.to_nearby[1]).unwrap();
    assert_eq!((added.guid, added.tag_id, added.composite_effect_id), (7, fifth, 16517));
    assert_eq!(added.source_guid, GUID, "the p2p beam draws from the user");
    assert!(stack.visual().auras.is_empty(), "a burst is not an aura");

    assert!(stack.tick(GUID, 10_000).is_empty(), "nothing is due before its 10 s");
    let second = stack.tick(GUID, 10_001);
    assert_eq!(
        decode_remove_composite_effect(&second.to_nearby[0]).unwrap(),
        (GUID, tags[1])
    );
    let ended = stack.tick(GUID, 10_004);
    let gone: Vec<_> = ended
        .to_nearby
        .iter()
        .map(|p| decode_remove_composite_effect(p).unwrap())
        .collect();
    assert_eq!(gone, [(GUID, tags[2]), (GUID, tags[3]), (7, fifth)]);
    assert!(stack.tick(GUID, 60_000).is_empty());
    assert_eq!(
        stack.admits(&Spec {
            kind: Kind::Burst {
                composite_effect_id: 0,
                target: 0
            },
            ..burst(0)
        }),
        Err(Refusal::Id)
    );
}

/// B1: a minigame stat is the owner's 38/7 with a buff row; another stat buff replaces it and
/// clears the first stat; the expiry sets 0; the stat goes out again after a zone change.
#[test]
fn b1_a_stat_buff_sets_its_stat_and_clears_it() {
    let stat = |stat_id, value| Spec {
        kind: Kind::Stat { stat_id, value },
        duration_ms: 30 * MIN,
        icon_id: 1234,
        name_id: 5678,
    };
    let record = |packet: &[u8]| -> (u64, i32, i32, i32) {
        let word = |at: usize| i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
        assert!(packet.starts_with(&[38, 0, 7, 0]) && word(12) == 1, "one record");
        let guid = u64::from_le_bytes(packet[4..12].try_into().unwrap());
        (guid, word(16), word(20), word(24))
    };
    let mut stack = EffectStack::default();
    let (timing, emit) = stack.apply(GUID, POS, stat(27, 2), 0).unwrap();
    assert!(emit.to_nearby.is_empty(), "a stat is the owner's");
    assert_eq!(record(&emit.to_self[0]), (GUID, 27, 0, 2));
    assert_eq!(decode_add_effect_tag(&emit.to_self[1]).unwrap().0, timing);
    assert!(
        stack
            .resend_to_self(GUID, 1_000)
            .iter()
            .any(|p| p.starts_with(&[38, 0, 7, 0]) && record(p) == (GUID, 27, 0, 2))
    );

    let (talent, emit) = stack.apply(GUID, POS, stat(24, 2), 1_000).unwrap();
    assert_eq!(record(&emit.to_self[0]), (GUID, 27, 0, 0), "Timing cleared first");
    assert_eq!(decode_remove_effect_tag(&emit.to_self[1]).unwrap(), timing);
    assert_eq!(record(&emit.to_self[2]), (GUID, 24, 0, 2));

    let ended = stack.tick(GUID, 1_000 + 30 * MIN);
    assert_eq!(record(&ended.to_self[0]), (GUID, 24, 0, 0));
    assert_eq!(decode_remove_effect_tag(&ended.to_self[1]).unwrap(), talent);
    assert_eq!(stack.admits(&stat(2, 2)), Err(Refusal::Stat), "speed is not a minigame stat");
    assert_eq!(stack.admits(&stat(27, 0)), Err(Refusal::Stat));
}
