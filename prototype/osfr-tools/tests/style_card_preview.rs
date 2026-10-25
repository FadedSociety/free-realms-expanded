//! T520: 42/11 PreviewStyleCard (the store try-on) through the real `Gateway`, and the 35/65
//! customization data at world entry. The client readers are `0091aff0` (35/39: u64 guid,
//! u8 preview, then records) and `00925460` (35/65: records only, which replace the whole set
//! through `00bd9730`); the decoder below consumes every byte, as part 1 pins in
//! `tests/style_cards.rs`. The cards are rows of the real item catalogue and the strings the
//! client's own mapping tables, as in `tests/style_card_use.rs`.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    identity::{CharacterIdentity, NativeCreation},
    style_cards::{
        CustomizationRecord, FACE_PAINT, HAIR, HEAD, Look, MODEL_CUSTOMIZATION, SKIN_TONE,
        customization_change,
    },
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/style_card_fixture.rs"]
#[allow(dead_code)]
mod fixture;
#[path = "common/gateway_packets.rs"]
mod packets;
#[path = "common/round70_content.rs"]
mod round70_content;
use fixture::*;

const NOW: u64 = 1_000;
const CARD: i32 = 900_001;

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

/// The application bodies whose message (after the 5/1 tunnel header) starts with `prefix`.
fn of<'a>(messages: &'a [Vec<u8>], prefix: &[u8]) -> Vec<&'a [u8]> {
    messages
        .iter()
        .filter(|message| message.len() > 7 && message[7..].starts_with(prefix))
        .map(|message| &message[7..])
        .collect()
}

/// Bounds-checked cursor; a short read panics, so a test fails instead of guessing.
struct Wire<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Wire<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let bytes = &self.bytes[self.at..self.at + n];
        self.at += n;
        bytes
    }

    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    /// `00913230` count, then `008e9710` per record: i32 id, i32 length and bytes, i32 param,
    /// i32 item id.
    fn records(&mut self) -> Vec<CustomizationRecord> {
        (0..self.i32())
            .map(|_| {
                let id = self.i32();
                let length = usize::try_from(self.i32()).unwrap();
                let string_param = String::from_utf8(self.take(length).to_vec()).unwrap();
                CustomizationRecord {
                    id,
                    string_param,
                    param: self.i32(),
                    item_id: self.i32(),
                }
            })
            .collect()
    }

    fn done(&self) {
        assert_eq!(self.at, self.bytes.len(), "the reader consumes every byte");
    }
}

/// 35/39 as `0091aff0` reads it: u16 35, u16 39, u64 guid, u8 preview, then the records.
fn read_change(bytes: &[u8]) -> (u64, bool, Vec<CustomizationRecord>) {
    let mut wire = Wire { bytes, at: 0 };
    assert_eq!(wire.take(4), [35, 0, 39, 0]);
    let guid = u64::from_le_bytes(wire.take(8).try_into().unwrap());
    let preview = wire.take(1)[0] != 0;
    let records = wire.records();
    wire.done();
    (guid, preview, records)
}

/// 35/65 as `00925460` reads it: u16 35, u16 65, then the records; no guid and no preview.
fn read_data(bytes: &[u8]) -> Vec<CustomizationRecord> {
    let mut wire = Wire { bytes, at: 0 };
    assert_eq!(wire.take(4), [35, 0, 65, 0]);
    let records = wire.records();
    wire.done();
    records
}

/// The records the client's `00bd8e00` switch and the 35/65 filler use for a look: ids 0-6 and
/// 8, with a pixie's wings as record 7 instead of 5 (R). Item ids are 0 (R): the look does not
/// keep which card set a slot.
fn expected_records(look: &Look, wings: bool) -> Vec<CustomizationRecord> {
    let row = |id, text: &str, param| CustomizationRecord {
        id,
        string_param: text.to_owned(),
        param,
        item_id: 0,
    };
    vec![
        row(0, &look.head, look.ids[HEAD]),
        row(1, &look.skin_tone, look.ids[SKIN_TONE]),
        row(2, &look.hair, look.ids[HAIR]),
        row(3, "", look.hair_color),
        row(4, "", look.eye_color),
        row(
            if wings { 7 } else { 5 },
            &look.model_customization,
            look.ids[MODEL_CUSTOMIZATION],
        ),
        row(6, &look.face_paint, look.ids[FACE_PAINT]),
        row(8, "", look.model),
    ]
}

/// A pixie boy with the creepy wings, for the record-7 rule.
fn pixie() -> CharacterIdentity {
    let mut choices = [0; 21];
    // Model, head, hair, model customization, face paint, skin tone; eye and hair tints.
    choices[..6].copy_from_slice(&[2, 2, 1, 13, 0, 3]);
    choices[18..20].copy_from_slice(&[6, 5]);
    CharacterIdentity {
        guid: 1,
        first_name: "A".into(),
        last_name: String::new(),
        model: 2,
        head: "<race>_<gender>_head_african.adr".into(),
        hair: "<race>_m_hair_afro_<hairtype>.adr".into(),
        model_customization: "fairy_m_wings_creepy.adr".into(),
        face_paint: "none".into(),
        skin_tone: "skintone3".into(),
        eye_color: 6,
        hair_color: 5,
        gender: 1,
        creation: NativeCreation {
            first_name: "A".into(),
            last_name: String::new(),
            temporary_first_name: String::new(),
            temporary_last_name: String::new(),
            choices,
        },
        attachments: Vec::new(),
    }
}

struct Rig {
    reference: PathBuf,
    content: PathBuf,
    root: PathBuf,
}

impl Rig {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "osfr-t520-gateway-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self {
            reference: round70_content::reference(),
            content: round70_content::build(&root, "content", &[]),
            root,
        }
    }

    /// A gateway for `identity`, already through login, ready and loaded. Returns the startup
    /// replies too. `tables` off is the server without `--resources`.
    fn login(
        &self,
        identity: &CharacterIdentity,
        tables: bool,
        save: Option<(CharacterState, Option<GameplayState>)>,
    ) -> (Gateway, Vec<Vec<u8>>) {
        let (character, state) = save.unwrap_or_else(|| {
            let mut state = Gameplay::load(&self.content, None).unwrap().state();
            state.inventory.items.push(card_item(CARD, CLEAN_SHAVEN, 1));
            state.inventory.next_instance_id = Some(i64::from(CARD) + 1);
            (
                Gateway::reference_state(&self.reference).unwrap(),
                Some(state),
            )
        });
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, state).unwrap();
        gateway.set_identity(identity.clone()).unwrap();
        if tables {
            gateway.set_style_tables(Some(fixture::tables()));
        }
        let mut replies = gateway.handle_at(&packets::login(1), 0).unwrap().replies;
        for inner in [[13, 0], [10, 0]] {
            replies.extend(gateway.handle_at(&wrap(&inner), 100).unwrap().replies);
        }
        (gateway, replies)
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn customization_data_at_world_entry() {
    let rig = Rig::new();
    let (mut gateway, replies) = rig.login(&boy(), true, None);
    let sent = of(&replies, &[35, 0, 65, 0]);
    assert_eq!(sent.len(), 1, "exactly one 35/65 at world entry");
    let records = read_data(sent[0]);
    assert_eq!(
        records,
        expected_records(&Look::from_identity(&boy()), false),
        "the byte layout the client reader consumes"
    );
    assert_eq!(
        records.iter().map(|record| record.id).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5, 6, 8]
    );
    assert!(
        records.iter().all(|record| record.item_id == 0),
        "creation looks carry item id 0"
    );

    // Never per replayed load, tick or location report.
    let replay = gateway.handle_at(&wrap(&[10, 0]), 200).unwrap();
    assert!(of(&replay.replies, &[35, 0, 65, 0]).is_empty());
    let tick = gateway.tick_at(300).unwrap();
    assert!(of(&tick.replies, &[35, 0, 65, 0]).is_empty());
    let mut move_packet = vec![125, 0];
    move_packet.extend(1u64.to_le_bytes());
    move_packet.extend(
        gateway
            .snapshot()
            .character
            .position
            .map(f32::to_le_bytes)
            .concat(),
    );
    move_packet.extend([0.0f32; 3].map(f32::to_le_bytes).concat());
    move_packet.extend([0, 0]);
    let location = gateway.handle_at(&wrap(&move_packet), 400).unwrap();
    assert!(
        of(&location.replies, &[35, 0, 65, 0]).is_empty(),
        "no 35/65 on a location report"
    );

    // A pixie's wings are record 7 instead of record 5 (R).
    let (_, pixie_replies) = rig.login(&pixie(), true, None);
    let sent = of(&pixie_replies, &[35, 0, 65, 0]);
    assert_eq!(sent.len(), 1, "exactly one 35/65 for a pixie");
    let records = read_data(sent[0]);
    assert_eq!(
        records,
        expected_records(&Look::from_identity(&pixie()), true)
    );
    assert_eq!(
        records.iter().map(|record| record.id).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 7, 6, 8]
    );

    // Without the client tables there is nothing to resolve the look through.
    let (_, no_tables) = rig.login(&boy(), false, None);
    assert!(of(&no_tables, &[35, 0, 65, 0]).is_empty());
}

#[test]
fn preview_reaches_self_only_and_saves_nothing() {
    let rig = Rig::new();
    let (mut gateway, _) = rig.login(&boy(), true, None);
    let before = gateway.snapshot();
    let record = CustomizationRecord {
        id: 6,
        string_param: "skull".into(),
        param: 1,
        item_id: 0,
    };
    let expected = customization_change(1, true, std::slice::from_ref(&record));

    // 36004 is the skull face paint; the character owns only CLEAN_SHAVEN, so a try-on needs no
    // ownership.
    let outcome = gateway
        .handle_at(&wrap(&request(11, SKULL_FACE_PAINT)), NOW)
        .unwrap();
    assert!(
        outcome.nearby.is_empty(),
        "a preview never reaches another player"
    );
    assert!(
        !outcome.gameplay_changed && !outcome.state_changed,
        "a preview is not a change to commit"
    );
    assert_eq!(
        outcome.replies,
        [wrap(&expected)],
        "one 35/39 preview 1 to its sender and nothing else"
    );
    assert_eq!(
        read_change(&of(&outcome.replies, &[35, 0, 39, 0])[0]),
        (1, true, vec![record])
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "the save and the look are unchanged"
    );
    assert!(
        gateway.snapshot().gameplay.unwrap().appearance.is_none(),
        "nothing was saved"
    );
}

#[test]
fn refused_previews_send_nothing_and_change_nothing() {
    let rig = Rig::new();
    let (mut gateway, _) = rig.login(&boy(), true, None);
    // Unknown definition, not a type 12 card, a female-only card, a pixie wing set on a human,
    // the China heads' slot 0, the model slot 8 and a hair style for the other gender.
    for definition in [
        424_242,
        NOT_A_CARD,
        BUTTERFLY_WINGS,
        FAIRY_WINGS,
        CHINA_HEAD,
        MODEL_CARD,
        6834,
    ] {
        let before = gateway.snapshot();
        let outcome = gateway
            .handle_at(&wrap(&request(11, definition)), NOW)
            .unwrap();
        assert!(
            outcome.replies.is_empty(),
            "a refused preview sends no reply ({definition})"
        );
        assert!(outcome.nearby.is_empty(), "{definition}");
        assert!(
            !outcome.gameplay_changed && !outcome.state_changed,
            "{definition}"
        );
        assert_eq!(gateway.snapshot(), before, "{definition}");
    }
    // A card the look fits is answered, so the refusals above are not a blanket silence.
    let accepted = gateway
        .handle_at(&wrap(&request(11, SKULL_FACE_PAINT)), NOW)
        .unwrap();
    assert_eq!(of(&accepted.replies, &[35, 0, 39, 0]).len(), 1);
}

#[test]
fn customization_data_after_relog_has_the_new_look() {
    let rig = Rig::new();
    let (mut gateway, _) = rig.login(&boy(), true, None);
    let used = gateway.handle_at(&wrap(&request(10, CARD)), NOW).unwrap();
    assert!(used.gameplay_changed, "the card changed the look");
    let saved = gateway.snapshot();
    assert!(saved.gameplay.as_ref().unwrap().appearance.is_some());

    let (_, replies) = rig.login(&boy(), true, Some((saved.character, saved.gameplay)));
    let sent = of(&replies, &[35, 0, 65, 0]);
    assert_eq!(sent.len(), 1, "one 35/65 after the relog");
    let records = read_data(sent[0]);
    let clean_shaven = &records[5];
    assert_eq!(
        (
            clean_shaven.id,
            clean_shaven.string_param.as_str(),
            clean_shaven.param
        ),
        (5, "", 0),
        "the saved style-card look is the committed set"
    );
    let mut expected = expected_records(&Look::from_identity(&boy()), false);
    expected[5] = CustomizationRecord {
        id: 5,
        string_param: String::new(),
        param: 0,
        item_id: 0,
    };
    assert_eq!(records, expected);
}
