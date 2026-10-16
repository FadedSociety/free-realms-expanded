use osfr_tools::{inventory::InventoryState, startup_selfdata as startup};
use startup::{
    AccountState, Appearance, CharacterName, Identity, JobState, PostQuestRecords, QuestRecords,
    SelfData, Transform,
};

pub fn self_data_packet(
    guid: u64,
    position: [f32; 3],
    rotation: [f32; 3],
) -> (Vec<u8>, usize, usize) {
    let appearance = Appearance {
        head: "head".into(),
        hair: "hair".into(),
        hair_color: 0,
        eye_color: 0,
        skin_tone: "skin".into(),
        face_paint: "paint".into(),
        model_customization: "custom".into(),
        customization_ids: [0; 5],
    };
    let body_position_offset = 20
        + 4
        + appearance.head.len()
        + 4
        + appearance.hair.len()
        + 8
        + 4
        + appearance.skin_tone.len()
        + 4
        + appearance.face_paint.len()
        + 4
        + appearance.model_customization.len()
        + 20;
    let data = SelfData {
        identity: Identity {
            unknown_identity_u64: 0,
            guid,
            model: 123,
        },
        appearance,
        transform: Transform {
            position: [position[0], position[1], position[2], 1.0].map(f32::to_bits),
            rotation: [rotation[0], rotation[1], rotation[2], 0.0].map(f32::to_bits),
        },
        name: CharacterName {
            fields: [0; 3],
            first: String::new(),
            last: String::new(),
        },
        account: AccountState {
            birthday: 0,
            age: 0,
            play_time: 0,
            underage: false,
            open_chat: false,
            membership_status: 0,
            show_member_nag: false,
            chat_country_id: 0,
            chat_language_id: 0,
            preferred_language: 0,
            chat_language: 0,
            login_count: 0,
            grandfathered: false,
            active_vehicle_loadouts: [0; 2],
        },
        jobs: JobState {
            profiles: Vec::new(),
            active_profile_id: 0,
        },
        profile_types: Vec::new(),
        collections: Vec::new(),
        inventory: InventoryState {
            coins: 0,
            items: Vec::new(),
            equipment: Vec::new(),
            next_instance_id: None,
        },
        gender: 1,
        quest_records: QuestRecords {
            first: Vec::new(),
            first_tail: [0; 17],
            second: Vec::new(),
            second_tail: [0; 4],
        },
        quests: startup::empty_quest_state(),
        quest_bindings: Vec::new(),
        post_quest_records: PostQuestRecords {
            keyed_strings: Vec::new(),
            recipes: Vec::new(),
            variable_records: Vec::new(),
            pet_source_marker: -1,
            pet_source_value: 0,
            pre_action_records: Vec::new(),
        },
        action_bars: Vec::new(),
        action_bar_flag: false,
        action_keys: Vec::new(),
        tutorial_keys: Vec::new(),
        effects: Vec::new(),
        post_effect_tail: vec![90, 91, 92],
    };
    (
        startup::build_self(&data),
        13 + body_position_offset,
        13 + body_position_offset + 16,
    )
}
