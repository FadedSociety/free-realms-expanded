//! W150: activity portal list, refresh and Play (module `activity_portal`).
//!
//! The list reader below is written from the client reader chain (`00be79d0` -> `00be75c0` ->
//! `00be6fa0` -> `00be63e0`), not from the writer under test, and fails on any trailing or missing
//! byte exactly as the client does. All content is synthetic; no player names or chat appear.
use osfr_tools::activity_portal::{
    Activity, Catalogue, ChessActivity, Kind, Refusal, Request, Viewer, parse,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
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
            level: 9
        }),
        [1, 1]
    );
    assert_eq!(
        joins(Viewer {
            combat_job: false,
            level: 20
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
    };
    assert_eq!(catalogue.admit(12302, nine, false).unwrap().id, 12302);
}

#[test]
fn forged_ids_never_admit_and_reveal_nothing() {
    let catalogue = Catalogue::new(vec![row(96, 0)]).unwrap();
    let sleeper = Viewer {
        combat_job: false,
        level: 0,
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
    assert!(Catalogue::new((1..=64).map(|i| row(i, 0)).collect()).is_ok());
    assert!(Catalogue::new((1..=65).map(|i| row(i, 0)).collect()).is_err());
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
        .with_chess_rows(vec![chess_row(20), chess_row(66)])
        .unwrap();
    let civilian = Viewer {
        combat_job: false,
        level: 0,
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
            .with_chess_rows(vec![chess_row(96)])
            .is_err()
    );
    assert!(
        Catalogue::new(vec![row(96, 0)])
            .unwrap()
            .with_chess_rows(vec![chess_row(0)])
            .is_err()
    );
}
