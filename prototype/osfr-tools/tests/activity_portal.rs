//! W150: activity portal list, refresh and Play (module `activity_portal`).
//!
//! The list reader below is written from the client reader chain (`00be79d0` -> `00be75c0` ->
//! `00be6fa0` -> `00be63e0`), not from the writer under test, and fails on any trailing or missing
//! byte exactly as the client does. All content is synthetic; no player names or chat appear.
use osfr_tools::{
    activity_portal::{
        self, Activity, Catalogue, ChessActivity, Kind, LiveRow, Refusal, Request, Viewer,
        game_row, parse,
    },
    card_game,
    harvest_nodes::Gather,
    recipes::CraftingJob,
};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Dir(PathBuf);
impl Dir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "activity-portal-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, text: &str) -> PathBuf {
        let path = self.0.join("field_battles.json");
        fs::write(&path, text).unwrap();
        path
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn row(id: u32, level: u32) -> Activity {
    Activity {
        id,
        name_id: 42_000 + id,
        description_id: 43_000 + id,
        icon_id: 4_500 + id,
        difficulty: 2,
        required_level: level,
        kind: Kind::Battle,
    }
}

const FIGHTER: Viewer = Viewer {
    combat_job: true,
    level: 5,
    crafts: [0; 3],
    postman: 0,
};

/// Bounded little-endian reader that mirrors the client stream (flag on short read).
struct Stream<'a>(&'a [u8]);
impl Stream<'_> {
    fn take(&mut self, n: usize) -> &[u8] {
        assert!(self.0.len() >= n, "client stream ran out of bytes");
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        head
    }
    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }
    fn string(&mut self) -> String {
        let len = self.u32() as i32;
        assert!(len >= 0);
        String::from_utf8(self.take(len as usize).to_vec()).unwrap()
    }
}

struct Decoded {
    key: u32,
    rows: Vec<Vec<u32>>,
    flags: Vec<(u8, u8)>,
    strings: Vec<(String, String)>,
}

fn decode_list(packet: &[u8]) -> Decoded {
    let mut s = Stream(packet);
    assert_eq!(s.take(2), 167u16.to_le_bytes());
    assert_eq!(s.u8(), 1, "service");
    assert_eq!(s.u8(), 1, "sub-op list");
    let key = s.u32();
    let count = s.u32() as i32;
    let mut out = Decoded {
        key,
        rows: vec![],
        flags: vec![],
        strings: vec![],
    };
    for _ in 0..count {
        // +4 +8 +0x10 +0x14 +0x18 +0x20 +0x24 +0x28 +0x2c +0x1c
        let mut dwords: Vec<u32> = (0..10).map(|_| s.u32()).collect();
        let can_join = s.u8();
        dwords.push(s.u32()); // +0x34
        let featured = s.u32();
        assert_eq!(featured, 0, "no featured windows");
        dwords.push(s.u32()); // +0x38
        let members = s.u8();
        out.strings.push((s.string(), s.string()));
        dwords.extend((0..4).map(|_| s.u32())); // +0x60 +0xc +0x68 +0x64
        out.rows.push(dwords);
        out.flags.push((can_join, members));
    }
    assert!(
        s.0.is_empty(),
        "trailing bytes make the client drop the list"
    );
    out
}

#[test]
fn list_matches_the_client_reader_for_every_row() {
    let catalogue = Catalogue::new(vec![row(96, 0), row(12302, 9)]).unwrap();
    let list = decode_list(&catalogue.list_packet(FIGHTER));
    assert_eq!(list.key, 1, "entry type 1 = world tunnel");
    assert_eq!(list.rows.len(), 2);
    // Id, AppSystemId, Category, ImageSetId, Position, Name, Desc, NameId, DescId, EntryType.
    // Category 10 is the client's "Battles" row (`ActivityCategories.txt`); 18 is Micro-Games.
    assert_eq!(
        list.rows[0][..10],
        [96, 0, 10, 4_596, 1, 42_096, 43_096, 42_096, 43_096, 1]
    );
    assert_eq!(list.rows[1][0], 12302);
    assert_eq!(list.rows[1][4], 2, "positions follow list order");
    // +0x34, +0x38, Difficulty, +0xc, +0x68, MysteryChestIcon
    assert_eq!(list.rows[0][10..], [0, 0, 2, 0, 0, 0]);
    assert_eq!(list.strings[0].0, "bandit_hideout_detail.dds");
    assert_eq!(list.strings[0].1, "bandit_hideout_detail.dds");
    assert!(
        list.flags
            .iter()
            .all(|(_, members_only)| *members_only == 0)
    );
}

#[test]
fn empty_catalogue_publishes_an_empty_list() {
    let packet = Catalogue::default().list_packet(FIGHTER);
    assert_eq!(packet, [167, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0]);
    assert!(decode_list(&packet).rows.is_empty());
}

#[test]
fn can_join_is_per_player() {
    let catalogue = Catalogue::new(vec![row(1, 0), row(2, 9)]).unwrap();
    let joins = |viewer| {
        decode_list(&catalogue.list_packet(viewer))
            .flags
            .iter()
            .map(|(can, _)| *can)
            .collect::<Vec<_>>()
    };
    assert_eq!(joins(FIGHTER), [1, 0], "level 5 misses the level 9 row");
    assert_eq!(
        joins(Viewer {
            combat_job: true,
            level: 9,
            crafts: [0; 3],
            postman: 0,
        }),
        [1, 1]
    );
    assert_eq!(
        joins(Viewer {
            combat_job: false,
            level: 20,
            crafts: [0; 3],
            postman: 0,
        }),
        [0, 0],
        "a non-combat job joins nothing"
    );
}

#[test]
fn refresh_and_play_decode_exactly() {
    assert_eq!(parse(&[167, 0, 1, 4]).unwrap(), Some(Request::Refresh));
    let mut play = vec![167, 0, 1, 2];
    play.extend(96u32.to_le_bytes());
    play.extend(0u32.to_le_bytes());
    assert_eq!(parse(&play).unwrap(), Some(Request::Play { activity: 96 }));
    // The client clock dword is never trusted: changing it changes nothing.
    play[8..].copy_from_slice(&0xdead_beef_u32.to_le_bytes());
    assert_eq!(parse(&play).unwrap(), Some(Request::Play { activity: 96 }));
}

#[test]
fn malformed_lengths_are_errors_not_panics() {
    // Canary: the exact shapes are accepted, every other length of the same sub-op is refused.
    assert!(parse(&[167, 0, 1, 4]).unwrap().is_some());
    let mut play = vec![167, 0, 1, 2, 1, 0, 0, 0, 0, 0, 0, 0];
    assert!(parse(&play).unwrap().is_some());
    for extra in [vec![0], vec![0; 4]] {
        let mut longer = play.clone();
        longer.extend(extra);
        assert!(parse(&longer).is_err());
    }
    for cut in 4..play.len() {
        assert!(parse(&play[..cut]).is_err(), "short play of {cut} bytes");
    }
    assert!(parse(&[167, 0, 1, 4, 0]).is_err(), "refresh with a body");
    play.truncate(3);
    assert_eq!(parse(&play).unwrap(), None, "no sub-op yet");
}

#[test]
fn other_traffic_is_not_ours() {
    assert_eq!(parse(&[]).unwrap(), None);
    assert_eq!(parse(&[167]).unwrap(), None);
    assert_eq!(parse(&[102, 0, 1, 0]).unwrap(), None, "lobby catalogue");
    assert_eq!(
        parse(&[167, 0, 2, 2, 1, 2, 3, 4]).unwrap(),
        None,
        "scheduled service"
    );
    assert_eq!(
        parse(&[167, 0, 1, 5, 9, 9, 9, 9]).unwrap(),
        None,
        "featured update"
    );
    assert_eq!(parse(&[167, 1, 1, 2]).unwrap(), None, "not family 167");
}

#[test]
fn every_short_or_garbled_input_is_handled() {
    let mut state = 0x1234_5678_u32;
    for len in 0..40 {
        for _ in 0..64 {
            let bytes: Vec<u8> = (0..len)
                .map(|_| {
                    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    (state >> 24) as u8
                })
                .collect();
            let _ = parse(&bytes);
            let mut forged = vec![167, 0, 1, 2];
            forged.extend(&bytes);
            let _ = parse(&forged);
        }
    }
}

#[test]
fn admission_follows_the_rules_in_order() {
    let catalogue = Catalogue::new(vec![row(96, 0), row(12302, 9)]).unwrap();
    assert_eq!(catalogue.admit(96, FIGHTER, false).unwrap().id, 96);
    // Canary: each refusal flips with exactly one change.
    assert_eq!(catalogue.admit(97, FIGHTER, false), Err(Refusal::Unknown));
    assert_eq!(catalogue.admit(96, FIGHTER, true), Err(Refusal::Busy));
    assert_eq!(
        catalogue.admit(
            96,
            Viewer {
                combat_job: false,
                ..FIGHTER
            },
            false
        ),
        Err(Refusal::NotCombatJob)
    );
    assert_eq!(
        catalogue.admit(12302, FIGHTER, false),
        Err(Refusal::LevelTooLow(9))
    );
    let nine = Viewer {
        combat_job: true,
        level: 9,
        crafts: [0; 3],
        postman: 0,
    };
    assert_eq!(catalogue.admit(12302, nine, false).unwrap().id, 12302);
}

#[test]
fn forged_ids_never_admit_and_reveal_nothing() {
    let catalogue = Catalogue::new(vec![row(96, 0)]).unwrap();
    let sleeper = Viewer {
        combat_job: false,
        level: 0,
        crafts: [0; 3],
        postman: 0,
    };
    for id in [0, 1, 95, 97, 0x8000_0000, u32::MAX] {
        for busy in [false, true] {
            assert_eq!(
                catalogue.admit(id, sleeper, busy),
                Err(Refusal::Unknown),
                "id {id} busy {busy}"
            );
        }
    }
    assert_eq!(
        Catalogue::default().admit(96, FIGHTER, false),
        Err(Refusal::Unknown)
    );
}

#[test]
fn refusals_have_player_text() {
    for refusal in [
        Refusal::Unknown,
        Refusal::Busy,
        Refusal::NotCombatJob,
        Refusal::LevelTooLow(9),
    ] {
        assert!(!refusal.message().is_empty());
    }
    assert!(Refusal::LevelTooLow(9).message().contains('9'));
}

#[test]
fn catalogue_validation() {
    assert!(Catalogue::new(vec![row(0, 0)]).is_err(), "zero id");
    assert!(
        Catalogue::new(vec![row(5, 0), row(5, 1)]).is_err(),
        "duplicate id"
    );
    assert!(Catalogue::new((1..=256).map(|i| row(i, 0)).collect()).is_ok());
    assert!(Catalogue::new((1..=257).map(|i| row(i, 0)).collect()).is_err());
}

const BATTLES: &str = r#"{"schema_version":1,"battles":[
 {"id":96,"title_id":42056,"description_id":42059,"objective_id":1,"difficulty":1,
  "required_combat_level":0,"icon_id":4543,"spawner":{"x":1},"extra":"ignored"},
 {"id":12302,"title_id":42054,"description_id":42068,"difficulty":3,"required_combat_level":5,
  "icon_id":-1}]}"#;

#[test]
fn loads_the_battle_file_shape() {
    let dir = Dir::new();
    let catalogue = Catalogue::load(&dir.write(BATTLES)).unwrap();
    assert_eq!(
        catalogue.activities(),
        [
            Activity {
                id: 96,
                name_id: 42056,
                description_id: 42059,
                icon_id: 4543,
                difficulty: 1,
                required_level: 0,
                kind: Kind::Battle
            },
            Activity {
                id: 12302,
                name_id: 42054,
                description_id: 42068,
                icon_id: 0,
                difficulty: 3,
                required_level: 5,
                kind: Kind::Battle
            },
        ]
    );
}

#[test]
fn bad_files_are_refused() {
    let dir = Dir::new();
    assert!(Catalogue::load(&dir.0.join("missing.json")).is_err());
    assert!(Catalogue::load(&dir.write("not json")).is_err());
    assert!(Catalogue::load(&dir.write(r#"{"battles":[{"id":1}]}"#)).is_err());
    let duplicate = BATTLES.replace("12302", "96");
    assert!(Catalogue::load(&dir.write(&duplicate)).is_err());
    let huge = format!(r#"{{"battles":[],"pad":"{}"}}"#, "x".repeat(1 << 20));
    assert!(Catalogue::load(&dir.write(&huge)).is_err());
}

#[test]
fn shipped_battle_content_publishes_every_battle() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/field_battles.json");
    let catalogue = Catalogue::load(&path).unwrap();
    let ids: Vec<u32> = catalogue.activities().iter().map(|a| a.id).collect();
    assert!(ids.contains(&96), "Troll Summoner Madness");
    assert!(!ids.is_empty());
    // And the packet it makes is one the client reader accepts.
    assert_eq!(
        decode_list(&catalogue.list_packet(FIGHTER)).rows.len(),
        ids.len()
    );
}

fn chess_row(row_id: u32) -> ChessActivity {
    ChessActivity {
        row_id,
        name_id: 950,
        description_id: 951,
        icon_id: 917,
        difficulty: 1,
    }
}

#[test]
fn chess_rows_follow_battles_and_open_without_a_combat_job() {
    let catalogue = Catalogue::new(vec![row(96, 0)])
        .unwrap()
        .append([chess_row(20), chess_row(66)].map(Activity::from))
        .unwrap();
    let civilian = Viewer {
        combat_job: false,
        level: 0,
        crafts: [0; 3],
        postman: 0,
    };
    let list = decode_list(&catalogue.list_packet(civilian));
    assert_eq!(list.rows.len(), 3);
    // Battles first, then chess in content order; ids are the client minigame rows.
    assert_eq!(list.rows[0][0], 96);
    assert_eq!(list.rows[1][0], 20);
    assert_eq!(list.rows[2][0], 66);
    // Category 0 (the type-6 CATEGORY_ID), icon 917, positions follow the list.
    assert_eq!(
        list.rows[1][..10],
        [20, 0, 0, 917, 2, 950, 951, 950, 951, 1]
    );
    assert_eq!(list.rows[2][4], 3);
    assert_eq!(
        list.strings[1],
        ("chess_detail.dds".to_owned(), "chess_thumb.dds".to_owned())
    );
    assert_eq!(list.flags[1].0, 1, "a non-combat job can join chess");
    assert_eq!(list.flags[1].1, 0, "chess rows are not members-only");

    // Admission: no job or level gate, but a busy player is refused.
    assert_eq!(catalogue.admit(20, civilian, false).unwrap().id, 20);
    assert_eq!(catalogue.admit(20, civilian, true), Err(Refusal::Busy));
    assert_eq!(catalogue.admit(21, civilian, false), Err(Refusal::Unknown));
    assert_eq!(catalogue.kind(20), Some(Kind::Chess { row_id: 20 }));
    assert_eq!(catalogue.kind(96), Some(Kind::Battle));
    assert_eq!(catalogue.kind(21), None);

    // A battle id cannot be republished as chess, and a zero id is refused.
    assert!(
        Catalogue::new(vec![row(96, 0)])
            .unwrap()
            .append([chess_row(96).into()])
            .is_err()
    );
    assert!(
        Catalogue::new(vec![row(96, 0)])
            .unwrap()
            .append([chess_row(0).into()])
            .is_err()
    );
}

fn cards() -> Arc<card_game::Content> {
    card_game::Content::load_optional(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"),
    )
    .unwrap()
    .expect("card_game.json ships")
}

/// T504: every card row decodes through the client reader with no trailing bytes, and each field
/// is its data row's value (`Kind::CardGame` rows read the data row, not a compiled table).
#[test]
fn card_rows_match_the_client_reader() {
    let cards = cards();
    let catalogue = Catalogue::assemble(None, None, None, Some(&cards)).unwrap();
    let list = decode_list(&catalogue.list_packet(Viewer {
        combat_job: false,
        level: 0,
        crafts: [0; 3],
        postman: 0,
    }));
    assert_eq!(list.rows.len(), 17);
    for (n, data) in cards.rows().iter().enumerate() {
        let row = &list.rows[n];
        let dwords = [
            data.id,
            data.app_system_id,
            data.category,
            data.image_set_id,
            data.position,
            data.display_name_id,
            data.display_description_id,
            data.name_id,
            data.description_id,
            1, // (R) entry type: the world tunnel, not Legacy's server_type 2
            0, // preferred
            data.tutorial_activity_id.unwrap_or(0),
            data.difficulty as u32, // the lobby's -1 is 0xFFFFFFFF
            0,
            0,
            0,
        ];
        assert_eq!(row[..], dwords, "row {}", data.id);
        assert_eq!(
            list.strings[n],
            (data.detail_image.clone(), data.thumbnail_image.clone())
        );
        assert_eq!(list.flags[n], (u8::from(data.can_player_join), 0));
    }
    let lobby = &list.rows[0];
    assert_eq!((lobby[0], lobby[12]), (7, u32::MAX));
    // 14 deck rows point at the tutorial; the lobby, tutorial and Treasure War do not.
    let tutorials = list.rows.iter().filter(|row| row[11] == 1049).count();
    assert_eq!(tutorials, 14);
}

#[test]
fn card_rows_are_refused_in_order_unknown_busy_closed() {
    let cards = cards();
    let catalogue = Catalogue::assemble(None, None, None, Some(&cards))
        .unwrap()
        .append([chess_row(20).into()])
        .unwrap();
    assert_eq!(
        catalogue.kind(490),
        Some(Kind::CardGame { minigame_id: 500 })
    );
    for id in [7, 490, 1049] {
        assert_eq!(catalogue.admit(id, FIGHTER, true), Err(Refusal::Busy));
        let refusal = catalogue.admit(id, FIGHTER, false).unwrap_err();
        assert_eq!(refusal, Refusal::CardGameClosed, "row {id}");
        assert!(refusal.message().contains("currently unavailable"));
    }
    assert_eq!(
        catalogue.admit(1078, FIGHTER, false),
        Err(Refusal::TreasureWarClosed)
    );
    assert_eq!(catalogue.admit(1079, FIGHTER, true), Err(Refusal::Unknown));
    // The chess row beside them still opens.
    assert!(catalogue.admit(20, FIGHTER, false).is_ok());
}

#[test]
fn assembly_refuses_collisions_and_too_many_rows() {
    let cards = cards();
    // A chess row may not reuse a card id, and chess ids repeat nowhere.
    let published = Catalogue::assemble(None, None, None, Some(&cards)).unwrap();
    assert!(published.append([chess_row(7).into()]).is_err());
    assert!(
        Catalogue::default()
            .append([chess_row(20).into(), chess_row(20).into()])
            .is_err()
    );
    // 17 card rows plus 239 battles fit (L962 cap 256); one more does not.
    let battles = |n: u32| (2000..2000 + n).map(|id| row(id, 0)).collect::<Vec<_>>();
    assert!(
        Catalogue::new(battles(239))
            .unwrap()
            .append(cards.portal_rows())
            .is_ok()
    );
    assert!(
        Catalogue::new(battles(240))
            .unwrap()
            .append(cards.portal_rows())
            .is_err()
    );
}

fn live_row(entry_type: u32, id: u32, category: u32, name_id: u32, content_id: u32) -> LiveRow {
    LiveRow {
        entry_type,
        id,
        app_system_id: entry_type,
        category,
        image_set_id: 7_000 + id,
        position: 3,
        display_name_id: name_id + 1,
        display_description_id: name_id + 2,
        name_id,
        description_id: name_id + 3,
        recommended_set_id: 15,
        tutorial_activity_id: 1_048,
        difficulty: 2,
        content_id,
        detail_image: format!("row{id}_detail.dds"),
        thumbnail_image: format!("row{id}_thumb.dds"),
        members_only: id % 2 == 1,
    }
}

/// L095: live rows go out exactly as the live server wrote them, the minigames in their own
/// entry-type list. A live row starts the service row joined to it (Play names the live id, the
/// service id starts); every other live row is greyed out and refused; a field battle no live row
/// runs is listed hidden; other unjoined service rows are not listed.
#[test]
fn live_rows_publish_the_live_values_and_play_their_service_rows() {
    let battle = row(12_370, 0);
    let catalogue = Catalogue::new(vec![battle.clone(), row(96, 0)])
        .unwrap()
        .append([chess_row(20), chess_row(66)].map(Activity::from))
        .unwrap()
        .with_live(
            vec![
                live_row(1, 37, 10, battle.name_id, 37),
                live_row(2, 515, 18, 950, 20),
                live_row(2, 63, 16, 37_933, 37),
                live_row(2, 8, 17, 409_962, 8),
                live_row(2, 62, 16, 17_958, 28),
                live_row(2, 65, 16, 37_938, 39),
                live_row(2, 516, 21, 15_293, 77),
            ],
            Some(8),
            vec![
                game_row(28, Kind::Kart),
                game_row(39, Kind::Derby),
                game_row(
                    77,
                    Kind::Recipe {
                        recipe_id: 4,
                        job: CraftingJob::Miner,
                    },
                ),
            ],
        )
        .unwrap();

    let world = decode_list(&catalogue.list_packet(FIGHTER));
    assert_eq!(world.key, 1);
    let ids: Vec<u32> = world.rows.iter().map(|r| r[0]).collect();
    assert_eq!(ids, [37, 96], "the live battle row, then the hidden battle");
    let name = battle.name_id;
    assert_eq!(
        world.rows[0][..10],
        [37, 1, 10, 7_037, 3, name + 1, name + 2, name, name + 3, 1]
    );
    // RecommendedSetId, TutorialActivityId, Difficulty, ContentId, +0x68, MysteryChestIcon.
    assert_eq!(world.rows[0][10..], [15, 1_048, 2, 37, 0, 0]);
    assert_eq!(world.flags[0], (1, 1), "joinable, members only");
    assert_eq!(
        world.strings[0],
        ("row37_detail.dds".into(), "row37_thumb.dds".into())
    );
    assert_eq!(world.rows[1][2], 99, "hidden, as the live field battles");
    assert_eq!(
        world.strings[1],
        (
            "placeholder_detail.dds".into(),
            "placeholder_thumb.dds".into()
        )
    );

    let minigames = decode_list(&catalogue.minigame_list_packet(FIGHTER).unwrap());
    assert_eq!(minigames.key, 2);
    assert!(minigames.rows.iter().all(|r| r[9] == 2), "row entry types");
    let joinable: Vec<(u32, u8)> = (minigames.rows.iter().zip(&minigames.flags))
        .map(|(r, f)| (r[0], f.0))
        .collect();
    assert_eq!(
        joinable,
        [(515, 1), (63, 0), (8, 1), (62, 1), (65, 1), (516, 1)],
        "chess, a kart row with no race, wheel, the race and the derby (L097)"
    );

    assert_eq!(catalogue.admit(37, FIGHTER, false).unwrap().id, 12_370);
    assert_eq!(
        catalogue.admit(515, FIGHTER, false).unwrap().kind,
        Kind::Chess { row_id: 20 }
    );
    assert_eq!(catalogue.admit(96, FIGHTER, false).unwrap().id, 96);
    assert_eq!(catalogue.admit(63, FIGHTER, false), Err(Refusal::NotOpen));
    assert_eq!(catalogue.admit(63, FIGHTER, true), Err(Refusal::NotOpen));
    for service_id in [12_370, 20, 66] {
        assert_eq!(
            catalogue.admit(service_id, FIGHTER, false),
            Err(Refusal::Unknown),
            "{service_id} is not a client id any more"
        );
    }
    assert_eq!(catalogue.kind(37), Some(Kind::Battle));
    assert_eq!(catalogue.kind(63), None);
    assert_eq!(catalogue.live().len(), 7);
    // L098: a recipe row starts its recipe by MiniGameData row.
    assert_eq!(
        catalogue.admit(516, FIGHTER, false).unwrap().kind,
        Kind::Recipe {
            recipe_id: 4,
            job: CraftingJob::Miner,
        }
    );
    // L097: the race and derby rows start their services by MiniGameData row.
    assert_eq!(
        catalogue.admit(62, FIGHTER, true).map(|a| (a.id, a.kind)),
        Err(Refusal::Busy)
    );
    assert_eq!(
        catalogue.admit(62, FIGHTER, false).unwrap().kind,
        Kind::Kart
    );
    assert_eq!(catalogue.admit(65, FIGHTER, false).unwrap().id, 39);
    assert_eq!(catalogue.admit(28, FIGHTER, false), Err(Refusal::Unknown));
}

/// L100: a recipe row is joinable from its recipe's rank in its job, whichever job is active
/// (the live tin smelting row asked Miner rank 5); a gather row joins the game its engine offers
/// by MiniGameData row, and a live gather row no engine offers stays closed.
#[test]
fn recipe_rows_open_at_their_rank_and_gather_rows_join_their_game() {
    let tin = Activity {
        required_level: 5,
        ..game_row(
            134,
            Kind::Recipe {
                recipe_id: 5,
                job: CraftingJob::Miner,
            },
        )
    };
    let harvest = Kind::Gather {
        gather: Gather::Harvesting,
    };
    let catalogue = Catalogue::new(vec![row(96, 0)])
        .unwrap()
        .with_live(
            vec![
                live_row(2, 569, 21, 74_000, 134),
                live_row(2, 410, 15, 100_833, 109),
                live_row(2, 412, 15, 100_838, 113),
            ],
            None,
            vec![tin, game_row(109, harvest), game_row(111, harvest)],
        )
        .unwrap();
    let joins = |crafts: [u32; 3]| -> Vec<(u32, u8)> {
        let viewer = Viewer { crafts, ..FIGHTER };
        let list = decode_list(&catalogue.minigame_list_packet(viewer).unwrap());
        (list.rows.iter().zip(&list.flags))
            .map(|(r, f)| (r[0], f.0))
            .collect()
    };
    // Chef, Blacksmith, Miner.
    assert_eq!(joins([20, 20, 4]), [(569, 0), (410, 1), (412, 0)]);
    assert_eq!(joins([1, 1, 5]), [(569, 1), (410, 1), (412, 0)]);

    let started = catalogue.admit(410, FIGHTER, false).unwrap();
    assert_eq!((started.id, started.kind), (109, harvest));
    assert_eq!(catalogue.admit(412, FIGHTER, false), Err(Refusal::NotOpen));
    assert_eq!(
        catalogue.admit(111, FIGHTER, false),
        Err(Refusal::Unknown),
        "an offered game with no live row is not listed"
    );
}

/// L106: a mail town's row opens at its Postman rank floor (Seaside 415, game 377: rank 5).
#[test]
fn a_mail_row_opens_at_its_postman_rank() {
    let seaside = Activity {
        required_level: 5,
        ..game_row(
            377,
            Kind::Gather {
                gather: Gather::Mail,
            },
        )
    };
    let catalogue = Catalogue::new(vec![row(96, 0)])
        .unwrap()
        .with_live(
            vec![live_row(2, 415, 18, 102_033, 377)],
            None,
            vec![seaside],
        )
        .unwrap();
    let joins = |postman: u32| {
        let list = decode_list(
            &catalogue
                .minigame_list_packet(Viewer { postman, ..FIGHTER })
                .unwrap(),
        );
        list.flags[0].0
    };
    assert_eq!((joins(4), joins(5)), (0, 1));
}

#[test]
fn live_rows_refuse_bad_rows_and_a_battle_shadowing_a_live_id() {
    let base = || Catalogue::new(vec![row(96, 0)]).unwrap();
    let with = |rows: Vec<LiveRow>| base().with_live(rows, None, vec![]);
    assert!(with(vec![live_row(1, 5, 10, 1, 5), live_row(2, 5, 18, 2, 6)]).is_err());
    assert!(with(vec![live_row(1, 0, 10, 1, 5)]).is_err(), "zero id");
    assert!(with(vec![live_row(3, 5, 10, 1, 5)]).is_err(), "entry type");
    let mut long = live_row(2, 5, 18, 1, 5);
    long.detail_image = "x".repeat(65);
    assert!(with(vec![long]).is_err(), "image name");
    // The client keys rows by id: a hidden battle may not reuse a live id.
    assert!(with(vec![live_row(2, 96, 18, 1, 5)]).is_err());
    assert_eq!(with(vec![]).unwrap(), base(), "no live rows: unchanged");
    // L097: without live rows nothing lists the race, so it is not added either.
    let games = vec![game_row(28, Kind::Kart)];
    assert_eq!(base().with_live(vec![], None, games).unwrap(), base());
}

/// L095: the shipped live rows cover all 16 Games window tiles with client art names, among them
/// the Frost Ridge kart row of the owner's 2010 frames.
#[test]
fn the_shipped_live_rows_cover_every_games_window_tile() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let rows = activity_portal::load_live(&dir).unwrap();
    assert_eq!(rows.len(), 211);
    let tiles: BTreeSet<u32> = rows.iter().map(|r| r.category).collect();
    assert_eq!(
        tiles.into_iter().collect::<Vec<_>>(),
        [
            10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26
        ]
    );
    assert!(
        rows.iter()
            .all(|r| r.detail_image.ends_with(".dds") && r.thumbnail_image.ends_with(".dds"))
    );
    let kart = rows.iter().find(|r| r.id == 63).unwrap();
    assert_eq!(
        (kart.entry_type, kart.category, kart.content_id),
        (2, 16, 37)
    );
    assert_eq!(kart.tutorial_activity_id, 1_048);
    assert_eq!(
        (kart.detail_image.as_str(), kart.thumbnail_image.as_str()),
        (
            "frost_ridge_racing_detail.dds",
            "frost_ridge_racing_thumb.dds"
        )
    );
    assert!(
        activity_portal::load_live(&dir.join("missing"))
            .unwrap()
            .is_empty()
    );
}

/// L098: every shipped recipe with a MiniGameData row is a live crafting row's ContentId, so its
/// Games window row starts it: 31 of 32 (Strong Arm Stew's live row is hidden, category 99).
/// L099: the 25 cooking recipes carry their own rows since the movie runs any recipe's game.
#[test]
fn shipped_recipes_join_their_live_crafting_rows() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let live = activity_portal::load_live(&dir).unwrap();
    let recipes: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("recipes.json")).unwrap()).unwrap();
    let records = recipes["records"].as_array().unwrap();
    let joined = (records.iter())
        .filter_map(|recipe| recipe["minigame_id"].as_u64())
        .filter(|id| {
            live.iter()
                .any(|row| matches!(row.category, 12 | 14 | 21) && u64::from(row.content_id) == *id)
        })
        .count();
    assert_eq!(joined, 31);
}
