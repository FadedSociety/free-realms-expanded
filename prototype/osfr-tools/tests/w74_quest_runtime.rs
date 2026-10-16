use osfr_tools::{
    gameplay::Gameplay,
    gateway::Gateway,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use std::{fs, path::PathBuf};

const QUEST_ID: u32 = 3_000_021;
const GIVER_ID: u64 = 9_000_000_189;

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as u32).to_le_bytes());
    bytes.extend(value.as_bytes());
}

fn login(guid: u64, version: &str) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    put_string(&mut bytes, "w74-test-ticket");
    bytes.extend(guid.to_le_bytes());
    put_string(&mut bytes, version);
    put_string(&mut bytes, "");
    bytes
}

fn application(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn packet(opcode: u16, body: &[u8]) -> Vec<u8> {
    let mut inner = opcode.to_le_bytes().to_vec();
    inner.extend(body);
    application(&inner)
}

fn message_body(packet: &[u8]) -> &[u8] {
    assert!(packet.len() >= 7 && packet[..3] == [5, 0, 1]);
    let length = u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize;
    assert_eq!(packet.len(), length + 7);
    &packet[7..]
}

fn private_paths() -> (PathBuf, PathBuf, PathBuf) {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    (
        manifest.join("../../../lab/references/rust-gateway-reference"),
        manifest.join("../../../lab/references/rust-gameplay"),
        manifest.join("data/content"),
    )
}

#[test]
#[ignore = "requires the private gateway and gameplay references"]
fn saved_regional_quest_syncs_on_world_entry_and_bad_application_messages_are_dropped() {
    let (reference, gameplay_content, quest_content) = private_paths();
    let character = Gateway::reference_state(&reference).unwrap();
    let base = Gameplay::load(&gameplay_content, None).unwrap();
    let mut quests = QuestEngine::from_split_content(
        &quest_content,
        character.guid,
        CharacterEligibility {
            member: true,
            nonmember_quest_limit: 20,
            member_quest_limit: 30,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(quests.offer(QUEST_ID, GIVER_ID).changed);
    assert!(quests.accept(QUEST_ID).changed);
    quests
        .on_event(osfr_tools::quest_engine::QuestEvent::TalkToNpc {
            npc_id: 9_500_000_254,
        })
        .unwrap();
    assert_eq!(quests.state().pending_turnin, Some(QUEST_ID));
    let accepted_record = quests.quest_add_packet(QUEST_ID).unwrap();

    let mut saved = base.state();
    saved.npc_quests = Some(quests.state().clone());
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
    let mut gateway = Gateway::load_gameplay(
        &reference,
        character.clone(),
        &gameplay_content,
        Some(saved),
    )
    .unwrap();
    gateway.set_trace_enabled(true);
    gateway
        .handle(&login(
            character.guid,
            manifest["client_version"].as_str().unwrap(),
        ))
        .unwrap();
    gateway.handle(&packet(13, &[])).unwrap();
    let entered = gateway.handle(&packet(10, &[])).unwrap();
    assert!(entered.entered_world);
    let records: Vec<_> = entered
        .replies
        .iter()
        .map(|packet| message_body(packet))
        .filter(|body| {
            body.len() >= 10
                && body[..6] == [49, 0, 3, 0, 0, 0]
                && u32::from_le_bytes(body[6..10].try_into().unwrap()) == QUEST_ID
        })
        .collect();
    assert_eq!(records, [&accepted_record[..]]);

    let mut malformed = vec![49, 0, 2, 0, 0, 0];
    malformed.extend(QUEST_ID.to_le_bytes());
    let rejected = gateway.handle(&application(&malformed)).unwrap();
    assert!(rejected.trace.iter().any(|entry| {
        entry.family == Some(49)
            && entry.subtype == Some(2)
            && entry.outcome == "rejected"
            && entry.reason.as_deref() == Some("invalid quest accept/decline reply")
            && entry.body_head_hex.is_some()
    }));

    let unknown = application(&[49, 0, 99, 0, 0, 0]);
    let rejected = gateway.handle(&unknown).unwrap();
    assert!(rejected.trace.iter().any(|entry| {
        entry.family == Some(49)
            && entry.subtype == Some(99)
            && entry.outcome == "rejected"
            && entry.reason.is_some()
            && entry.body_head_hex.is_some()
    }));
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .unwrap()
            .npc_quests
            .as_ref()
            .unwrap()
            .pending_turnin,
        Some(QUEST_ID)
    );

    let mut thanks = vec![49, 0, 14, 0, 0, 0];
    thanks.extend(QUEST_ID.to_le_bytes());
    let completed = gateway.handle(&application(&thanks)).unwrap();
    let completion_packets: Vec<_> = completed
        .replies
        .iter()
        .map(|packet| message_body(packet))
        .collect();
    assert!(
        completed.gameplay_changed,
        "trace={:?} bodies={:?}",
        completed.trace, completion_packets
    );
    assert!(completion_packets.iter().any(|body| {
        body.len() == 10
            && body[..6] == [49, 0, 4, 0, 0, 0]
            && u32::from_le_bytes(body[6..10].try_into().unwrap()) == QUEST_ID
    }));
    assert!(completion_packets.iter().any(|body| {
        body.len() == 10
            && body[..6] == [49, 0, 12, 0, 0, 0]
            && u32::from_le_bytes(body[6..10].try_into().unwrap()) == 1
    }));
    let saved = gateway.snapshot().gameplay.unwrap().npc_quests.unwrap();
    assert_eq!(saved.pending_turnin, None);
    assert_eq!(saved.quests[&QUEST_ID].status, QuestStatus::Completed);
    assert!(gateway.handle(&application(&[26, 0, 4, 0])).is_ok());
}
