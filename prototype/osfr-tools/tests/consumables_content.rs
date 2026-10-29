//! T511: the shipped `data/content/consumables.json` through the real loaders.
//!
//! The file is generated offline by `scripts/build_consumables.py` from the pinned
//! Sanctuary/Legacy item definitions and the client asset tables. These checks pin the
//! loader contract (the production `ItemBar::load` path), the two potion rows, the two
//! named rows the brief calls out, the exclusions and every row's provenance.
use osfr_tools::item_bar::{EffectKind, Effects, ItemBar};
use std::path::PathBuf;

const POTIONS: [i32; 2] = [2173, 11002];
/// Every definition the content rules exclude. `900000..` (the OSFR synthetic abilities) is
/// checked separately.
const EXCLUDED: [&[i32]; 4] = [
    // Cakes (the birthday cakes 7966..=7974 are placed_cake rows since T557, checked below)
    &[17597, 69819, 48100],
    // RandomTransformFoods
    &[69828, 48104],
    // Boomboxes
    &[
        878, 1180, 1913, 8185, 35759, 48244, 56774, 69825, 76927, 76928, 76930, 76933, 76935,
        77329, 76934, 900059, 900060, 900061, 76877, 900063, 78075,
    ],
    // PartyFavors
    &[62516, 67961, 67962, 67963, 67964, 68909, 68910, 68911],
];

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

#[test]
fn the_shipped_rows_load_through_the_real_loader_with_their_client_checked_values() {
    let directory = content();
    // The production entry point the game uses at bind; it calls `Effects::load` itself.
    ItemBar::load(&directory, Default::default(), None).expect("the shipped rows load");
    let effects = Effects::load(&directory).expect("consumables.json parses");
    let rows = effects.rows();
    assert!(
        (90..=110).contains(&rows.len()),
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
        assert!(matches!(row.effect, EffectKind::Heal { percent: 35 }));
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
    let (mut heals, mut auras, mut transforms, mut cakes) = (0, 0, 0, 0);
    for row in rows {
        match row.effect {
            EffectKind::Heal { .. } => heals += 1,
            EffectKind::Aura { .. } => auras += 1,
            EffectKind::Transform { .. } => transforms += 1,
            EffectKind::Scale { .. } => {}
            EffectKind::PlacedCake { .. } => cakes += 1,
        }
    }
    assert_eq!(heals, 2);
    assert_eq!(auras, 23);
    assert_eq!(transforms, 69);
    assert_eq!(cakes, 9);

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
    assert!(rows.iter().all(|row| row.definition < 900_000));

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
