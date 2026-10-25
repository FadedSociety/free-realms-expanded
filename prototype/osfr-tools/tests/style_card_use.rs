//! T478: a style card through the real `Gateway`: 42/10 and 42/12 change what self-data and
//! AddPc show, the change is saved, and a relogged character shows it again. The gateway's own
//! session rig (`src/bin/osfr_gateway/style_card_use.rs`) covers the viewers around the player.
//! The cards are rows of the real item catalogue, the strings the client's own mapping tables.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    style_cards::{CustomizationRecord, customization_change},
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

const CARD: i32 = 900_001;
const BEARD_STRING: &[u8] = b"human_m_facialhair_beard.adr";

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn contains(replies: &[Vec<u8>], needle: &[u8]) -> bool {
    replies
        .iter()
        .any(|reply| reply.windows(needle.len()).any(|window| window == needle))
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
            "osfr-t478-gateway-{}-{}",
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

    /// A gateway for the bearded boy, from a save (or a fresh one with the card), already
    /// through login, ready and loaded. Returns the startup replies too.
    fn login(
        &self,
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
        gateway.set_identity(boy()).unwrap();
        gateway.set_style_tables(Some(tables()));
        let login = packets::login(1);
        let mut replies = gateway.handle_at(&login, 0).unwrap().replies;
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

fn expected_change() -> Vec<u8> {
    customization_change(
        1,
        false,
        &[CustomizationRecord {
            id: 5,
            string_param: String::new(),
            param: 0,
            item_id: CARD,
        }],
    )
}

#[test]
fn clean_shaven_changes_self_data_and_add_pc_and_survives_relog() {
    let rig = Rig::new();
    let (mut gateway, startup) = rig.login(None);
    assert!(
        contains(&startup, BEARD_STRING),
        "the creation look has its beard"
    );
    assert_eq!(
        gateway
            .player_presentation(false)
            .unwrap()
            .model_customization
            .as_bytes(),
        BEARD_STRING
    );

    let outcome = gateway.handle_at(&wrap(&request(10, CARD)), 1_000).unwrap();
    assert!(outcome.gameplay_changed && outcome.state_changed);
    assert_eq!(
        outcome.nearby,
        [expected_change()],
        "the viewers get the 35/39"
    );
    assert!(
        contains(&outcome.replies, &wrap(&expected_change())),
        "and so does the player"
    );
    assert!(
        contains(
            &outcome.replies,
            &wrap(&[&[38, 0, 4, 0][..], &CARD.to_le_bytes()].concat())
        ),
        "the last card is deleted"
    );
    let look = gateway.player_presentation(false).unwrap();
    assert_eq!(look.model_customization, "", "AddPc for a late joiner");
    assert_eq!(
        look.head, "<race>_<gender>_head_african.adr",
        "the rest of the look is kept"
    );

    // Relog from the saved snapshot: the same identity, the saved look.
    let saved = gateway.snapshot();
    assert!(saved.gameplay.as_ref().unwrap().appearance.is_some());
    let (relogged, startup) = rig.login(Some((saved.character, saved.gameplay)));
    assert!(
        !contains(&startup, BEARD_STRING),
        "self-data shows the new look"
    );
    assert_eq!(
        relogged
            .player_presentation(false)
            .unwrap()
            .model_customization,
        ""
    );
}

#[test]
fn a_gateway_without_the_tables_refuses_and_keeps_the_card() {
    let rig = Rig::new();
    let (mut gateway, _) = rig.login(None);
    gateway.set_style_tables(None);
    let before = gateway.snapshot();
    let outcome = gateway
        .handle_at(&wrap(&request(12, CLEAN_SHAVEN)), 1_000)
        .unwrap();
    assert!(outcome.nearby.is_empty() && !outcome.gameplay_changed && !outcome.state_changed);
    assert!(contains(
        &outcome.replies,
        b"Style cards are not available on this server."
    ));
    assert_eq!(gateway.snapshot(), before);
}
