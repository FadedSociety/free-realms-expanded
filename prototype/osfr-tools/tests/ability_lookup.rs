//! W141: 36/12 definition lookup and explicit refusal of unsupported 36/10 casts.
//! Wire shapes: native writers `00a34410` (36/12) and `00a342e0` (36/10), reader
//! `00a32930` (36/13). Everything about which casts are served is (R).
use osfr_tools::{
    abilities::{
        definition_reply, parse_definition_request, unsupported_cast, unsupported_cast_failure,
    },
    opening_abilities::OpeningAbilities,
};
use std::path::Path;

const COMBAT_PROFILES: [i32; 6] = [2, 11, 12, 32, 35, 43];

fn roster() -> OpeningAbilities {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let mut abilities = OpeningAbilities::load(&content).unwrap();
    abilities
        .load_job_abilities(&content.join("job_abilities.json"))
        .unwrap();
    abilities
}

fn definitions(bar: &[Vec<u8>]) -> Vec<Vec<u8>> {
    bar.iter()
        .filter(|p| p.starts_with(&[36, 0, 13, 0]))
        .cloned()
        .collect()
}

fn id_of(definition: &[u8]) -> i32 {
    i32::from_le_bytes(definition[4..8].try_into().unwrap())
}

fn request(id: i32) -> Vec<u8> {
    [vec![36, 0, 12, 0], id.to_le_bytes().to_vec()].concat()
}

fn cast(bar: i32, slot: i32, mode: i32, tail: &[u8]) -> Vec<u8> {
    let mut bytes = vec![36, 0, 10, 0];
    for value in [bar, slot, mode] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(tail);
    bytes
}

fn failure_286() -> Vec<u8> {
    [vec![36, 0, 1, 0], 286_i32.to_le_bytes().to_vec()].concat()
}

#[test]
fn request_parser_accepts_only_the_exact_native_shape() {
    for id in [0, 1, 4899, -1, i32::MIN, i32::MAX] {
        assert_eq!(parse_definition_request(&request(id)).unwrap(), id);
    }
    let good = request(4899);
    for bad in [
        vec![],
        good[..4].to_vec(),
        good[..7].to_vec(),
        [good.clone(), vec![0]].concat(),
        [good.clone(), vec![0; 4]].concat(),
        [vec![36, 0, 13, 0], good[4..].to_vec()].concat(),
        [vec![35, 0, 12, 0], good[4..].to_vec()].concat(),
        [vec![36, 1, 12, 0], good[4..].to_vec()].concat(),
    ] {
        assert!(parse_definition_request(&bad).is_err(), "{bad:?}");
    }
}

#[test]
fn every_toolbar_definition_resolves_byte_identical() {
    let abilities = roster();
    for profile in COMBAT_PROFILES {
        let loaded = definitions(&abilities.activation(profile, 20).unwrap());
        assert!(
            loaded.len() >= 4,
            "profile {profile} has a basic and skills"
        );
        for definition in &loaded {
            let id = parse_definition_request(&request(id_of(definition))).unwrap();
            assert_eq!(
                definition_reply(&loaded, id).as_deref(),
                Some(definition.as_slice()),
                "profile {profile} id {id}"
            );
        }
    }
}

#[test]
fn definitions_above_the_player_level_or_of_another_job_are_not_served() {
    let abilities = roster();
    // Medic (11) Vitamins-class skill 62002 unlocks at level 10; Warrior-class 62004 is another job.
    let level_1 = definitions(&abilities.activation(11, 1).unwrap());
    let level_20 = definitions(&abilities.activation(11, 20).unwrap());
    assert!(level_1.iter().all(|d| id_of(d) != 62002));
    assert_eq!(definition_reply(&level_1, 62002), None, "level-locked");
    assert!(definition_reply(&level_20, 62002).is_some());
    assert_eq!(definition_reply(&level_20, 62004), None, "other job's id");
    // Non-combat profiles have no definitions at all.
    let none = definitions(&abilities.activation(9, 20).unwrap());
    assert!(none.is_empty());
    assert_eq!(definition_reply(&none, 4895), None);
}

#[test]
fn missing_ids_are_bounded_and_answer_nothing() {
    let abilities = roster();
    let loaded = definitions(&abilities.activation(11, 20).unwrap());
    for id in [
        0,
        -1,
        i32::MIN,
        i32::MAX,
        4894,
        4896,
        4900,
        62000,
        1_000_000,
    ] {
        assert_eq!(definition_reply(&loaded, id), None, "id {id}");
    }
    // Whatever is asked, at most one 160-byte 36/13 can come back.
    for id in -50..70_000 {
        if let Some(reply) = definition_reply(&loaded, id) {
            assert_eq!(reply.len(), 160);
            assert_eq!(id_of(&reply), id);
        }
    }
    // The input list is never trusted to be well-formed: wrong length or header is skipped.
    let mut poisoned = vec![
        vec![36, 0, 13, 0, 7, 0, 0, 0],
        [vec![36, 0, 12, 0, 7, 0, 0, 0], vec![0; 152]].concat(),
        vec![],
    ];
    assert_eq!(definition_reply(&poisoned, 7), None);
    poisoned.extend(loaded.iter().cloned());
    assert!(definition_reply(&poisoned, 4895).is_some());
}

#[test]
fn unsupported_casts_get_a_valid_failure_packet() {
    let target = 0x1234_5678_9abc_def0_u64.to_le_bytes();
    let ground = [0; 16];
    let refused = [
        cast(1, 0, 1, &ground),        // ground mode on the job bar
        cast(1, 3, 1, &ground),        // ground ability slot
        cast(1, 0, 2, &ground),        // unknown non-zero mode, four-dword body
        cast(0, 0, 0, &target),        // bar 0
        cast(2, 0, 0, &target),        // another bar (item bar is a separate build)
        cast(-1, 0, 0, &target),       // negative bar
        cast(1, 8, 0, &target),        // slot past the 8-slot toolbar
        cast(1, -1, 0, &target),       // pulse sentinel never arrives as 36/10
        cast(1, i32::MAX, 0, &target), // out of range
        cast(i32::MIN, i32::MIN, 0, &target),
    ];
    for request in refused {
        assert_eq!(
            unsupported_cast(&request).unwrap(),
            Some(failure_286()),
            "{request:?}"
        );
    }
    assert_eq!(unsupported_cast_failure(), failure_286());
}

#[test]
fn served_casts_are_left_to_the_existing_routes() {
    let target = 7_u64.to_le_bytes();
    for slot in 0..=7 {
        assert_eq!(unsupported_cast(&cast(1, slot, 0, &target)).unwrap(), None);
    }
    // Other families and subtypes are not this function's business.
    for other in [
        vec![],
        vec![36, 0, 12, 0, 1, 0, 0, 0],
        vec![36, 0, 11, 0],
        vec![26, 0, 10, 0, 1, 0, 0, 0],
    ] {
        assert_eq!(unsupported_cast(&other).unwrap(), None);
    }
}

#[test]
fn malformed_cast_shapes_error_instead_of_guessing() {
    let target = 7_u64.to_le_bytes();
    let ground = [0; 16];
    for bad in [
        vec![36, 0, 10, 0],
        cast(1, 0, 0, &[]),
        cast(1, 0, 0, &target[..7]),
        cast(1, 0, 0, &[&target[..], &[0]].concat()),
        cast(1, 0, 0, &ground), // mode 0 must carry a u64, not four dwords
        cast(1, 0, 1, &target), // ground mode must carry four dwords
        cast(1, 0, 1, &[&ground[..], &[0]].concat()),
        cast(1, 0, 1, &ground[..15]),
        cast(1, 0, i32::MIN, &target),
    ] {
        assert!(unsupported_cast(&bad).is_err(), "{bad:?}");
    }
    assert!(unsupported_cast(&[36, 0, 10, 0, 1]).is_err());
}

#[test]
fn classifier_never_panics_on_arbitrary_short_input() {
    // Canary against indexing slips: every prefix of every shape, with edge-valued words.
    let words = [0, 1, 2, 7, 8, -1, i32::MIN, i32::MAX];
    for bar in words {
        for slot in words {
            for mode in words {
                let full = cast(bar, slot, mode, &[0xff; 16]);
                for end in 0..=full.len() {
                    let _ = unsupported_cast(&full[..end]);
                }
            }
        }
    }
}
