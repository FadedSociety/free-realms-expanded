//! T511: the shipped `data/content/consumables.json` through the real loaders.
//!
//! The file is generated offline by `scripts/build_consumables.py` from the pinned
//! Sanctuary/Legacy item definitions and the client asset tables. These checks pin the
//! loader contract (the production `ItemBar::load` path), the two potion rows, the two
//! named rows the brief calls out, the exclusions and every row's provenance.
use osfr_tools::consumable_effects::Boosted;
use osfr_tools::item_bar::{EffectKind, Effects, ItemBar};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const POTIONS: [i32; 2] = [2173, 11002];
/// Every definition the content rules exclude. An id with no client item row (the OSFR
/// synthetic abilities) is checked separately. C3: the Scaredy Cake 69819, Mystery Cake 48100 and
/// Jack-O-Lantern 69828 are placed props, the Slice of Mystery Cake 48104 a transform and the
/// silly-string cans bursts (`scripts/event_effects.py`): Sanctuary's own handlers for them were
/// never ported, so the item-behaviour audit found them doing nothing.
const EXCLUDED: [&[i32]; 2] = [
    // Cakes (the birthday cakes 7966..=7974 are placed_cake rows since T557, checked below)
    &[17597],
    // Boomboxes
    &[
        878, 1180, 1913, 8185, 35759, 48244, 56774, 69825, 76927, 76928, 76930, 76933, 76935,
        77329, 76934, 900059, 900060, 900061, 76877, 900063, 78075,
    ],
];

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(text.trim_start_matches('\u{feff}')).unwrap()
}

/// Every item id of the release item table and its supplement.
fn item_ids() -> BTreeSet<i32> {
    let release = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release");
    ["item-table.json", "item-table-supplement.json"]
        .iter()
        .flat_map(|name| json(&release.join(name))["rows"].as_array().unwrap().clone())
        .map(|row| i32::try_from(row[0].as_i64().unwrap()).unwrap())
        .collect()
}

#[test]
fn the_shipped_rows_load_through_the_real_loader_with_their_client_checked_values() {
    let directory = content();
    // The production entry point the game uses at bind; it calls `Effects::load` itself.
    ItemBar::load(&directory, Default::default(), None).expect("the shipped rows load");
    let effects = Effects::load(&directory).expect("consumables.json parses");
    let rows = effects.rows();
    assert!(
        (90..=512).contains(&rows.len()),
        "{} consumable rows, outside the content bounds",
        rows.len()
    );

    // Rule 1: the built-in potion rows come first and keep their values.
    assert_eq!(
        rows[..2]
            .iter()
            .map(|row| row.definition)
            .collect::<Vec<_>>(),
        POTIONS
    );
    for definition in POTIONS {
        let row = effects.get(definition).expect("a potion row");
        assert!(matches!(
            row.effect,
            EffectKind::Heal {
                percent: 35,
                instant_percent: 2,
                over_ms: 5_000
            }
        ));
        assert_eq!(row.cooldown_ms, 10_000);
        assert!(row.single_use);
        assert_eq!((row.name_id, row.description_id), (422909, 422916));
    }
    // Rule 6: every generated row is sorted by definition id.
    assert!(
        rows[2..]
            .windows(2)
            .all(|pair| pair[0].definition < pair[1].definition)
    );

    // 2982 Electric Veal Meat: the brief's aura row.
    let veal = effects.get(2982).expect("Electric Veal Meat");
    assert!(matches!(
        veal.effect,
        EffectKind::Aura {
            composite_effect_id: 5030,
            duration_ms: 1_800_000,
            delay_ms: 0,
            icon_id: 3843,
        }
    ));
    assert_eq!((veal.name_id, veal.description_id), (277, 2707));
    assert_eq!(veal.cooldown_ms, 1_800_000);
    assert!(veal.single_use);

    // 4042 Bark Berries: the brief's transformation treat.
    let berries = effects.get(4042).expect("Bark Berries");
    assert!(matches!(
        berries.effect,
        EffectKind::Transform {
            model_id: 169,
            poof_effect_id: 21,
            duration_ms: 1_800_000,
            icon_id: 3843,
        }
    ));
    assert_eq!((berries.name_id, berries.description_id), (1201, 2772));
    assert_eq!(berries.cooldown_ms, 1_800_000);

    // The retained joins: 23 food items and 69 transformation items after the exclusions, and
    // the nine birthday cakes (T557).
    let (mut heals, mut auras, mut transforms, mut cakes, mut boosts) = (0, 0, 0, 0, 0);
    let (mut strikes, mut scales, mut bursts, mut stats, mut restores) = (0, 0, 0, 0, 0);
    for row in rows {
        match row.effect {
            EffectKind::Heal { .. } => heals += 1,
            EffectKind::Aura { .. } => auras += 1,
            EffectKind::Transform { .. } => transforms += 1,
            EffectKind::Scale { .. } => scales += 1,
            EffectKind::PlacedCake { .. } => cakes += 1,
            EffectKind::Boost { .. } => boosts += 1,
            EffectKind::Strike { .. } => strikes += 1,
            EffectKind::Burst { .. } => bursts += 1,
            EffectKind::Stat { .. } => stats += 1,
            EffectKind::Restore { .. } => restores += 1,
        }
    }
    // B1 (`scripts/event_effects.py`): the event, code and free-item consumables add 4 heals,
    // 11 auras, 23 transforms (and fix 3), 2 scales, 16 bursts, 2 stat buffs, 2 coin boosters
    // (the Prospector's Scrolls) and 2 placed props (Turkleton Treats, Glerga's Birthday Cake).
    // C3 (the item-behaviour audit's 207, then the checker's round): 29 restores (B1's two
    // Medium Replenishment heals among them), 29 auras, 23 transforms (and B1's three pending
    // masks and jelly beans), 4 scales, 28 bursts, 59 stat buffs, 5 boosters, 3 strikes and 14
    // placed props (and B1's Toxic Barrel).
    assert_eq!(heals, 4);
    assert_eq!(restores, 29);
    assert_eq!(auras, 63);
    assert_eq!(
        transforms, 121,
        "L092: and the Scare Orb, Scare Sphere and Spook Sphere"
    );
    assert_eq!((scales, bursts, stats), (6, 44, 61));
    let scare = effects.get(306).expect("Scare Orb");
    assert_eq!(scare.scare.map(|s| s.flee_ms), Some(4_000));
    assert!(effects.get(896).expect("Spook Sphere").scare.is_none());
    assert_eq!(cakes, 26);
    // L087: three +50% star boosters (ability 203), the Wheeler star and coin scrolls (wiki).
    // P-games-window: and the Super Coin Booster the minigame offer sells.
    assert_eq!(boosts, 13);
    // L088: the Frag and Flabbergast orbs and spheres; L089: Sleep, Unmoving and Confusion (wiki, client text).
    assert_eq!(
        strikes, 21,
        "L091: and the Blast Orb and Spheres; C3: Cocoon Orb, Frost Grenade, Flute of Slumber"
    );
    let blast = effects.get(3072).expect("Blast Orb");
    assert!(matches!(blast.effect, EffectKind::Strike { knockback, .. } if knockback == 5.5));
    let sleep = effects.get(11011).expect("Sleep Orb");
    assert!(matches!(
        sleep.effect,
        EffectKind::Strike {
            sleep_ms: 10_000,
            stun_ms: 0,
            ..
        }
    ));
    let frag = effects.get(11008).expect("Frag Orb");
    assert!(matches!(
        frag.effect,
        EffectKind::Strike {
            damage: 254,
            stun_ms: 0,
            ..
        }
    ));
    assert_eq!((frag.name_id, frag.description_id), (20441, 20438));
    let booster = effects.get(78).expect("Super Star Booster");
    assert!(matches!(
        booster.effect,
        EffectKind::Boost {
            raises: Boosted::Stars,
            percent: 50,
            duration_ms: 1_800_000,
            icon_id: 1157,
        }
    ));
    assert_eq!((booster.name_id, booster.description_id), (2440, 2446));

    // The boss cake: never spent, its recast is the cake row's (placed_cakes.json), and its row
    // is its own definition.
    let boss = effects.get(7974).expect("Boss Birthday Cake");
    assert!(matches!(boss.effect, EffectKind::PlacedCake { row: 7974 }));
    assert_eq!((boss.name_id, boss.description_id), (30109, 30118));
    assert_eq!(boss.cooldown_ms, 60_000);
    assert!(!boss.single_use);

    // No excluded item and no OSFR synthetic ability id is present.
    for group in EXCLUDED {
        for definition in group {
            assert!(
                effects.get(*definition).is_none(),
                "excluded definition {definition} is present"
            );
        }
    }
    // No OSFR synthetic id slips in: a row at 900000 or above is one of our own reconstructed item
    // rows (the release supplement: B3's 911000 band, the recipe outputs).
    let items = item_ids();
    for row in rows.iter().filter(|row| row.definition >= 900_000) {
        assert!(items.contains(&row.definition), "{} has no item row", row.definition);
    }

    // Rule 7: every row names the pinned data it came from; the sampled rows also name the
    // client asset the effect resolves to.
    for row in rows {
        assert!(
            row.source.contains("10f129e"),
            "definition {} has no Sanctuary provenance",
            row.definition
        );
        assert!(
            row.source.contains("Legacy@9e08202"),
            "definition {} has no Legacy cross-check",
            row.definition
        );
    }
    assert!(veal.source.contains("PFX_barrier-electricity_blue_loop"));
    assert!(berries.source.contains("dog_large_floppy.adr"));
    assert!(
        effects
            .get(36)
            .expect("Can of Beans")
            .source
            .contains("EffectDelayMs 300")
    );
}

/// C3: the scope of KNOWN-BUGS C3: every potion, food, consumable, scroll, orb and illusion id the
/// item-behaviour audit lists as a player consumable (`item_behaviour.json`, corrected on main
/// `0c55fa4c`, 2026-11-12), as client items: the item table, its supplement and the captured
/// `inventory.json`.
const C3_AUDITED: [i32; 411] = [
    30, 32, 34, 35, 36, 78, 79, 81, 83, 175, 176, 213,
    306, 308, 841, 878, 896, 1146, 1147, 1148, 1167, 1168, 1169, 1180,
    1194, 1195, 1196, 1197, 1913, 1914, 2112, 2173, 2265, 2271, 2466, 2467,
    2580, 2581, 2582, 2585, 2586, 2589, 2590, 2591, 2592, 2593, 2595, 2596,
    2598, 2602, 2646, 2654, 2655, 2656, 2659, 2660, 2675, 2676, 2677, 2687,
    2697, 2939, 2940, 2943, 2947, 2950, 2951, 2954, 2956, 2957, 2958, 2959,
    2963, 2964, 2965, 2966, 2968, 2969, 2970, 2971, 2972, 2973, 2974, 2975,
    2976, 2977, 2978, 2979, 2980, 2981, 2982, 3005, 3006, 3007, 3011, 3013,
    3015, 3021, 3023, 3025, 3072, 3074, 3082, 3084, 3087, 3089, 3991, 3992,
    3993, 3995, 4001, 4002, 4005, 4011, 4012, 4013, 4014, 4015, 4016, 4017,
    4018, 4019, 4020, 4021, 4022, 4023, 4024, 4025, 4026, 4027, 4028, 4029,
    4030, 4031, 4032, 4033, 4034, 4035, 4036, 4037, 4038, 4042, 4043, 4044,
    4045, 4046, 4047, 4051, 4052, 4053, 4055, 4056, 4057, 4058, 4059, 4060,
    4061, 4062, 4063, 4064, 4065, 4066, 4067, 4070, 4071, 4072, 4828, 4829,
    7966, 7967, 7968, 7969, 7970, 7971, 7972, 7973, 7974, 8157, 8185, 9114,
    9116, 9117, 9295, 9296, 10352, 10433, 10482, 10516, 10517, 10518, 10519, 10520,
    11002, 11005, 11008, 11011, 11014, 11017, 15896, 15915, 15958, 15959, 15960, 15961,
    16033, 16034, 16035, 16036, 16537, 21807, 21809, 21810, 21811, 21812, 21813, 21814,
    21815, 21816, 21817, 21818, 21819, 21820, 21821, 21822, 21823, 21824, 21825, 21826,
    21827, 21828, 21829, 21830, 21831, 21832, 21833, 21834, 31221, 31223, 31224, 31230,
    31231, 31232, 31240, 31244, 31250, 31253, 31255, 31256, 31259, 31301, 31313, 31315,
    38822, 38823, 38827, 48100, 48104, 48154, 48155, 48156, 48244, 49997, 49998, 49999,
    55404, 55405, 55406, 55407, 55408, 55409, 55410, 55411, 55412, 55413, 55414, 55415,
    55416, 55417, 55418, 55419, 55420, 55421, 55422, 55831, 55850, 56774, 56776, 57010,
    57011, 57013, 57022, 57023, 57024, 57025, 57026, 57027, 57028, 60259, 60262, 60267,
    60269, 60271, 60272, 60273, 60275, 60277, 60280, 60563, 60986, 62516, 62529, 65820,
    65821, 65822, 65831, 65833, 67888, 67961, 67962, 67963, 67964, 68152, 68153, 68154,
    68155, 68156, 68157, 68158, 68159, 68160, 68161, 68162, 68163, 68164, 68783, 68849,
    68908, 68909, 68910, 68911, 68932, 69818, 69819, 69820, 69821, 69822, 69823, 69825,
    69828, 69839, 69840, 69841, 69842, 69845, 69846, 69847, 69848, 70327, 70328, 70329,
    76609, 76610, 76611, 76612, 76620, 76621, 76622, 76624, 76625, 76626, 76627, 76634,
    76641, 76642, 76643, 76710, 76713, 76800, 76801, 76802, 76803, 76804, 76805, 76806,
    77039, 77041, 77043, 77329, 77330, 77331, 77417, 77418, 77419, 77420, 77435, 77436,
    77437, 77438, 77439, 77440, 77442, 77443, 77459, 78137, 910206, 910224, 911060, 911137,
    911158, 911312, 911313,
];

/// C3: each audited player consumable is a client item and has an effect row, a boombox row or a
/// reason it has none, in the generator's `LEFT` table (`scripts/event_effects.py`). Repo data
/// only, so it runs everywhere.
#[test]
fn c3_every_audited_player_consumable_has_an_effect_or_a_reason() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let effects = Effects::load(&content()).unwrap();
    let boxes: BTreeSet<i64> = json(&content().join("boomboxes.json"))["boomboxes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["item_id"].as_i64().unwrap())
        .collect();
    let script = std::fs::read_to_string(root.join("../../scripts/event_effects.py")).unwrap();
    // A CRLF checkout reads the same.
    let script = script.replace("\r\n", "\n");
    let left = script.split("\nLEFT = {\n").nth(1).expect("the LEFT table");
    let left = &left[..left.find("\n}").unwrap()];
    // `    2112: "Bravery Biscuit summons ...",`: a numeric key and a reason worth reading.
    let reasons: BTreeMap<i64, &str> = left
        .lines()
        .filter_map(|line| {
            let (id, reason) = line.trim().split_once(": \"")?;
            let reason = reason.trim_end_matches("\",");
            (reason.len() > 20).then_some((id.parse().ok()?, reason))
        })
        .collect();
    let release = root.join("data/release");
    let mut items = item_ids();
    let captured = json(&release.join("inventory.json"));
    let captured = captured["definitions"].as_array().unwrap().iter();
    items.extend(captured.map(|row| i32::try_from(row["id"].as_i64().unwrap()).unwrap()));
    let mut missing = Vec::new();
    for id in C3_AUDITED {
        assert!(items.contains(&id), "{id} is no client item");
        let covered = effects.get(id).is_some()
            || boxes.contains(&i64::from(id))
            || reasons.contains_key(&i64::from(id));
        if !covered {
            missing.push(id);
        }
    }
    assert!(missing.is_empty(), "no effect and no reason: {missing:?}");
}
