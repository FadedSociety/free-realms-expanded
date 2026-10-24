//! T496: quest marker packets (35/10, 35/11) and the marker rule.
//!
//! The byte tests decode with a model of the client readers (`008db310` per 35/10 element,
//! `008db550` per 35/11 element; counts at `0090aa90` and `008fc330`), so a field moved or
//! resized fails here. `marker_for` runs on a small synthetic `QuestEngine` (it takes the
//! engine, which `Gameplay` does not expose) and on the shipped catalogue for disabled quests.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::content_dir;
use osfr_tools::{
    quest_engine::{
        CharacterEligibility, ObjectiveKind, QuestDefinition, QuestEngine, QuestEvent,
        QuestObjective, QuestProgress, QuestRequirements, QuestState, QuestStatus,
    },
    quest_markers::{
        CONTENT_FILE, MAX_ENTRIES, QuestMarker, QuestMarkerContent, encode_add_notifications,
        encode_remove_notifications, marker_for,
    },
    rewards::{RewardDisplay, RewardGrant},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};

const CHARACTER: u64 = 4_960;
const GIVER: u64 = 9_001;
const TARGET: u64 = 9_002;
const SECOND_GIVER: u64 = 9_003;
const HOLLOW_GIVER: u64 = 9_004;
const HIGH_LEVEL_GIVER: u64 = 9_005;
const KILL_GIVER: u64 = 9_006;
const KILL_TURNIN: u64 = 9_007;
const FIRST: u32 = 11;
const SECOND: u32 = 12;
const HOLLOW: u32 = 13;
const HIGH_LEVEL: u32 = 14;
const KILL: u32 = 15;

fn objective(id: u32, kind: ObjectiveKind) -> QuestObjective {
    QuestObjective {
        id,
        required: 1,
        description: format!("Goal {id}"),
        kind,
    }
}

fn definition(
    id: u32,
    giver: u64,
    turnin: u64,
    prerequisites: &[u32],
    objectives: Vec<QuestObjective>,
) -> QuestDefinition {
    QuestDefinition {
        id,
        title: format!("Quest {id}"),
        description: String::new(),
        giver_npc_id: giver,
        turnin_npc_id: turnin,
        prerequisite_quest_ids: prerequisites.to_vec(),
        requirements: QuestRequirements::default(),
        ordered: false,
        objectives,
        reward: RewardGrant {
            coins: 0,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            sc: 0,
            display: RewardDisplay {
                source_guid: 1,
                player_guid: CHARACTER,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        },
    }
}

fn definitions() -> Vec<QuestDefinition> {
    let talk = || vec![objective(1, ObjectiveKind::TalkToNpc { npc_id: TARGET })];
    let mut high = definition(HIGH_LEVEL, HIGH_LEVEL_GIVER, HIGH_LEVEL_GIVER, &[], talk());
    high.requirements.player_level = Some(50);
    vec![
        definition(FIRST, GIVER, GIVER, &[], talk()),
        definition(SECOND, SECOND_GIVER, SECOND_GIVER, &[FIRST], talk()),
        definition(HOLLOW, HOLLOW_GIVER, HOLLOW_GIVER, &[], Vec::new()),
        high,
        definition(
            KILL,
            KILL_GIVER,
            KILL_TURNIN,
            &[],
            vec![objective(
                1,
                ObjectiveKind::KillActorKind { actor_kind_id: 77 },
            )],
        ),
    ]
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 10,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 5,
        member_quest_limit: 5,
    }
}

fn engine(state: Option<QuestState>) -> QuestEngine {
    QuestEngine::new(CHARACTER, definitions(), eligibility(), state).unwrap()
}

fn saved_state(quests: &[(u32, QuestStatus, u32)]) -> QuestState {
    QuestState {
        version: 1,
        character_id: CHARACTER,
        quests: quests
            .iter()
            .cloned()
            .map(|(id, status, count)| {
                (
                    id,
                    QuestProgress {
                        status,
                        counts: vec![count],
                    },
                )
            })
            .collect(),
        pending_offer: None,
        pending_turnin: None,
        pending_turnin_actor: None,
        selected_quest_id: None,
        auto_switch_selected: false,
    }
}

const NPCS: [u64; 7] = [
    GIVER,
    TARGET,
    SECOND_GIVER,
    HOLLOW_GIVER,
    HIGH_LEVEL_GIVER,
    KILL_GIVER,
    KILL_TURNIN,
];

fn markers(engine: &QuestEngine) -> Vec<Option<QuestMarker>> {
    NPCS.iter().map(|npc| marker_for(engine, *npc)).collect()
}

/// Little-endian cursor over a packet, like the client's bounded reader.
struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take(&mut self, n: usize) -> &[u8] {
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        head
    }
    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take(4).try_into().unwrap())
    }
    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().unwrap())
    }
    fn bool(&mut self) -> bool {
        self.take(1)[0] != 0
    }
}

/// `008db310` order for a non-compact element.
#[derive(Debug, PartialEq)]
struct Notification {
    guid: u64,
    compact: bool,
    kind: i32,
    a: i32,
    image_id: i32,
    c: i32,
    d: i32,
    e: i32,
    f: bool,
    g: i32,
    trailing: bool,
}

fn read_add(bytes: &[u8]) -> Vec<Notification> {
    let mut reader = Reader(bytes);
    assert_eq!(reader.take(4), [35, 0, 10, 0]);
    let count = reader.i32();
    let rows = (0..count)
        .map(|_| {
            let guid = reader.u64();
            let compact = reader.bool();
            let kind = reader.i32();
            assert!(!compact, "the encoder writes the full form");
            Notification {
                guid,
                compact,
                kind,
                a: reader.i32(),
                image_id: reader.i32(),
                c: reader.i32(),
                d: reader.i32(),
                e: reader.i32(),
                f: reader.bool(),
                g: reader.i32(),
                trailing: reader.bool(),
            }
        })
        .collect();
    assert!(reader.0.is_empty(), "the client refuses trailing bytes");
    rows
}

#[test]
fn add_notifications_bytes_match_the_reader() {
    let one = encode_add_notifications(&[(0x0102_0304_0506_0708, 2, 4_321)]).unwrap();
    let mut expected = vec![35, 0, 10, 0, 1, 0, 0, 0];
    expected.extend(0x0102_0304_0506_0708u64.to_le_bytes());
    expected.push(0); // compact
    expected.extend(0i32.to_le_bytes()); // type
    expected.extend(0i32.to_le_bytes()); // a
    expected.extend(2i32.to_le_bytes()); // image id
    expected.extend(0i32.to_le_bytes()); // c
    expected.extend(4_321i32.to_le_bytes()); // d, the name string id
    expected.extend(0i32.to_le_bytes()); // e
    expected.push(0); // f
    expected.extend(0i32.to_le_bytes()); // g
    expected.push(0); // trailing
    assert_eq!(one, expected);
    assert_eq!(one.len(), 4 + 4 + 39);

    let rows = [(11, 2, 100), (12, 6, 200), (u64::MAX, 6, i32::MAX)];
    let three = encode_add_notifications(&rows).unwrap();
    assert_eq!(three.len(), 4 + 4 + 39 * 3);
    let decoded = read_add(&three);
    for (note, (guid, image, name)) in decoded.iter().zip(rows) {
        assert_eq!(
            *note,
            Notification {
                guid,
                compact: false,
                kind: 0,
                a: 0,
                image_id: image,
                c: 0,
                d: name,
                e: 0,
                f: false,
                g: 0,
                trailing: false,
            }
        );
    }
}

#[test]
fn remove_notifications_bytes_match_the_reader() {
    let guids = [5, 0x0A0B_0C0D_0E0F_1011, u64::MAX];
    let bytes = encode_remove_notifications(&guids).unwrap();
    assert_eq!(bytes.len(), 4 + 4 + 16 * 3);
    let mut reader = Reader(&bytes);
    assert_eq!(reader.take(4), [35, 0, 11, 0]);
    assert_eq!(reader.i32(), 3);
    for guid in guids {
        // `008db550`: u64 guid, then two i32 words.
        assert_eq!(reader.u64(), guid);
        assert_eq!((reader.i32(), reader.i32()), (0, 0));
    }
    assert!(reader.0.is_empty());
}

#[test]
fn encoders_refuse_a_count_the_packet_cannot_carry() {
    let rows = vec![(1, 2, 3); MAX_ENTRIES + 1];
    assert!(encode_add_notifications(&rows).is_none());
    assert!(encode_add_notifications(&rows[..MAX_ENTRIES]).is_some());
    let guids = vec![1; MAX_ENTRIES + 1];
    assert!(encode_remove_notifications(&guids).is_none());
    assert!(encode_remove_notifications(&guids[..MAX_ENTRIES]).is_some());
    assert_eq!(
        encode_add_notifications(&[]).unwrap(),
        [35, 0, 10, 0, 0, 0, 0, 0]
    );
}

#[test]
fn marker_rule() {
    let mut quests = engine(None);
    // A fresh player: only the open giver shows '!'. The prerequisite, level and
    // objective-less quests show nothing.
    assert_eq!(
        markers(&quests),
        [
            Some(QuestMarker::Available),
            None,
            None,
            None,
            None,
            Some(QuestMarker::Available),
            None,
        ]
    );

    // Offered then accepted: the giver stops showing '!' and the talk target shows '?'.
    assert!(quests.offer(FIRST, GIVER).changed);
    assert_eq!(marker_for(&quests, GIVER), None);
    assert!(quests.accept(FIRST).changed);
    assert_eq!(marker_for(&quests, GIVER), None);
    assert_eq!(marker_for(&quests, TARGET), Some(QuestMarker::TurnIn));

    // The talk finishes the quest and opens the end screen at the target: the target keeps
    // '?' and the formal turn-in NPC (the giver) does not also show one.
    quests
        .on_event(QuestEvent::TalkToNpc { npc_id: TARGET })
        .unwrap();
    assert_eq!(quests.pending_turnin_actor(), Some(TARGET));
    assert_eq!(marker_for(&quests, TARGET), Some(QuestMarker::TurnIn));
    assert_eq!(marker_for(&quests, GIVER), None);

    // Completed chain: nothing left to offer or hand in; the next quest's prerequisite is met.
    let done = engine(Some(saved_state(&[(FIRST, QuestStatus::Completed, 1)])));
    assert_eq!(marker_for(&done, GIVER), None);
    assert_eq!(marker_for(&done, TARGET), None);
    assert_eq!(
        marker_for(&done, SECOND_GIVER),
        Some(QuestMarker::Available)
    );
    let chain = engine(Some(saved_state(&[
        (FIRST, QuestStatus::Completed, 1),
        (SECOND, QuestStatus::Completed, 1),
    ])));
    assert_eq!(markers(&chain)[..3], [None, None, None]);
}

#[test]
fn a_finished_kill_quest_marks_its_turnin_npc_and_turnin_wins_over_available() {
    // Objectives done but no Talk opened the screen yet: '?' on the turn-in NPC, none on
    // the giver (its quest is active).
    let ready = engine(Some(saved_state(&[(KILL, QuestStatus::Active, 1)])));
    assert_eq!(marker_for(&ready, KILL_TURNIN), Some(QuestMarker::TurnIn));
    assert_eq!(marker_for(&ready, KILL_GIVER), None);
    // Still killing: nothing yet.
    let hunting = engine(Some(saved_state(&[(KILL, QuestStatus::Active, 0)])));
    assert_eq!(marker_for(&hunting, KILL_TURNIN), None);

    // The target is also a second giver: the active talk step wins over its own '!'.
    let mut both = definitions();
    both.push(definition(
        16,
        TARGET,
        TARGET,
        &[],
        vec![objective(1, ObjectiveKind::TalkToNpc { npc_id: GIVER })],
    ));
    let state = saved_state(&[(FIRST, QuestStatus::Active, 0)]);
    let quests = QuestEngine::new(CHARACTER, both, eligibility(), Some(state)).unwrap();
    assert!(quests.can_offer(16, TARGET));
    assert_eq!(marker_for(&quests, TARGET), Some(QuestMarker::TurnIn));
}

#[test]
fn an_active_quest_without_objectives_marks_nobody() {
    // A save can hold one (nothing validates the count of an empty goal list); with no open
    // goal it would otherwise read as finished and mark its turn-in NPC.
    let mut state = saved_state(&[]);
    state.quests.insert(
        HOLLOW,
        QuestProgress {
            status: QuestStatus::Active,
            counts: Vec::new(),
        },
    );
    assert_eq!(marker_for(&engine(Some(state)), HOLLOW_GIVER), None);
}

#[test]
fn marker_for_never_mutates_quest_state() {
    let states = [
        None,
        Some(saved_state(&[(FIRST, QuestStatus::Active, 0)])),
        Some(saved_state(&[(KILL, QuestStatus::Active, 1)])),
        Some(saved_state(&[(FIRST, QuestStatus::Completed, 1)])),
    ];
    for state in states {
        let quests = engine(state);
        let before = quests.state().clone();
        for npc in NPCS {
            marker_for(&quests, npc);
        }
        assert_eq!(*quests.state(), before);
    }
}

#[test]
fn shipped_disabled_quests_never_mark_their_giver() {
    let root = content_dir();
    let eligibility = CharacterEligibility {
        player_level: 100,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 100,
        member_quest_limit: 100,
    };
    let quests = QuestEngine::from_split_content(&root, CHARACTER, eligibility, None).unwrap();
    let records = quest_records::read_quests(&root);
    let enabled_givers: BTreeSet<u64> = records
        .iter()
        .filter(|quest| quest.enabled && !quest.objectives.is_empty())
        .map(|quest| quest.giver_npc_id)
        .collect();
    let hollow_givers: BTreeSet<u64> = records
        .iter()
        .filter(|quest| !quest.enabled || quest.objectives.is_empty())
        .map(|quest| quest.giver_npc_id)
        .filter(|giver| !enabled_givers.contains(giver))
        .collect();
    assert!(
        !hollow_givers.is_empty(),
        "the catalogue has disabled-only givers"
    );
    for giver in hollow_givers {
        assert_eq!(marker_for(&quests, giver), None, "giver {giver}");
    }
    assert!(
        enabled_givers
            .iter()
            .any(|giver| marker_for(&quests, *giver) == Some(QuestMarker::Available)),
        "an enabled giver shows '!'"
    );
}

struct Scratch(PathBuf);
impl Scratch {
    fn write(text: &str) -> Self {
        static SERIAL: AtomicU32 = AtomicU32::new(0);
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t496-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CONTENT_FILE);
        fs::write(&path, text).unwrap();
        Self(path)
    }
    fn load(&self) -> std::io::Result<QuestMarkerContent> {
        QuestMarkerContent::load(&self.0)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.0.parent().unwrap());
    }
}

fn marker_json(available: &str, turnin: &str) -> String {
    format!(
        r#"{{"schema_version": 1, "available_image_id": {available}, "turnin_image_id": {turnin}}}"#
    )
}

#[test]
fn quest_markers_json_bounds() {
    let shipped =
        QuestMarkerContent::load(&content_dir().join(CONTENT_FILE)).expect("shipped file loads");
    assert_eq!(shipped.image_id(QuestMarker::Available), 2);
    assert_eq!(shipped.image_id(QuestMarker::TurnIn), 6);
    assert!(Scratch::write(&marker_json("1", "10000")).load().is_ok());

    for (available, turnin) in [
        ("0", "6"),
        ("2", "0"),
        ("-1", "6"),
        ("2", "-6"),
        ("10001", "6"),
        ("2", "10001"),
    ] {
        let scratch = Scratch::write(&marker_json(available, turnin));
        assert!(scratch.load().is_err(), "{available}/{turnin}");
    }
    let missing = r#"{"schema_version": 1, "available_image_id": 2}"#;
    assert!(Scratch::write(missing).load().is_err());
    let version = r#"{"schema_version": 2, "available_image_id": 2, "turnin_image_id": 6}"#;
    assert!(Scratch::write(version).load().is_err());
    let oversized = format!(
        r#"{{"schema_version": 1, "available_image_id": 2, "turnin_image_id": 6, "source": ["{}"]}}"#,
        "x".repeat(64 * 1024)
    );
    assert!(Scratch::write(&oversized).load().is_err());
}
